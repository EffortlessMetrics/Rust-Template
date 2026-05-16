//! CLI entry point for development tasks and CI orchestration.
//!
//! This is the `xtask` binary that provides a single entrypoint for all dev and CI operations.
//! It follows the `cargo-xtask` pattern: a binary crate in the workspace that acts as a
//! task runner for development workflows.
//!
//! # Command categories
//!
//! - **Onboarding**: Environment setup and validation (`doctor`, `dev-up`, `install-hooks`)
//! - **Validation Gates**: Checks and tests (`selftest`, `check`, `precommit`, `bdd`)
//! - **Acceptance Criteria**: AC management and testing (`ac-status`, `ac-new`, `test-ac`)
//! - **Design & Documentation**: ADRs, design docs, spellcheck (`adr-new`, `docs-check`)
//! - **Governance Artifacts**: Skills, agents, friction log, questions (`skills-lint`, `friction-list`)
//! - **Tasks & Hints**: Work tracking and agent guidance (`tasks-list`, `suggest-next`)
//! - **Releases**: Version management and release process (`release-prepare`, `release-bundle`)
//! - **Security & Policy**: Audits and policy testing (`audit`, `policy-test`, `coverage`)
//! - **LLM/Agent Support**: Context bundles and workflows (`bundle`, `help-flows`)
//! - **Infrastructure**: Build, cleanup, and utilities (`clean`, `graph-export`, `migrate`)
//!
//! # Usage
//!
//! ```bash
//! cargo xtask <command> [options]
//! ```
//!
//! For a list of all commands, run:
//!
//! ```bash
//! cargo xtask --help
//! ```
//!
//! The tool automatically wraps execution in `nix develop` when Nix is available,
//! ensuring hermetic builds and perfect CI/local parity per ADR-0002.

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
mod runtime;
mod validation;

pub use cli::Verbosity;
pub use docs_help::all_command_names;
pub use runtime::{cargo_cmd, run_cmd};

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    // Universal Nix wrapper - ALL commands run in hermetic environment when available.
    // This aligns with ADR-0002 (Nix-first development) and ensures perfect CI/local parity.
    if runtime::should_wrap_with_nix() {
        // Silent when Nix is present - it's the expected default.
        runtime::exec_via_nix()?;
        unreachable!(); // Process will be replaced by nix develop.
    }

    // Warn when Nix is missing (gentle reminder, not an error).
    if !cli.quiet && std::env::var("IN_NIX_SHELL").is_err() {
        eprintln!("{}", "[WARN] Running without Nix (hermetic environment unavailable)".yellow());
        eprintln!("{}", "   Install Nix for full CI parity: https://nixos.org/download".dimmed());
        eprintln!();
    }

    // Determine verbosity level.
    let verbosity = if cli.quiet {
        Verbosity::Quiet
    } else if cli.verbose {
        Verbosity::Verbose
    } else {
        Verbosity::Normal
    };

    if cli.help_docs {
        docs_help::show_command_docs(&cli.command)?;
        return Ok(());
    }

    dispatch::dispatch(cli.command, verbosity)
}
