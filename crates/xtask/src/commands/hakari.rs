use crate::run_cmd;
use anyhow::Result;
use colored::Colorize;
use std::process::Command;

pub fn run() -> Result<()> {
    // Check if cargo-hakari is installed
    if which::which("cargo-hakari").is_err() {
        eprintln!("{}", "Error: cargo-hakari is not installed.".red());
        eprintln!();
        eprintln!("Install it with:");
        eprintln!("  {}", "cargo install cargo-hakari".cyan());
        eprintln!();
        eprintln!("Or if using Nix:");
        eprintln!("  {}", "nix develop -c cargo install cargo-hakari".cyan());
        anyhow::bail!("cargo-hakari not found");
    }

    println!("{}", "Running cargo hakari generate...".blue());
    let mut cmd = Command::new("cargo");
    cmd.args(["hakari", "generate"]);
    run_cmd(&mut cmd)?;

    println!("{}", "Running cargo hakari verify...".blue());
    let mut cmd = Command::new("cargo");
    cmd.args(["hakari", "verify"]);
    run_cmd(&mut cmd)?;

    println!("{}", "✅ Hakari completed successfully!".green());
    Ok(())
}
