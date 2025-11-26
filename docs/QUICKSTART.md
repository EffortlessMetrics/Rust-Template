---
id: GUIDE-TPL-QUICKSTART-001
title: Quick Start Guide
doc_type: guide
status: published
audience: developers
tags: [onboarding, quickstart, setup, tutorial]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-ONBOARDING, REQ-PLT-DEVEX-CONTRACT]
acs: [AC-PLT-001, AC-PLT-015, AC-PLT-019]
adrs: [ADR-0002, ADR-0005]
last_updated: 2025-11-26
---

# Quick Start Guide

**Goal:** Get productive in under 15 minutes.

---

## Defaults & Opinions

**This template is opinionated by design.** It's not a generic Rust starter—it encodes specific beliefs about how governed, agent-friendly service development should work.

### Key Opinions (Encoded as Acceptance Criteria)

The template enforces these opinions through contracts in `specs/spec_ledger.yaml`:

1. **Environment: Nix-First (Tier-1)**
   - Nix with flakes provides the canonical development environment
   - Native Windows/macOS are supported as Tier-2 (functional but slower)
   - CI uses Nix to guarantee parity between local and remote validation
   - **Why:** Eliminates "works on my machine" by making the environment reproducible

2. **CI Gate: `cargo xtask selftest` Required**
   - All PRs must pass an 8-step governance validation
   - Includes: code quality, BDD scenarios, policy tests, graph integrity
   - Selftest passing means "governed and ready to review"
   - **Why:** Prevents drift between specs, tests, docs, and code

3. **Governance Artifacts: First-Class Citizens**
   - Questions (`QUESTIONS.yaml`), friction (`FRICTION_LOG.md`), and forks (`FORKS.yaml`) are tracked artifacts
   - Exposed via HTTP APIs (`/platform/questions`, `/platform/friction`, `/platform/forks`)
   - CLI commands with `--json` output for machine consumption
   - Governance artifacts support `refs` field for REQ-*/AC-* IDs (AC-TPL-ARTIFACTS-HAVE-REFS)
   - **Why:** Governance data is as important as the code—make it queryable

4. **BDD Harness: Exit Codes for Automation (AC-TPL-BDD-EXIT-CODES)**
   - BDD test harness returns exit 0 when all non-@wip scenarios pass
   - Skipped scenarios (@skip) don't affect exit code
   - Returns non-zero only if at least one non-@wip scenario fails
   - **Why:** CI/CD needs reliable, deterministic test outcomes

5. **Fork Visibility: Platform Status Integration (AC-TPL-FORKS-STATUS-SUMMARY)**
   - Fork registry visible in `/platform/status` (governance.forks.total, forks.ids)
   - `cargo xtask fork-list --json` reflects fork registry state
   - **Why:** IDPs and agents need to discover known forks without reading raw YAML

6. **Agent Surfaces: APIs, Bundles, JSON CLI**
   - Platform introspection via `/platform/*` endpoints (status, graph, docs, hints)
   - Context bundles for LLM workflows (`cargo xtask bundle <task>`)
   - Core reporting commands support `--json` for programmatic use (version, friction-list, questions-list, fork-list)
   - **Why:** Agents and platform tooling need structured, machine-readable interfaces

### How to Customize

**Don't edit config files randomly.** Opinions are encoded as Acceptance Criteria (ACs) in `specs/spec_ledger.yaml`.

To change an opinion:

1. Identify the relevant AC(s) in the spec ledger
2. Modify or remove the AC and its associated BDD scenarios
3. Update implementation to match your preference
4. Run `cargo xtask selftest` to verify consistency

For step-by-step guidance, see:
- **Add your AC**: `docs/how-to/add-acceptance-criterion.md`
- **Fork for your service**: `docs/how-to/new-service-from-template.md`
- **Report upstream**: `docs/how-to/report-fork-feedback.md`

**Example:** If you don't want Nix, remove ACs related to `REQ-PLT-NIX-DEVSHELL`, update CI workflows, and document your alternative environment in a new AC.

---

## Prerequisites

**Required:** [Nix with flakes](https://nixos.org/download.html) (for Tier-1 environment matching CI)
**Optional:** Docker, WSL2 (if on Windows), VS Code (recommended editor)

**Note:** You can develop without Nix (Tier-2), but policy tests and exact CI parity require it. See [Platform Support](reference/platform-support.md).

---

## Setup (3 Commands)

```bash
# Clone
git clone https://github.com/EffortlessMetrics/Rust-Template.git my-service && cd my-service

# Enter Nix devshell (installs all tools, matches CI)
nix develop

# Bootstrap (hooks, checks, tests) - 3-5 minutes first run
cargo xtask dev-up
```

---

## Start the Service

```bash
cargo run -p app-http
# → http://localhost:8080/ui (dashboard)
# → http://localhost:8080/platform/status (API)

# Smoke test
curl http://localhost:8080/health
# Expected: {"status":"ok"}
```

---

## Run Tests

**During development** (fast):
```bash
cargo xtask check           # fmt + clippy + tests (~30s)
cargo xtask test-changed    # Test what you edited (seconds to minutes)
cargo xtask test-ac AC-PLT-001  # Test specific AC
```

**Before PR** (full validation):
```bash
cargo xtask selftest        # 8-step governance validation (10-20 min)
```

**Validation ladder:** `check` → `test-changed` → `test-ac` → `selftest`
See [SELECTIVE_TESTING.md](SELECTIVE_TESTING.md) for details.

---

## Editor Setup (VS Code)

VS Code integration is pre-configured in `.vscode/`:

**Recommended extensions** (auto-suggested on open):
- rust-analyzer (Rust language support)
- Even Better TOML (TOML syntax)
- crates (Cargo.toml dependency management)
- Docker (container support)
- Code Spell Checker (with repo dictionary)
- YAML (for spec files)

**Built-in tasks** (Ctrl+Shift+B / Cmd+Shift+B):
- `kernel: smoke` (default build) - Quick validation
- `dev: check` - Fast code quality check
- `dev: test-changed` - Test affected code
- `kernel: selftest` - Full governance validation

**Debug configurations** (F5):
- `Run app-http` - Launch the service with debugger
- `Debug unit tests` - Debug all tests
- `Debug current test` - Debug selected test

**Settings configured**:
- Format on save for Rust
- Clippy on save
- Spell checker with repo dictionary
- Search exclusions (target/, .llm/)

Open the command palette (Ctrl+Shift+P / Cmd+Shift+P) and type "Tasks: Run Task" to see all available tasks.

---

## Make Your First Change

### Example: Modify health check response

```bash
# 1. Find relevant AC
cargo xtask ac-coverage | grep HEALTH
# → ✅ AC-TPL-HEALTH-001: Service responds to /health

# 2. Edit code
# → crates/app-http/src/routes/health.rs

# 3. Test your change
cargo xtask test-changed

# 4. Validate
cargo xtask check
cargo xtask selftest  # Before PR
```

---

## Command Cheat Sheet

### Environment
```bash
cargo xtask doctor          # Check setup
cargo xtask status          # Governance dashboard
cargo xtask version         # Kernel version
```

### Development
```bash
cargo xtask check           # Fast validation
cargo xtask test-changed    # Test affected code
cargo xtask bdd             # Run BDD scenarios
cargo xtask selftest        # Full governance gate
```

### Specs & ACs
```bash
cargo xtask ac-coverage     # Show AC test coverage
cargo xtask ac-status       # Generate status report
cargo xtask ac-new <ID> "Desc" --requirement <REQ>
cargo xtask tasks-list      # Available work items
```

### Documentation
```bash
cargo xtask docs-check      # Validate docs
cargo xtask adr-new "Title" # New ADR
cargo xtask spellcheck      # Run spellcheck
```

### Governance
```bash
cargo xtask policy-test     # Run OPA/Rego policies
cargo xtask graph-export    # Export governance graph
cargo xtask audit           # Security audit
```

### LLM/Agent
```bash
cargo xtask bundle <task>             # Generate LLM context
cargo xtask suggest-next --task <id>  # Get workflow guidance
cargo xtask help-flows                # Available flows
```

---

## Key Concepts (2-Minute Primer)

**Acceptance Criteria (ACs):** Testable behaviors in `specs/spec_ledger.yaml`, verified by BDD scenarios tagged `@AC-XXX`.

**Spec Ledger:** `specs/spec_ledger.yaml` - Stories → Requirements → ACs → Tests → Docs. Single source of truth.

**Selftest:** `cargo xtask selftest` - 8-step validation (code quality + BDD + policies + graph integrity). Must be green before merge.

**Platform APIs:** `http://localhost:8080/platform/*` - `/status` (health), `/graph` (REQ/AC/Doc graph), `/tasks` (work items), `/agent/hints` (prioritized suggestions).

**Validation Ladder:**
1. `check` - Fast feedback (30s)
2. `test-changed` - Test affected code
3. `test-ac <ID>` - Test specific AC
4. `selftest` - Full gate (10-20 min)

---

## Troubleshooting

**`dev-up` fails with "conftest not found"**
→ Run `nix develop` first

**`selftest` slow (2+ hours on Windows)**
→ Expected on native Windows (Tier-2). Use WSL2 + Nix (Tier-1) for canonical validation.

**AC shows ❌ in `ac-coverage`**
→ No BDD tests wired. Run `cargo xtask ac-suggest-scenarios <AC-ID>`, add to `specs/features/*.feature`.

**Policy tests fail**
→ Run `cargo xtask policy-test` to see which policy failed. Common fixes:
- Add `tests` array to AC in spec_ledger.yaml
- Add ACs for `must_have_ac: true` requirements
- Fix typo in feature file tag

---

## Next Steps

**Humans:**
- Architecture: [why-this-exists.md](why-this-exists.md)
- Workflows: [AGENT_GUIDE.md](AGENT_GUIDE.md)
- Testing: [SELECTIVE_TESTING.md](SELECTIVE_TESTING.md)

**Agents:**
- Instructions: [CLAUDE.md](../CLAUDE.md)
- APIs: `/platform/agent/hints` for prioritized work
- Guidance: `cargo xtask suggest-next --task <id>`

**Common Tasks:**
- Add feature: [governed-feature-dev skill](.claude/skills/governed-feature-dev/SKILL.md)
- Fix drift: [governed-maintenance skill](.claude/skills/governed-maintenance/SKILL.md)
- Release: [governed-release skill](.claude/skills/governed-release/SKILL.md)

---

## Help

**In-repo:**
- `cargo xtask help` - All commands
- `cargo xtask doctor` - Environment check
- `http://localhost:8080/ui` - Web dashboard

**Docs:**
- **README.md** - Overview
- **CLAUDE.md** - Agent instructions
- **docs/AGENT_GUIDE.md** - Detailed workflows
- **docs/INDEX.md** - Full index

**Stuck?**
1. `cargo xtask doctor`
2. `cargo xtask status`
3. [MISSING_MANUAL.md](MISSING_MANUAL.md)

---

**You're ready!** Core commands learned. Environment working. Go build.
