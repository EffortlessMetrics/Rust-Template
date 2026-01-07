---
id: GUIDE-RECEIPTS-001
title: Receipts Schema and Usage
doc_type: guide
status: published
audience: contributors, reviewers, tooling-authors
tags: [receipts, evidence, verification, schema]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-DOCS-CONSISTENCY]
acs: [AC-PLT-009, AC-PLT-010]
adrs: []
last_updated: 2026-01-07
---

# Receipts Schema and Usage

Receipts are the source of truth for claims in PR cover sheets.

---

## Philosophy

**Claims must be backed by evidence.** A receipt is machine-generated proof that a gate ran and what it found.

Receipts live in `.runs/` (ephemeral, gitignored) during development. Version-controlled exhibits in `docs/audit/EXHIBITS/` preserve the claims for posterity.

---

## Directory Structure

```
.runs/
  pr/<n>/
    <run-id>/
      receipts/
        gate.json        # What gates ran, pass/fail
        economics.json   # DevLT + compute spend
        tests.json       # Test execution details
        security.json    # Security scan results
        perf.json        # Performance measurements (if any)
        dossier.json     # Structured PR analysis (future)
```

---

## Minimal Receipt Set

### gate.json

The core receipt: what ran and whether it passed.

<!-- doclint:disable orphan-version (example data, not managed versions) -->
```json
{
  "schema_version": "1.0",
  "run_id": "2026-01-07T14:32Z-pr209",
  "pr": 209,
  "commit": "abc123def",
  "started_at": "2026-01-07T14:32:00Z",
  "finished_at": "2026-01-07T14:35:42Z",
  "gates": [
    {
      "name": "fmt",
      "command": "cargo fmt --all --check",
      "status": "pass",
      "duration_ms": 1234
    },
    {
      "name": "clippy",
      "command": "cargo clippy --all-targets",
      "status": "pass",
      "duration_ms": 45678
    },
    {
      "name": "tests",
      "command": "cargo test --all",
      "status": "pass",
      "duration_ms": 23456,
      "details": {
        "passed": 142,
        "failed": 0,
        "ignored": 3
      }
    },
    {
      "name": "selftest",
      "command": "cargo xtask selftest",
      "status": "pass",
      "duration_ms": 120000,
      "steps_passed": 11,
      "steps_total": 11
    }
  ],
  "overall_status": "pass",
  "repo_version": "vX.Y.Z",
  "environment": {
    "os": "linux",
    "rust_version": "1.xx.0",
    "nix_shell": true
  }
}
```

### economics.json

DevLT and compute tracking. Allows unknowns.

```json
{
  "schema_version": "1.0",
  "pr": 209,
  "run_id": "2026-01-07T14:32Z-pr209",
  "devlt_minutes": {
    "author": 25,
    "author_confidence": "estimated",
    "review": "unknown",
    "review_confidence": "unknown",
    "interventions": 2,
    "notes": "Two fix-loops after initial clippy failures"
  },
  "compute": {
    "tokens_usd": 4.20,
    "confidence": "estimated",
    "runs": 3,
    "notes": "Three selftest runs during iteration"
  },
  "iterations": {
    "failed_gates": 2,
    "fix_loops": 2,
    "notes": "Clippy warnings, then test failure"
  },
  "value_delivered": {
    "uncertainty_reduced": "Confirmed BDD scenarios cover error paths",
    "rework_prevented": "Caught missing 400 handler before merge"
  }
}
```

### dossier.json

Structured analysis for casebook generation.

```json
{
  "schema_version": "1.0",
  "pr": 209,
  "title": "Add pagination error contract BDD scenarios",
  "merged_at": "2026-01-07T15:00:00Z",
  "scope": {
    "top_dirs": ["specs/features", "crates/gov-http"],
    "files_changed": 5,
    "lines_added": 120,
    "lines_removed": 15
  },
  "intent": {
    "issue_links": ["#76"],
    "spec_links": ["REQ-PLT-ISSUES-001"],
    "ac_links": ["AC-PLT-ISSUES-PAGINATION"]
  },
  "findings": [],
  "errata": [],
  "exhibit_score": {
    "scope_clarity": 5,
    "proof_completeness": 5,
    "errata_quality": 5,
    "factory_delta": 3,
    "total": 18,
    "max": 25
  },
  "factory_delta": {
    "gates_added": ["BDD pagination scenarios"],
    "contracts_tightened": ["400 error responses"],
    "docs_updated": []
  }
}
```

---

## Schema Files

Receipt structures are defined by JSON schemas in `specs/schemas/`:

| Schema | Path | Purpose |
|--------|------|---------|
| Gate receipt | `specs/schemas/gate.schema.json` | Gate execution results (fmt, clippy, tests, selftest) |
| Economics receipt | `specs/schemas/economics.schema.json` | DevLT + compute tracking |
| Dossier receipt | `specs/schemas/dossier.schema.json` | Structured PR analysis for casebook generation |

Canonical receipt output paths (relative to run directory):

- `receipts/gate.json` - Gate pass/fail evidence
- `receipts/economics.json` - Time and cost tracking
- `receipts/dossier.json` - PR analysis (future)

---

## CLI Commands

### `receipts-gate` - Run gates and emit receipt

```bash
# Run gates and generate gate.json
cargo xtask receipts-gate --pr 209 --output-dir .runs/pr/209/run-01

# Without PR number (for local validation)
cargo xtask receipts-gate --output-dir .runs/current
```

Output: `.runs/current/receipts/gate.json`

### `receipts-economics` - Generate economics receipt

```bash
# Generate economics receipt with estimates
cargo xtask receipts-economics --pr 209 \
  --author-minutes 25 --author-confidence estimated \
  --compute-usd 4.20 --compute-confidence estimated \
  --runs 3 --failed-gates 2 --fix-loops 2 \
  --uncertainty-reduced "Confirmed BDD scenarios cover error paths"
```

Output: `.runs/current/receipts/economics.json`

### `receipts-validate` - Validate receipt against schema

```bash
# Validate a gate receipt (future command)
cargo xtask receipts-validate .runs/current/receipts/gate.json
```

---

## Example Workflow

```bash
# 1. Run gates and capture evidence
cargo xtask receipts-gate --pr 209

# 2. Record economics (after PR work is done)
cargo xtask receipts-economics --pr 209 \
  --author-minutes 30 --author-confidence estimated \
  --compute-usd 5.00 --compute-confidence estimated

# 3. Generate PR cover sheet from receipts
cargo xtask pr-cover --pr 209

# 4. Update PR body and save exhibit
cargo xtask pr-update --pr 209 --save-exhibit
```

---

## Generating Receipts (Legacy/Manual)

For contexts where CLI commands aren't available:

```bash
# Run gates and capture output
cargo xtask selftest 2>&1 | tee .runs/pr/209/2026-01-07/selftest.log

# Create receipts manually from output
# (Use CLI commands above when available)
```

---

## Using Receipts

### In PR Cover Sheets

Link to receipts, don't copy their content:

```markdown
### Proof (receipts)
| Check | Status | Receipt |
|-------|--------|---------|
| Selftest | PASS | `.runs/pr/209/2026-01-07/receipts/gate.json` |
```

### For Casebook Generation

```bash
# Generate dossier from receipts
cargo xtask pr-dossier --pr 209

# Update casebook from all dossiers
cargo xtask casebook-gen
```

---

## Schema Evolution

Receipts are versioned via `schema_version`. Rules:

1. **Additive changes** (new optional fields) → same version
2. **Breaking changes** (required field changes) → bump version
3. **Old receipts** are valid; tooling must handle missing fields gracefully

---

## Confidence Levels

Use these consistently:

| Level | Meaning |
|-------|---------|
| `measured` | Actual measurement (timer, counter) |
| `estimated` | Reasonable guess based on evidence |
| `unknown` | No basis for estimate |

Never fake precision. `"unknown"` is better than a wrong number.

---

## Storage Policy

| Location | Retention | Purpose |
|----------|-----------|---------|
| `.runs/` | Ephemeral (gitignored) | Working artifacts during PR |
| `docs/audit/EXHIBITS/` | Permanent (committed) | Curated examples |
| CI artifacts | 30-90 days | Debugging failed runs |

Don't commit `.runs/` to git. The dossier + exhibit captures what matters.
