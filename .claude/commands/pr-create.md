---
description: Create PR (current branch)
argument-hint: [optional: context/intent e.g. "Issue #218", "ready", "draft", "base=main", "no-ci", "local-gate-canonical"]
allowed-tools: >
  Bash(git status:*), Bash(git branch:*), Bash(git rev-parse:*), Bash(git symbolic-ref:*), Bash(git remote:*),
  Bash(git merge-base:*), Bash(git log:*), Bash(git show:*), Bash(git diff:*),
  Bash(ls:*), Bash(find:*), Bash(rg:*), Bash(sed:*), Bash(awk:*), Bash(wc:*),
  Bash(mkdir:*), Bash(cat:*), Bash(tee:*), Bash(date:*),
  Bash(gh:*),
  Bash(make:*), Bash(just:*), Bash(nix:*),
  Bash(cargo:*), Bash(cargo-*:*) , Bash(pytest:*), Bash(ruff:*), Bash(mypy:*), Bash(pyright:*),
  Bash(node:*), Bash(npm:*), Bash(pnpm:*), Bash(yarn:*),
  Bash(tokei:*), Bash(scc:*), Bash(lizard:*), Bash(radon:*),
  Bash(lychee:*),
  Bash(pip-audit:*), Bash(pip:*), Bash(uv:*), Bash(poetry:*)
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

Before doing anything else, gather this information yourself:

1. **Branch info**: Run `git branch --show-current` and `git status --porcelain=v1 -b`
2. **Default branch**: Try `git symbolic-ref refs/remotes/origin/HEAD` or assume `main`
3. **Merge base**: Find the merge-base with the default branch
4. **Changed files**: `git diff --name-only <merge-base>..HEAD`
5. **Diff stats**: `git diff --stat <merge-base>..HEAD`
6. **Commit history**: `git log --oneline <merge-base>..HEAD`
7. **Tooling detection**: Look for `Cargo.toml`, `pyproject.toml`, `package.json`, `Makefile`, `justfile`, `flake.nix`, etc.

Optionally create a receipts directory if you want to save artifacts:
- Suggested path: `target/pr-create/<timestamp>-<branch>/`
- Only create if you'll actually save useful outputs there

## How to work (agents in waves)

### Wave 1 — Explore (map the change)

Invoke the **Explore** subagent to:
- map where behavior changed (review map + semantic hotspots)
- separate mechanical vs semantic changes
- identify interface/contract touchpoints (API/schema/CLI/config)
- flag risk surface deltas (unsafe/concurrency/IO/deps)
- identify the repo's likely "gate" command(s)

Explore should use git + repo inspection directly and report back with anchors (paths, commits, commands).

### Wave 2 — Plan (compose the story + evidence plan)

Invoke the **Plan** subagent to:
- propose a coherent PR narrative arc (intent → design → review path → risk/evidence)
- produce a crisp **Interface & compatibility verdict** (and how it is supported)
- recommend which tools are worth running here to support claims (best available, not exhaustive)
- surface key decision points that affect maintainability (boundaries, invariants, compatibility intent)

### Wave 3 — Improve (tighten the PR content)

Invoke specialist subagents (or general-purpose helpers) to refine:
- Diff Scout / Maintainability: review map + future change-cost interpretation (hotspots, modularity, complexity proxies)
- Evidence / Verification: what was actually validated, reproduction path, what remains unverified
- Docs Verifier (if docs touched): drift/executable example issues
- Risk Surface: unsafe/concurrency/IO/deps delta + rollback story
- Complexity Analyst: tool-backed if available; otherwise defensible proxies with interpretation

### Wave 4 — Create the PR (gh)

Once you have the PR title and body ready, create the PR with `gh pr create`.

Default: create as **draft**, unless `$ARGUMENTS` clearly indicates "ready".

## Known workflow issues

### `gh pr edit` fails with Projects (classic)

If you need to update a PR body after creation, `gh pr edit` may fail with:

```
GraphQL: Cannot query field "projectCards" on type "PullRequest"
```

This is a GitHub CLI bug affecting repos with Projects (classic) enabled.

**Workaround:** Use REST API directly:

```bash
gh api -X PATCH /repos/{owner}/{repo}/pulls/{pr_number} -f body='...'
```

Or use a heredoc for complex bodies:

```bash
gh api -X PATCH /repos/{owner}/{repo}/pulls/{pr_number} \
  -f body="$(cat <<'EOF'
Your PR body here
EOF
)"
```

## Useful tools (guidance)

Use what fits the repo and what supports the claims you plan to make:
- Rust: `cargo fmt`, `cargo clippy`, `cargo test`/`cargo nextest`, `cargo semver-checks`/`cargo-semver-checks`, `cargo-audit`, `cargo-deny`, `cargo-geiger`, `cargo llvm-lines`, `tokei`
- Python: `ruff`, `mypy`/`pyright`, `pytest`, `pip-audit`, `radon`
- JS/TS: `eslint`, `tsc`, `jest`, `npm audit`
- Docs: doctests, link checks (`lychee`)

Save outputs you cite into the receipts dir if you created one.

## PR body format (narrative + modern signals)

Use these sections:

### Summary

1–3 paragraphs: what changed + why, trade-offs, what should be true after merge.

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

### Complexity (future change-cost)

Tool-backed if available; otherwise proxies (hotspots/churn, module splits, API delta, unsafe delta, deps delta).
Interpret implications rather than scoring.

### Risk & rollback

Blast radius, failure modes, rollback/recovery.

### Known limits / follow-ups

Explicit deferrals and next steps.

### Retrospective (earnest)

Surprises, corrections, and what to mechanize next time (new gate/receipt/invariant).

## Checklist (use TodoWrite to track)

Use TodoWrite to create and track these steps:

1. **Gather context** - Get branch info, merge-base, changed files, diff stats, commit history, detect tooling
2. **Wave 1: Explore** - Map behavior changes, separate mechanical vs semantic, find interfaces and risk surfaces
3. **Wave 2: Plan** - Compose PR narrative arc, interface verdict, evidence plan
4. **Wave 3: Improve** - Refine maintainability notes, verification evidence, risk assessment
5. **Wave 4: Create PR** - Write title + body, run `gh pr create` (draft by default)
6. **Confirm** - Print PR URL and summary

Now proceed through the checklist.
