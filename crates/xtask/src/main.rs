use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
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
    /// Generate AC status report from acceptance tests
    AcStatus,
    /// Run all checks: fmt, clippy, tests
    Check,
    /// Run BDD acceptance tests
    Bdd,
    /// Generate LLM context bundle for a task
    Bundle {
        /// Task name from .llm/contextpack.yaml
        task: String,
    },
    /// Test Rego policies with conftest
    PolicyTest,
    /// Quick validation of template functionality
    Quickstart,
    /// Run full template self-test suite (check + bdd + ac-status + bundler + policies)
    Selftest,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::AcStatus => {
            commands::ac_status::run(commands::ac_status::AcStatusArgs::default())
        }
        Commands::Check => commands::check::run(),
        Commands::Bdd => commands::bdd::run(),
        Commands::Bundle { task } => commands::bundle::run(&task),
        Commands::PolicyTest => commands::policy_test::run(),
        Commands::Quickstart => commands::quickstart::run(),
        Commands::Selftest => commands::selftest::run(),
    }
}

/// Helper to run a command and propagate failures
///
/// Captures stdout/stderr and displays them on failure for better debugging.
pub fn run_cmd(cmd: &mut Command) -> Result<()> {
    let cmd_repr = format_command(cmd);

    let output = cmd
        .output()
        .with_context(|| format!("Failed to execute: {}", cmd_repr))?;

    if !output.status.success() {
        eprintln!("\n{} Command failed: {}", "✗".bright_red(), cmd_repr);

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
            if s.contains(' ') {
                format!("\"{}\"", s)
            } else {
                s.to_string()
            }
        })
        .collect();

    if args.is_empty() {
        program.to_string()
    } else {
        format!("{} {}", program, args.join(" "))
    }
}
