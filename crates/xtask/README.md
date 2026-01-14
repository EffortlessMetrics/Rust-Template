# xtask

Development task automation CLI for build, test, and maintenance workflows.

## What It Is

`xtask` is the `cargo-xtask` pattern binary that provides a single entrypoint for all dev and CI operations. It follows the convention of a workspace binary crate that acts as a task runner for development workflows.

This tool automatically wraps execution in `nix develop` when Nix is available, ensuring hermetic builds and perfect CI/local parity per ADR-0002.

### What It Owns

| Module | Responsibility |
|--------|----------------|
| `main.rs` | CLI entry point, command routing, Nix wrapper |
| `commands/` | Individual command implementations |
| `contracts.rs` | Governed facts validation and synchronization |
| `env.rs` | Environment detection (CI, noninteractive, low-resources) |
| `kernel.rs` | Kernel health and validation |
| `validation.rs` | Spec and config validation utilities |
| `docs_index/` | Documentation index management |

### What It Is Not

- **Not a production service**: This is a dev tool, not deployed to production
- **Not a library**: This is a binary crate (`[[bin]]`)
- **Not a test framework**: Uses `cucumber` for acceptance tests

## Quick Start

### Installation

```bash
# Run xtask commands via cargo
cargo xtask <command>

# Or build and run directly
cargo run -p xtask -- <command>
```

### Common Commands

```bash
# Onboarding
cargo xtask doctor          # Diagnose development environment
cargo xtask dev-up            # One-command developer bootstrap
cargo xtask install-hooks    # Install git hooks

# Validation
cargo xtask check             # Run fmt, clippy, unit tests
cargo xtask selftest          # Run full governance validation
cargo xtask precommit         # Run pre-commit guardrail checks

# Acceptance Criteria
cargo xtask ac-status         # Generate AC coverage report
cargo xtask ac-new            # Create new acceptance criterion
cargo xtask bdd               # Run BDD acceptance tests

# Documentation
cargo xtask adr-new           # Create new architecture decision record
cargo xtask docs-check        # Verify documentation consistency
cargo xtask spellcheck        # Run spellcheck across docs/specs

# Cleanup
cargo xtask clean             # Remove target/, generated docs, etc.
```

## Command Categories

### 🚀 Onboarding

| Command | Description |
|---------|-------------|
| `doctor` | Diagnose development environment setup |
| `dev-up` | One-command developer bootstrap (nix + hooks + checks) |
| `install-hooks` | Install git hooks for pre-commit governance |
| `ci-local` | Run CI checks locally (doctor + selftest + audit + docs-check) |

### ✅ Validation Gates

| Command | Description |
|---------|-------------|
| `kernel-smoke` | Quick kernel smoke test – validate template baseline |
| `kernel-status` | Show aggregated kernel health (specs, docs, governance) |
| `selftest` | Run full template self-test suite (8-step governance validation) |
| `quickstart` | Quick validation of template functionality |
| `check` | Run all checks: fmt, clippy, unit tests |
| `precommit` | Run pre-commit guardrail checks (fmt/clippy/tests/docs/spellcheck) |
| `test-changed` | Run tests affected by git changes |

### 📋 Acceptance Criteria

| Command | Description |
|---------|-------------|
| `ac-status` | Generate AC status report from acceptance tests |
| `ac-new` | Create new acceptance criterion |
| `ac-coverage` | Show AC coverage grouped by requirement |
| `ac-suggest-scenarios` | Suggest BDD scenarios for a given AC |
| `ac-tests` | Show all tests mapped to a specific AC |
| `test-ac` | Run tests for a specific acceptance criterion |
| `bdd` | Run BDD acceptance tests |
| `ac-report` | Generate human-readable AC governance report |
| `ac-history` | Analyze AC coverage trends from CI snapshots |
| `ac-slo` | Check if AC coverage meets SLO thresholds |
| `ac-ensure-kernel-mapped` | Verify all kernel ACs have test mappings |
| `ac-lint` | Lint spec_ledger.yaml for structural integrity |

### 📚 Design & Documentation

| Command | Description |
|---------|-------------|
| `adr-new` | Create new architecture decision record |
| `adr-check` | Validate ADR references in spec ledger |
| `design-new` | Create new design document with front-matter |
| `docs-check` | Verify documentation consistency |
| `docs-frontmatter-sync` | Sync front-matter in design docs from doc_index.yaml |
| `spellcheck` | Run spellcheck across docs/specs |
| `contracts-check` | Check that governed facts in docs match their sources |
| `contracts-fmt` | Synchronize governed facts from code/specs to docs |
| `ui-contract-check` | Validate UI contract (specs/ui_contract.yaml) and DOM anchors |

### 🏛️ Governance Artifacts

| Command | Description |
|---------|-------------|
| `friction-list` | List friction log entries (track process/tooling issues) |
| `friction-new` | Create a new friction log entry |
| `friction-resolve` | Resolve a friction entry (mark as resolved) |
| `friction-gh-create` | Create GitHub issue from friction entry |
| `friction-gh-link` | Link existing GitHub issue to friction entry |
| `questions-list` | List questions from questions/ directory |
| `question-new` | Create a new question artifact (capture ambiguity) |
| `question-resolve` | Resolve a question (mark as answered/resolved/obsolete) |
| `issues-search` | Search across friction, questions, and tasks |
| `fork-list` | List registered template forks |
| `fork-register` | Register a new template fork |
| `skills-fmt` | Format Agent Skills (SKILL.md) |
| `skills-lint` | Lint Agent Skills (SKILL.md) |
| `agents-fmt` | Format Claude Code agents (.claude/agents/*.md) |
| `agents-lint` | Lint Claude Code agents (.claude/agents/*.md) |

### 📌 Tasks & Hints

| Command | Description |
|---------|-------------|
| `tasks-list` | List tasks from specs/tasks.yaml |
| `task-create` | Create a new task in specs/tasks.yaml |
| `task-update` | Update an existing task in specs/tasks.yaml |
| `suggest-next` | Suggest next steps for a task (agent guidance) |

### 🚢 Releases

| Command | Description |
|---------|-------------|
| `release-prepare` | Prepare release (bump versions, update changelog) |
| `release-bundle` | Generate release evidence bundle |
| `release-verify` | Verify release readiness (selftest + audit + docs-check) |
| `sbom-local` | Generate local SBOM (software bill of materials) |
| `pr-cover` | Generate PR cover sheet from receipts |
| `pr-update` | Update PR body with cover sheet (bounded replacement) |

### 📋 Publishing & Forensics

| Command | Description |
|---------|-------------|
| `receipts-gate` | Run gates and emit gate.json receipt |
| `receipts-economics` | Generate economics.json receipt for DevLT tracking |
| `receipts-validate` | Validate receipt JSON files against their schemas |
| `receipts-quality` | Generate quality.json receipt with code quality metrics |
| `receipts-telemetry` | Generate telemetry.json receipt with probe execution results |
| `receipts-timeline` | Generate timeline.json receipt from commit history |
| `receipts-forensic` | Run all receipt emitters for comprehensive PR forensics |

### ⚙️ Service Setup

| Command | Description |
|---------|-------------|
| `service-init` | Initialize service branding (ID, name, description) |
| `service-descriptor` | Generate service descriptor (e.g., Backstage catalog info) |
| `config-validate` | Validate config schema for an environment |

### 🔐 Security & Policy

| Command | Description |
|---------|-------------|
| `audit` | Run security and dependency audit (cargo audit + cargo deny) |
| `policy-test` | Test Rego policies with conftest (OPA policy verification) |
| `coverage` | Run test coverage analysis with tarpaulin (baseline: 65%) |

### 📊 Build Metrics

| Command | Description |
|---------|-------------|
| `build-time-capture` | Capture build time metrics (clean release build) |
| `build-time-compare` | Compare two build time metric files |

### 🤖 LLM/Agent Support

| Command | Description |
|---------|-------------|
| `bundle` | Generate LLM context bundle for a task (AI-native development) |
| `help-flows` | Show flow-based command map (available workflows) |

### 🔧 Infrastructure & Utilities

| Command | Description |
|---------|-------------|
| `fmt-all` | Format all code (Rust, YAML validation, etc.) |
| `tools-checksum-update` | Update tool checksums in scripts/tools.sha256 |
| `tools-checksum-verify` | Verify tool checksums are present and valid |
| `clean` | Clean workspace (remove target/, generated docs, etc.) |
| `graph-export` | Export dependency graph |
| `hakari` | Manage workspace dependencies with hakari |
| `migrate` | Run database migrations |
| `pin-actions` | Pin GitHub Actions to commit SHAs |
| `deploy` | Deploy application to specified environment |

### ℹ️ Status & Metadata

| Command | Description |
|---------|-------------|
| `status` | Show governance status dashboard (health check) |
| `version` | Show kernel/template version |
| `version-check` | Validate version consistency across all version-bearing files |
| `env-mode` | Show environment detection mode (CI, noninteractive, low-resources) |

### 🔌 IDP Integration

| Command | Description |
|---------|-------------|
| `idp-snapshot` | Generate IDP snapshot (consolidated governance + task state) |
| `idp-check` | Validate IDP integration surface (OpenAPI lint + Backstage plugin checks) |

## Nix Integration

The tool automatically wraps all commands in `nix develop` when available:

```bash
# If Nix is installed, this runs via: nix develop -c cargo run -p xtask -- <command>
cargo xtask <command>
```

To disable Nix wrapping temporarily:

```bash
# Set IN_NIX_SHELL to skip wrapper
IN_NIX_SHELL=1 cargo xtask <command>
```

## Verbosity Control

All commands support verbosity flags:

```bash
# Verbose output (show detailed output)
cargo xtask --verbose <command>

# Quiet output (suppress non-error output)
cargo xtask --quiet <command>
```

## Environment Variables

| Variable | Description |
|-----------|-------------|
| `IN_NIX_SHELL` | Set to skip Nix wrapper |
| `XTASK_LOW_RESOURCES` | Set to limit build jobs and disable sccache |
| `RUST_LOG` | Log level filter (e.g., `debug`, `xtask=trace`) |
| `OTLP_ENDPOINT` | OTLP collector endpoint (for telemetry) |

## Usage Examples

### Running Pre-commit Checks

```bash
# Full mode (all checks regardless of what changed)
cargo xtask precommit --mode full

# Fast mode (change-aware routing)
cargo xtask precommit --mode fast --staged-only
```

### Managing Acceptance Criteria

```bash
# Generate AC status report
cargo xtask ac-status

# Check mode: verify existing file matches computed state
cargo xtask ac-status --check

# Show summary only
cargo xtask ac-status --summary

# Output in JSON format
cargo xtask ac-status --json

# Filter to specific AC
cargo xtask ac-status --ac AC-KERN-001
```

### Creating Governance Artifacts

```bash
# Create new friction entry
cargo xtask friction-new \
  --category tooling \
  --severity high \
  --summary "Slow build times" \
  --flow build \
  --phase compile

# Create new question
cargo xtask question-new \
  --category TPL \
  --summary "Ambiguous requirement" \
  --flow design \
  --phase specification \
  --description "The requirement doesn't specify..."

# Create new ADR
cargo xtask adr-new "Use PostgreSQL for primary data store"
```

### Running Receipt Generation

```bash
# Generate gate receipt
cargo xtask receipts-gate --pr 123

# Generate forensic receipts (all emitters)
cargo xtask receipts-forensic --pr 123 --profile fast

# Generate economics receipt
cargo xtask receipts-economics --pr 123 \
  --author-minutes 60 \
  --author-confidence measured \
  --review-minutes 30 \
  --review-confidence estimated
```

## Architecture

The tool follows the `cargo-xtask` pattern:

```
User runs: cargo xtask <command>
         ↓
    [Nix wrapper] (if available)
         ↓
    xtask binary (this crate)
         ↓
    Command routing (clap)
         ↓
    Individual command implementations
         ↓
    Execute cargo/git/other tools
```

## Consumers

This crate is used by:

| Consumer | Usage |
|----------|-------|
| Developers | Local development and testing |
| CI/CD | Automated validation and release workflows |
| Git Hooks | Pre-commit validation |

## See Also

- [`docs/reference/xtask-commands.md`](../../docs/reference/xtask-commands.md) - Complete command reference
- [`docs/how-to/add-xtask-command.md`](../../docs/how-to/add-xtask-command.md) - Guide for adding commands
- [`gov-xtask-core/README.md`](../gov-xtask-core/README.md) - Shared xtask utilities
