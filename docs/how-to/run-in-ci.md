---
id: GUIDE-TPL-RUN-IN-CI-001
doc_type: how_to
title: "Running xtask Commands in CI/CD Pipelines"
status: published
audience: platform-engineers, ci-maintainers
tags: [ci, automation, devex, governance]
stories: [US-TPL-PLT-001]
requirements:
  - REQ-PLT-DEVEX-CONTRACT
acs: []
adrs: []
last_updated: 2025-12-01
---

# Running xtask Commands in CI/CD Pipelines

This guide documents how to run xtask commands in CI/CD environments (GitHub Actions, GitLab CI, Jenkins, etc.) with proper non-interactive mode configuration.

## Environment Variables

The xtask system recognizes several environment variables that affect behavior in CI:

### `CI`

**Purpose:** Standard CI detection flag recognized by most CI systems.

**Effect:**
- Treats missing `conftest` as a **hard failure** (instead of warning)
- Enables stricter validation in `selftest`
- Automatically set by GitHub Actions, GitLab CI, CircleCI, etc.

**Example:**
```bash
CI=1 cargo xtask selftest
```

### `XTASK_NONINTERACTIVE`

**Purpose:** Explicit non-interactive mode flag.

**Effect:**
- Guarantees no prompts or interactive input
- Commands exit with non-zero codes on failure
- Useful for automation scripts outside traditional CI

**Example:**
```bash
XTASK_NONINTERACTIVE=1 cargo xtask doctor
```

### `XTASK_LOW_RESOURCES`

**Purpose:** Resource-constrained mode for limited CI runners.

**Effect:**
- Reduces parallelism (`CARGO_BUILD_JOBS=1`)
- Skips expensive checks (BDD suite, full format)
- Suitable for ARM runners, free-tier CI, or constrained environments

**Example:**
```bash
XTASK_LOW_RESOURCES=1 cargo xtask check
```

### `XTASK_SKIP_BDD`

**Purpose:** Internal circuit breaker to prevent nested BDD runs (harness use only).

**Effect:**
- Bypasses `conftest` and feature file execution
- Selftest cannot validate AC status or regenerate `docs/feature_status.md`
- Normal users should leave this unset; Tier-1 selftest is expected to run BDD

**Example:**
```bash
XTASK_SKIP_BDD=1 cargo xtask check
```

### `XTASK_CHANGED_BASE`

**Purpose:** Specify base branch for `test-changed` command.

**Effect:**
- Controls which commits are considered for change detection
- Defaults to `origin/main`

**Example:**
```bash
XTASK_CHANGED_BASE=origin/develop cargo xtask test-changed
```

---

## CI Platform Examples

### GitHub Actions

```yaml
name: CI

on: [push, pull_request]

env:
  CI: "1"
  XTASK_NONINTERACTIVE: "1"

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Nix
        uses: DeterminateSystems/nix-installer-action@main

      - name: Setup Nix cache
        uses: DeterminateSystems/magic-nix-cache-action@main

      - name: Run selftest
        run: nix develop --command cargo xtask selftest

  low-resource:
    runs-on: ubuntu-latest
    env:
      XTASK_LOW_RESOURCES: "1"
    steps:
      - uses: actions/checkout@v4
      - name: Quick check
        run: cargo xtask check
```

### GitLab CI

```yaml
variables:
  CI: "1"
  XTASK_NONINTERACTIVE: "1"

stages:
  - validate

selftest:
  stage: validate
  image: nixos/nix
  script:
    - nix develop --command cargo xtask selftest

quick-check:
  stage: validate
  variables:
    XTASK_LOW_RESOURCES: "1"
  script:
    - cargo xtask check
```

### Jenkins

```groovy
pipeline {
    agent any

    environment {
        CI = '1'
        XTASK_NONINTERACTIVE = '1'
    }

    stages {
        stage('Validate') {
            steps {
                sh 'cargo xtask selftest'
            }
        }
    }
}
```

---

## Debugging Environment Detection

Use `cargo xtask env-mode` to see how xtask detects your environment:

```bash
$ cargo xtask env-mode

Environment Mode
  Mode: interactive

Detection Flags
  is_ci:            false
  is_noninteractive: false
  is_low_resources: false
  should_skip_bdd:  false

Raw Environment Variables
  CI=               (unset)
  GITHUB_ACTIONS=   (unset)
  XTASK_NONINTERACTIVE= (unset)
  XTASK_LOW_RESOURCES=  (unset)
  XTASK_SKIP_BDD=       (unset)
  IN_NIX_SHELL=         "1"
```

With CI variables set:

```bash
$ CI=1 XTASK_LOW_RESOURCES=1 cargo xtask env-mode

Environment Mode
  Mode: CI (low-resources)

Detection Flags
  is_ci:            true
  is_noninteractive: true
  is_low_resources: true
  should_skip_bdd:  true
```

For machine-readable output:

```bash
$ cargo xtask env-mode --json
```

### Environment Mode Matrix

| Variables Set | Mode | BDD Skipped? |
|--------------|------|--------------|
| (none) | `interactive` | No |
| `CI=1` | `CI` | No |
| `XTASK_NONINTERACTIVE=1` | `non-interactive` | No |
| `XTASK_LOW_RESOURCES=1` | `low-resources` | Yes |
| `CI=1 XTASK_LOW_RESOURCES=1` | `CI (low-resources)` | Yes |
| `XTASK_SKIP_BDD=1` | `interactive` | Yes |

> **Note:** `XTASK_SKIP_BDD=1` only affects the `should_skip_bdd` flag; it does
> not change the reported mode. The mode remains `interactive` unless other
> environment variables (`CI`, `XTASK_NONINTERACTIVE`, `XTASK_LOW_RESOURCES`) are set.

---

## Commands and Their CI Behavior

| Command | CI Detection | Non-Interactive | Notes |
|---------|--------------|-----------------|-------|
| `doctor` | ✅ | ✅ | Runs without prompts, exits 0/1 |
| `check` | ✅ | ✅ | Respects `XTASK_LOW_RESOURCES` |
| `selftest` | ✅ | ✅ | Strict mode with `CI=1` |
| `ac-status` | ✅ | ✅ | Generates report silently |
| `docs-check` | ✅ | ✅ | Full validation, no prompts |
| `bundle` | ✅ | ✅ | Generates bundles silently |
| `env-mode` | ✅ | ✅ | Debug env detection (--json for CI) |

All commands:
- **Never prompt** for input when `CI` or `XTASK_NONINTERACTIVE` is set
- **Exit with code 0** on success
- **Exit with non-zero** on any failure
- **Write errors to stderr** for log capture

---

## Exit Codes

The xtask system uses standard Unix exit codes:

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General failure (tests failed, validation errors) |
| `2` | Missing dependencies (conftest, rustc, etc.) |
| `127` | Command not found (library loading issues) |

---

## Troubleshooting

### sccache Library Issues

If you see errors like:
```
error while loading shared libraries: libz.so.1
```

**Fix:** Set `RUSTC_WRAPPER=""` or ensure you're in the Nix shell:
```bash
RUSTC_WRAPPER="" cargo xtask selftest
# or
nix develop --command cargo xtask selftest
```

### Missing conftest

If `selftest` fails with conftest errors:

**In CI:** This is a hard failure. Install conftest or skip BDD:
```bash
XTASK_SKIP_BDD=1 cargo xtask check
```

**Locally:** Warning only. Install via:
```bash
nix develop  # Provides conftest
# or
brew install conftest  # macOS
```

### Hung Commands

If commands appear to hang in CI:
1. Ensure `CI=1` or `XTASK_NONINTERACTIVE=1` is set
2. Check for stdin redirection issues
3. Verify no interactive git operations

---

## Best Practices

1. **Always set `CI=1`** in CI pipelines (often automatic)
2. **Use `XTASK_NONINTERACTIVE=1`** for scripts and automation
3. **Use `XTASK_LOW_RESOURCES=1`** for free-tier runners
4. **Run in Nix shell** for full reproducibility
5. **Capture exit codes** to fail pipelines on errors
6. **Log stderr** for debugging validation failures

---

## Related Documentation

- [CI Workflows Reference](../reference/ci-workflows.md) - Detailed workflow configurations
- [CI Examples](../ci-examples.md) - Greenfield and brownfield CI templates
- [TROUBLESHOOTING](../TROUBLESHOOTING.md) - Common issues and fixes
- [DevEx Flows](../../specs/devex_flows.yaml) - Available xtask commands
