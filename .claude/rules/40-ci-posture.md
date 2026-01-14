# CI Posture Rule

When CI is disabled, non-canonical, or green-but-meaningless, local gates become the source of truth.

## Background

GitHub Actions may be intentionally disabled during certain phases (template evolution,
release prep, cost management). The GitHub UI may still show stale check results that
don't reflect current branch state.

## Rules

### 1. Never claim "CI passed" when CI is disabled

If CI workflows are disabled or non-canonical:

- Do not claim "checks passed" as quality evidence
- Do not reference GitHub UI check status
- Explicitly state: "CI disabled; local gate canonical"

### 2. Local gate is canonical

When CI is unavailable:

- `cargo xtask selftest` output is the authoritative gate
- Provide reproduce commands so reviewers can verify locally
- Save gate receipts to `target/receipts/` or PR-specific locations

### 3. PR cockpit must reflect actual verification

In PR bodies:

**If CI is active and meaningful:**

```markdown
### Evidence & Verification
CI workflow `tier1-selftest.yml` ran successfully.
Gate receipts: [link or embedded]
```

**If CI is disabled:**

```markdown
### Evidence & Verification
CI disabled; local gate canonical.

Reproduce:
```bash
cargo xtask selftest
```

Gate receipts: `target/receipts/gate.json` (committed or linked)

```

### 4. Friction log captures CI posture changes

When CI is disabled or re-enabled:

- Update `friction/FRICTION-CI-001.yaml` or create new entry
- Document reason, expected duration, and workaround
- Update status when CI returns to normal operation

## Checking CI Status

```bash
# Check if workflows are active
ls -la .github/workflows/

# Check recent workflow runs (if gh CLI available)
gh run list --limit 5

# Check friction log for CI status
cargo xtask friction-list --category ci
```

## Why This Matters

Claiming "CI green" when CI isn't running creates false confidence. This rule ensures:

- Truth surfaces are explicit about their provenance
- Reviewers know exactly what verification occurred
- The gap between GitHub UI and actual verification is visible
