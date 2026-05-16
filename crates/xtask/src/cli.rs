//! Clap definitions for the xtask command-line interface.

use crate::commands;
use clap::{Parser, Subcommand};

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
pub struct Cli {
    /// Increase verbosity (show detailed output)
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Decrease verbosity (suppress non-error output)
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Show relevant documentation for the command
    #[arg(long, global = true)]
    pub help_docs: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    // ============================================================================
    // ONBOARDING (Getting started & environment setup)
    // ============================================================================
    /// Diagnose development environment setup
    #[command(next_help_heading = "🚀 Onboarding")]
    Doctor,

    /// One-command developer bootstrap (nix setup + hooks + first checks)
    #[command(next_help_heading = "🚀 Onboarding")]
    DevUp,

    /// Install git hooks for pre-commit governance
    #[command(next_help_heading = "🚀 Onboarding")]
    InstallHooks,

    /// Run CI checks locally (doctor + selftest + audit + docs-check)
    #[command(next_help_heading = "🚀 Onboarding")]
    CiLocal,

    // ============================================================================
    // VALIDATION GATES (Selftest, check, smoke tests)
    // ============================================================================
    /// Quick kernel smoke test – validate template baseline (docs + core tests)
    #[command(next_help_heading = "✅ Validation Gates")]
    KernelSmoke,

    /// Show aggregated kernel health (specs, docs, governance)
    #[command(next_help_heading = "✅ Validation Gates")]
    KernelStatus,

    /// Run full template self-test suite (8-step governance validation – run before PR)
    #[command(next_help_heading = "✅ Validation Gates")]
    Selftest,

    /// Quick validation of template functionality (abbreviated selftest for smoke tests)
    #[command(next_help_heading = "✅ Validation Gates")]
    Quickstart,

    /// Run all checks: fmt, clippy, unit tests (fast, local feedback)
    #[command(next_help_heading = "✅ Validation Gates")]
    Check,

    /// Run pre-commit guardrail checks (fmt/clippy/tests/docs/spellcheck)
    ///
    /// Modes:
    ///   - full (default): All checks regardless of what changed
    ///   - fast: Change-aware routing (only check affected categories)
    ///
    /// The git hook uses --mode fast --staged-only for speed.
    #[command(next_help_heading = "✅ Validation Gates")]
    Precommit {
        /// Precommit mode: full (all checks) or fast (change-aware routing)
        #[arg(long, default_value = "full", value_parser = ["fast", "full"])]
        mode: String,
        /// Only consider staged changes (for git hooks)
        #[arg(long)]
        staged_only: bool,
    },

    /// Run tests affected by git changes (selective testing for faster iteration)
    #[command(next_help_heading = "✅ Validation Gates")]
    TestChanged {
        /// Git ref to compare against (default: origin/main)
        #[arg(long, default_value = "origin/main")]
        base: String,
        /// Plan-only mode: compute test plan without executing it (env: XTASK_TEST_CHANGED_PLAN_ONLY)
        #[arg(long, action = clap::ArgAction::SetTrue)]
        plan_only: bool,
    },

    /// Check for breaking changes in contract crate public APIs
    #[command(next_help_heading = "✅ Validation Gates")]
    CheckApiDiff {
        /// Path to ADR approving the change (optional)
        #[arg(long)]
        adr: Option<String>,
    },

    /// Check for breaking changes in OpenAPI spec (/platform/* endpoints)
    #[command(next_help_heading = "✅ Validation Gates")]
    CheckOpenapiDiff,

    /// Check for breaking changes in CLI JSON output schemas
    #[command(next_help_heading = "✅ Validation Gates")]
    CheckJsonSchemas {
        /// Generate golden snapshots instead of checking
        #[arg(long)]
        generate: bool,
    },

    /// Check dependency layering rules for contract and foundation crates
    #[command(next_help_heading = "✅ Validation Gates")]
    CheckLayering,

    // ============================================================================
    // ACCEPTANCE CRITERIA (AC management & testing)
    // ============================================================================
    /// Generate AC status report from acceptance tests
    ///
    /// Two modes of operation:
    ///   - Write mode (default): Generates/updates docs/feature_status.md
    ///   - Check mode (--check): Verifies file is up-to-date without writing
    ///
    /// The --summary and --json flags produce stdout output only, so --check
    /// is ignored when combined with these flags.
    ///
    /// See also: /platform/coverage API for programmatic access
    #[command(next_help_heading = "📋 Acceptance Criteria")]
    AcStatus {
        /// Print concise summary instead of generating full markdown file.
        /// Note: When used, --check flag has no effect (no file operation).
        #[arg(long)]
        summary: bool,
        /// Output in JSON format.
        /// Note: When used, --check flag has no effect (no file operation).
        #[arg(long)]
        json: bool,
        /// Filter to a specific AC ID (e.g., AC-KERN-001)
        #[arg(long)]
        ac: Option<String>,
        /// Check mode: verify existing file matches computed state without writing.
        /// Used by CI or manual verification. Fails if docs/feature_status.md would differ.
        /// Has no effect when combined with --summary or --json.
        #[arg(long)]
        check: bool,
        /// Require coverage data to exist before computing status.
        /// Fails with helpful guidance if coverage.jsonl is missing or empty.
        /// Prevents churn from regenerating feature_status.md with incomplete data.
        #[arg(long)]
        require_coverage: bool,
    },

    /// Create new acceptance criterion
    #[command(next_help_heading = "📋 Acceptance Criteria")]
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

    /// Show AC coverage grouped by requirement (which ACs need BDD scenarios)
    #[command(next_help_heading = "📋 Acceptance Criteria")]
    AcCoverage {
        /// Show only ACs with Unknown status (coverage backlog)
        #[arg(long)]
        todo: bool,
        /// When used with --todo, filter to only kernel ACs (must_have_ac=true)
        #[arg(long, requires = "todo")]
        must_have: bool,
    },

    /// Suggest BDD scenarios for a given AC
    #[command(next_help_heading = "📋 Acceptance Criteria")]
    AcSuggestScenarios {
        /// AC ID to suggest scenarios for (e.g., AC-TPL-001)
        ac_id: String,
    },

    /// Show all tests mapped to a specific AC
    #[command(next_help_heading = "📋 Acceptance Criteria")]
    AcTests {
        /// AC ID to show tests for (e.g., AC-TPL-001)
        ac_id: String,
    },

    /// Run tests for a specific acceptance criterion
    #[command(next_help_heading = "📋 Acceptance Criteria")]
    TestAc {
        /// AC ID to test (e.g., AC-TPL-001)
        ac_id: String,
    },

    /// Run BDD acceptance tests
    #[command(next_help_heading = "📋 Acceptance Criteria")]
    Bdd,

    /// Generate human-readable AC governance report (consumes ac-status --json)
    #[command(next_help_heading = "📋 Acceptance Criteria")]
    AcReport {
        /// Only show must_have_ac=true ACs
        #[arg(long)]
        must_have: bool,
        /// Filter by status (pass, fail, unknown)
        #[arg(long)]
        status: Option<String>,
        /// Group by story instead of requirement
        #[arg(long)]
        by_story: bool,
        /// Output format (text, markdown, html, json)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Analyze AC coverage trends from CI snapshots
    #[command(next_help_heading = "📋 Acceptance Criteria")]
    AcHistory {
        /// Directory containing ac-status JSON snapshots
        #[arg(long, default_value = "artifacts/ac-status")]
        dir: String,
        /// Output format (text, markdown, csv, json)
        #[arg(long, default_value = "text")]
        format: String,
        /// Only show must_have_ac=true ACs
        #[arg(long)]
        must_have: bool,
    },

    /// Check if AC coverage meets SLO thresholds
    #[command(next_help_heading = "📋 Acceptance Criteria")]
    AcSlo {
        /// Directory containing ac-status JSON snapshots
        #[arg(long, default_value = "artifacts/ac-status")]
        dir: String,
        /// Minimum required coverage percentage
        #[arg(long, default_value = "80.0")]
        min_coverage: f64,
        /// Maximum allowed kernel blockers
        #[arg(long, default_value = "0")]
        max_blockers: usize,
        /// Maximum allowed unknown status ACs (no limit if not specified)
        #[arg(long)]
        max_unknown: Option<usize>,
        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Verify all kernel ACs have test mappings in spec_ledger.yaml
    ///
    /// Kernel ACs (must_have_ac=true) should have at least one test mapping
    /// (unit, bdd, or integration) to ensure they're not forgotten.
    /// This is a guardrail per ADR-0024.
    #[command(next_help_heading = "📋 Acceptance Criteria")]
    AcEnsureKernelMapped {
        /// Fail if any kernel ACs are unmapped (default: warn only)
        #[arg(long)]
        strict: bool,
    },

    /// Lint spec_ledger.yaml for structural integrity
    ///
    /// Validates invariants: no duplicate IDs, naming conventions,
    /// kernel ACs have test mappings, test types are known, etc.
    #[command(next_help_heading = "📋 Acceptance Criteria")]
    AcLint {
        /// Fail on warnings (default: only errors fail)
        #[arg(long)]
        strict: bool,
        /// Check that test file references exist on disk
        #[arg(long)]
        check_files: bool,
    },

    // ============================================================================
    // DESIGN & DOCUMENTATION (Docs, ADRs, design docs)
    // ============================================================================
    /// Create new architecture decision record
    #[command(next_help_heading = "📚 Design & Documentation")]
    AdrNew {
        /// ADR title
        title: String,
    },

    /// Validate ADR references in spec ledger
    #[command(next_help_heading = "📚 Design & Documentation")]
    AdrCheck,

    /// Create new design document with front-matter
    #[command(next_help_heading = "📚 Design & Documentation")]
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

    /// Verify documentation consistency
    #[command(next_help_heading = "📚 Design & Documentation")]
    DocsCheck,

    /// Sync front-matter in design docs from doc_index.yaml
    #[command(next_help_heading = "📚 Design & Documentation")]
    DocsFrontmatterSync {
        /// Apply changes instead of just checking
        #[arg(long)]
        fix: bool,
    },

    /// Run spellcheck across docs/specs
    #[command(next_help_heading = "📚 Design & Documentation")]
    Spellcheck,

    /// Check that governed facts in docs match their sources (selftest steps, kernel AC count, etc.)
    #[command(next_help_heading = "📚 Design & Documentation")]
    ContractsCheck,

    /// Synchronize governed facts from code/specs to docs
    #[command(next_help_heading = "📚 Design & Documentation")]
    ContractsFmt,

    /// Validate UI contract (specs/ui_contract.yaml) and DOM anchors
    #[command(next_help_heading = "📚 Design & Documentation")]
    UiContractCheck,

    // ============================================================================
    // GOVERNANCE ARTIFACTS (Friction log, questions, forks, skills)
    // ============================================================================
    /// List friction log entries (track process/tooling issues)
    #[command(next_help_heading = "🏛️ Governance Artifacts")]
    FrictionList {
        /// Filter by status (open, investigating, in_progress, resolved, wont_fix)
        #[arg(long)]
        status: Option<String>,
        /// Filter by severity (low, medium, high, critical)
        #[arg(long)]
        severity: Option<String>,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Create a new friction log entry
    #[command(next_help_heading = "🏛️ Governance Artifacts")]
    FrictionNew {
        /// Category (tooling, process, documentation, devex, ci_cd, platform, api, testing, governance, other)
        #[arg(long)]
        category: String,
        /// Severity level (low, medium, high, critical)
        #[arg(long)]
        severity: String,
        /// Brief summary of the friction point
        #[arg(long)]
        summary: String,
        /// Detailed description (optional, defaults to summary)
        #[arg(long)]
        description: Option<String>,
        /// Flow where friction occurred (optional)
        #[arg(long)]
        flow: Option<String>,
        /// Phase within flow where friction occurred (optional)
        #[arg(long)]
        phase: Option<String>,
        /// Who discovered this friction (optional, defaults to "human")
        #[arg(long)]
        discovered_by: Option<String>,
        /// REQ-*/AC-* IDs this friction entry is about (repeatable)
        #[arg(long = "refs")]
        refs: Vec<String>,
    },

    /// Resolve a friction entry (mark as resolved with resolution details)
    #[command(next_help_heading = "🏛️ Governance Artifacts")]
    FrictionResolve {
        /// Friction ID to resolve (e.g., FRICTION-TOOL-001)
        #[arg(long)]
        id: String,
        /// Who resolved it (e.g., "agent", "human", username)
        #[arg(long)]
        resolved_by: String,
        /// Description of how it was fixed
        #[arg(long)]
        fix_description: Option<String>,
        /// PR links (repeatable)
        #[arg(long = "pr")]
        pr_links: Vec<String>,
        /// Verification notes
        #[arg(long)]
        verification: Option<String>,
        /// New status (defaults to "resolved", can be "wont_fix")
        #[arg(long, default_value = "resolved")]
        status: String,
    },

    /// Create GitHub issue from friction entry
    #[command(next_help_heading = "🏛️ Governance Artifacts")]
    FrictionGhCreate {
        /// Friction ID to create issue from (e.g., FRICTION-TOOL-001)
        friction_id: String,
        /// Additional labels (comma-separated)
        #[arg(long)]
        labels: Option<String>,
        /// Preview issue without creating (dry run)
        #[arg(long)]
        dry_run: bool,
        /// Open issue in browser after creation
        #[arg(long)]
        open: bool,
    },

    /// Link existing GitHub issue to friction entry
    #[command(next_help_heading = "🏛️ Governance Artifacts")]
    FrictionGhLink {
        /// Friction ID to link
        friction_id: String,
        /// GitHub issue number (e.g., 123 or #123)
        issue_number: String,
    },

    /// List questions from questions/ directory
    #[command(next_help_heading = "🏛️ Governance Artifacts")]
    QuestionsList {
        /// Filter by status (open, answered, resolved, obsolete)
        #[arg(long)]
        status: Option<String>,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Create a new question artifact (capture ambiguity during flows)
    #[command(next_help_heading = "🏛️ Governance Artifacts")]
    QuestionNew {
        /// Question category/component (e.g., TPL, BUNDLE, SUGGEST)
        #[arg(long)]
        category: String,
        /// Brief summary of the question
        #[arg(long)]
        summary: String,
        /// Flow that generated this question
        #[arg(long)]
        flow: String,
        /// Phase within the flow
        #[arg(long)]
        phase: String,
        /// Detailed description of the ambiguity
        #[arg(long)]
        description: String,
        /// Who created this question (agent, human, flow)
        #[arg(long, default_value = "human")]
        created_by: String,
        /// Related task ID (optional)
        #[arg(long)]
        task_id: Option<String>,
        /// REQ-*/AC-* IDs this question is about (repeatable)
        #[arg(long = "refs")]
        refs: Vec<String>,
    },

    /// Resolve a question (mark as answered/resolved/obsolete)
    #[command(next_help_heading = "🏛️ Governance Artifacts")]
    QuestionResolve {
        /// Question ID to resolve (e.g., Q-TPL-001)
        #[arg(long)]
        id: String,
        /// Who resolved it (agent, human, flow)
        #[arg(long)]
        resolved_by: String,
        /// Which option was chosen (label from options list, optional)
        #[arg(long)]
        chosen_option: Option<String>,
        /// Resolution notes
        #[arg(long)]
        notes: Option<String>,
        /// New status (answered, resolved, obsolete)
        #[arg(long, default_value = "resolved")]
        status: String,
    },

    /// Search across friction, questions, and tasks
    #[command(next_help_heading = "🏛️ Governance Artifacts")]
    IssuesSearch {
        /// Search query (matches ID, summary, description)
        query: String,
        /// Limit to specific type: friction, question, task (omit for all)
        #[arg(long = "type")]
        type_filter: Option<String>,
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
        /// Filter by REQ/AC reference (e.g., REQ-TPL-001, AC-TPL-001)
        #[arg(long)]
        refs: Option<String>,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
        /// Maximum results to return
        #[arg(long, default_value = "50")]
        limit: usize,
    },

    /// List registered template forks
    #[command(next_help_heading = "🏛️ Governance Artifacts")]
    ForkList {
        /// Filter by status (active, archived, experimental)
        #[arg(long)]
        status: Option<String>,
        /// Filter by domain substring
        #[arg(long)]
        domain: Option<String>,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Register a new template fork
    #[command(next_help_heading = "🏛️ Governance Artifacts")]
    ForkRegister {
        /// Fork name
        #[arg(long)]
        name: String,
        /// Knowledge domain (e.g., rust-sdk, python-ml, knowledge-hub)
        #[arg(long)]
        domain: String,
        /// Kernel version fork is based on (e.g., v3.3.3)
        #[arg(long)]
        kernel_version: String,
        /// Repository URL (optional)
        #[arg(long)]
        url: Option<String>,
        /// Maintainer name (optional)
        #[arg(long)]
        maintainer_name: Option<String>,
        /// Maintainer contact (optional)
        #[arg(long)]
        maintainer_contact: Option<String>,
        /// Fork status (active, archived, experimental)
        #[arg(long)]
        status: Option<String>,
        /// Free-form notes (optional)
        #[arg(long)]
        notes: Option<String>,
    },

    /// Format Agent Skills (SKILL.md)
    #[command(next_help_heading = "🏛️ Governance Artifacts")]
    SkillsFmt,

    /// Lint Agent Skills (SKILL.md)
    #[command(next_help_heading = "🏛️ Governance Artifacts")]
    SkillsLint,

    /// Format Claude Code agents (.claude/agents/*.md)
    #[command(next_help_heading = "🏛️ Governance Artifacts")]
    AgentsFmt,

    /// Lint Claude Code agents (.claude/agents/*.md)
    #[command(next_help_heading = "🏛️ Governance Artifacts")]
    AgentsLint,

    // ============================================================================
    // TASKS & HINTS (Work tracking & agent guidance)
    // ============================================================================
    /// List tasks from specs/tasks.yaml
    #[command(next_help_heading = "📌 Tasks & Hints")]
    TasksList,

    /// Create a new task in specs/tasks.yaml
    #[command(next_help_heading = "📌 Tasks & Hints")]
    TaskCreate {
        /// Task ID (e.g., TASK-123)
        #[arg(long)]
        id: String,
        /// Task title
        #[arg(long)]
        title: String,
        /// Requirement ID the task belongs to
        #[arg(long = "req")]
        requirement: String,
        /// Acceptance criteria linked to the task (repeatable)
        #[arg(long = "ac")]
        acs: Vec<String>,
        /// Optional owner label
        #[arg(long)]
        owner: Option<String>,
        /// Task status (Todo, InProgress, Review, Done)
        #[arg(long)]
        status: Option<String>,
        /// Labels to attach to the task
        #[arg(long)]
        labels: Vec<String>,
    },

    /// Update an existing task in specs/tasks.yaml
    #[command(next_help_heading = "📌 Tasks & Hints")]
    TaskUpdate {
        /// Task ID to update
        #[arg(long)]
        id: String,
        /// New title (optional)
        #[arg(long)]
        title: Option<String>,
        /// New owner (optional)
        #[arg(long)]
        owner: Option<String>,
        /// New status (Todo, InProgress, Review, Done)
        #[arg(long)]
        status: Option<String>,
    },

    /// Suggest next steps for a task (agent guidance)
    #[command(next_help_heading = "📌 Tasks & Hints")]
    SuggestNext(commands::suggest_next::SuggestNextArgs),

    // ============================================================================
    // RELEASES (Version management & release process)
    // ============================================================================
    /// Prepare release (bump versions, update changelog)
    #[command(next_help_heading = "🚢 Releases")]
    ReleasePrepare {
        /// Version to release (e.g., 2.5.0)
        version: String,
        /// Preview changes without applying them
        #[arg(long)]
        dry_run: bool,
    },

    /// Generate release evidence bundle
    #[command(next_help_heading = "🚢 Releases")]
    ReleaseBundle {
        /// Version to generate evidence for (e.g., 3.1.0)
        version: String,
    },

    /// Generate kernel pack manifest and tarball
    #[command(next_help_heading = "🚢 Releases")]
    KernelPack {
        /// Output directory for the manifest (default: target/kernel-pack)
        #[arg(long, default_value = "target/kernel-pack")]
        output_dir: String,
    },

    /// Verify repo matches kernel pack manifest
    #[command(next_help_heading = "🚢 Releases")]
    KernelCheck {
        /// Path to manifest file (default: target/kernel-pack/kernel-pack.manifest.json)
        #[arg(long)]
        manifest: Option<String>,
    },

    /// Verify release readiness (selftest + audit + docs-check)
    #[command(next_help_heading = "🚢 Releases")]
    ReleaseVerify,

    /// Generate local SBOM (software bill of materials)
    #[command(next_help_heading = "🚢 Releases")]
    SbomLocal,

    /// Check crate publish readiness for crates.io
    #[command(next_help_heading = "🚢 Releases")]
    PublishCheck {
        /// Check a specific crate (default: all publishable crates)
        #[arg(long = "crate")]
        crate_name: Option<String>,
        /// Also run cargo publish --dry-run
        #[arg(long)]
        dry_run: bool,
    },

    /// Generate PR cover sheet from receipts
    #[command(next_help_heading = "🚢 Releases")]
    PrCover {
        /// PR number
        #[arg(long)]
        pr: u32,
        /// Run directory (must contain 'receipts/' subdirectory). Default: .runs/pr/{pr}/latest/
        #[arg(long)]
        run_dir: Option<std::path::PathBuf>,
        /// Output file (default: stdout)
        #[arg(long, short)]
        output: Option<std::path::PathBuf>,
        /// Description of what changed (1-3 sentences)
        #[arg(long, short, required = true)]
        description: String,
    },

    /// Update PR body with cover sheet (bounded replacement)
    #[command(next_help_heading = "🚢 Releases")]
    PrUpdate {
        /// PR number
        #[arg(long)]
        pr: u32,
        /// Directory containing receipts (default: .runs/pr/{pr}/latest/)
        #[arg(long)]
        run_dir: Option<std::path::PathBuf>,
        /// Description of what changed (optional)
        #[arg(long, short)]
        description: Option<String>,
        /// Save a copy to docs/audit/EXHIBITS/PR-{n}.md
        #[arg(long)]
        save_exhibit: bool,
        /// Dry run: show what would be updated without making changes
        #[arg(long)]
        dry_run: bool,
    },

    // ============================================================================
    // PUBLISHING & FORENSICS (Receipts, evidence, audit trails)
    // ============================================================================
    /// Run gates and emit gate.json receipt
    ///
    /// Executes validation gates (fmt, clippy, tests) and generates a structured
    /// JSON receipt in `.runs/current/receipts/gate.json` for CI pipelines,
    /// IDP integrations, and audit trails.
    #[command(next_help_heading = "📋 Publishing & Forensics")]
    ReceiptsGate {
        /// PR number (optional, included in receipt metadata)
        #[arg(long)]
        pr: Option<u32>,
        /// Output directory for receipts (default: .runs/current)
        #[arg(long, default_value = ".runs/current")]
        output_dir: std::path::PathBuf,
    },

    /// Generate economics.json receipt for DevLT and compute tracking
    ///
    /// Records developer time, compute spend, iteration counts, and value delivered.
    /// Supports confidence levels (measured/estimated/unknown) for honest reporting.
    #[command(next_help_heading = "📋 Publishing & Forensics")]
    ReceiptsEconomics {
        /// PR number (required)
        #[arg(long)]
        pr: u32,
        /// Output directory for receipts (default: .runs/current)
        #[arg(long, default_value = ".runs/current")]
        output_dir: std::path::PathBuf,
        /// Author time in minutes
        #[arg(long)]
        author_minutes: Option<u32>,
        /// Author time confidence: measured, estimated, unknown (default: unknown)
        #[arg(long, default_value = "unknown")]
        author_confidence: String,
        /// Review time in minutes
        #[arg(long)]
        review_minutes: Option<u32>,
        /// Review time confidence: measured, estimated, unknown (default: unknown)
        #[arg(long, default_value = "unknown")]
        review_confidence: String,
        /// Number of human interventions required
        #[arg(long, default_value = "0")]
        interventions: u32,
        /// Compute cost in USD
        #[arg(long)]
        compute_usd: Option<f64>,
        /// Compute confidence: measured, estimated, unknown (default: unknown)
        #[arg(long, default_value = "unknown")]
        compute_confidence: String,
        /// Number of CI/gate runs
        #[arg(long, default_value = "0")]
        runs: u32,
        /// Number of failed gate runs before success
        #[arg(long, default_value = "0")]
        failed_gates: u32,
        /// Number of fix-and-retry loops
        #[arg(long, default_value = "0")]
        fix_loops: u32,
        /// Description of uncertainty reduced
        #[arg(long)]
        uncertainty_reduced: Option<String>,
        /// Description of rework prevented
        #[arg(long)]
        rework_prevented: Option<String>,
        /// DevLT notes
        #[arg(long)]
        devlt_notes: Option<String>,
        /// Compute notes
        #[arg(long)]
        compute_notes: Option<String>,
        /// Iteration notes
        #[arg(long)]
        iteration_notes: Option<String>,
    },

    /// Validate receipt JSON files against their schemas
    ///
    /// Finds all `receipts/*.json` files in the run directory, matches each
    /// to its schema (gate.json -> gate.schema.json), and validates.
    /// Exits with non-zero code if any validation fails.
    #[command(next_help_heading = "📋 Publishing & Forensics")]
    ReceiptsValidate {
        /// Run directory containing receipts/ subdirectory
        #[arg(long, default_value = ".runs/current")]
        dir: std::path::PathBuf,
        /// Schema directory (default: specs/schemas/)
        #[arg(long, default_value = "specs/schemas")]
        schema_dir: std::path::PathBuf,
    },

    /// Generate quality.json receipt with code quality metrics
    ///
    /// Computes hard metrics from git diff (files changed, modules touched,
    /// unsafe delta, test/impl LOC) and optionally accepts LLM assessments
    /// for boundary integrity and test depth ratings.
    ///
    /// With --llm, obtains semantic analysis from the Historian agent and merges
    /// boundary/test-depth/risk assessments into the receipt. Use --historian-output
    /// for offline/testing with pre-generated historian output.
    #[command(next_help_heading = "📋 Publishing & Forensics")]
    ReceiptsQuality {
        /// PR number (optional, included in receipt metadata)
        #[arg(long)]
        pr: Option<u32>,
        /// Output directory for receipts (default: .runs/current)
        #[arg(long, default_value = ".runs/current")]
        output_dir: std::path::PathBuf,
        /// Base branch for comparison (default: origin/main)
        #[arg(long, default_value = "origin/main")]
        base_branch: String,
        /// LLM-provided boundary rating (improved/neutral/degraded)
        #[arg(long)]
        boundary_rating: Option<String>,
        /// LLM-provided test depth rating (hardened/mixed/shallow)
        #[arg(long)]
        test_depth_rating: Option<String>,
        /// LLM-provided notes (repeatable)
        #[arg(long)]
        notes: Vec<String>,
        /// Enable LLM semantic analysis via Historian agent
        #[arg(long)]
        llm: bool,
        /// Path to existing historian output (for offline/testing use)
        #[arg(long)]
        historian_output: Option<std::path::PathBuf>,
        /// Command template for running historian (use {input} as placeholder).
        /// Precedence: CLI arg > HISTORIAN_CMD env var > error if not configured.
        #[arg(long)]
        historian_cmd: Option<String>,
    },

    /// Generate telemetry.json receipt with probe execution results
    ///
    /// Captures change surface from git diff, contract change detection,
    /// and probe execution status based on the selected profile.
    #[command(next_help_heading = "📋 Publishing & Forensics")]
    ReceiptsTelemetry {
        /// PR number (optional, included in receipt metadata)
        #[arg(long)]
        pr: Option<u32>,
        /// Output directory for receipts (default: .runs/current)
        #[arg(long, default_value = ".runs/current")]
        output_dir: std::path::PathBuf,
        /// Probe profile: fast (quick CI), full (comprehensive), exhibit (forensic)
        #[arg(long, default_value = "fast")]
        profile: String,
        /// Base branch for comparison (default: origin/main)
        #[arg(long, default_value = "origin/main")]
        base_branch: String,
    },

    /// Generate timeline.json receipt from commit history
    ///
    /// Analyzes commit history to identify sessions, friction zones,
    /// oscillation patterns, and classifies overall development topology.
    #[command(next_help_heading = "📋 Publishing & Forensics")]
    ReceiptsTimeline {
        /// PR number (optional, included in receipt metadata)
        #[arg(long)]
        pr: Option<u32>,
        /// Output directory for receipts (default: .runs/current)
        #[arg(long, default_value = ".runs/current")]
        output_dir: std::path::PathBuf,
        /// Base branch for comparison (default: origin/main)
        #[arg(long, default_value = "origin/main")]
        base_branch: String,
        /// Session gap threshold in minutes (default: 30)
        #[arg(long, default_value = "30")]
        session_gap_minutes: u32,
        /// Additional path prefixes to exclude from friction analysis (repeatable)
        #[arg(long = "exclude-prefix")]
        exclude_prefixes: Vec<String>,
        /// Include ephemeral directories (.runs/, target/) in analysis (debug only)
        #[arg(long)]
        include_ephemeral: bool,
    },

    /// Run all receipt emitters for comprehensive PR forensics
    ///
    /// Generates a complete forensic receipt set by running:
    /// 1. telemetry - change surface facts and probe results
    /// 2. timeline - development pattern analysis from git history
    /// 3. quality - code quality metrics from diff analysis (optionally with LLM)
    /// 4. validate - schema validation of all generated receipts
    #[command(next_help_heading = "📋 Publishing & Forensics")]
    ReceiptsForensic {
        /// PR number (required)
        #[arg(long)]
        pr: u32,
        /// Probe profile: fast (quick CI), full (comprehensive), exhibit (forensic)
        #[arg(long, default_value = "fast")]
        profile: String,
        /// Base branch for comparison (default: origin/main)
        #[arg(long, default_value = "origin/main")]
        base_branch: String,
        /// Output directory for receipts (default: .runs/current)
        #[arg(long, default_value = ".runs/current")]
        output_dir: std::path::PathBuf,
        /// Additional path prefixes to exclude from friction analysis (repeatable)
        #[arg(long = "exclude-prefix")]
        exclude_prefixes: Vec<String>,
        /// Include ephemeral directories (.runs/, target/) in analysis (debug only)
        #[arg(long)]
        include_ephemeral: bool,
        /// Enable LLM semantic analysis for quality receipt via Historian agent
        #[arg(long)]
        llm: bool,
        /// Path to existing historian markdown output (for offline/testing use)
        #[arg(long)]
        historian_output: Option<std::path::PathBuf>,
        /// Command template for running historian (use {input} as placeholder).
        /// Precedence: CLI arg > HISTORIAN_CMD env var > error if not configured.
        #[arg(long)]
        historian_cmd: Option<String>,
    },

    // ============================================================================
    // SERVICE SETUP (Initialization & configuration)
    // ============================================================================
    /// Initialize service branding (ID, name, description)
    #[command(next_help_heading = "⚙️ Service Setup")]
    ServiceInit {
        /// Service ID (kebab-case, e.g., agile-hr)
        #[arg(long)]
        id: String,
        /// Display name (e.g., "Agile HR Hub")
        #[arg(long)]
        name: String,
        /// Service description
        #[arg(long)]
        description: String,
        /// Tags (repeatable, e.g., --tags hr --tags payroll)
        #[arg(long)]
        tags: Vec<String>,
        /// Register this fork in fork_registry.yaml
        #[arg(long)]
        register_fork: bool,
    },

    /// Generate service descriptor (e.g., Backstage catalog info)
    #[command(next_help_heading = "⚙️ Service Setup")]
    ServiceDescriptor {
        /// Output format (backstage)
        #[arg(long, default_value = "backstage")]
        format: String,
    },

    /// Validate config schema for an environment
    #[command(next_help_heading = "⚙️ Service Setup")]
    ConfigValidate {
        /// Target environment: dev, staging, prod
        #[arg(short, long, default_value = "dev")]
        env: String,
    },

    // ============================================================================
    // SECURITY & POLICY (Audits, policies, supply chain)
    // ============================================================================
    /// Run security and dependency audit (cargo audit + cargo deny)
    #[command(next_help_heading = "🔐 Security & Policy")]
    Audit,

    /// Test Rego policies with conftest (OPA policy verification)
    #[command(next_help_heading = "🔐 Security & Policy")]
    PolicyTest,

    /// Run test coverage analysis with tarpaulin (baseline: 65%)
    #[command(next_help_heading = "🔐 Security & Policy")]
    Coverage,

    // ============================================================================
    // BUILD METRICS (Build time tracking & analysis)
    // ============================================================================
    /// Capture build time metrics (clean release build)
    #[command(next_help_heading = "📊 Build Metrics")]
    BuildTimeCapture,

    /// Compare two build time metric files
    #[command(next_help_heading = "📊 Build Metrics")]
    BuildTimeCompare {
        /// Path to baseline metrics file
        #[arg(long)]
        baseline: String,
        /// Path to current metrics file
        #[arg(long)]
        current: String,
    },

    // ============================================================================
    // LLM/AGENT SUPPORT (Bundles, hints, workflows)
    // ============================================================================
    /// Generate LLM context bundle for a task (AI-native development)
    ///
    /// See also: /platform/tasks API for available tasks
    #[command(next_help_heading = "🤖 LLM/Agent Support")]
    Bundle {
        /// Task name from .llm/contextpack.yaml
        task: String,
    },

    /// Show flow-based command map (available workflows for agents/humans)
    #[command(next_help_heading = "🤖 LLM/Agent Support")]
    HelpFlows,

    // ============================================================================
    // INFRASTRUCTURE & UTILITIES (Build, cleanup, migrations)
    // ============================================================================
    /// Format all code (Rust, YAML validation, etc.)
    #[command(next_help_heading = "🔧 Infrastructure & Utilities")]
    FmtAll,

    /// Update tool checksums in scripts/tools.sha256
    #[command(next_help_heading = "🔧 Infrastructure & Utilities")]
    ToolsChecksumUpdate,

    /// Verify tool checksums are present and valid
    #[command(next_help_heading = "🔧 Infrastructure & Utilities")]
    ToolsChecksumVerify,

    /// Clean workspace (remove target/, generated docs, etc.)
    #[command(next_help_heading = "🔧 Infrastructure & Utilities")]
    Clean,

    /// Export dependency graph
    #[command(next_help_heading = "🔧 Infrastructure & Utilities")]
    GraphExport {
        #[arg(long, value_enum, default_value = "json")]
        format: commands::graph_export::OutputFormat,
        /// Validate graph invariants instead of emitting graph output
        #[arg(long)]
        check: bool,
        /// Report format for invariant checks (text or json)
        #[arg(long, value_enum, default_value = "text")]
        report_format: commands::graph_export::ReportFormat,
    },

    /// Manage workspace dependencies with hakari
    #[command(next_help_heading = "🔧 Infrastructure & Utilities")]
    Hakari,

    /// Run database migrations
    #[command(next_help_heading = "🔧 Infrastructure & Utilities")]
    Migrate,

    /// Pin GitHub Actions to commit SHAs
    #[command(next_help_heading = "🔧 Infrastructure & Utilities")]
    PinActions,

    /// Deploy application to specified environment (dev, staging, prod)
    #[command(next_help_heading = "🔧 Infrastructure & Utilities")]
    Deploy {
        /// Target environment: dev, staging, or prod
        #[arg(short, long, default_value = "dev")]
        env: String,
    },

    // ============================================================================
    // STATUS & METADATA (Service status, version, help)
    // ============================================================================
    /// Show governance status dashboard (health check)
    #[command(next_help_heading = "ℹ️ Status & Metadata")]
    Status,

    /// Show kernel/template version
    #[command(next_help_heading = "ℹ️ Status & Metadata")]
    Version {
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Validate version consistency across all version-bearing files
    #[command(next_help_heading = "ℹ️ Status & Metadata")]
    VersionCheck {
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Show environment detection mode (CI, noninteractive, low-resources)
    #[command(next_help_heading = "ℹ️ Status & Metadata")]
    EnvMode {
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    // ============================================================================
    // IDP INTEGRATION (Machine-readable snapshots for IDPs)
    // ============================================================================
    /// Generate IDP snapshot (consolidated governance + task state for IDPs)
    #[command(next_help_heading = "🔌 IDP Integration")]
    IdpSnapshot {
        /// Output file path (default: stdout)
        #[arg(long)]
        output: Option<String>,
        /// Pretty-print JSON output
        #[arg(long)]
        pretty: bool,
    },

    /// Validate IDP integration surface (OpenAPI lint + Backstage plugin checks)
    #[command(next_help_heading = "🔌 IDP Integration")]
    IdpCheck,
}

impl Commands {
    pub fn name(&self) -> &'static str {
        match self {
            Commands::Doctor => "doctor",
            Commands::DevUp => "dev-up",
            Commands::InstallHooks => "install-hooks",
            Commands::CiLocal => "ci-local",
            Commands::KernelCheck { .. } => "kernel-check",
            Commands::KernelPack { .. } => "kernel-pack",
            Commands::KernelSmoke => "kernel-smoke",
            Commands::KernelStatus => "kernel-status",
            Commands::Selftest => "selftest",
            Commands::Quickstart => "quickstart",
            Commands::Check => "check",
            Commands::Precommit { .. } => "precommit",
            Commands::TestChanged { .. } => "test-changed",
            Commands::CheckApiDiff { .. } => "check-api-diff",
            Commands::CheckOpenapiDiff => "check-openapi-diff",
            Commands::CheckJsonSchemas { .. } => "check-json-schemas",
            Commands::CheckLayering => "check-layering",
            Commands::AcStatus { .. } => "ac-status",
            Commands::AcNew { .. } => "ac-new",
            Commands::AcCoverage { .. } => "ac-coverage",
            Commands::AcSuggestScenarios { .. } => "ac-suggest-scenarios",
            Commands::AcTests { .. } => "ac-tests",
            Commands::TestAc { .. } => "test-ac",
            Commands::Bdd => "bdd",
            Commands::AcReport { .. } => "ac-report",
            Commands::AcHistory { .. } => "ac-history",
            Commands::AcSlo { .. } => "ac-slo",
            Commands::AcEnsureKernelMapped { .. } => "ac-ensure-kernel-mapped",
            Commands::AcLint { .. } => "ac-lint",
            Commands::AdrNew { .. } => "adr-new",
            Commands::AdrCheck => "adr-check",
            Commands::DesignNew { .. } => "design-new",
            Commands::DocsCheck => "docs-check",
            Commands::DocsFrontmatterSync { .. } => "docs-frontmatter-sync",
            Commands::Spellcheck => "spellcheck",
            Commands::ContractsCheck => "contracts-check",
            Commands::ContractsFmt => "contracts-fmt",
            Commands::UiContractCheck => "ui-contract-check",
            Commands::FrictionList { .. } => "friction-list",
            Commands::FrictionNew { .. } => "friction-new",
            Commands::FrictionResolve { .. } => "friction-resolve",
            Commands::FrictionGhCreate { .. } => "friction-gh-create",
            Commands::FrictionGhLink { .. } => "friction-gh-link",
            Commands::QuestionsList { .. } => "questions-list",
            Commands::QuestionNew { .. } => "question-new",
            Commands::QuestionResolve { .. } => "question-resolve",
            Commands::IssuesSearch { .. } => "issues-search",
            Commands::ForkList { .. } => "fork-list",
            Commands::ForkRegister { .. } => "fork-register",
            Commands::SkillsFmt => "skills-fmt",
            Commands::SkillsLint => "skills-lint",
            Commands::AgentsFmt => "agents-fmt",
            Commands::AgentsLint => "agents-lint",
            Commands::TasksList => "tasks-list",
            Commands::TaskCreate { .. } => "task-create",
            Commands::TaskUpdate { .. } => "task-update",
            Commands::SuggestNext(..) => "suggest-next",
            Commands::ReleasePrepare { .. } => "release-prepare",
            Commands::ReleaseBundle { .. } => "release-bundle",
            Commands::ReleaseVerify => "release-verify",
            Commands::SbomLocal => "sbom-local",
            Commands::PublishCheck { .. } => "publish-check",
            Commands::PrCover { .. } => "pr-cover",
            Commands::PrUpdate { .. } => "pr-update",
            Commands::ReceiptsGate { .. } => "receipts-gate",
            Commands::ReceiptsEconomics { .. } => "receipts-economics",
            Commands::ReceiptsValidate { .. } => "receipts-validate",
            Commands::ReceiptsQuality { .. } => "receipts-quality",
            Commands::ReceiptsTelemetry { .. } => "receipts-telemetry",
            Commands::ReceiptsTimeline { .. } => "receipts-timeline",
            Commands::ReceiptsForensic { .. } => "receipts-forensic",
            Commands::ServiceInit { .. } => "service-init",
            Commands::ServiceDescriptor { .. } => "service-descriptor",
            Commands::ConfigValidate { .. } => "config-validate",
            Commands::Audit => "audit",
            Commands::PolicyTest => "policy-test",
            Commands::Coverage => "coverage",
            Commands::BuildTimeCapture => "build-time-capture",
            Commands::BuildTimeCompare { .. } => "build-time-compare",
            Commands::Bundle { .. } => "bundle",
            Commands::HelpFlows => "help-flows",
            Commands::FmtAll => "fmt-all",
            Commands::ToolsChecksumUpdate => "tools-checksum-update",
            Commands::ToolsChecksumVerify => "tools-checksum-verify",
            Commands::Clean => "clean",
            Commands::GraphExport { .. } => "graph-export",
            Commands::Hakari => "hakari",
            Commands::Migrate => "migrate",
            Commands::PinActions => "pin-actions",
            Commands::Deploy { .. } => "deploy",
            Commands::Status => "status",
            Commands::Version { .. } => "version",
            Commands::VersionCheck { .. } => "version-check",
            Commands::EnvMode { .. } => "env-mode",
            Commands::IdpSnapshot { .. } => "idp-snapshot",
            Commands::IdpCheck => "idp-check",
        }
    }
}
