---
description: Create PR (current branch)
argument-hint: [optional: context/intent e.g. "Issue #218", "ready", "draft", "base=main", "no-ci", "local-gate-canonical"]
---

# Create PR (current branch)

Create a pull request from the **CURRENT WORKING TREE state** of this branch.

Write the PR like maintainer notes: narrative is welcome. Center it on modern review signals:
- **Interface integrity** (public API / contracts / schemas / CLI/config surface)
- **Risk surface delta** (unsafe, concurrency, IO/networking/serialization, deps)
- **Verification depth** (what evidence exists and how to reproduce it)
- **Future change-cost** (hotspots, modularity, complexity proxies, doc rot prevention)

Use any extra context I provide: **$ARGUMENTS**

## First: Gather Context

Get the branch state before doing anything else:
- Current branch name and status (clean/dirty)
- Default branch (usually `main`)
- Whether branch tracks a remote and is up-to-date
- Merge-base with default branch
- Changed files and diff stats since merge-base
- Commit history on this branch
- Full diff for analysis
- Detect tooling: look for `Cargo.toml`, `pyproject.toml`, `package.json`, etc.

## How to work (agents in waves)

Work through these waves, using subagents for exploration and planning. Launch independent work in parallel where possible.

### Wave 1 — Explore (map the change)

Spawn an **Explore agent** to analyze the changes:
- Map where behavior changed (semantic hotspots vs mechanical)
- Identify interface/contract touchpoints (public API, schemas, CLI/config)
- Flag risk surface deltas (unsafe, concurrency, IO, deps changes)
- Identify the repo's gate command(s)
- Report with anchors: file:line, function names, commit refs

The explore agent should analyze only, not make changes.

### Wave 2 — Plan (compose the story + evidence plan)

Spawn a **Plan agent** to structure the PR:
- Propose a coherent narrative arc (intent → design → review path → evidence)
- Produce a crisp Interface & compatibility verdict
- Recommend which verification tools support the claims
- Surface key decision points affecting maintainability

The plan agent should plan only, not write the final PR body.

### Wave 3 — Improve (gather evidence)

Gather verification evidence yourself or spawn a helper:
- Run repo's gate command if not already run
- Collect tool outputs that support interface/risk claims
- Note what was verified and what wasn't

### Wave 4 — Create the PR

Once you have the PR title and body ready:
1. Push the branch if needed (with `-u` to set upstream)
2. Create the PR with `gh pr create`

Default: create as **draft**, unless `$ARGUMENTS` clearly indicates "ready".

## Known workflow issues

### `gh pr edit` fails with Projects (classic)

If you need to update a PR body after creation, `gh pr edit` may fail with:

```
GraphQL: Cannot query field "projectCards" on type "PullRequest"
```

**Workaround:** Use REST API: `gh api -X PATCH /repos/{owner}/{repo}/pulls/{pr_number} -f body='...'`

## PR body format (narrative + modern signals)

Use these sections:

### Summary

1-3 paragraphs: what changed + why, trade-offs, what should be true after merge.

### Interface & compatibility verdict

Crisp top-line statements (supported by tools or concrete deltas):
- Public API: unchanged | additive | breaking | not measured
- Schemas/contracts: unchanged | updated | breaking | not measured
- CLI/config surface: unchanged | changed | not measured

### Design & maintainability notes

Boundaries, modularity, and what changed future change-cost.

### What changed (narrative)

System-level explanation (not a file dump).

### How to review (fast path)

A practical map: key dirs/files + semantic hotspots.

### Evidence & verification

What you ran, what it proves, and how to reproduce.
If something wasn't run, say so.

**CI posture:** If CI is disabled, state "CI disabled; local gate canonical" and provide reproduce commands.

### Complexity (future change-cost)

Tool-backed if available; otherwise proxies (hotspots/churn, module splits, API delta, unsafe delta, deps delta).
Interpret implications rather than scoring.

### Risk & rollback

Blast radius, failure modes, rollback/recovery.

### Known limits / follow-ups

Explicit deferrals and next steps.

### Retrospective (earnest)

Surprises, corrections, and what to mechanize next time.

## Progress tracking

Track your progress through the waves. Mark each wave as you start and complete it so the user can see status.

Now proceed through the waves.
