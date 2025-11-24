# Agent Guidelines: Rust-as-Spec Platform Cell

**You are a team member working in a self-governing platform cell.**

---

## Core Directive

**Do not guess workflows.** This repository has formal governance contracts that must be followed.

Your authority comes from:
1. **Skills** (`.claude/skills/*`) - Defined workflows
2. **Platform APIs** (`/platform/*`) - System state
3. **xtask commands** - Validated operations
4. **Selftest** - Ground truth validation

---

## Available Skills

Use skills from `.claude/skills/` to execute governed workflows:

### 1. Feature Development
**Skill:** `governed-feature-dev`  
**When:** Adding new features, implementing requirements  
**Contract:** AC → BDD → Code → Selftest

### 2. Release Management
**Skill:** `governed-release`  
**When:** Cutting versions, tagging releases  
**Contract:** Prepare → Verify → Tag → CI

### 3. Maintenance
**Skill:** `governed-maintenance`  
**When:** Fixing environment, updating deps, diagnosing drift  
**Contract:** Doctor → Audit → Docs → Graph → Selftest

---

## Operational Commands

### Discovery
```bash
# What tasks are available?
cargo xtask tasks-list

# What should I do next?
cargo xtask suggest-next --task <id>

# What workflows exist?
cargo xtask help-flows
```

### Platform State
```bash
# Check health
curl http://localhost:8080/platform/status

# View governance graph
curl http://localhost:8080/platform/graph

# Browse visually
open http://localhost:8080/ui
```

### Bounded Context
```bash
# Get relevant context for a task
cargo xtask bundle <task-id>
# Output: .llm/bundle/<task-id>.md (max 250KB)
```

### Selective Testing (Fast Iteration)
```bash
# Test only what changed (fast)
cargo xtask test-changed

# Test specific acceptance criterion
cargo xtask test-ac AC-PLT-001

# Compare against different base
cargo xtask test-changed --base main
```

### Validation
```bash
# Quick check (fmt, clippy, unit tests)
cargo xtask check

# Check AC test coverage
cargo xtask ac-coverage

# Full governance validation (use Tier-1: Nix+Linux/WSL2)
cargo xtask selftest
```

**Performance Note:**
- On **native Windows** (Tier-2): Use `test-changed` and `test-ac` for fast iteration. Reserve `selftest` for CI or WSL2.
- On **Nix+Linux/WSL2** (Tier-1): All commands run efficiently. Use `selftest` as pre-merge gate.

See `docs/SELECTIVE_TESTING.md` for complete guide.

---

## The Golden Rule

**If `cargo xtask selftest` fails, you are not done.**

Selftest is the single source of truth for "is this work acceptable?"

It validates:
1. Core checks (fmt, clippy, tests)
2. BDD (behavior matches ACs)
3. AC mapping (traceability)
4. LLM bundler (context generation)
5. Policy tests (compliance)
6. DevEx contract (commands exist)
7. Graph invariants (structural integrity)

**Never:**
- ❌ Bypass selftest
- ❌ Claim work is complete without running `ac-coverage` (kernel ACs must be green)
- ❌ Force-merge failing work
- ❌ Hand-edit YAML specs (use `xtask ac-new`, `adr-new`, etc.)
- ❌ Guess at workflows (check skills first)

---

## Decision Boundaries

### You Can Do Autonomously
- ✅ Implement ACs with clear specs
- ✅ Fix failing tests
- ✅ Update docs to match code
- ✅ Run maintenance commands (audit, doctor)

### Requires Human Review
- ⚠️ **High-risk tasks** (REQ-TPL-AGENT-INTERFACE defines risk levels)
- ⚠️ **Architecture decisions** (need ADR)
- ⚠️ **Security changes** (auth, crypto, secrets)
- ⚠️ **Graph invariant changes** (e.g., removing `must_have_ac`)
- ⚠️ **Policy modifications** (`.rego` files)

### Never Do
- ❌ **Bypass governance gates**
- ❌ **Make breaking changes without approval**
- ❌ **Invent new workflows** (use defined skills)

---

## Orientation for First-Time Systems

When you first encounter this repository:

1. **Read strategic docs:**
   - `docs/ROADMAP.md` - Current state and pilot plan
   - `docs/AGENT_GUIDE.md` - Detailed operational guide
   - `docs/MISSING_MANUAL.md` - Operational realities

2. **Check platform health:**
   ```bash
   cargo xtask doctor
   curl http://localhost:8080/platform/status
   ```

3. **Discover tasks:**
   ```bash
   cargo xtask tasks-list
   ```

4. **Before doing any work:**
   - Confirm task exists or create AC via `ac-new`
   - Use `suggest-next` to get step-by-step guidance
   - Follow the appropriate skill workflow

---

## Example Interaction

**User:** "Add an endpoint to list users"

**Your workflow:**
```bash
# 1. Check if AC exists
grep -i "list users" specs/spec_ledger.yaml

# 2. If not, ask user for details and create:
cargo xtask ac-new AC-MYSERV-USERS-LIST \
  "GET /users returns list of users" \
  --story US-MYSERV-001 \
  --requirement REQ-MYSERV-USERS

# 3. Follow governed-feature-dev skill:
cargo xtask bundle implement_ac   # Get context
# Write BDD scenario
# Implement code
cargo xtask bdd                   # Test
cargo xtask selftest              # Validate

# 4. Report status to user
```

---

## Further Reading

- **Operational Guide:** `docs/AGENT_GUIDE.md`
- **Technical Architecture:** `docs/explanation/rust-as-spec-overview.md`
- **Platform APIs:** `http://localhost:8080/platform/status`
- **Skills:** `.claude/skills/*/SKILL.md`

---

## Remember

This is not a normal codebase. It's a **self-governing cell** where:
- Specs are contracts, not documentation
- Tests validate governance, not just features
- Drift is a build failure, not technical debt

Your job is to work **within** these contracts, not around them. The skills and platform APIs make this ergonomic. Use them.
