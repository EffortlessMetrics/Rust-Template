use anyhow::{Context, Result};
use chrono::Utc;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

/// Build time metrics captured from a single build
#[derive(Debug, Serialize, Deserialize)]
pub struct BuildTimeMetrics {
    /// Timestamp when build was measured
    pub timestamp: String,
    /// Git SHA at time of measurement
    pub git_sha: String,
    /// Template version
    pub version: String,
    /// Total build time in seconds
    pub total_time_sec: f64,
    /// Codegen time in seconds (estimated from timings)
    pub codegen_time_sec: Option<f64>,
    /// Linker time in seconds (estimated from timings)
    pub linker_time_sec: Option<f64>,
    /// Debug binary size in MB (if available)
    pub debug_size_mb: Option<f64>,
    /// Release binary size in MB (if available)
    pub release_size_mb: Option<f64>,
}

/// Capture build time metrics for the workspace
pub fn run_capture() -> Result<()> {
    println!("{}", "Capturing build time metrics".blue().bold());
    println!();

    // Clean first to get accurate cold build time
    println!("Cleaning workspace...");
    let mut clean_cmd = crate::cargo_cmd("clean", &[]);
    clean_cmd.output().context("Failed to clean workspace")?;

    // Capture release build time
    println!("Building release mode (this will take a while)...");
    let start = Instant::now();
    let mut build_cmd = crate::cargo_cmd("build", &["--release", "--verbose"]);
    let output = build_cmd.output().context("Failed to run cargo build")?;
    let build_time = start.elapsed().as_secs_f64();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Build failed:\n{}", stderr);
    }

    // Parse build output for timing information (if available)
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}\n{}", stdout, stderr);

    // Extract timings (cargo --timings would be better, but this is a simple fallback)
    let codegen_time = extract_timing(&combined, "Codegen");
    let linker_time = extract_timing(&combined, "Linker");

    // Get git SHA
    let git_sha = get_git_sha()?;

    // Get version
    let version = get_workspace_version()?;

    // Get binary sizes
    let debug_size = get_binary_size("target/debug/app-http");
    let release_size = get_binary_size("target/release/app-http");

    let metrics = BuildTimeMetrics {
        timestamp: Utc::now().to_rfc3339(),
        git_sha,
        version,
        total_time_sec: build_time,
        codegen_time_sec: codegen_time,
        linker_time_sec: linker_time,
        debug_size_mb: debug_size,
        release_size_mb: release_size,
    };

    // Save metrics
    let output_path = PathBuf::from("build_times.json");
    save_metrics(&metrics, &output_path)?;

    // Print summary
    println!();
    println!("{}", "Build Time Metrics:".bold());
    println!("  Timestamp: {}", metrics.timestamp);
    println!("  Git SHA: {}", metrics.git_sha);
    println!("  Version: {}", metrics.version);
    println!("  Total time: {:.2}s", metrics.total_time_sec);
    if let Some(codegen) = metrics.codegen_time_sec {
        println!("  Codegen time: {:.2}s", codegen);
    }
    if let Some(linker) = metrics.linker_time_sec {
        println!("  Linker time: {:.2}s", linker);
    }
    if let Some(debug_size) = metrics.debug_size_mb {
        println!("  Debug binary: {:.2} MB", debug_size);
    }
    if let Some(release_size) = metrics.release_size_mb {
        println!("  Release binary: {:.2} MB", release_size);
    }
    println!();
    println!("{} Metrics saved to: {}", "✓".green(), output_path.display());

    Ok(())
}

/// Compare two build time metrics files
pub fn run_compare(baseline_path: &str, current_path: &str) -> Result<()> {
    println!("{}", "Comparing build time metrics".blue().bold());
    println!();

    let baseline = load_metrics(baseline_path)?;
    let current = load_metrics(current_path)?;

    println!("{}", "Baseline:".bold());
    println!("  Version: {} ({})", baseline.version, baseline.git_sha);
    println!("  Total time: {:.2}s", baseline.total_time_sec);

    println!();
    println!("{}", "Current:".bold());
    println!("  Version: {} ({})", current.version, current.git_sha);
    println!("  Total time: {:.2}s", current.total_time_sec);

    println!();
    println!("{}", "Comparison:".bold());

    // Compare total time
    let time_diff = current.total_time_sec - baseline.total_time_sec;
    let time_pct = (time_diff / baseline.total_time_sec) * 100.0;

    if time_diff > 0.0 {
        println!("  Total time: {} (+{:.2}s, +{:.1}%)", "slower".red(), time_diff, time_pct);
    } else {
        println!("  Total time: {} ({:.2}s, {:.1}%)", "faster".green(), time_diff, time_pct);
    }

    // Compare release binary size if available
    if let (Some(baseline_size), Some(current_size)) =
        (baseline.release_size_mb, current.release_size_mb)
    {
        let size_diff = current_size - baseline_size;
        let size_pct = (size_diff / baseline_size) * 100.0;

        if size_diff > 0.0 {
            println!(
                "  Release size: {} (+{:.2} MB, +{:.1}%)",
                "larger".red(),
                size_diff,
                size_pct
            );
        } else {
            println!(
                "  Release size: {} ({:.2} MB, {:.1}%)",
                "smaller".green(),
                size_diff,
                size_pct
            );
        }
    }

    Ok(())
}

fn extract_timing(output: &str, phase: &str) -> Option<f64> {
    // This is a placeholder - cargo doesn't emit detailed timing by default
    // Use `cargo build --timings` for real timing data
    // For now, return None
    let _ = (output, phase);
    None
}

fn get_git_sha() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .context("Failed to get git SHA")?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Ok("unknown".to_string())
    }
}

fn get_workspace_version() -> Result<String> {
    // Read version from workspace Cargo.toml
    let cargo_toml = fs::read_to_string("Cargo.toml").context("Failed to read Cargo.toml")?;

    for line in cargo_toml.lines() {
        if line.contains("version = ") && !line.starts_with('#') {
            if let Some(version) = line.split('"').nth(1) {
                return Ok(version.to_string());
            }
        }
    }

    Ok("unknown".to_string())
}

fn get_binary_size(path: &str) -> Option<f64> {
    if let Ok(metadata) = fs::metadata(path) {
        let size_mb = metadata.len() as f64 / 1_048_576.0; // Convert to MB
        Some(size_mb)
    } else {
        None
    }
}

fn save_metrics(metrics: &BuildTimeMetrics, path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(metrics).context("Failed to serialize metrics")?;
    fs::write(path, json).context("Failed to write metrics file")?;
    Ok(())
}

fn load_metrics(path: &str) -> Result<BuildTimeMetrics> {
    let json = fs::read_to_string(path).context("Failed to read metrics file")?;
    let metrics: BuildTimeMetrics =
        serde_json::from_str(&json).context("Failed to parse metrics JSON")?;
    Ok(metrics)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_time_metrics_serialization() {
        let metrics = BuildTimeMetrics {
            timestamp: "2025-12-01T12:00:00Z".to_string(),
            git_sha: "abc123".to_string(),
            version: "3.3.6".to_string(),
            total_time_sec: 156.42,
            codegen_time_sec: Some(48.3),
            linker_time_sec: Some(12.5),
            debug_size_mb: Some(4230.0),
            release_size_mb: Some(156.0),
        };

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: BuildTimeMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.version, "3.3.6");
        assert_eq!(deserialized.total_time_sec, 156.42);
    }

    #[test]
    fn test_run_capture_exists() {
        // Verify the function signature
        let _: fn() -> Result<()> = run_capture;
    }

    #[test]
    fn test_run_compare_exists() {
        // Verify the function signature
        let _: fn(&str, &str) -> Result<()> = run_compare;
    }
}
