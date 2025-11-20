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
