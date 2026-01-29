---
description: Create PR (current branch)
argument-hint: [optional: context/intent e.g. "Issue #218", "ready", "draft", "base=main", "local-gate-canonical"]
---

# Create PR

This command produces a PR description that a maintainer can review quickly and trust.

Default posture: **draft**, unless the argument clearly indicates "ready".

Use any extra context I provide: **$ARGUMENTS**

## Wave 0 — Context

Establish:

- Branch name + working tree status
- Default branch + merge base
- Changed files + diff stats
- Commit list on this branch
- Repo gate posture (CI required vs local gate canonical)

## Wave 1 — Explore

Build a reviewer map:

- What behavior changed (semantic hotspots vs mechanical churn)
- Contract touchpoints (public API, schemas, CLI output formats)
- Risk deltas (unsafe/concurrency/IO/deps)
- Rollback story (what reverting would mean)

**No edits in this wave.**

## Wave 2 — Structure

Define:

- One-sentence intent
- Scope boundaries (what's in/out)
- Compatibility verdict (and what supports it)
- Evidence plan (what you ran / will run)

## Wave 3 — Evidence

Run the repo's gate and any targeted checks that support your claims.

If CI is disabled, say: **"CI disabled; local gate canonical"** and include reproduce commands.

## Wave 4 — Create

Push the branch if needed and create the PR (draft by default).

## PR body format

### Summary
1–3 paragraphs: what changed + why, trade-offs, what should be true after merge.

### Scope
- Type (governance/devex/docs/runtime/etc.)
- Touchpoints (key crates/files/surfaces)

### Interface & compatibility verdict
- Public API: unchanged | additive | breaking | not measured
- Schemas/contracts: unchanged | updated | breaking | not measured
- CLI/config surface: unchanged | changed | not measured

### Evidence & verification
What you ran, what it proves, and how to reproduce.

### Risk & rollback
What could go wrong; how to back out safely.

### Follow-ups (explicitly out of scope)
Short list; no vague "we should".

### Retrospective
Surprises, corrections, what to mechanize next time.

## Known workflow issue: PR body edits may fail in some repos

If PR body edits fail due to GitHub Projects (classic) API behavior, update via a REST call instead of relying on interactive edit flows.
