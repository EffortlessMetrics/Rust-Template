//! Runtime process helpers shared by xtask commands.

use anyhow::{Context, Result};
use colored::Colorize;
use std::process::Command;
use std::thread;
use std::time::Duration;

/// Check if we should wrap execution with Nix
pub fn should_wrap_with_nix() -> bool {
    // Don't re-wrap if already inside Nix shell
    if std::env::var("IN_NIX_SHELL").is_ok() {
        return false;
    }

    // Check if nix command is available
    which::which("nix").is_ok()
}

/// Execute xtask via Nix wrapper, forwarding all arguments
pub fn exec_via_nix() -> Result<()> {
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
