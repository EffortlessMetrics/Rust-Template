use anyhow::{Result, anyhow};
use colored::Colorize;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn run() -> Result<()> {
    println!("{}", "🦀 Rust-as-Spec dev-up starting...".bold());

    // 1. Install pre-commit hooks if missing
    println!("→ Pre-commit hooks check...");
    let hook_path = Path::new(".git").join("hooks").join("pre-commit");
    if !hook_path.exists() {
        println!("  Installing pre-commit hooks...");
        crate::commands::install_hooks::run()?;
    } else {
        println!("  {}", "✓ Pre-commit hooks already installed".green());
    }

    // 2. Check Docker availability
    println!("→ Docker status check...");
    let docker_ok = Command::new("docker")
        .arg("ps")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !docker_ok {
        println!("  {}", "⚠ Docker status: not running".yellow());
        println!("  Please start Docker and run:");
        println!("  {}", "docker compose up -d".cyan());
    } else if Path::new("docker-compose.yaml").exists() {
        println!("  Docker status: running");
        println!("  Ensuring local services are running (docker compose up -d)...");
        let _ = Command::new("docker").arg("compose").arg("up").arg("-d").status();
    } else {
        println!("  Docker status: running (no docker-compose.yaml found)");
    }

    // 3. Light governance check (low-resource mode)
    println!("→ Running governance check...");
    let low_resource_mode = std::env::var("XTASK_LOW_RESOURCES").unwrap_or_default() == "1";
    if low_resource_mode {
        println!("  Running in low-resource mode");
    }

    // Use current executable to avoid file locking issues
    let current_exe = std::env::current_exe().unwrap_or_else(|_| "xtask".into());
    let mut cmd = Command::new(current_exe);
    cmd.env("XTASK_LOW_RESOURCES", "1").arg("check");
    let status = cmd.status()?;
    if !status.success() {
        println!(
            "{}",
            "❌ Governance check failed. Please fix issues above before continuing.".red()
        );
        return Err(anyhow!("governance check failed"));
    }

    // 4. Print next steps
    println!();
    println!("{}", "✅ dev-up complete.".green().bold());
    println!("Next steps:");
    println!("  1. {}", "cargo run -p app-http".cyan());
    println!("  2. Open {}", "http://localhost:8080/ui".cyan());

    Ok(())
}
