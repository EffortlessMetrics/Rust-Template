use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::process::Command;

mod commands;

/// xtask: Single entrypoint for all dev and CI operations
#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Development and CI orchestration tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all checks: fmt, clippy, tests
    Check,
    /// Run BDD acceptance tests
    Bdd,
    /// Generate LLM context bundle for a task
    Bundle {
        /// Task name from .llm/contextpack.yaml
        task: String,
    },
    /// Quick validation of template functionality
    Quickstart,
    /// Run full template self-test suite (check + bdd + ac-status + bundler + policies)
    Selftest,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check => commands::check::run(),
        Commands::Bdd => commands::bdd::run(),
        Commands::Bundle { task } => commands::bundle::run(&task),
        Commands::Quickstart => commands::quickstart::run(),
        Commands::Selftest => commands::selftest::run(),
    }
}

/// Helper to run a command and propagate failures
pub fn run_cmd(cmd: &mut Command) -> Result<()> {
    let status = cmd.status().with_context(|| format!("Failed to spawn command: {:?}", cmd))?;

    if !status.success() {
        anyhow::bail!("Command {:?} failed with status {:?}", cmd, status);
    }

    Ok(())
}
