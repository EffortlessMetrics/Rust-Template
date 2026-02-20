//! Kernel pack manifest generation and verification.
//!
//! Implements two commands:
//! - `kernel-pack`: Generate a kernel pack manifest listing all governance files with SHA-256 checksums
//! - `kernel-check`: Verify repo state against a kernel pack manifest
//!
//! See ADR-0031 for the kernel pack distribution design.

use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Glob patterns for files included in the kernel pack.
///
/// These are the governance assets that downstream repos consume.
/// Order is cosmetic (manifest is sorted by path).
const KERNEL_FILE_PATTERNS: &[&str] = &[
    ".github/workflows/*.yml",
    ".claude/skills/*/SKILL.md",
    ".claude/agents/*.md",
    ".claude/rules/*.md",
    "flake.nix",
    "flake.lock",
    "deny.toml",
    ".pre-commit-config.yaml",
    "specs/spec_ledger.yaml",
    "specs/devex_flows.yaml",
    "specs/schemas/*.json",
    "specs/features/**/*.feature",
    "docs/AGENT_GUIDE.md",
    "docs/GLOSSARY.md",
    "docs/how-to/*.md",
    "Justfile",
    "cliff.toml",
    "CLAUDE.md",
];

/// Wave 1 publishable crates (same list as publish_check.rs, kept in sync)
const PUBLISHABLE_CRATES: &[&str] = &[
    "rust-as-spec-ac-kernel",
    "rust-as-spec-business-core",
    "rust-as-spec-gov-contracts",
    "rust-as-spec-gov-model",
    "rust-as-spec-gov-policy",
    "rust-as-spec-gov-receipts",
    "rust-as-spec-gov-xtask-core",
    "rust-as-spec-model",
    "rust-as-spec-runtime",
    "rust-as-spec-telemetry",
];

/// Manifest structure written to kernel-pack.manifest.json
#[derive(Debug, Serialize, Deserialize)]
pub struct KernelPackManifest {
    /// Template/kernel version from workspace Cargo.toml
    pub version: String,
    /// ISO 8601 timestamp of generation
    pub generated_at: String,
    /// Map of relative path -> file metadata
    pub files: BTreeMap<String, FileEntry>,
    /// Total number of files in the manifest
    pub file_count: usize,
    /// Crate names eligible for crates.io publishing
    pub publishable_crates: Vec<String>,
}

/// Metadata for a single file in the kernel pack
#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    /// SHA-256 hex digest
    pub sha256: String,
    /// File size in bytes
    pub size: u64,
}

/// Minimal as_spec.toml structure for kernel-check
#[derive(Debug, Deserialize)]
struct AsSpecConfig {
    #[serde(default)]
    kernel: Option<AsSpecKernel>,
}

#[derive(Debug, Deserialize)]
struct AsSpecKernel {
    #[serde(default)]
    version: Option<String>,
}

// ---------------------------------------------------------------------------
// kernel-pack command
// ---------------------------------------------------------------------------

/// Generate a kernel pack manifest.
///
/// Collects all governance files matching [`KERNEL_FILE_PATTERNS`], computes
/// SHA-256 checksums, and writes `kernel-pack.manifest.json` to `output_dir`.
pub fn run_pack(output_dir: &str) -> Result<()> {
    let root = workspace_root()?;
    let output_path = if Path::new(output_dir).is_absolute() {
        PathBuf::from(output_dir)
    } else {
        root.join(output_dir)
    };

    println!("{}", "Generating kernel pack manifest...".blue().bold());
    println!();

    // Read workspace version from root Cargo.toml
    let version = read_workspace_version(&root)?;
    println!("  {} Workspace version: {}", "i".cyan(), version);

    // Collect matching files
    let mut files = BTreeMap::new();
    let mut missing_patterns = Vec::new();

    for pattern in KERNEL_FILE_PATTERNS {
        let full_pattern = root.join(pattern).to_string_lossy().to_string();
        let matches: Vec<_> = glob::glob(&full_pattern)
            .with_context(|| format!("Invalid glob pattern: {}", pattern))?
            .filter_map(|entry| entry.ok())
            .collect();

        if matches.is_empty() {
            missing_patterns.push(*pattern);
        }

        for path in matches {
            if path.is_file() {
                let relative = path
                    .strip_prefix(&root)
                    .with_context(|| {
                        format!("File {} is not under workspace root", path.display())
                    })?
                    .to_string_lossy()
                    .to_string();

                let sha256 = compute_sha256(&path)?;
                let size = fs::metadata(&path)
                    .with_context(|| format!("Failed to stat {}", path.display()))?
                    .len();

                files.insert(relative, FileEntry { sha256, size });
            }
        }
    }

    // Report missing patterns (warning, not error)
    if !missing_patterns.is_empty() {
        println!();
        for pat in &missing_patterns {
            println!("  {} No files matched pattern: {}", "!".yellow(), pat);
        }
    }

    let file_count = files.len();
    let publishable_crates: Vec<String> =
        PUBLISHABLE_CRATES.iter().map(|s| s.to_string()).collect();

    let manifest = KernelPackManifest {
        version: version.clone(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        files,
        file_count,
        publishable_crates,
    };

    // Write manifest
    fs::create_dir_all(&output_path)
        .with_context(|| format!("Failed to create output directory: {}", output_path.display()))?;

    let manifest_path = output_path.join("kernel-pack.manifest.json");
    let json = serde_json::to_string_pretty(&manifest)
        .context("Failed to serialize kernel pack manifest")?;
    fs::write(&manifest_path, &json)
        .with_context(|| format!("Failed to write manifest: {}", manifest_path.display()))?;

    println!();
    println!("  {} {} files collected", "OK".green().bold(), file_count);
    println!("  {} {} publishable crates listed", "OK".green().bold(), PUBLISHABLE_CRATES.len());
    println!("  {} Manifest written to: {}", "OK".green().bold(), manifest_path.display());
    println!();
    println!(
        "{} Kernel pack manifest v{} generated successfully.",
        "Done.".green().bold(),
        version
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// kernel-check command
// ---------------------------------------------------------------------------

/// Verify repo state against a kernel pack manifest.
///
/// - Reads `as_spec.toml` (if present) for version pinning
/// - Reads the manifest from `manifest_path` (or default location)
/// - Verifies SHA-256 checksums for every file in the manifest
/// - Reports mismatches and missing files
pub fn run_check(manifest_path: Option<&str>) -> Result<()> {
    let root = workspace_root()?;

    println!("{}", "Verifying repo against kernel pack manifest...".blue().bold());
    println!();

    // Determine manifest location
    let manifest_file = match manifest_path {
        Some(p) => {
            let path = PathBuf::from(p);
            if path.is_absolute() { path } else { root.join(p) }
        }
        None => root.join("target/kernel-pack/kernel-pack.manifest.json"),
    };

    if !manifest_file.exists() {
        anyhow::bail!(
            "Manifest not found at: {}\n\
             Run `cargo xtask kernel-pack` first to generate the manifest.",
            manifest_file.display()
        );
    }

    // Load manifest
    let manifest_json = fs::read_to_string(&manifest_file)
        .with_context(|| format!("Failed to read manifest: {}", manifest_file.display()))?;
    let manifest: KernelPackManifest = serde_json::from_str(&manifest_json)
        .with_context(|| format!("Failed to parse manifest: {}", manifest_file.display()))?;

    println!(
        "  {} Manifest version: {} ({} files)",
        "i".cyan(),
        manifest.version,
        manifest.file_count
    );

    // Check as_spec.toml if present
    let as_spec_path = root.join("as_spec.toml");
    if as_spec_path.exists() {
        let as_spec_content =
            fs::read_to_string(&as_spec_path).context("Failed to read as_spec.toml")?;
        let config: AsSpecConfig =
            toml_parse(&as_spec_content).context("Failed to parse as_spec.toml")?;

        if let Some(kernel) = &config.kernel
            && let Some(pinned_version) = &kernel.version
        {
            println!("  {} as_spec.toml pins kernel version: {}", "i".cyan(), pinned_version);
            if *pinned_version != manifest.version {
                println!(
                    "  {} Version mismatch: as_spec.toml pins {} but manifest is {}",
                    "!".yellow(),
                    pinned_version,
                    manifest.version
                );
            }
        }
    } else {
        println!(
            "  {} No as_spec.toml found (standalone mode, validating manifest consistency)",
            "i".cyan()
        );
    }

    println!();

    // Verify each file in the manifest
    let mut ok_count = 0;
    let mut mismatch_count = 0;
    let mut missing_count = 0;
    let mut mismatches: Vec<String> = Vec::new();
    let mut missing: Vec<String> = Vec::new();

    for (relative_path, expected) in &manifest.files {
        let full_path = root.join(relative_path);

        if !full_path.exists() {
            missing_count += 1;
            missing.push(relative_path.clone());
            continue;
        }

        let actual_sha256 = compute_sha256(&full_path)?;
        if actual_sha256 == expected.sha256 {
            ok_count += 1;
        } else {
            mismatch_count += 1;
            mismatches.push(relative_path.clone());
        }
    }

    // Report results
    if !missing.is_empty() {
        println!("  {} Missing files:", "MISS".red().bold());
        for path in &missing {
            println!("    - {}", path);
        }
        println!();
    }

    if !mismatches.is_empty() {
        println!("  {} Checksum mismatches:", "DIFF".yellow().bold());
        for path in &mismatches {
            println!("    - {}", path);
        }
        println!();
    }

    println!(
        "  {} {} OK, {} mismatched, {} missing (of {} total)",
        if mismatch_count == 0 && missing_count == 0 {
            "PASS".green().bold()
        } else {
            "FAIL".red().bold()
        },
        ok_count,
        mismatch_count,
        missing_count,
        manifest.file_count
    );

    if mismatch_count > 0 || missing_count > 0 {
        println!();
        anyhow::bail!(
            "Kernel check failed: {} mismatch(es), {} missing file(s)",
            mismatch_count,
            missing_count
        );
    }

    println!();
    println!(
        "{} All {} kernel files verified against manifest v{}.",
        "Done.".green().bold(),
        ok_count,
        manifest.version
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Compute SHA-256 checksum of a file using `sha256sum` CLI.
fn compute_sha256(path: &Path) -> Result<String> {
    let output = Command::new("sha256sum")
        .arg(path)
        .output()
        .with_context(|| format!("Failed to run sha256sum on {}", path.display()))?;

    if !output.status.success() {
        anyhow::bail!(
            "sha256sum failed for {}: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let hash = stdout
        .split_whitespace()
        .next()
        .with_context(|| format!("Unexpected sha256sum output for {}", path.display()))?;

    Ok(hash.to_string())
}

/// Read the workspace version from the root Cargo.toml.
///
/// Parses `version = "X.Y.Z"` from the `[workspace.package]` section.
fn read_workspace_version(root: &Path) -> Result<String> {
    let cargo_toml_path = root.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_toml_path)
        .with_context(|| format!("Failed to read {}", cargo_toml_path.display()))?;

    // Find the version in [workspace.package] section
    let mut in_workspace_package = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[workspace.package]" {
            in_workspace_package = true;
            continue;
        }
        if in_workspace_package && trimmed.starts_with('[') {
            // Left the section
            break;
        }
        if in_workspace_package && trimmed.starts_with("version") {
            // Parse version = "X.Y.Z"
            if let Some(version) =
                trimmed.split('=').nth(1).map(|v| v.trim().trim_matches('"').to_string())
            {
                return Ok(version);
            }
        }
    }

    anyhow::bail!("Could not find version in [workspace.package] section of Cargo.toml")
}

/// Minimal TOML parser for as_spec.toml (avoids adding a `toml` crate dependency).
///
/// We only need to extract `[kernel].version`, so a serde_json roundtrip via
/// a simple key-value parser is sufficient.
fn toml_parse(content: &str) -> Result<AsSpecConfig> {
    // Use a simple line-based parser for the subset we need.
    // This avoids pulling in the `toml` crate just for one config file.
    let mut kernel_version: Option<String> = None;
    let mut current_section = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = trimmed[1..trimmed.len() - 1].trim().to_string();
            continue;
        }
        if current_section == "kernel"
            && let Some((key, value)) = trimmed.split_once('=')
        {
            let key = key.trim();
            let value = value.trim().trim_matches('"');
            if key == "version" {
                kernel_version = Some(value.to_string());
            }
        }
    }

    Ok(AsSpecConfig { kernel: kernel_version.map(|v| AsSpecKernel { version: Some(v) }) })
}

/// Determine workspace root from CARGO_MANIFEST_DIR.
fn workspace_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .context("Could not determine workspace root")?;
    Ok(root.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toml_parse_kernel_version() {
        let content = r#"
# Kernel version pin
[kernel]
version = "3.3.15"

[overlay]
spec_ledger = "specs/spec_ledger.yaml"
"#;
        let config = toml_parse(content).unwrap();
        assert_eq!(config.kernel.as_ref().unwrap().version.as_deref(), Some("3.3.15"));
    }

    #[test]
    fn test_toml_parse_empty() {
        let config = toml_parse("").unwrap();
        assert!(config.kernel.is_none());
    }

    #[test]
    fn test_toml_parse_no_kernel_section() {
        let content = r#"
[overlay]
spec_ledger = "specs/spec_ledger.yaml"
"#;
        let config = toml_parse(content).unwrap();
        assert!(config.kernel.is_none());
    }

    #[test]
    fn test_read_workspace_version() {
        // This test runs against the real workspace Cargo.toml
        let root = workspace_root().unwrap();
        let version = read_workspace_version(&root).unwrap();
        assert!(!version.is_empty());
        // Should look like a semver version
        assert!(version.split('.').count() == 3, "Expected semver format, got: {}", version);
    }
}
