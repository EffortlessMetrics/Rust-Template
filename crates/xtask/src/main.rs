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

use anyhow::{Context, Result};
use colored::Colorize;
use std::process::Command;
use std::thread;
use std::time::Duration;

use clap::Parser;
use cli::Cli;
pub use cli::{Verbosity, all_command_names};

mod cli;
mod command_dispatch;
mod commands;
mod contracts;
mod devex;
mod docs_help;
mod docs_index;
pub mod env;
pub mod kernel;
mod validation;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Universal Nix wrapper - ALL commands run in hermetic environment when available
    // This aligns with ADR-0002 (Nix-first development) and ensures perfect CI/local parity
    if should_wrap_with_nix() {
        // Silent when Nix is present - it's the expected default
        exec_via_nix()?;
        unreachable!(); // Process will be replaced by nix develop
    }

    // Warn when Nix is missing (gentle reminder, not an error)
    if !cli.quiet && std::env::var("IN_NIX_SHELL").is_err() {
        eprintln!("{}", "[WARN] Running without Nix (hermetic environment unavailable)".yellow());
        eprintln!("{}", "   Install Nix for full CI parity: https://nixos.org/download".dimmed());
        eprintln!();
    }

    // Determine verbosity level
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

    command_dispatch::run(cli.command, verbosity)
}

/// Check if we should wrap execution with Nix
fn should_wrap_with_nix() -> bool {
    // Don't re-wrap if already inside Nix shell
    if std::env::var("IN_NIX_SHELL").is_ok() {
        return false;
    }

    // Check if nix command is available
    which::which("nix").is_ok()
}

/// Execute xtask via Nix wrapper, forwarding all arguments
fn exec_via_nix() -> Result<()> {
    let mut cmd = Command::new("nix");
    cmd.args(["develop", "-c", "cargo", "run", "-p", "xtask", "--"]);

    // Forward ALL arguments after the program name
    cmd.args(std::env::args().skip(1));

    // Execute and replace current process
    let status =
        cmd.status().map_err(|e| anyhow::anyhow!("Failed to execute nix develop: {}", e))?;

    std::process::exit(status.code().unwrap_or(1));
}

/// Helper to run a command and propagate failures
///
/// Captures stdout/stderr and displays them on failure for better debugging.
pub fn run_cmd(cmd: &mut Command) -> Result<()> {
    let cmd_repr = format_command(cmd);

    // Some environments (CI, constrained containers) intermittently refuse to spawn new
    // processes with `Os { kind: WouldBlock }`. Retry a few times and drop RUSTC_WRAPPER to
    // avoid sccache overhead in those cases.
    let mut attempts = 0;
    let output = loop {
        let result = cmd.output();
        match result {
            Ok(out) => break out,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock && attempts < 3 => {
                cmd.env_remove("RUSTC_WRAPPER");
                attempts += 1;
                thread::sleep(Duration::from_millis(200 * attempts));
                continue;
            }
            Err(e) => {
                return Err(e).with_context(|| format!("Failed to execute: {}", cmd_repr));
            }
        }
    };

    if !output.status.success() {
        eprintln!("\n{} Command failed: {}", "[FAIL]".bright_red(), cmd_repr);

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !stdout.trim().is_empty() {
            eprintln!("\n--- stdout ---");
            eprintln!("{}", stdout);
        }

        if !stderr.trim().is_empty() {
            eprintln!("\n--- stderr ---");
            eprintln!("{}", stderr);
        }

        anyhow::bail!("Command failed with exit code: {:?}", output.status.code());
    }

    Ok(())
}

/// Format a Command for display
fn format_command(cmd: &Command) -> String {
    use std::ffi::OsStr;

    let program = cmd.get_program().to_string_lossy();
    let args: Vec<String> = cmd
        .get_args()
        .map(OsStr::to_string_lossy)
        .map(|s| {
            // Quote arguments with spaces
            if s.contains(' ') { format!("\"{}\"", s) } else { s.to_string() }
        })
        .collect();

    if args.is_empty() { program.to_string() } else { format!("{} {}", program, args.join(" ")) }
}

/// Create a cargo command with optional low-resource overrides
///
/// If XTASK_LOW_RESOURCES is set:
/// - CARGO_BUILD_JOBS=1
/// - RUSTC_WRAPPER is removed (disabling sccache)
pub fn cargo_cmd(subcommand: &str, args: &[&str]) -> Command {
    let mut cmd = Command::new("cargo");
    cmd.arg(subcommand).args(args);

    if std::env::var_os("XTASK_LOW_RESOURCES").is_some() {
        cmd.env("CARGO_BUILD_JOBS", "1");
        cmd.env_remove("RUSTC_WRAPPER");
    }

    cmd
}
