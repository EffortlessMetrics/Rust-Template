---
description: Cleanup pass to make the current branch PR-ready (agents in waves; apply fixes; save purposeful receipts; report)
argument-hint: [optional: intent/constraints e.g. "minimal churn", "run full gate", "skip benches", "prep for issue #218"]
allowed-tools: >
  Bash(git status:*), Bash(git branch:*), Bash(git rev-parse:*), Bash(git symbolic-ref:*), Bash(git remote:*),
  Bash(git merge-base:*), Bash(git log:*), Bash(git show:*), Bash(git diff:*),
  Bash(git add:*), Bash(git restore:*), Bash(git checkout:*), Bash(git stash:*), Bash(git commit:*),
  Bash(ls:*), Bash(find:*), Bash(rg:*), Bash(sed:*), Bash(awk:*), Bash(wc:*),
  Bash(mkdir:*), Bash(cat:*), Bash(tee:*), Bash(date:*),
  Bash(make:*), Bash(just:*), Bash(nix:*),
  Bash(cargo:*), Bash(cargo-*:*) , Bash(pytest:*), Bash(ruff:*), Bash(mypy:*), Bash(pyright:*),
  Bash(node:*), Bash(npm:*), Bash(pnpm:*), Bash(yarn:*),
  Bash(tokei:*), Bash(scc:*), Bash(lizard:*), Bash(radon:*),
  Bash(lychee:*),
  Bash(pip-audit:*), Bash(pip:*), Bash(uv:*), Bash(poetry:*),
  Bash(gitleaks:*), Bash(trufflehog:*)
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

Before doing anything else, gather this information yourself:

1. **Branch info**: Run `git branch --show-current` and `git status --porcelain=v1 -b`
2. **Default branch**: Try `git symbolic-ref refs/remotes/origin/HEAD` or assume `main`
3. **Merge base**: Find the merge-base with the default branch
4. **Changed files**: `git diff --name-only <merge-base>..HEAD`
5. **Diff stats**: `git diff --stat <merge-base>..HEAD`
6. **Tooling detection**: Look for `Cargo.toml`, `pyproject.toml`, `package.json`, `Makefile`, `justfile`, `flake.nix`, etc.

Optionally create a receipts directory if you want to save artifacts:
- Suggested path: `target/pr-cleanup/<timestamp>-<branch>/`
- Only create if you'll actually save useful outputs there

## How to work (agents in waves)

### Wave 1 — Explore (find what matters)

Invoke **Explore** to:
- map semantic hotspots and what's mechanical
- flag interface/contract touchpoints
- flag risk surface deltas (unsafe/concurrency/IO/deps)
- identify repo-native "gate" commands and how to run them locally

Explore should report back with anchors (paths, commands, commit references), not raw diffs.

### Wave 2 — Plan (cleanup plan with maintainability intent)

Invoke **Plan** to:
- propose a cleanup plan that improves maintainability/PR quality without scope creep
- separate "quick wins" vs "follow-ups"
- choose which tooling to run (gate-first, then targeted checks)
- suggest a commit plan if it will materially improve review (mechanical vs semantic)

### Wave 3 — Improve & fix (apply changes in the working tree)

Invoke appropriate fixing agents (general-purpose or specialist) to:
- run the repo's best available gate (just/make/scripts/xtask/nix) and address findings
- apply safe mechanical fixes (format/lint/docs drift) and straightforward correctness fixes
- tighten boundaries / reduce future change-cost where it's clearly beneficial (especially in hotspots)
- save tool outputs you will cite into the receipts dir (gate logs, audit outputs, link checks, etc.)

### Wave 4 — Verify & report (prove readiness)

After fixes:
- re-run the relevant gate/checks
- save "after" snapshots + key logs into the receipts dir
- produce a narrative cleanup report with a crisp interface verdict and evidence pointers

## Useful tools (guidance)

Prefer repo-native commands; otherwise use what fits:
- Rust: `cargo fmt`, `cargo clippy`, `cargo test`/`cargo nextest`, `cargo semver-checks`/`cargo-semver-checks`, `cargo-audit`, `cargo-deny`, `cargo-geiger`, `cargo llvm-lines`, `tokei`
- Python: `ruff`, `mypy`/`pyright`, `pytest`, `pip-audit`, `radon`
- JS/TS: `eslint`, `tsc`, `jest`, `npm audit`
- Docs: doctests, `lychee`

Save outputs you cite into the receipts dir.

## Output (cleanup report)

At the end, provide a cleanup report (print it, and optionally save to receipts dir). Include:

### Cleanup summary (narrative)

What you tightened and why (maintainability + reviewability), and what you deliberately didn't touch.

### Interface & compatibility verdict (crisp)

- Public API: unchanged | additive | breaking | not measured
- Schemas/contracts: unchanged | updated | breaking | not measured
- CLI/config surface: unchanged | changed | not measured
Back each with anchors (paths, commands, or saved tool outputs).

### Evidence & receipts

What you ran and where you saved it (if applicable).

### What changed during cleanup

Key files/dirs touched + "before → after" highlights (lint/test/docs/risk surface).

### Remaining concerns / follow-ups

What's still worth doing and what you'd mechanize next time.

### PR readiness verdict

Ready / not ready + blockers.
If ready, recommend running `/pr-create` next (with suggested context).

## Checklist (use TodoWrite to track)

Use TodoWrite to create and track these steps:

1. **Gather context** - Get branch info, merge-base, changed files, diff stats, detect tooling
2. **Wave 1: Explore** - Map hotspots, interfaces, risk surfaces, find gate commands
3. **Wave 2: Plan** - Create cleanup plan, separate quick wins from follow-ups
4. **Wave 3: Fix** - Run gate, apply mechanical fixes, tighten boundaries
5. **Wave 4: Verify** - Re-run checks, save receipts, produce cleanup report
6. **Report** - Print final cleanup report with PR readiness verdict

Now proceed through the checklist.
