//! CLI entry point for development tasks and CI orchestration.
//!
//! This is the `xtask` binary that provides a single entrypoint for all dev and CI operations.
//! It follows the `cargo-xtask` pattern: a binary crate in the workspace that acts as a
//! task runner for development workflows.
//!
//! The entry point is intentionally small: CLI definitions, dispatch, documentation help, and
//! process helpers live in focused sibling modules so each unit has one reason to change.

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

mod cli;
mod commands;
mod contracts;
mod devex;
mod dispatch;
mod docs_help;
mod docs_index;
pub mod env;
pub mod kernel;
mod process;
mod validation;

pub use cli::{Verbosity, all_command_names};
pub use process::{cargo_cmd, run_cmd};

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    // Universal Nix wrapper - ALL commands run in hermetic environment when available.
    // This aligns with ADR-0002 (Nix-first development) and ensures CI/local parity.
    if process::should_wrap_with_nix() {
        process::exec_via_nix()?;
        unreachable!("process is replaced by nix develop");
    }

    // Warn when Nix is missing (gentle reminder, not an error).
    if !cli.quiet && std::env::var("IN_NIX_SHELL").is_err() {
        eprintln!("{}", "[WARN] Running without Nix (hermetic environment unavailable)".yellow());
        eprintln!("{}", "   Install Nix for full CI parity: https://nixos.org/download".dimmed());
        eprintln!();
    }

    let verbosity = cli.verbosity();

    if cli.help_docs {
        docs_help::show_command_docs(&cli.command)?;
        return Ok(());
    }

    dispatch::run(cli.command, verbosity)
}
