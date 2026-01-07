---
id: GUIDE-FAILURE-MODES-001
title: Failure Modes Taxonomy
doc_type: guide
status: published
audience: contributors, reviewers, maintainers
tags: [failure-modes, taxonomy, prevention, gates]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-DOCS-CONSISTENCY]
acs: [AC-PLT-009, AC-PLT-010]
adrs: []
last_updated: 2026-01-07
---

# Failure Modes Taxonomy

This document catalogs recurring failure modes, how we detect them, and what gates prevent recurrence.

---

## Purpose

When something goes wrong, we:
1. **Tag it** with a category from this taxonomy
2. **Log the errata** in the PR cover sheet
3. **Add a gate** to prevent recurrence
4. **Update this doc** with the new pattern

This turns "we got something wrong" into "we hardened the factory."

---

## Taxonomy

### Measurement Drift (`measurement_drift`)

**What:** Benchmark semantics changed, denominator changed, units changed.

**Examples:**
- Perf ratio invalid because baseline changed
- p50/p95 numbers not comparable across runs
- "10x faster" claim based on different workloads

**Detection:**
- Perf receipts with explicit methodology
- Baseline pinning in benchmark configs
- Schema validation for measurement units

**Prevention:**
- [ ] Measurement integrity contract (planned)
- [x] No multiplier claims policy (see `PR_COVER_SHEET.md`)

---

### Claim Drift (`claim_drift`)

**What:** PR text contradicts receipts or checks.

**Examples:**
- Cover sheet says "tests pass" but no receipt link
- Security claim without scan output
- "Production-ready" without criteria

**Detection:**
- PR review checking for receipt links
- Cover sheet linting (planned)

**Prevention:**
- [x] Cover sheet format requires receipt links
- [ ] `xtask pr-lint` to validate cover sheets (planned)

---

### Spec Drift (`spec_drift`)

**What:** AC/contract doesn't match implementation.

**Examples:**
- Endpoint returns different schema than documented
- BDD scenario doesn't match actual behavior
- Config option documented but not implemented

**Detection:**
- [x] `cargo xtask selftest` step 9 (graph invariants)
- [x] BDD tests with AC tags
- [x] OpenAPI schema validation

**Prevention:**
- [x] Selftest gates on spec/test/doc alignment
- [x] BDD harness exit semantics (`AC-TPL-BDD-EXIT-CODES`)

---

### Environment/Global State (`env_global_state`)

**What:** CWD, env vars, test isolation failures.

**Examples:**
- Test pollutes working directory
- `SPEC_ROOT` not set correctly
- Tests depend on execution order

**Detection:**
- [x] BDD test isolation (fixed in 3.3.x line)
- [x] `SPEC_ROOT` contract (`AC-TPL-XTASK-SPEC-ROOT`)

**Prevention:**
- [x] Tests clean up after themselves
- [x] `SPEC_ROOT` honored by all xtask commands
- [x] Worktree cleanup in acceptance tests

---

### Non-Determinism (`non_determinism`)

**What:** Order dependence, flaky tests, race conditions.

**Examples:**
- Test passes locally, fails in CI
- Results depend on HashMap iteration order
- Concurrent test interference

**Detection:**
- CI runs catch flaky tests
- Repeated local runs with `--test-threads=1`

**Prevention:**
- [x] BDD runs sequentially by default
- [ ] Flaky test quarantine (planned)

---

### Boundary Violation (`boundary_violation`)

**What:** Sync I/O in async, blocking calls, architectural violations.

**Examples:**
- Blocking file read in async handler
- Mutex held across await point
- Direct database call bypassing repository

**Detection:**
- [x] Clippy lints for some patterns
- Code review for architectural violations

**Prevention:**
- [x] Hexagonal architecture enforced by crate structure
- [ ] tokio-console integration for runtime detection (planned)

---

### Security Boundary (`security_boundary`)

**What:** Path traversal, input validation, auth bypass.

**Examples:**
- `../` in file path not sanitized
- Missing size limit on upload
- Auth check skipped for certain routes

**Detection:**
- [x] Security middleware (CORS, headers, JWT validation)
- [x] Fail-closed auth mode
- [x] Supply chain CI (CodeQL, gitleaks, cargo-audit)

**Prevention:**
- [x] Security headers on all responses
- [x] `PLATFORM_AUTH_MODE=basic` rejects unauthed writes
- [x] Path validation in file handlers

---

### Documentation Falsehood (`docs_falsehood`)

**What:** Examples don't compile, docs imply non-existent APIs.

**Examples:**
- Code snippet has syntax errors
- "Run `foo`" but `foo` command doesn't exist
- API endpoint documented but returns 404

**Detection:**
- [x] `cargo xtask docs-check` for version alignment
- [ ] Doc snippet compilation (planned)
- [x] OpenAPI endpoint validates schema

**Prevention:**
- [x] Version authority in `spec_ledger.yaml`
- [x] `docs-check` in precommit
- [ ] Executable documentation tests (planned)

---

### Packaging Drift (`packaging_drift`)

**What:** Release artifacts don't match source.

**Examples:**
- SBOM lists wrong versions
- Release bundle missing files
- Version in binary doesn't match tag

**Detection:**
- [x] `cargo xtask release-bundle` generates evidence
- [x] Version manifest (`specs/version_manifest.yaml`)
- [x] `release-prepare --dry-run` for preview

**Prevention:**
- [x] Manifest-driven versioning engine
- [x] Release evidence bundle with checksums
- [x] SBOM generation (`cargo xtask sbom-local`)

---

## Using the Taxonomy

### In PR Cover Sheets

```markdown
### Errata (what we got wrong)
- **Wrong:** [measurement_drift] Perf claim invalid due to baseline change
  - Detected by: Reviewer noticed p95 numbers not comparable
  - Fix: #210
  - Prevention: Added baseline pinning to bench config
```

### In Casebook Entries

```markdown
### PR #76 — Pagination error contract
- **Failure mode:** claim_drift risk (400 behavior could silently regress)
- **Detection:** New BDD scenarios
- **Prevention:** Contract tests in `specs/features/platform_issues.feature`
```

### Tracking Trends

Count failure modes across PRs to identify hotspots:

```bash
# Future: aggregate from dossiers
cargo xtask failure-mode-stats
```

---

## Adding New Failure Modes

When you encounter a new category:

1. Add it to this taxonomy with:
   - Name (snake_case)
   - What it is
   - Examples
   - Detection mechanism
   - Prevention gate

2. Tag the errata in the PR cover sheet

3. Add the prevention gate (test, lint, check)

4. Update this doc

---

## Factory Learning Backlog

Recurring failure modes that need better gates:

| Mode | Occurrences | Next Action |
|------|-------------|-------------|
| `measurement_drift` | 2 | Add measurement integrity contract |
| `docs_falsehood` | 3 | Add doc snippet validation |
| `non_determinism` | 1 | Add flaky test quarantine |

This backlog is updated when errata accumulate in a category.
