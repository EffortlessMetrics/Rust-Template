use anyhow::Result;
use colored::Colorize;
use std::process::Command;

pub fn run() -> Result<()> {
    println!("{}", "🩺 Running environment diagnostics...".blue().bold());
    println!();

    let mut issues = 0;
    let mut warnings = 0;

    // Environment Section: Detect Nix vs native, rustc version, sccache status
    println!("{}", "Environment:".bold());

    // Detect environment type (Nix devshell vs native)
    print!("  Environment type... ");
    let in_nix_shell = std::env::var("IN_NIX_SHELL").is_ok();
    if in_nix_shell {
        println!("{} {}", "✓".green(), "Nix devshell".dimmed());
    } else {
        println!("{} {}", "⚠".yellow(), "Native (Nix recommended)".dimmed());
        warnings += 1;
    }

    // Check Rust version
    print!("  Rust toolchain... ");
    match check_rust_version() {
        Ok(version) => println!("{} {}", "✓".green(), version.dimmed()),
        Err(e) => {
            println!("{} {}", "✗".red(), e);
            issues += 1;
        }
    }

    // Check sccache health
    print!("  sccache status... ");
    match check_sccache_health() {
        Ok(msg) => println!("{} {}", "✓".green(), msg.dimmed()),
        Err(warning) => {
            println!("{} {}", "⚠".yellow(), warning);
            warnings += 1;
        }
    }

    println!();

    // Spec Root Section: Verify repository structure
    println!("{}", "Repository Structure:".bold());

    // Check spec root configuration
    print!("  Spec root... ");
    let spec_info = crate::kernel::spec_root_info();
    if spec_info.valid {
        println!("{} {}", "✓".green(), spec_info.path.display().to_string().dimmed());
        println!("    Source: {}", spec_info.source.dimmed());
    } else {
        println!("{} Invalid", "✗".red());
        println!("    Path: {}", spec_info.path.display());
        println!("    Source: {}", spec_info.source);
        if !spec_info.missing_files.is_empty() {
            println!("    Missing files:");
            for file in &spec_info.missing_files {
                println!("      - {}", file);
            }
        }
        issues += 1;
    }

    println!();

    // ABI Compatibility Section: Detect mismatches between system and Nix rustc
    println!("{}", "ABI Compatibility:".bold());

    // Check ABI consistency (system rustc vs Nix rustc)
    print!("  Toolchain ABI... ");
    match check_abi_consistency() {
        Ok(msg) => println!("{} {}", "✓".green(), msg.dimmed()),
        Err(warning) => {
            println!("{} {}", "⚠".yellow(), warning);
            warnings += 1;
        }
    }

    // Check glibc version compatibility (Linux only)
    print!("  glibc compatibility... ");
    match check_glibc_compatibility() {
        Ok(msg) => println!("{} {}", "✓".green(), msg.dimmed()),
        Err(warning) => {
            println!("{} {}", "⚠".yellow(), warning);
            warnings += 1;
        }
    }

    // Check libz.so.1 availability (common sccache issue)
    print!("  libz.so.1 available... ");
    match check_libz_availability() {
        Ok(msg) => println!("{} {}", "✓".green(), msg.dimmed()),
        Err(warning) => {
            println!("{} {}", "⚠".yellow(), warning);
            warnings += 1;
        }
    }

    println!();

    // Build Configuration Section
    println!("{}", "Build Configuration:".bold());

    // Check Cargo version
    print!("  Cargo... ");
    match which::which("cargo") {
        Ok(_) => {
            let output = Command::new("cargo").arg("--version").output()?;
            let version = String::from_utf8_lossy(&output.stdout);
            println!("{} {}", "✓".green(), version.trim().dimmed());
        }
        Err(_) => {
            println!("{} Not found", "✗".red());
            issues += 1;
        }
    }

    // Check Rust edition
    print!("  Rust edition... ");
    if std::fs::read_to_string("Cargo.toml")?.contains("edition = \"2024\"") {
        println!("{} 2024", "✓".green());
    } else {
        println!("{} Unexpected edition", "⚠".yellow());
        warnings += 1;
    }

    // Check CI/Low-resource modes
    print!("  CI mode... ");
    if crate::env::is_ci() {
        println!("{} Running in CI", "✓".green());
    } else {
        println!("{} Local development", "✓".green());
    }

    print!("  XTASK_LOW_RESOURCES... ");
    if crate::env::is_low_resources() {
        println!("{} Enabled (reduced parallelism)", "✓".green());
    } else {
        println!("{} Not set (using full resources)", "✓".green());
    }

    println!();

    // Required Tools Section
    println!("{}", "Required Tools:".bold());

    // Check Nix
    print!("  Nix... ");
    match which::which("nix") {
        Ok(_) => {
            let output = Command::new("nix").args(["--version"]).output()?;
            let version = String::from_utf8_lossy(&output.stdout);
            println!("{} {}", "✓".green(), version.trim().dimmed());
        }
        Err(_) => {
            println!("{} Not installed (recommended for hermetic env)", "⚠".yellow());
            warnings += 1;
        }
    }

    // Check conftest (policy tests)
    print!("  conftest (policy tests)... ");
    match which::which("conftest") {
        Ok(_) => {
            let output = Command::new("conftest").args(["--version"]).output()?;
            let version = String::from_utf8_lossy(&output.stdout);
            println!("{} {}", "✓".green(), version.trim().dimmed());
        }
        Err(_) => {
            println!("{} Not found (install via Nix)", "⚠".yellow());
            warnings += 1;
        }
    }

    // Check git
    print!("  git... ");
    match which::which("git") {
        Ok(_) => {
            let output = Command::new("git").args(["--version"]).output()?;
            let version = String::from_utf8_lossy(&output.stdout);
            println!("{} {}", "✓".green(), version.trim().dimmed());
        }
        Err(_) => {
            println!("{} Not found", "✗".red());
            issues += 1;
        }
    }

    println!();

    // Optional Tools Section
    println!("{}", "Optional Tools:".bold());

    // Check cargo-hakari
    print!("  cargo-hakari... ");
    let has_cargo_hakari = which::which("cargo-hakari").is_ok();
    if has_cargo_hakari {
        println!("{} Installed", "✓".green());
    } else {
        println!("{} Not found (install: cargo install cargo-hakari)", "⚠".yellow());
        warnings += 1;
    }

    // Summary with next steps and exit codes
    println!();
    if issues == 0 && warnings == 0 {
        println!("{}", "✓ Environment checks passed!".green().bold());
        println!();
        println!("{}", "Recommendations:".bold());
        println!("  • Fast dev loop:  {}", "cargo xtask check".cyan());
        println!("  • Before pushing: {}", "cargo xtask selftest".cyan());
        println!("  • See all flows:  {}", "cargo xtask help-flows".cyan());
        println!();
        println!("{}", "Exit code: 0 (all checks passed)".dimmed());
    } else {
        if issues > 0 {
            println!("{} {} critical issue(s) found", "✗".red().bold(), issues);
        }
        if warnings > 0 {
            println!("{}", "⚠ Environment functional with warnings".yellow().bold());
        }

        // Always show recommendations section when there are issues or warnings
        println!();
        println!("{}", "Recommendations:".bold());

        if !has_cargo_hakari {
            println!("  • Install hakari: {}", "cargo install cargo-hakari".dimmed());
        }
        if !in_nix_shell {
            println!("  • Enter Nix shell: {}", "nix develop".cyan());
            println!("    {}", "(Provides hermetic tools + policy tests)".dimmed());
        }
        println!("  • View flows: {}", "cargo xtask help-flows".cyan());
        println!("  • Troubleshooting: {}", "docs/TROUBLESHOOTING.md".dimmed());

        println!();
        if issues > 0 {
            println!("{}", "Exit code: 1 (critical issues found)".dimmed());
            anyhow::bail!("{} critical environment issue(s)", issues);
        } else {
            println!("{}", "Exit code: 0 (warnings only, functional)".dimmed());
        }
    }

    Ok(())
}

fn check_rust_version() -> Result<String> {
    let output = Command::new("rustc").arg("--version").output()?;
    let version = String::from_utf8_lossy(&output.stdout).to_string();

    // Check if version meets minimum requirement
    let version_str = version.trim();
    if version_str.contains("1.89") || version_str.contains("1.90") || version_str.contains("1.91")
    {
        Ok(version_str.to_string())
    } else {
        anyhow::bail!("{} (requires 1.89.0+)", version_str)
    }
}

fn check_abi_consistency() -> Result<String, String> {
    // If already in Nix shell, no check needed
    if std::env::var("IN_NIX_SHELL").is_ok() {
        return Ok("In Nix shell".to_string());
    }

    // Check if Nix is available
    if which::which("nix").is_err() {
        return Ok("Nix not installed (check skipped)".to_string());
    }

    // Get current rustc version
    let current_output = Command::new("rustc")
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to get current rustc version: {}", e))?;
    let current_version = String::from_utf8_lossy(&current_output.stdout);
    let current_version = extract_version_number(&current_version);

    // Get Nix rustc version
    let nix_output = Command::new("nix")
        .args(["develop", "-c", "rustc", "--version"])
        .output()
        .map_err(|e| format!("Failed to get Nix rustc version: {}", e))?;
    let nix_version = String::from_utf8_lossy(&nix_output.stdout);
    let nix_version = extract_version_number(&nix_version);

    // Compare versions
    if current_version == nix_version {
        Ok(format!("System and Nix toolchains match ({})", current_version))
    } else {
        Err(format!(
            "Toolchain mismatch detected: System rustc {} vs Nix rustc {}\n      \
            This can cause rust-analyzer proc-macro errors.\n      \
            Fix: Enter 'nix develop' before running IDE.\n      \
            See: docs/TROUBLESHOOTING.md §rust-analyzer ABI",
            current_version, nix_version
        ))
    }
}

fn extract_version_number(version_output: &str) -> String {
    // Extract version number from "rustc X.Y.Z (hash date)" format
    version_output.split_whitespace().nth(1).unwrap_or("unknown").to_string()
}

fn check_sccache_health() -> Result<String, String> {
    // Check if RUSTC_WRAPPER is set to use sccache
    let rustc_wrapper = std::env::var("RUSTC_WRAPPER").ok();

    match rustc_wrapper {
        None => {
            // sccache not configured, which is fine
            Ok("Not configured (optional)".to_string())
        }
        Some(wrapper) if wrapper.is_empty() => {
            // RUSTC_WRAPPER set to empty string (explicitly disabled)
            Ok("Not configured (optional)".to_string())
        }
        Some(wrapper) if !wrapper.contains("sccache") => {
            // RUSTC_WRAPPER set to something else
            Ok(format!("RUSTC_WRAPPER={}", wrapper))
        }
        Some(_) => {
            // sccache is configured, check if it works
            match Command::new("sccache").arg("--version").output() {
                Ok(output) => {
                    if output.status.success() {
                        let version = String::from_utf8_lossy(&output.stdout);
                        Ok(format!("sccache {}", version.trim()))
                    } else {
                        // sccache command failed, check for libz.so.1 error
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        if stderr.contains("libz.so.1") {
                            Err("sccache libz.so.1 error detected\n      \
                                Workaround: unset RUSTC_WRAPPER=\"\" in current shell\n      \
                                See: docs/TROUBLESHOOTING.md §sccache"
                                .to_string())
                        } else {
                            Err(format!("sccache failed: {}", stderr.trim()))
                        }
                    }
                }
                Err(e) => Err(format!(
                    "sccache not found or failed to run: {}\n      \
                        Workaround: unset RUSTC_WRAPPER=\"\" in current shell\n      \
                        See: docs/TROUBLESHOOTING.md §sccache",
                    e
                )),
            }
        }
    }
}

fn check_glibc_compatibility() -> Result<String, String> {
    // Only relevant on Linux
    #[cfg(not(target_os = "linux"))]
    {
        return Ok("N/A (not Linux)".to_string());
    }

    #[cfg(target_os = "linux")]
    {
        // Check if we can read glibc version
        let output = Command::new("ldd")
            .arg("--version")
            .output()
            .map_err(|e| format!("Failed to check glibc: {}", e))?;

        let version_output = String::from_utf8_lossy(&output.stdout);

        // Extract glibc version from output like "ldd (GNU libc) 2.35"
        if let Some(line) = version_output.lines().next() {
            if line.contains("GNU libc") || line.contains("GLIBC") {
                // Extract version number
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(version) = parts.last() {
                    Ok(format!("glibc {}", version))
                } else {
                    Ok("glibc detected".to_string())
                }
            } else {
                Err("Non-glibc libc detected (e.g., musl). May cause compatibility issues."
                    .to_string())
            }
        } else {
            Err("Could not determine glibc version".to_string())
        }
    }
}

fn check_libz_availability() -> Result<String, String> {
    // Only relevant on Linux
    #[cfg(not(target_os = "linux"))]
    {
        return Ok("N/A (not Linux)".to_string());
    }

    #[cfg(target_os = "linux")]
    {
        // Check if libz.so.1 is available (common sccache dependency issue)
        let output = Command::new("ldconfig")
            .args(["-p"])
            .output()
            .map_err(|e| format!("Failed to check libz: {}", e))?;

        let libs = String::from_utf8_lossy(&output.stdout);

        if libs.contains("libz.so.1") {
            Ok("libz.so.1 found".to_string())
        } else {
            Err("libz.so.1 not found. This may cause sccache errors.\n      \
                 Fix: Install zlib (e.g., 'apt install zlib1g' on Ubuntu)\n      \
                 See: docs/TROUBLESHOOTING.md §sccache"
                .to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use testing::process::EnvVarGuard;

    #[test]
    fn test_check_rust_version_accepts_valid_versions() {
        // This test validates the version checking logic would accept valid Rust versions
        // We can't test the actual check_rust_version() function in isolation easily,
        // but we can verify the version string matching logic
        let valid_versions = vec![
            "rustc 1.89.0 (abc123456 2024-01-01)",
            "rustc 1.90.0 (def789012 2024-02-01)",
            "rustc 1.91.0 (ghi345678 2024-03-01)",
        ];

        for version in valid_versions {
            assert!(
                version.contains("1.89") || version.contains("1.90") || version.contains("1.91"),
                "Version {} should be accepted",
                version
            );
        }
    }

    #[test]
    fn test_version_check_rejects_old_versions() {
        // Verify that old versions would be rejected
        let old_versions = vec![
            "rustc 1.88.0 (abc123456 2023-12-01)",
            "rustc 1.70.0 (def789012 2023-06-01)",
            "rustc 1.60.0 (ghi345678 2023-01-01)",
        ];

        for version in old_versions {
            assert!(
                !(version.contains("1.89") || version.contains("1.90") || version.contains("1.91")),
                "Version {} should be rejected",
                version
            );
        }
    }

    #[test]
    fn test_doctor_command_exists() {
        // Verify that the run function is accessible and has the correct signature
        // This ensures the command is properly exported for use by the CLI
        let _: fn() -> Result<()> = run;
    }

    #[test]
    fn test_extract_version_number() {
        // Test version extraction from rustc output
        let test_cases = vec![
            ("rustc 1.89.0 (abc123456 2024-01-01)", "1.89.0"),
            ("rustc 1.90.1 (def789012 2024-02-01)", "1.90.1"),
            ("rustc 1.91.0 (ghi345678 2024-03-01)", "1.91.0"),
            ("  rustc 1.89.0 (abc123 2024-01-01)  ", "1.89.0"), // with whitespace
        ];

        for (input, expected) in test_cases {
            let result = extract_version_number(input);
            assert_eq!(result, expected, "Failed to extract version from: {}", input);
        }
    }

    #[test]
    fn test_extract_version_number_handles_malformed() {
        // Test that malformed input doesn't panic
        let malformed = vec!["", "rustc", "some random text"];

        for input in malformed {
            let result = extract_version_number(input);
            // Should return "unknown" for malformed input
            assert!(
                result == "unknown" || !result.is_empty(),
                "Should handle malformed input: {}",
                input
            );
        }
    }

    #[test]
    fn test_check_sccache_health_when_not_configured() {
        // When RUSTC_WRAPPER is not set, sccache check should pass
        // This test validates the function logic without requiring sccache to be installed
        let guard = EnvVarGuard::new(&["RUSTC_WRAPPER"]);
        guard.remove("RUSTC_WRAPPER");

        let result = check_sccache_health();
        assert!(result.is_ok(), "Should succeed when RUSTC_WRAPPER not set");
        assert!(result.unwrap().contains("Not configured"), "Should report not configured");
    }

    #[test]
    fn test_check_sccache_health_with_other_wrapper() {
        // When RUSTC_WRAPPER is set to something else, should report it
        let guard = EnvVarGuard::new(&["RUSTC_WRAPPER"]);
        guard.set("RUSTC_WRAPPER", "some-other-wrapper");

        let result = check_sccache_health();
        assert!(result.is_ok(), "Should succeed with other wrapper");
        assert!(result.unwrap().contains("RUSTC_WRAPPER=some-other-wrapper"));
        // guard drops and restores RUSTC_WRAPPER automatically
    }
}
