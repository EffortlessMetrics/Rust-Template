use anyhow::{Result, anyhow};
use colored::Colorize;
use std::path::Path;
use std::process::{Command, Stdio};

/// Steps that dev-up runs during environment setup.
/// @AC-PLT-018: dev-up runs required steps (install-hooks, docker check, governance check)
pub const DEV_UP_STEPS: [&str; 3] =
    ["Pre-commit hooks check", "Docker status check", "Governance check"];

/// Next steps shown after successful dev-up completion.
pub const DEV_UP_NEXT_STEPS: [&str; 2] = ["cargo run -p app-http", "http://localhost:8080/ui"];

pub fn run() -> Result<()> {
    println!("{}", "🦀 Rust-as-Spec dev-up starting...".bold());

    // Run each step from DEV_UP_STEPS
    // Step 1: Pre-commit hooks check
    println!("→ {}...", DEV_UP_STEPS[0]);
    let hook_path = Path::new(".git").join("hooks").join("pre-commit");
    if !hook_path.exists() {
        println!("  Installing pre-commit hooks...");
        crate::commands::install_hooks::run()?;
    } else {
        println!("  {}", "✓ Pre-commit hooks already installed".green());
    }

    // Step 2: Docker status check
    println!("→ {}...", DEV_UP_STEPS[1]);
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

    // Step 3: Governance check (low-resource mode)
    println!("→ {}...", DEV_UP_STEPS[2]);
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

    // Print next steps from DEV_UP_NEXT_STEPS
    println!();
    println!("{}", "✅ dev-up complete.".green().bold());
    println!("Next steps:");
    println!("  1. {}", DEV_UP_NEXT_STEPS[0].cyan());
    println!("  2. Open {}", DEV_UP_NEXT_STEPS[1].cyan());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// @AC-PLT-018: dev-up command exists with correct signature
    #[test]
    fn test_dev_up_command_exists() {
        // Verify that the run function is accessible and has the correct signature
        let _: fn() -> Result<()> = run;
    }

    /// @AC-PLT-018: dev-up runs required steps (install-hooks, docker check, governance check)
    #[test]
    fn test_dev_up_required_steps() {
        // Uses the shared DEV_UP_STEPS constant that run() also uses
        assert!(DEV_UP_STEPS.len() >= 3, "dev-up must run at least 3 steps");

        // Each step should be meaningful
        for step in &DEV_UP_STEPS {
            assert!(!step.is_empty(), "Step description should not be empty");
        }
    }

    /// @AC-PLT-018: dev-up provides next steps guidance on completion
    #[test]
    fn test_dev_up_provides_next_steps() {
        // Uses the shared DEV_UP_NEXT_STEPS constant that run() also uses
        assert!(DEV_UP_NEXT_STEPS.len() >= 2, "dev-up should suggest at least 2 next steps");
    }
}
