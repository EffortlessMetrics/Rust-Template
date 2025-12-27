---
id: HOWTO-TRUST-CELL-001
title: How to Trust a Cell
doc_type: how_to
status: published
# doclint:disable orphan-version
audience: platform-engineers, idp-operators, integration-developers
tags: [trust, validation, governance, idp, cell]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-PLATFORM-APIS, REQ-TPL-IDP-SNAPSHOT]
acs: [AC-PLT-015, AC-TPL-IDP-SNAPSHOT, AC-TPL-BDD-EXIT-CODES]
adrs: [ADR-0005, ADR-0017]
last_updated: 2025-12-22
---

# How to Trust a Cell

> **For IDP teams and platform engineers:** This one-pager shows how to verify that a Rust-as-Spec cell is healthy and trustworthy in under 5 minutes.

**Kernel Version:** v3.3.9-kernel

---

## The Trust Checklist

A cell built on this template is **trustworthy** when all of these pass:

| # | Check | Command | Expected |
|---|-------|---------|----------|
| 1 | **tier1-selftest CI is green** | Check GitHub Actions | `tier1-selftest` workflow passing on `main` |
| 2 | **kernel-status is healthy** | `cargo xtask kernel-status` | All sections green, no red flags |
| 3 | **idp-snapshot returns valid JSON** | `cargo xtask idp-snapshot` | Valid JSON with `governance_health.status: "healthy"` |
| 4 | **platform/status responds** | `curl localhost:8080/platform/status` | `governance.selftest_status: "pass"` |

---

## Quick Validation Script

Copy and run this script to validate a cell:

```bash
#!/bin/bash
set -e

echo "=== Cell Trust Validation ==="

# 1. Kernel status (local)
echo "1. Checking kernel status..."
cargo xtask kernel-status

# 2. IDP snapshot (machine-readable)
echo "2. Generating IDP snapshot..."
cargo xtask idp-snapshot --pretty | head -30

# 3. Start service and check platform status
echo "3. Checking platform status..."
cargo run -p app-http &
sleep 3
curl -s http://localhost:8080/platform/status | jq '.governance | {
  selftest_status,
  ac_pass,
  ac_total,
  policies_pass,
  policies_total
}'

# Cleanup
kill %1 2>/dev/null || true

echo "=== Validation Complete ==="
```

---

## What Each Check Validates

### 1. tier1-selftest CI (Remote)

**Why:** This is the authoritative gate. If it's green on `main`, the kernel contract is intact.

**Where to check:**
- GitHub Actions → `tier1-selftest.yml` workflow
- Must show green checkmark on the latest commit to `main`

**What it enforces:**
- `XTASK_STRICT_AC_COVERAGE=1` – all kernel ACs must pass
- `XTASK_STRICT_PRECOMMIT=1` – strict mode enabled
- All 11 selftest steps (fmt, clippy, tests, BDD, policies, graph invariants)

---

### 2. kernel-status (Local)

**Why:** Quick local validation of kernel health without running full selftest.

**Command:**
```bash
cargo xtask kernel-status
```

**Expected output sections:**
- `Template version: v3.3.13` – matches expected kernel version
- `Kernel tag: v3.3.9-kernel (HEAD is at tag: yes)` – at kernel tag
- `Tree clean: yes` – no uncommitted changes
- `Kernel ACs: Total: 72, PASS: 72` – all passing
- `Docs-as-Code: version alignment: OK` – docs match spec_ledger

**Red flags (investigate if you see):**
- `Kernel ACs: UNKNOWN > 5` – tests not running properly
- `version alignment: DRIFT` – docs out of sync
- `Tree clean: no` – uncommitted changes affecting validation

---

### 3. idp-snapshot (Machine-Readable)

**Why:** This is the contract surface for IDP integrations. If it returns valid JSON with healthy status, the cell is consumable.

**Command:**
```bash
cargo xtask idp-snapshot --pretty
```

**Key fields to check:**
```json
{
  "template_version": "3.3.13",
  "governance_health": {
    "status": "healthy",
    "ac_coverage": {
      "total": 72,
      "passing": 72,
      "failing": 0
    }
  }
}
```

**Trust threshold:**
- `governance_health.status` must be `"healthy"`
- `ac_coverage.failing` must be `0`
- `ac_coverage.unknown` should be `< 5` (some meta-ACs are CI-only)

---

### 4. platform/status (Runtime)

**Why:** Validates the running service exposes correct governance state.

**Command:**
```bash
curl -s http://localhost:8080/platform/status | jq '.governance'
```

**Key fields:**
```json
{
  "selftest_status": "pass",
  "ac_pass": 72,
  "ac_total": 72,
  "policies_pass": 22,
  "policies_total": 22
}
```

**Trust threshold:**
- `selftest_status` must be `"pass"`
- `ac_pass` should equal `ac_total` (or close)
- `policies_pass` should equal `policies_total`

---

## When to Validate

| Scenario | Recommended Checks |
|----------|-------------------|
| **Before forking** | All 4 checks |
| **Before IDP integration** | idp-snapshot + platform/status |
| **Daily health check** | kernel-status |
| **After pulling updates** | kernel-status + selftest |
| **Before deploying** | tier1-selftest CI green |

---

## What to Do If Validation Fails

1. **tier1-selftest CI red:**
   - Do not integrate until CI is green
   - Check GitHub Actions for failure details
   - File issue if this is a kernel bug

2. **kernel-status shows issues:**
   - Run `cargo xtask selftest` for full details
   - Check `cargo xtask ac-status` for specific AC failures
   - If version drift: run `cargo xtask docs-check`

3. **idp-snapshot fails:**
   - Ensure you're on the kernel tag: `git checkout v3.3.9-kernel`
   - Run `cargo build -p xtask` to rebuild
   - Check `cargo xtask idp-snapshot 2>&1` for errors

4. **platform/status unavailable:**
   - Ensure service is running: `cargo run -p app-http`
   - Check port: default is 8080
   - Check logs for startup errors

---

## Related Documentation

- **[IDP_CELL_CONTRACT.md](../IDP_CELL_CONTRACT.md)** – Full IDP integration contract
- **[KERNEL_SNAPSHOT.md](../KERNEL_SNAPSHOT.md)** – Frozen kernel baseline
- **[ci-workflows.md](../reference/ci-workflows.md)** – CI gate details
- **[ADR-0017](../adr/0017-tier1-selftest-gate.md)** – Why tier1-selftest is authoritative
