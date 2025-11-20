use crate::run_cmd;
use anyhow::Result;
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::process::Command;

fn project_root() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    Path::new(manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("Failed to find project root")
        .to_path_buf()
}

pub fn run() -> Result<()> {
    println!("{}", "🚀 Running database migrations for dev environment...".blue());

    let root = project_root();
    let crate_dir = root.join("crates/adapters-db-sqlx");

    if !crate_dir.exists() {
        anyhow::bail!("Database adapter crate not found at {}", crate_dir.display());
    }

    println!("{}", "📦 Running sqlx migrate run...".blue());

    let mut cmd = Command::new("cargo");
    cmd.current_dir(&crate_dir);
    cmd.args(["sqlx", "migrate", "run", "--source", "migrations"]);

    // Pass through DATABASE_URL if present, otherwise let sqlx complain or use .env
    if let Ok(url) = std::env::var("DATABASE_URL") {
        cmd.env("DATABASE_URL", url);
    }

    run_cmd(&mut cmd)?;

    println!("{}", "✅ Migrations completed successfully!".green());

    // Check for cargo-sqlx binary to run prepare
    if which::which("cargo-sqlx").is_ok() {
        println!("{}", "🔄 Regenerating sqlx types...".blue());
        let mut cmd = Command::new("cargo");
        cmd.current_dir(&crate_dir);
        cmd.args(["sqlx", "prepare", "--", "--lib"]);

        if let Err(e) = run_cmd(&mut cmd) {
            eprintln!("{} Failed to regenerate sqlx types: {}", "⚠".yellow(), e);
            // Don't fail the whole command for this optional step
        }
    }

    Ok(())
}
