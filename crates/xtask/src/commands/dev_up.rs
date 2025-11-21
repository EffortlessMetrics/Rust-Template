use anyhow::{Result, anyhow};
use colored::Colorize;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn run() -> Result<()> {
    println!("{}", "🦀 Rust-as-Spec dev-up starting...".bold());

    // 1. Install pre-commit hooks if missing
    let hook_path = Path::new(".git").join("hooks").join("pre-commit");
    if !hook_path.exists() {
        println!("→ Installing pre-commit hooks...");
        crate::commands::install_hooks::run()?;
    } else {
        println!("{}", "✓ Pre-commit hooks already installed".green());
    }

    // 2. Check Docker availability
    let docker_ok = Command::new("docker")
        .arg("ps")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !docker_ok {
        println!("{}", "⚠ Docker does not appear to be running.".yellow());
        println!("   Please start Docker and run:");
        println!("   {}", "docker compose up -d".cyan());
    } else if Path::new("docker-compose.yaml").exists() {
        println!("→ Ensuring local services are running (docker compose up -d)...");
        let _ = Command::new("docker").arg("compose").arg("up").arg("-d").status();
    } else {
        println!("ℹ No docker-compose.yaml found; skipping service startup.");
    }

    // 3. Light governance check (low-resource mode)
    println!("→ Running governance check (low-resource mode)...");
    let mut cmd = Command::new("cargo");
    cmd.env("XTASK_LOW_RESOURCES", "1").arg("run").arg("-p").arg("xtask").arg("--").arg("check");
    let status = cmd.status()?;
    if !status.success() {
        println!("{}", "❌ xtask check failed. Please fix issues above before continuing.".red());
        return Err(anyhow!("xtask check failed"));
    }

    // 4. Print next steps
    println!();
    println!("{}", "✅ dev-up complete.".green().bold());
    println!("Next steps:");
    println!("  1. {}", "cargo run -p app-http".cyan());
    println!("  2. Open {}", "http://localhost:3000/ui".cyan());

    Ok(())
}
