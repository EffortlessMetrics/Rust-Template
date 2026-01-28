---
id: HOWTO-KERNEL-DRY-RUN-001
title: How to Conduct a Kernel Dry-Run
doc_type: how_to
status: published
# doclint:disable orphan-version
audience: maintainers, adopters, qa
tags: [dry-run, validation, fork, adoption, testing]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-ONBOARDING, REQ-PLT-DEVEX-CONTRACT]
acs: [AC-PLT-ENV-CHECK, AC-PLT-015]
adrs: [ADR-0005]
last_updated: 2025-12-22
---

# How to Conduct a Kernel Dry-Run

> **For kernel maintainers and adopters:** This guide explains how to validate that the kernel is fork-ready by conducting a systematic dry-run.

**Kernel Version:** v3.3.9-kernel

---

## What is a Kernel Dry-Run?

A **dry-run** is a structured validation exercise that proves:

1. A new adopter can fork the template following only the docs
2. The kernel reaches selftest-green immediately after checkout
3. Service identity can be changed without breaking governance
4. Platform introspection works with the new identity

It's the "acceptance test" for the kernel's adoptability.

---

## When to Conduct a Dry-Run

| Scenario | Recommended |
|----------|-------------|
| **Before tagging a new kernel release** | Required |
| **After major kernel changes** | Required |
| **Before first adoption by a new team** | Recommended |
| **Quarterly health check** | Recommended |
| **After environment changes (Nix, Rust)** | Optional |

---

## Prerequisites

Before conducting a dry-run:

1. **Clean environment** – Fresh terminal, no cached state
2. **Kernel tag exists** – The version you're testing is tagged
3. **Receipt template** – Copy of `docs/receipts/FORK_DRY_RUN_TEMPLATE.md`
4. **Time allocation** – Allow 30-60 minutes for full dry-run

---

## Dry-Run Procedure

### Step 1: Setup

Create a new directory for the dry-run (don't use an existing clone):

```bash
# Create isolated workspace
mkdir -p ~/dry-runs
cd ~/dry-runs

# Clone fresh
git clone https://github.com/EffortlessMetrics/Rust-Template.git dry-run-$(date +%Y%m%d)
cd dry-run-$(date +%Y%m%d)

# Checkout the kernel tag
git checkout v3.3.9-kernel
```

**Record in receipt:**
- Clone time
- Any network or auth issues

---

### Step 2: Environment Bootstrap

```bash
# Enter Nix devshell
nix develop

# Run one-command setup
cargo xtask dev-up

# Verify full selftest
cargo xtask selftest
```

**Expected outcome:**
- `dev-up` completes without errors
- All 11 selftest steps pass
- No manual intervention needed

**Record in receipt:**
- Total time for bootstrap
- Any warnings or issues
- Selftest gate results (all 12 steps)

---

### Step 3: Identity Change

Test that a fork can rebrand without breaking governance:

```bash
# Change service identity
cargo xtask service-init \
  --id my-test-service \
  --name "My Test Service" \
  --description "Dry-run validation service" \
  --tags test validation

# Verify selftest still passes
cargo xtask selftest
```

**Expected outcome:**
- `service-init` updates all identity files
- Selftest remains green after identity change
- No manual file edits required

**Record in receipt:**
- Files changed by service-init
- Any files needing manual edits
- Selftest result after identity change

---

### Step 4: Introspection Validation

Start the service and verify platform APIs reflect new identity:

```bash
# Start service
cargo run -p app-http &
sleep 3

# Check platform status
curl -s http://localhost:8080/platform/status | jq '.service'

# Check docs index
curl -s http://localhost:8080/platform/docs/index | jq '.docs | length'

# Check tasks
curl -s http://localhost:8080/platform/tasks | jq '.tasks | length'

# Visit UI
echo "Open http://localhost:8080/ui in browser"
```

**Expected outcome:**
- `/platform/status` shows new service_id
- `/platform/docs/index` returns valid doc inventory
- `/ui` displays correctly with new identity

**Record in receipt:**
- API response validation
- UI screenshot or notes
- Any discrepancies

---

### Step 5: Domain Smoke Test

Verify domain-specific functionality still works:

```bash
# Check example endpoints (MyService todos)
curl -s http://localhost:8080/todos | jq

# Verify domain ACs still pass
cargo xtask ac-status | grep MYSERV
```

**Expected outcome:**
- Domain endpoints respond correctly
- Domain ACs show `[PASS]`

**Record in receipt:**
- Domain endpoint results
- AC status for domain ACs

---

### Step 6: Cleanup

```bash
# Stop service
kill %1 2>/dev/null || true

# Return to original directory
cd ~/dry-runs

# Optionally remove dry-run directory
rm -rf dry-run-$(date +%Y%m%d)
```

---

## Interpreting Results

### Success Criteria

The kernel is **fork-ready** when:

| Criterion | Threshold |
|-----------|-----------|
| Selftest green on checkout | 12/12 steps pass |
| Selftest green after service-init | 12/12 steps pass |
| Manual edits required | ≤ 3 files |
| Platform APIs working | All return valid JSON |
| UI reflects identity | New service name shown |
| Time to complete | < 30 minutes |

### Interpreting Gaps

| Gap Type | Severity | Action |
|----------|----------|--------|
| **Selftest fails on checkout** | Critical | Block release until fixed |
| **service-init misses files** | High | Fix service-init, retest |
| **Docs unclear for a step** | Medium | Update docs, note in friction |
| **Minor UI glitches** | Low | File issue, don't block |

---

## Recording Results

### Use the Receipt Template

Copy `docs/receipts/FORK_DRY_RUN_TEMPLATE.md` to:

```bash
cp docs/receipts/FORK_DRY_RUN_TEMPLATE.md docs/receipts/FORK_DRY_RUN_$(date +%Y-%m-%d).md
```

Fill in all sections as you complete each step.

### Filing Friction Entries

For each gap discovered:

```bash
# Create friction entry
cargo xtask friction-new \
  --category kernel \
  --severity [low|medium|high] \
  --summary "Gap discovered in dry-run: [description]"
```

### Feeding Back to Kernel

If gaps require kernel changes:

1. File a GitHub issue with the friction ID
2. Reference the dry-run receipt
3. Propose a fix or doc update
4. Tag for next kernel release

---

## Automation (Future)

The dry-run is currently manual, but could be automated:

```yaml
# .github/workflows/dry-run.yml (example)
name: Kernel Dry-Run
on:
  workflow_dispatch:
  schedule:
    - cron: '0 0 1 * *'  # Monthly

jobs:
  dry-run:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          ref: v3.3.9-kernel
      - uses: DeterminateSystems/nix-installer-action@main
      - run: nix develop --command cargo xtask dev-up
      - run: nix develop --command cargo xtask selftest
      - run: |
          nix develop --command cargo xtask service-init \
            --id ci-dry-run --name "CI Dry Run" \
            --description "Automated dry-run"
      - run: nix develop --command cargo xtask selftest
```

---

## Related Documentation

- **[FORK_DRY_RUN_TEMPLATE.md](../receipts/FORK_DRY_RUN_TEMPLATE.md)** – Receipt template
- **[FIRST_FORK.md](./FIRST_FORK.md)** – Full fork setup guide
- **[adopt-kernel.md](./adopt-kernel.md)** – Kernel adoption guide
- **[trust-a-cell.md](./trust-a-cell.md)** – Cell trust validation
- **[maintain-kernel.md](./maintain-kernel.md)** – Kernel maintenance
