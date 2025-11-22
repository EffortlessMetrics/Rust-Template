---
name: bootstrap-dev-env
description: "One-command environment setup and health check for the Rust-as-Spec platform cell. Use when first entering the repository, after major environment changes, or when the environment appears broken. Follows the 'onboarding' flow from \n_flows.yaml.\n"
allowed-tools:
- Read
- Grep
- Glob
- Bash
---

# Bootstrap Development Environment

## When to Use

Use this Skill when:

- First time in the repository
- After major environment changes (tool updates, Rust version changes, config changes)
- Environment appears broken (commands failing, dependencies missing)
- User says "setup environment", "fix environment", or "get started"

## Prerequisites

- Git repository cloned
- Basic shell access (bash/zsh)
- Internet connectivity for downloading dependencies

## Workflow

This Skill follows the **onboarding** flow from `specs/devex_flows.yaml`.

### 1. Run one-command setup

```bash
cargo xtask dev-up
```

**What this does:**

- Validates Rust toolchain (rustc, cargo)
- Checks for optional tools (Nix, conftest)
- Installs git hooks (pre-commit governance)
- Runs core quality checks (fmt, clippy, tests)
- Runs BDD acceptance tests
- Starts platform HTTP server
- Shows status and next steps

**Expected output:**

```text
✅ Environment validated
✅ Git hooks installed
✅ Core checks passed
✅ BDD tests passed
✅ Platform running at http://localhost:3000

Next steps:
  - View tasks: http://localhost:3000/ui/tasks
  - Check status: cargo xtask status
  - Platform API: http://localhost:3000/platform/status
```

### 2. Verify platform is running

```bash
# Check platform health
curl http://localhost:3000/platform/status | jq

# Expected response:
# {
#   "status": "healthy",
#   "version": "...",
#   "governance": {
#     "total_requirements": N,
#     "total_acs": M,
#     "total_tasks": K
#   }
# }
```

### 3. Explore available workflows

```bash
# See what you can do
cargo xtask help-flows
```

**Output:** Categorized command map showing onboarding, feature development, maintenance, and release workflows.

### 4. Check governance status

```bash
cargo xtask status
```

**Shows:**

- Environment health
- Task counts by status
- Policy compliance
- Next recommended actions

### 5. Browse task board

Open in browser:

```bash
open http://localhost:3000/ui/tasks
# or: xdg-open http://localhost:3000/ui/tasks (Linux)
# or: start http://localhost:3000/ui/tasks (Windows)
```

## Exit Criteria

Environment is ready when:

- ✅ `cargo xtask dev-up` exits with code 0
- ✅ `GET /platform/status` returns 200 with `"status": "healthy"`
- ✅ All URLs in output are reachable
- ✅ `cargo xtask status` shows no critical issues

## Error Handling

### If dev-up fails at environment validation

```bash
# Run detailed diagnostics
cargo xtask doctor
```

**Fix common issues:**

- **Rust too old:** Update via `rustup update`
- **conftest missing:** Install via Nix (`nix profile install nixpkgs#conftest`) or follow <https://www.conftest.dev/install/>
- **Git not configured:** Set `user.name` and `user.email`

### If dev-up fails at checks

```bash
# Run checks individually
cargo xtask check
# Fix any reported issues (fmt, clippy, test failures)
```

### If dev-up fails at BDD

```bash
# Run BDD with verbose output
cargo xtask bdd
# Check for missing step definitions or feature syntax errors
```

### If platform doesn't start

```bash
# Check if port 3000 is already in use
lsof -i :3000
# Kill existing process or change port in .env
```

## Examples

### Example 1: First-time setup

```bash
# Clone repository
git clone <repo-url>
cd rust-as-spec-template

# Bootstrap environment
cargo xtask dev-up

# ✅ Environment ready, platform running
# Open task board
open http://localhost:3000/ui/tasks
```

### Example 2: Fixing broken environment

```bash
# User reports: "xtask commands not working"

# Run bootstrap
cargo xtask dev-up

# Output shows: "✗ conftest not found"
# Install conftest
nix profile install nixpkgs#conftest

# Re-run
cargo xtask dev-up
# ✅ All checks pass
```

### Example 3: After pulling major changes

```bash
# Pull latest from main
git pull origin main

# Refresh environment
cargo xtask dev-up

# Ensures dependencies are updated, hooks reinstalled, platform restarted
```

## Success Criteria

Bootstrap complete when:

- ✅ `cargo xtask dev-up` passes all steps
- ✅ Platform API responding at <http://localhost:3000>
- ✅ Task board accessible at <http://localhost:3000/ui/tasks>
- ✅ `cargo xtask status` shows healthy state
- ✅ Developer can see next steps clearly

## References

- **Flow definition:** `specs/devex_flows.yaml` (onboarding flow)
- **Detailed commands:** `docs/reference/xtask-commands.md`
- **Operational guide:** `docs/AGENT_GUIDE.md`
- **Platform APIs:** <http://localhost:3000/platform/status>

## Notes

- **dev-up is idempotent:** Safe to run multiple times
- **Platform auto-starts:** No need to manually run `cargo run -p app-http`
- **Git hooks are governed:** They enforce pre-commit selftest checks
- **First run is slower:** Cargo downloads and compiles dependencies
