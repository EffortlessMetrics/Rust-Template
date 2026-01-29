---
description: Cleanup pass to make the current branch PR-ready (agents in waves; apply fixes; save purposeful receipts; report)
argument-hint: [optional: intent/constraints e.g. "minimal churn", "run full gate", "skip benches", "prep for issue #218"]
allowed-tools: >
  Read, Grep, Glob, Edit, Write,
  Bash(git status:*), Bash(git branch:*), Bash(git rev-parse:*), Bash(git symbolic-ref:*), Bash(git remote:*),
  Bash(git merge-base:*), Bash(git log:*), Bash(git show:*), Bash(git diff:*),
  Bash(git add:*), Bash(git restore:*), Bash(git checkout:*), Bash(git stash:*), Bash(git commit:*),
  Bash(mkdir:*), Bash(date:*),
  Bash(make:*), Bash(just:*), Bash(nix:*),
  Bash(cargo:*), Bash(cargo-*:*), Bash(pytest:*), Bash(ruff:*), Bash(mypy:*), Bash(pyright:*),
  Bash(node:*), Bash(npm:*), Bash(pnpm:*), Bash(yarn:*),
  Bash(tokei:*), Bash(scc:*), Bash(lizard:*), Bash(radon:*),
  Bash(lychee:*),
  Bash(pip-audit:*), Bash(pip:*), Bash(uv:*), Bash(poetry:*),
  Bash(gitleaks:*), Bash(trufflehog:*),
  Task, TaskCreate, TaskUpdate, TaskList, TaskGet
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

Run these git commands in parallel to gather branch state:

```bash
# Run in parallel (single message with multiple Bash tool calls)
git branch --show-current
git status --porcelain=v1 -b
git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null || echo "main"
```

Then sequentially (depends on merge-base):

```bash
MERGE_BASE=$(git merge-base HEAD origin/main)
git diff --name-only ${MERGE_BASE}..HEAD
git diff --stat ${MERGE_BASE}..HEAD
```

Use **Glob** tool (not `find`) to detect tooling:

```
Glob: Cargo.toml, pyproject.toml, package.json, Makefile, justfile, flake.nix
```

## How to work (Task tool with specialized agents)

Use the **Task** tool to spawn specialized agents. Launch independent agents in parallel (single message with multiple Task tool calls).

### Wave 1 — Explore (find what matters)

Use the Task tool with `subagent_type: "Explore"`:

```
Task(subagent_type="Explore", prompt="Analyze the changes on this branch for PR cleanup:
- Map semantic hotspots vs mechanical changes
- Flag interface/contract touchpoints (public APIs, schemas, CLI)
- Flag risk surface deltas (unsafe, concurrency, IO, deps)
- Identify repo-native gate commands (cargo xtask selftest, make check, etc.)
- Report with anchors: file paths, function names, commit refs
Do NOT make changes, just analyze and report.")
```

### Wave 2 — Plan (cleanup plan with maintainability intent)

Use the Task tool with `subagent_type: "Plan"`:

```
Task(subagent_type="Plan", prompt="Create a cleanup plan for this branch:
- Propose cleanup actions that improve maintainability without scope creep
- Separate 'quick wins' (format/lint) vs 'follow-ups' (bigger refactors)
- Recommend which tools to run (gate-first, then targeted checks)
- Suggest commit strategy if it helps review (mechanical vs semantic commits)
Do NOT implement, just plan.")
```

### Wave 3 — Improve & fix (apply changes)

Use the Task tool with `subagent_type: "general-purpose"` for fixing:

```
Task(subagent_type="general-purpose", prompt="Apply the cleanup plan:
- Run the repo's gate command and fix findings
- Apply safe mechanical fixes (format, lint, docs drift)
- Tighten boundaries where clearly beneficial
- Use Edit tool for file changes, not sed/awk
- Save tool outputs to receipts dir if created")
```

### Wave 4 — Verify & report (prove readiness)

After fixes, run verification yourself:

```bash
cargo xtask selftest  # or repo-native gate
```

Then produce the cleanup report.

## Native tool preferences

| Task | Use | Avoid |
|------|-----|-------|
| Read files | `Read` tool | `cat`, `head`, `tail` |
| Search content | `Grep` tool | `grep`, `rg` bash |
| Find files | `Glob` tool | `find`, `ls` |
| Edit files | `Edit` tool | `sed`, `awk` |
| Write files | `Write` tool | `echo >`, heredocs |

Reserve Bash for: git operations, build commands, test runners.

## Output (cleanup report)

At the end, provide a cleanup report. Include:

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

## Progress tracking (use TaskCreate/TaskUpdate)

Create tasks to track progress:

```
TaskCreate(subject="Gather branch context", description="Get branch info, merge-base, changed files, detect tooling", activeForm="Gathering context")
TaskCreate(subject="Wave 1: Explore changes", description="Map hotspots, interfaces, risk surfaces via Explore agent", activeForm="Exploring changes")
TaskCreate(subject="Wave 2: Plan cleanup", description="Create cleanup plan via Plan agent", activeForm="Planning cleanup")
TaskCreate(subject="Wave 3: Apply fixes", description="Run gate, apply mechanical fixes", activeForm="Applying fixes")
TaskCreate(subject="Wave 4: Verify and report", description="Re-run checks, produce cleanup report", activeForm="Verifying changes")
```

Update task status as you work:
- `TaskUpdate(taskId="...", status="in_progress")` when starting
- `TaskUpdate(taskId="...", status="completed")` when done

Now proceed through the tasks.
