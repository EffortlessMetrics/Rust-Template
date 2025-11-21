use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::process::Command;

mod commands;
mod devex;
mod docs_index;
mod validation;

/// Verbosity level for command output
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verbosity {
    Quiet,
    Normal,
    Verbose,
}

impl Verbosity {
    pub fn is_verbose(&self) -> bool {
        *self >= Verbosity::Verbose
    }

    pub fn is_quiet(&self) -> bool {
        *self == Verbosity::Quiet
    }

    pub fn is_normal(&self) -> bool {
        *self == Verbosity::Normal
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            Verbosity::Quiet => 0,
            Verbosity::Normal => 1,
            Verbosity::Verbose => 2,
        }
    }
}

/// xtask: Single entrypoint for all dev and CI operations
#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Development and CI orchestration tool", long_about = None)]
struct Cli {
    /// Increase verbosity (show detailed output)
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Decrease verbosity (suppress non-error output)
    #[arg(short, long, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate AC status report from acceptance tests
    AcStatus,
    /// Create new acceptance criterion
    AcNew {
        /// AC ID (e.g., AC-TPL-001)
        ac_id: String,
        /// AC description
        description: String,
        /// Story ID (e.g., US-TPL-001)
        #[arg(long)]
        story: String,
        /// Requirement ID (e.g., REQ-TPL-HEALTH)
        #[arg(long)]
        requirement: String,
    },
    /// Validate ADR references in spec ledger
    AdrCheck,
    /// Create new architecture decision record
    AdrNew {
        /// ADR title
        title: String,
    },
    /// Run all checks: fmt, clippy, tests
    Check,
    /// Run BDD acceptance tests
    Bdd,
    /// Generate LLM context bundle for a task
    Bundle {
        /// Task name from .llm/contextpack.yaml
        task: String,
    },
    /// Clean workspace (remove target/, generated docs, etc.)
    #[command(next_help_heading = "Infrastructure")]
    Clean,
    /// Run CI checks locally (doctor + selftest + audit + docs-check)
    #[command(next_help_heading = "Onboarding")]
    CiLocal,
    /// Deploy application to specified environment (dev, staging, prod)
    Deploy {
        /// Target environment: dev, staging, or prod
        #[arg(short, long, default_value = "dev")]
        env: String,
    },
    /// Create new design document with front-matter
    #[command(next_help_heading = "Design & Acceptance Criteria")]
    DesignNew {
        /// Document ID (e.g., DESIGN-TPL-HEALTH-001)
        id: String,
        /// Title for the design doc
        title: String,
        /// Linked requirements (repeatable)
        #[arg(long = "req")]
        requirements: Vec<String>,
        /// Linked ADRs (repeatable)
        #[arg(long = "adr")]
        adrs: Vec<String>,
        /// Optional owner label
        #[arg(long)]
        owner: Option<String>,
    },
    /// Run security and dependency audit
    Audit,
    /// Diagnose development environment setup
    Doctor,
    /// Verify documentation consistency
    DocsCheck,
    /// One-command developer bootstrap
    DevUp,
    /// Export dependency graph
    GraphExport {
        #[arg(long, value_enum, default_value = "json")]
        format: commands::graph_export::OutputFormat,
    },
    /// List tasks from specs/tasks.yaml
    TasksList,
    /// Format all code (Rust, YAML validation, etc.)
    FmtAll,
    /// Manage workspace dependencies with hakari
    Hakari,
    /// Run database migrations
    Migrate,
    /// Pin GitHub Actions to commit SHAs
    PinActions,
    /// Test Rego policies with conftest
    PolicyTest,
    /// Prepare release (bump versions, update changelog)
    ReleasePrepare {
        /// Version to release (e.g., 2.5.0)
        version: String,
    },
    /// Verify release readiness (selftest + audit + docs-check)
    ReleaseVerify,
    /// Generate local SBOM
    SbomLocal,
    /// Suggest next steps for a task
    SuggestNext(commands::suggest_next::SuggestNextArgs),
    /// Quick validation of template functionality
    Quickstart,
    /// Run full template self-test suite (check + bdd + ac-status + bundler + policies)
    Selftest,
    /// Show governance status dashboard
    #[command(next_help_heading = "Meta")]
    Status,
    /// Show flow-based command map
    #[command(next_help_heading = "Meta")]
    HelpFlows,
    /// Install git hooks for pre-commit governance
    #[command(next_help_heading = "Onboarding")]
    InstallHooks,
}

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
        eprintln!("{}", "⚠️  Running without Nix (hermetic environment unavailable)".yellow());
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

    match cli.command {
        Commands::AcStatus => commands::ac_status::run(commands::ac_status::AcStatusArgs {
            verbosity,
            ..Default::default()
        }),
        Commands::AcNew { ac_id, description, story, requirement } => {
            commands::ac_new::run(&ac_id, &description, &story, &requirement)
        }
        Commands::AdrCheck => commands::adr_check::run(commands::adr_check::AdrCheckArgs {
            verbosity,
            ..Default::default()
        }),
        Commands::AdrNew { title } => commands::adr_new::run(&title),
        Commands::Check => commands::check::run(),
        Commands::Bdd => commands::bdd::run(),
        Commands::Bundle { task } => commands::bundle::run(&task),
        Commands::Audit => commands::audit::run(),
        Commands::Clean => commands::clean::run(),
        Commands::CiLocal => commands::ci_local::run(),
        Commands::Deploy { env } => commands::deploy::run(&env),
        Commands::DesignNew { id, title, requirements, adrs, owner } => {
            commands::design_new::run(commands::design_new::DesignNewArgs {
                id,
                title,
                requirements,
                adrs,
                owner,
            })
        }
        Commands::Doctor => commands::doctor::run(),
        Commands::DocsCheck => commands::docs_check::run(),
        Commands::GraphExport { format } => {
            commands::graph_export::run(commands::graph_export::GraphExportArgs {
                format,
                check: false,
            })
        }
        Commands::TasksList => commands::tasks_list::run(),
        Commands::FmtAll => commands::fmt_all::run(),
        Commands::Hakari => commands::hakari::run(),
        Commands::Migrate => commands::migrate::run(),
        Commands::PinActions => commands::pin_actions::run(),
        Commands::PolicyTest => commands::policy_test::run().map_err(|e| anyhow::anyhow!("{}", e)),
        Commands::Quickstart => commands::quickstart::run(),
        Commands::ReleasePrepare { version } => commands::release_prepare::run(&version),
        Commands::ReleaseVerify => commands::release_verify::run(),
        Commands::SbomLocal => commands::sbom_local::run(),
        Commands::SuggestNext(args) => commands::suggest_next::run(args),
        Commands::Selftest => commands::selftest::run_with_verbosity(verbosity),
        Commands::Status => commands::status::run(),
        Commands::HelpFlows => commands::help_flows::run(),
        Commands::InstallHooks => commands::install_hooks::run(),
        Commands::DevUp => commands::dev_up::run(),
    }
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

    let output = cmd.output().with_context(|| format!("Failed to execute: {}", cmd_repr))?;

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
            if s.contains(' ') { format!("\"{}\"", s) } else { s.to_string() }
        })
        .collect();

    if args.is_empty() { program.to_string() } else { format!("{} {}", program, args.join(" ")) }
}

/// Returns list of all available xtask command names
/// Used by devex contract check to validate required commands exist
pub fn all_command_names() -> Vec<&'static str> {
    vec![
        "ac-status",
        "ac-new",
        "adr-check",
        "adr-new",
        "check",
        "bdd",
        "bundle",
        "clean",
        "ci-local",
        "deploy",
        "audit",
        "doctor",
        "docs-check",
        "fmt-all",
        "hakari",
        "install-hooks",
        "migrate",
        "pin-actions",
        "policy-test",
        "release-prepare",
        "release-verify",
        "sbom-local",
        "quickstart",
        "selftest",
        "status",
        "help-flows",
    ]
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
