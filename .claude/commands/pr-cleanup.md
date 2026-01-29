---
description: Cleanup pass to make the current branch PR-ready (agents in waves; apply fixes; save purposeful receipts; report)
argument-hint: [optional: intent/constraints e.g. "minimal churn", "run full gate", "skip benches", "prep for issue #218"]
---

# PR Cleanup Pass (current branch)

Do a **quality-first cleanup pass** on the **CURRENT WORKING TREE state** to make this branch PR-ready.

The goal is maintainability and reviewability:
- reduce future change-cost
- make interfaces/boundaries clearer
- make verification credible
- remove obvious footguns (lint/test/docs drift, dependency posture, risky patterns)

Use any extra context I provide: **$ARGUMENTS**

## First: Gather Context

Get the branch state before doing anything else:
- Current branch name and status (clean/dirty)
- Default branch (usually `main`)
- Merge-base with default branch
- Changed files and diff stats since merge-base
- Detect tooling: look for `Cargo.toml`, `pyproject.toml`, `package.json`, `Makefile`, `justfile`, etc.

## How to work (agents in waves)

Work through these waves, using subagents for exploration and planning. Launch independent work in parallel where possible.

### Wave 1 — Explore (find what matters)

Spawn an **Explore agent** to analyze the changes:
- Map semantic hotspots vs mechanical changes
- Flag interface/contract touchpoints (public APIs, schemas, CLI)
- Flag risk surface deltas (unsafe, concurrency, IO, deps)
- Identify repo-native gate commands (cargo xtask selftest, make check, etc.)
- Report with anchors: file paths, function names, commit refs

The explore agent should analyze only, not make changes.

### Wave 2 — Plan (cleanup plan with maintainability intent)

Spawn a **Plan agent** to create a cleanup strategy:
- Propose cleanup actions that improve maintainability without scope creep
- Separate "quick wins" (format/lint) vs "follow-ups" (bigger refactors)
- Recommend which tools to run (gate-first, then targeted checks)
- Suggest commit strategy if it helps review (mechanical vs semantic commits)

The plan agent should plan only, not implement.

### Wave 3 — Improve & fix (apply changes)

Apply the cleanup plan yourself or spawn a helper agent:
- Run the repo's best available gate and address findings
- Apply safe mechanical fixes (format, lint, docs drift)
- Tighten boundaries where clearly beneficial (especially in hotspots)
- Save tool outputs you'll cite (gate logs, audit outputs, etc.)

### Wave 4 — Verify & report (prove readiness)

After fixes:
- Re-run the relevant gate/checks
- Produce the cleanup report (see below)

## Output (cleanup report)

At the end, provide a cleanup report with these sections:

### Cleanup summary (narrative)

What you tightened and why (maintainability + reviewability), and what you deliberately didn't touch.

### Interface & compatibility verdict (crisp)

- Public API: unchanged | additive | breaking | not measured
- Schemas/contracts: unchanged | updated | breaking | not measured
- CLI/config surface: unchanged | changed | not measured

Back each with anchors (paths, functions, or tool outputs).

### Evidence & receipts

What you ran and key findings.

### What changed during cleanup

Key files/dirs touched + "before → after" highlights.

### Remaining concerns / follow-ups

What's still worth doing and what you'd mechanize next time.

### PR readiness verdict

Ready / not ready + blockers.
If ready, recommend running `/pr-create` next (with suggested context).

## Progress tracking

Track your progress through the waves. Mark each wave as you start and complete it so the user can see status.

Now proceed through the waves.
