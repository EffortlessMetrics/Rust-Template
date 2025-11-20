# Claude Code Usage Guide

**Version**: v2.4.0
**Last Updated**: 2025-11-19
**Purpose**: Define the LLM-assisted workflow for this governed Rust template

---

## Overview

This repository is designed for **LLM-assisted development** with built-in governance rails. The template provides:

- 📋 **Spec-as-Code**: All features tracked as acceptance criteria (ACs) in `specs/spec_ledger.yaml` ([ADR-0003](docs/adr/0003-spec-and-bdd-as-source-of-truth.md))
- 🤖 **Bounded Context**: LLM bundles generated via `xtask bundle` with automatic size limits ([ADR-0004](docs/adr/0004-policy-and-llm-governance.md))
- ✅ **Safety Rails**: Selftest validates fmt, clippy, tests, BDD scenarios, and policy compliance ([ADR-0005](docs/adr/0005-xtask-selftest-single-gate.md))
- 🎯 **AC-First Flow**: Always work from existing ACs, never invent new IDs
- 🔧 **Nix-First**: Declarative dev environment matching CI exactly ([ADR-0002](docs/adr/0002-nix-first-dev-env.md))
- 🏛️ **Hexagonal Architecture**: Business logic in core, adapters for HTTP/gRPC/DB ([ADR-0001](docs/adr/0001-hexagonal-architecture.md))

**Development Environment**: This template uses Nix for reproducible development. Always run commands inside `nix develop`:

```bash
# Enter the development shell (first time or new session)
nix develop

# Now all tools are pinned: Rust, conftest, yq, etc.
cargo run -p xtask -- selftest  # Full validation including policies
```

If you can't use Nix, see [docs/dev-environment.md](docs/dev-environment.md) for the fallback path. Policy tests will be skipped locally but still enforced in CI.

---

## Golden Path

### 1. Pick an AC

```bash
# View all acceptance criteria
cat specs/spec_ledger.yaml

# Check AC status
cargo run -p xtask -- ac-status
```

Example AC:
```yaml
- id: AC-TPL-001
  title: Template Repository Structure
  status: implemented
  feature_file: specs/features/template_core.feature
  scenario: Template provides hexagonal architecture
```

### 2. Generate Context Bundle

```bash
# For implementing an AC
cargo run -p xtask -- bundle implement_ac

# For creating a new service (pilot project)
cargo run -p xtask -- bundle new_service_guide
```

This creates `.llm/bundle/implement_ac.md` (or similar) with:
- Spec ledger and relevant features
- Core business logic
- HTTP/gRPC adapters
- Acceptance tests
- Architecture docs
- **Bounded to ~250KB** with automatic file selection

### 3. Use the Standard Prompt

Open `.llm/bundle/implement_ac.md` in your LLM client (Claude Code, Cursor, etc.) and use one of the standard prompts below.

### 4. Apply Changes & Validate

```bash
# Run full validation
cargo run -p xtask -- selftest

# Or individual checks
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p acceptance
```

### 5. Commit When Green

Only commit when selftest passes. The template enforces:
- ✅ Code formatting (rustfmt)
- ✅ Linting (clippy)
- ✅ Unit tests
- ✅ BDD scenarios
- ✅ AC mapping integrity
- ✅ LLM bundle generation
- ⚠️  Policy tests (enforced in CI via Nix+conftest)

---

## Golden Path Workflows

You have access to a comprehensive `xtask` command suite. **Prefer these flows over ad-hoc commands.**

**Onboarding (new dev / new machine)**

1. `nix develop`
2. `cargo xtask doctor`
3. `cargo xtask check`

**AC-first development (add behavior)**

1. `cargo xtask ac-new AC-ID "description" --story US-ID --requirement REQ-ID`
2. Add the AC snippet to `specs/spec_ledger.yaml`
3. Add a `@AC-ID` scenario to the appropriate file in `specs/features/`
4. `cargo xtask bundle implement_ac` (generates LLM context)
5. Implement code + tests with LLM assistance
6. `cargo xtask bdd` → `cargo xtask selftest`

**Architecture decisions**

1. `cargo xtask adr-new "Decision Title"`
2. Edit `docs/adr/NNNN-*.md`
3. Link the ADR from `specs/spec_ledger.yaml`
4. `cargo xtask adr-check`

**Dependencies & security**

- When dependencies change: `cargo xtask audit`
- For local SBOM inspection: `cargo xtask sbom-local`

**Release**

1. `cargo xtask release-prepare X.Y.Z`
2. Edit `CHANGELOG.md` entry
3. `cargo xtask release-verify`
4. Tag + push

**Discoverability**

- `cargo xtask help-flows` shows flow-based command groups
- `cargo xtask --help` shows grouped commands by category


---

## Standard Prompts

### Implement an Existing AC

```text
You are working in a governed Rust service template (v2.4.0).

Task: Implement ONE existing acceptance criterion from specs/spec_ledger.yaml.

Constraints:
- Do NOT create new AC IDs
- Work with the existing AC: [AC-ID-HERE]
- Keep selftest passing
- Follow hexagonal architecture patterns

Steps:
1. Identify the AC and its corresponding BDD scenario in specs/features/
2. Review existing code patterns in business-core/ and app-http/
3. Propose minimal changes:
   - Update specs/features/ if behavior changes
   - Implement business logic in business-core/
   - Wire endpoints in app-http/
   - Add/update tests in acceptance/
4. Ensure all changes follow existing patterns (error handling, telemetry, etc.)

After implementation, I will run: cargo run -p xtask -- selftest
```

### Fix a Failing BDD Scenario

```text
You are working in a governed Rust service template (v2.4.0).

Task: Fix a failing BDD scenario while maintaining governance.

Current failure: [paste cucumber output]

Constraints:
- Fix the scenario without breaking other tests
- Maintain AC mapping in specs/spec_ledger.yaml
- Keep hexagonal architecture boundaries clean
- Preserve existing error handling and telemetry patterns

Steps:
1. Identify which AC owns this scenario
2. Locate the relevant business logic in business-core/
3. Propose minimal fix (prefer fixing code over changing spec)
4. Update acceptance tests if needed

After fix, I will run: cargo run -p xtask -- selftest
```

### Add a New Endpoint (Following Existing Patterns)

```text
You are working in a governed Rust service template (v2.4.0).

Task: Add a new HTTP endpoint following existing patterns.

New endpoint: [describe endpoint]
Related AC: [AC-ID if exists, or note "needs new AC first"]

Constraints:
- Follow patterns from app-http/src/routes/
- Use existing middleware (telemetry, error handling)
- Keep business logic in business-core/
- Add OpenAPI documentation in specs/openapi/openapi.yaml
- Add BDD scenario in specs/features/

Steps:
1. Review existing endpoints (e.g., /version, /health)
2. Propose endpoint structure following axum patterns
3. Wire business logic from business-core/
4. Add OpenAPI spec
5. Add acceptance test

After implementation, I will run: cargo run -p xtask -- selftest
```

### Create a Pilot Project (New Service)

```text
You are helping create a new service from the Rust Template (v2.4.0).

Context: I've created a new project using GitHub's "Use this template" or manual git clone.

Task: Help implement the first feature for a new domain.

Domain: [describe domain, e.g., "task management API"]
First feature: [e.g., "create task endpoint"]

Steps:
1. Suggest AC structure for specs/spec_ledger.yaml
2. Propose BDD scenarios in specs/features/
3. Design domain model in business-core/
4. Implement HTTP handlers in app-http/
5. Add acceptance tests

Constraints:
- Reuse template patterns (error handling, telemetry, hexagonal arch)
- Keep selftest passing
- Log any friction in FRICTION_LOG.md

I will validate with: cargo run -p xtask -- selftest
```

---

## LLM Responsibilities vs Human Ownership

### ✅ LLM Can Change

- Business logic in `business-core/`
- HTTP handlers in `app-http/`
- gRPC adapters in `adapters-grpc/` (if needed)
- BDD scenarios in `specs/features/`
- Acceptance tests in `acceptance/`
- OpenAPI specs in `specs/openapi/`
- AC status in `specs/spec_ledger.yaml` (update status only, don't invent IDs)

### 🛡️ Human-Owned (LLM Should Ask First)

- **New AC IDs**: Must follow project convention and be added deliberately
- **ADRs** (`docs/adr/*.md`): Architectural decisions (see [ADR template](docs/templates/ADR-TEMPLATE.md))
- **Policy files** (`policies/*.rego`): Security/governance rules (see [controls-as-code](docs/explanation/controls-as-code.md))
- **Infrastructure** (`k8s/*.yaml`, Nix configs): Deployment concerns
- **Schema version changes**: Breaking changes to spec_ledger schema
- **CI workflows** (`.github/workflows/`): Build/deploy automation
- **Dependencies** (`Cargo.toml`): Version upgrades or new crates

---

## Bundle Configuration

The template uses **contextpack** (`.llm/contextpack.yaml`) to generate governed bundles:

```yaml
# Template Version: v2.4.0
# Schema Version: 1.0

tasks:
  implement_ac:
    description: "Context for implementing a single AC end-to-end"
    max_bytes: 250000  # ~250KB limit
    output: .llm/bundle/implement_ac.md

    include_globs:
      # Specs and features
      - specs/spec_ledger.yaml
      - specs/features/**/*.feature

      # Core business logic
      - crates/business-core/src/**/*.rs

      # HTTP adapter
      - crates/app-http/src/**/*.rs

      # Acceptance tests
      - crates/acceptance/**/*.rs

      # Docs
      - docs/explanation/*.md
      - docs/how-to/*.md
      - README.md
```

### Why Bounded Context?

- **Prevents token overflow**: LLMs have context limits
- **Focuses attention**: Only relevant files included
- **Maintains consistency**: Automatic file selection based on recency
- **Enables governance**: Bundle metadata includes version/schema info

---

## Workflow Integration

### Using Claude Code CLI

**Always work inside the Nix devshell for full validation:**

```bash
# Enter development environment
nix develop

# Generate bundle
cargo run -p xtask -- bundle implement_ac

# Open in editor with LLM context
code .llm/bundle/implement_ac.md

# Paste standard prompt, let LLM propose changes

# Validate (includes policy tests)
cargo run -p xtask -- selftest

# Commit when green
git add .
git commit -m "feat(core): implement AC-XXX-YYY - [description]"
```

### Using Cursor or Similar

**Run inside `nix develop` for CI-equivalent validation:**

1. Enter devshell: `nix develop`
2. Generate bundle: `cargo run -p xtask -- bundle implement_ac`
3. Open `.llm/bundle/implement_ac.md` in Cursor
4. Use Cursor's chat with the standard prompt
5. Apply suggested changes
6. Run `cargo run -p xtask -- selftest` in terminal (all 5 steps pass)
7. Commit when passing

### CI Integration

In CI (GitHub Actions), the workflow:
1. Runs `cargo run -p xtask -- selftest` (including policy tests via Nix+conftest)
2. Fails if any step fails (fmt, clippy, tests, BDD, policies)
3. Blocks merge if selftest fails

Locally, policy tests are skipped if `conftest` is not installed (shows ⚠️ warning only).

---

## Policy Tests (Local vs CI)

The template includes **Open Policy Agent (OPA)** policies in `policies/`:

- `ledger_governance.rego`: AC structure validation
- `feature_coverage.rego`: BDD feature requirements
- `k8s_standards.rego`: Kubernetes manifest validation
- `privacy_compliance.rego`: PII/sensitive data rules

**Behavior**:
- **Locally**: If `conftest` is not installed, selftest shows `⚠ Policy tests skipped` but still passes
- **CI**: Policy tests enforced via Nix environment (which provides `conftest`), failures block merge

### Running Policy Tests Locally (Optional)

You have two options to enable policy tests in `xtask selftest`:

#### Option 1: Use the Nix devshell (Recommended)

The flake already includes `conftest 0.52.0`, so just run selftest inside the devshell:

```bash
nix develop
cargo run -p xtask -- selftest
# Policy tests will now run in step 5 ✓
```

This gives you the same toolchain as CI with zero extra installs.

#### Option 2: Install conftest globally on WSL/Ubuntu

If you prefer to run `cargo xtask selftest` without `nix develop`:

```bash
# 1. Check what version CI uses
nix develop -c conftest --version
# Output: Conftest: 0.52.0

# 2. Install that version
CONFTEST_VERSION="0.52.0"
ARCH="$(uname -m)"
SYSTEM="$(uname -s)"
wget "https://github.com/open-policy-agent/conftest/releases/download/v${CONFTEST_VERSION}/conftest_${CONFTEST_VERSION}_${SYSTEM}_${ARCH}.tar.gz"
tar xzf "conftest_${CONFTEST_VERSION}_${SYSTEM}_${ARCH}.tar.gz"
chmod +x conftest
sudo mv conftest /usr/local/bin/conftest

# 3. Verify
conftest --version
# Should show: Conftest: 0.52.0
```

After installation, `cargo run -p xtask -- selftest` will run policy tests automatically.

**Note**: Without `conftest`, selftest still passes locally (with ⚠️ warning). CI always enforces policies.

---

## Troubleshooting

### Selftest Fails

```bash
# Check which step failed
cargo run -p xtask -- selftest

# Run steps individually
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p acceptance
cargo run -p xtask -- ac-status
cargo run -p xtask -- bundle implement_ac
```

### Bundle Too Large

Contextpack automatically limits to `max_bytes`. If you need more context:

1. Be more selective: edit `.llm/contextpack.yaml` to include only needed files
2. Split task: use multiple smaller bundles (e.g., `bundle_core` + `bundle_http`)
3. Review: check `.llm/bundle/implement_ac.md` to see which files were selected

### AC Mapping Broken

```bash
# Check AC status
cargo run -p xtask -- ac-status

# Common issues:
# - AC ID in spec_ledger.yaml doesn't match feature tag
# - Feature file path in spec_ledger.yaml is wrong
# - Scenario name doesn't match

# Example fix in spec_ledger.yaml:
acceptance_criteria:
  - id: AC-TPL-001
    feature_file: specs/features/template_core.feature  # ← must match actual path
    scenario: Template provides hexagonal architecture   # ← must match scenario name
```

### BDD Scenarios Fail

```bash
# Run with verbose output
cargo run -p acceptance -- --tags @template-core

# Check which steps failed
# Fix step definitions in acceptance/tests/steps/
# Or update scenario in specs/features/

# Re-run selftest
cargo run -p xtask -- selftest
```

---

## Examples

### Example 1: Implement Metrics Endpoint (AC-TPL-004)

```bash
# 1. Generate context
cargo run -p xtask -- bundle implement_ac

# 2. Review AC in specs/spec_ledger.yaml
# AC-TPL-004: Metrics Endpoint Returns Prometheus Format

# 3. Use standard prompt with LLM, pointing to AC-TPL-004

# 4. LLM proposes changes:
#    - Update app-http/src/routes/metrics.rs
#    - Update acceptance/tests/steps/metrics_steps.rs
#    - Ensure OpenAPI doc in specs/openapi/openapi.yaml

# 5. Apply changes

# 6. Validate
cargo run -p xtask -- selftest
# Output: ✓ All self-tests passed!

# 7. Commit
git commit -m "feat(metrics): implement AC-TPL-004 - prometheus endpoint"
```

### Example 2: Fix Failing Health Check

```bash
# 1. Notice failure
cargo run -p acceptance
# Output: scenario "Health check returns 200 OK" failed

# 2. Generate context
cargo run -p xtask -- bundle implement_ac

# 3. Use "Fix a Failing BDD Scenario" prompt with error output

# 4. LLM identifies issue in app-http/src/routes/health.rs

# 5. Apply fix

# 6. Verify
cargo run -p xtask -- selftest
# Output: ✓ All self-tests passed!

# 7. Commit
git commit -m "fix(health): correct status code in health endpoint"
```

---

## Version Metadata

All bundles include version headers:

```markdown
<!-- LLM Context Bundle -->
<!-- Template Version: v2.4.0 -->
<!-- Schema Version: 1.0 -->
<!-- Generated: 2025-11-19T... -->
```

This ensures the LLM knows:
- Which template version it's working with
- What schema/structure to expect
- When the bundle was generated (freshness)

See `docs/explanation/template-versioning.md` for the versioning scheme.

---

## Pilot Workflow

When creating a new service from the template:

**Option 1: GitHub "Use this template"**

```bash
# 1. In GitHub UI, click "Use this template" → create repo → clone locally
git clone git@github.com:your-org/my-service.git
cd my-service

# 2. Enter dev environment
nix develop

# 3. Verify template works
cargo run -p xtask -- selftest
```

**Option 2: Manual git clone**

```bash
# 1. Clone and reset git
git clone git@github.com:EffortlessMetrics/Rust-Template.git my-service
cd my-service
rm -rf .git
git init
git remote add origin git@github.com:your-org/my-service.git

# 2. Enter dev environment
nix develop

# 3. Verify template works
cargo run -p xtask -- selftest
```

**Development Loop (both options):**

```bash
# 4. Plan features
# - Review PILOT_FEATURE_IDEAS.md (if exists)
# - Choose 3-5 features
# - Add ACs to specs/spec_ledger.yaml

# 5. For each feature:
cargo run -p xtask -- bundle implement_ac
# Use LLM with "Create a Pilot Project" prompt
# Apply changes
# Run selftest
# Log friction in FRICTION_LOG.md

# 6. After 1-2 weeks, review FRICTION_LOG.md
# Feed findings back to template maintainer
```

---

## Best Practices

### ✅ Do

- Always generate a fresh bundle before asking the LLM
- Run `selftest` after every LLM-proposed change
- Keep commits atomic (one AC or fix per commit)
- Log friction in `FRICTION_LOG.md` during pilot projects
- Follow existing patterns (error handling, telemetry, hexagonal arch)
- Update OpenAPI specs when adding/changing endpoints
- Keep BDD scenarios in sync with code

### ❌ Don't

- Let the LLM invent new AC IDs without human review
- Skip selftest before committing
- Change policy files without understanding governance impact
- Modify infrastructure (k8s, Nix, CI) without careful review
- Commit failing tests with a plan to "fix later"
- Break hexagonal architecture boundaries (e.g., DB logic in HTTP handlers)

---

## Resources

### Tutorials and How-Tos

- **Tutorial**: `docs/tutorials/first-ac-change.md` - walkthrough of implementing your first AC
- **How-to**: `docs/how-to/new-service-from-template.md` - creating a new service

### Architectural Decisions (ADRs)

- **ADR Index**: `docs/adr/` - all architectural decision records
- **ADR Template**: `docs/templates/ADR-TEMPLATE.md` - template for new ADRs
- **Key ADRs**:
  - [ADR-0001: Hexagonal Architecture](docs/adr/0001-hexagonal-architecture.md)
  - [ADR-0002: Nix-First Dev Environment](docs/adr/0002-nix-first-dev-env.md)
  - [ADR-0003: Spec and BDD as Source of Truth](docs/adr/0003-spec-and-bdd-as-source-of-truth.md)
  - [ADR-0004: Policy and LLM Governance](docs/adr/0004-policy-and-llm-governance.md)
  - [ADR-0005: Selftest as Single Gate](docs/adr/0005-xtask-selftest-single-gate.md)
  - [ADR-0006: Supply Chain Hardening](docs/adr/0006-supply-chain-hardening.md)

### Explanations and Reference

- **Explanation**: `docs/explanation/hexagonal-architecture.md` - architecture principles
- **Explanation**: `docs/explanation/controls-as-code.md` - policy governance
- **Explanation**: `docs/explanation/supply-chain-hardening.md` - SBOM and provenance
- **Reference**: `docs/reference/xtask-commands.md` - all xtask commands
- **Testing**: `docs/testing-strategy.md` - complete test layer documentation

### Templates for Services

- **Service Metadata**: `docs/templates/SERVICE_METADATA.example.yaml` - service self-description
- **Runbook**: `docs/templates/RUNBOOK.example.md` - operations guide template

### Changelog and Plans

- **Changelog**: `CHANGELOG.md` - version history
- **Release Plans**: `docs/v2.*.md` - per-version release planning and retrospectives

---

## Summary

This template is **LLM-friendly by design**:

1. 📋 **Spec-as-Code** keeps features traceable
2. 🤖 **Contextpack bundles** give bounded, governed context
3. ✅ **Selftest** validates everything before commit
4. 🎯 **Standard prompts** guide LLM behavior
5. 🛡️ **Policies** enforce governance in CI

The golden path is simple:
```bash
cargo run -p xtask -- bundle implement_ac  # Generate context
# [Use LLM with standard prompt]
cargo run -p xtask -- selftest              # Validate
git commit                                  # Commit when green
```

**Next step**: Run a pilot project (use GitHub "Use this template" or manual git clone as shown in [Pilot Workflow](#pilot-workflow)) and track real-world friction in `FRICTION_LOG.md`.

---

**Last updated**: 2025-11-19
**Template version**: v2.4.0
