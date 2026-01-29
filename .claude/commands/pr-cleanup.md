---
description: Cleanup pass to make the current branch PR-ready (reduce review cost; prove correctness; avoid scope creep)
argument-hint: [optional: constraints e.g. "minimal churn", "doc-only", "tight scope", "prep for issue #218", "local-gate-canonical"]
---

# PR Cleanup Pass

This command turns "branch has changes" into "branch is reviewable".

The goal is not perfection. The goal is **lower review cost** and **higher confidence**.

Use any extra context I provide: **$ARGUMENTS**

## Defaults

- Don't widen scope.
- Prefer reversible/mechanical fixes over refactors.
- If CI is disabled, treat the repo's local gate as canonical and include reproduce commands.

## Wave 0 — Context

Before changing anything, capture:

- Branch name, whether the tree is clean/dirty
- Default branch + merge base
- Changed files + diff stats since merge base
- Repo gate posture (what "green" means here)
- Any obvious contracts touched (API/schema/CLI output/specs/policies)

## Wave 1 — Explore

Produce a review map:

- What's semantic vs mechanical
- Interfaces/contracts touched (public API, schema files, CLI outputs, policy surfaces)
- Risk deltas (unsafe/concurrency/IO/deps)
- The smallest verification ladder that supports the claims

**No changes in this wave.**

## Wave 2 — Plan

Produce a bounded cleanup plan:

- Quick wins to apply now (format/lint/docs drift, obvious correctness fixes)
- Follow-ups explicitly deferred (bigger refactors, behavior shifts)
- Verification plan (what you will run and why)
- Commit strategy if it materially improves review (mechanical vs semantic)

**No changes in this wave.**

## Wave 3 — Fix

Apply the bounded plan:

- Run the repo's best available gate and address findings
- Apply safe mechanical fixes
- Fix straightforward correctness issues revealed by the gate
- Tighten boundaries only where it clearly reduces future change-cost

Avoid "nice to have" refactors unrelated to the PR story.

## Wave 4 — Verify & Report

- Re-run the relevant gate(s)
- Capture the reproduce commands and key outputs
- Produce the cleanup report below

## Output: Cleanup report

### Summary
1–3 paragraphs: what changed during cleanup and why.

### Interface & compatibility verdict
- Public API: unchanged | additive | breaking | not measured
- Schemas/contracts: unchanged | updated | breaking | not measured
- CLI/config surface: unchanged | changed | not measured

### Evidence
What you ran and what it proves. If something wasn't run, say so.

### What changed during cleanup
Key files/dirs touched + before→after highlights.

### Remaining concerns / follow-ups
Concrete next items; explicitly out of scope for this run.

### PR readiness
Ready / not ready + blockers. If ready, recommend running `/pr-create`.
