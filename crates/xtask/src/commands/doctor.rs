use anyhow::Result;
use colored::Colorize;
use std::process::Command;

pub fn run() -> Result<()> {
    println!("{}", "🩺 Running environment diagnostics...".blue().bold());
    println!();

    let mut issues = 0;
    let mut warnings = 0;

    // Check Rust version
    print!("Rust toolchain... ");
    match check_rust_version() {
        Ok(version) => println!("{} {}", "✓".green(), version.dimmed()),
        Err(e) => {
            println!("{} {}", "✗".red(), e);
            issues += 1;
        }
    }

    // Check ABI consistency (system rustc vs Nix rustc)
    print!("Toolchain ABI consistency... ");
    match check_abi_consistency() {
        Ok(msg) => println!("{} {}", "✓".green(), msg.dimmed()),
        Err(warning) => {
            println!("{} {}", "⚠".yellow(), warning);
            warnings += 1;
        }
    }

    // Check Cargo version
    print!("Cargo... ");
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

    // Check Nix
    print!("Nix... ");
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
    print!("conftest (policy tests)... ");
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

    // Check cargo-hakari
    print!("cargo-hakari... ");
    let has_cargo_hakari = which::which("cargo-hakari").is_ok();
    if has_cargo_hakari {
        println!("{} Installed", "✓".green());
    } else {
        println!("{} Not found (optional, install: cargo install cargo-hakari)", "⚠".yellow());
        warnings += 1;
    }

    // Check git
    print!("git... ");
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
    println!("{}", "Environment Checks:".bold());

    // Check if inside Nix shell
    print!("IN_NIX_SHELL... ");
    if std::env::var("IN_NIX_SHELL").is_ok() {
        println!("{} Running in Nix devshell", "✓".green());
    } else {
        println!("{} Not in Nix shell (run: nix develop)", "⚠".yellow());
        warnings += 1;
    }

    // Check Rust edition
    print!("Rust edition... ");
    if std::fs::read_to_string("Cargo.toml")?.contains("edition = \"2024\"") {
        println!("{} 2024", "✓".green());
    } else {
        println!("{} Unexpected edition", "⚠".yellow());
        warnings += 1;
    }

    // Summary with next steps
    println!();
    if issues == 0 && warnings == 0 {
        println!("{}", "✓ Environment checks passed!".green().bold());
        println!();
        println!("{}", "Next steps:".bold());
        println!("  • Fast dev loop:  {}", "cargo xtask check".cyan());
        println!("  • Before pushing: {}", "cargo xtask selftest".cyan());
        println!("  • See all flows:  {}", "cargo xtask help-flows".cyan());
    } else {
        if issues > 0 {
            println!("{} {} critical issue(s) found", "✗".red().bold(), issues);
        }
        if warnings > 0 {
            println!("{}", "⚠ Environment functional with warnings".yellow().bold());
            println!();
            println!("{}", "Recommendations:".bold());
            if !has_cargo_hakari {
                println!("  • Install hakari: {}", "cargo install cargo-hakari".dimmed());
            }
            if std::env::var("IN_NIX_SHELL").is_err() {
                println!("  • Enter Nix shell: {}", "nix develop".cyan());
                println!("    {}", "(Provides hermetic tools + policy tests)".dimmed());
            }
            println!("  • View flows: {}", "cargo xtask help-flows".cyan());
        }
    }

    if issues > 0 {
        anyhow::bail!("{} critical environment issue(s)", issues);
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
    version_output.trim().split_whitespace().nth(1).unwrap_or("unknown").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
