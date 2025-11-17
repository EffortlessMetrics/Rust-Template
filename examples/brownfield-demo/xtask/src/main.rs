//! Xtask binary for brownfield-demo
//!
//! This binary delegates to rust_iac_xtask_core for core functionality,
//! demonstrating how brownfield projects can adopt the Rust IaC tooling.

use anyhow::Result;
use clap::{Parser, Subcommand};
use rust_iac_xtask_core::{commands, InitMode};

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Brownfield demo xtask - Rust IaC automation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Rust IaC structure
    Init {
        /// Initialization mode: brownfield or greenfield
        #[arg(long, default_value = "brownfield")]
        mode: String,
    },
    /// Run self-test to verify project structure
    Selftest,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { mode } => {
            let init_mode: InitMode = mode.parse()?;
            commands::init::init(init_mode, None)?;
            println!("\nNext steps:");
            println!("  1. Review generated files:");
            println!("     - RUST_IAC.toml (project configuration)");
            println!("     - specs/spec_ledger.yaml (requirements tracking)");
            println!("     - policy/example.rego (policy templates)");
            println!("  2. Run: cargo run -p xtask -- selftest");
        }
        Commands::Selftest => {
            commands::selftest::selftest(None)?;
        }
    }

    Ok(())
}
