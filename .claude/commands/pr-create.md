---
description: Create PR (current branch)
argument-hint: [optional: context/intent e.g. "Issue #218", "ready", "draft", "base=main", "no-ci", "local-gate-canonical"]
allowed-tools: >
  Read, Grep, Glob,
  Bash(git status:*), Bash(git branch:*), Bash(git rev-parse:*), Bash(git symbolic-ref:*), Bash(git remote:*),
  Bash(git merge-base:*), Bash(git log:*), Bash(git show:*), Bash(git diff:*),
  Bash(gh:*),
  Bash(mkdir:*), Bash(date:*),
  Bash(make:*), Bash(just:*), Bash(nix:*),
  Bash(cargo:*), Bash(cargo-*:*), Bash(pytest:*), Bash(ruff:*), Bash(mypy:*), Bash(pyright:*),
  Bash(node:*), Bash(npm:*), Bash(pnpm:*), Bash(yarn:*),
  Bash(tokei:*), Bash(scc:*), Bash(lizard:*), Bash(radon:*),
  Bash(lychee:*),
  Bash(pip-audit:*), Bash(pip:*), Bash(uv:*), Bash(poetry:*),
  Task, TaskCreate, TaskUpdate, TaskList, TaskGet
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

Run these git commands **in parallel** (single message with multiple Bash tool calls):

```bash
# All independent - run in parallel
git branch --show-current
git status --porcelain=v1 -b
git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null || echo "main"
git rev-parse --abbrev-ref HEAD@{upstream} 2>/dev/null || echo "not tracking"
```

Then sequentially (depends on merge-base):

```bash
MERGE_BASE=$(git merge-base HEAD origin/main)
git diff --name-only ${MERGE_BASE}..HEAD
git diff --stat ${MERGE_BASE}..HEAD
git log --oneline ${MERGE_BASE}..HEAD
git diff ${MERGE_BASE}..HEAD  # full diff for analysis
```

Use **Glob** tool (not `find`) to detect tooling:

```
Glob: Cargo.toml, pyproject.toml, package.json, Makefile, justfile, flake.nix
```

## How to work (Task tool with specialized agents)

Use the **Task** tool to spawn specialized agents. Launch independent agents in parallel (single message with multiple Task tool calls).

### Wave 1 — Explore (map the change)

Use the Task tool with `subagent_type: "Explore"`:

```
Task(subagent_type="Explore", prompt="Analyze this branch's changes for PR creation:
- Map where behavior changed (semantic hotspots vs mechanical)
- Identify interface/contract touchpoints (public API, schemas, CLI/config)
- Flag risk surface deltas (unsafe, concurrency, IO, deps changes)
- Identify the repo's gate command(s)
- Report with anchors: file:line, function names, commit refs
Do NOT make changes, just analyze and report findings.")
```

### Wave 2 — Plan (compose the story + evidence plan)

Use the Task tool with `subagent_type: "Plan"`:

```
Task(subagent_type="Plan", prompt="Plan the PR content structure:
- Propose a coherent narrative arc (intent → design → review path → evidence)
- Produce a crisp Interface & compatibility verdict
- Recommend which verification tools support the claims
- Surface key decision points affecting maintainability
Do NOT write the PR body yet, just plan the structure.")
```

### Wave 3 — Improve (gather evidence)

Use the Task tool with `subagent_type: "general-purpose"` to run verification:

```
Task(subagent_type="general-purpose", prompt="Gather verification evidence for the PR:
- Run repo's gate command if not already run (cargo xtask selftest, etc.)
- Collect tool outputs that support interface/risk claims
- Note what was verified and what wasn't
- Use Read tool to examine key changed files, not cat
Report findings for inclusion in PR body.")
```

### Wave 4 — Create the PR (gh)

Once you have the PR title and body ready, create the PR:

```bash
gh pr create --draft --title "title" --body "$(cat <<'EOF'
PR body here
EOF
)"
```

Default: create as **draft**, unless `$ARGUMENTS` clearly indicates "ready".

## Known workflow issues

### `gh pr edit` fails with Projects (classic)

If you need to update a PR body after creation, `gh pr edit` may fail with:

```
GraphQL: Cannot query field "projectCards" on type "PullRequest"
```

**Workaround:** Use REST API:

```bash
gh api -X PATCH /repos/{owner}/{repo}/pulls/{pr_number} \
  -f body="$(cat <<'EOF'
Your PR body here
EOF
)"
```

## Native tool preferences

| Task | Use | Avoid |
|------|-----|-------|
| Read files | `Read` tool | `cat`, `head`, `tail` |
| Search content | `Grep` tool | `grep`, `rg` bash |
| Find files | `Glob` tool | `find`, `ls` |

Reserve Bash for: git operations, `gh` CLI, build/test commands.

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

**CI posture rule:** If CI is disabled, state "CI disabled; local gate canonical" and provide reproduce commands.

### Complexity (future change-cost)

Tool-backed if available; otherwise proxies (hotspots/churn, module splits, API delta, unsafe delta, deps delta).
Interpret implications rather than scoring.

### Risk & rollback

Blast radius, failure modes, rollback/recovery.

### Known limits / follow-ups

Explicit deferrals and next steps.

### Retrospective (earnest)

Surprises, corrections, and what to mechanize next time.

## Progress tracking (use TaskCreate/TaskUpdate)

Create tasks to track progress:

```
TaskCreate(subject="Gather branch context", description="Get branch info, merge-base, changed files, commit history", activeForm="Gathering context")
TaskCreate(subject="Wave 1: Explore changes", description="Map behavior changes, interfaces, risk surfaces via Explore agent", activeForm="Exploring changes")
TaskCreate(subject="Wave 2: Plan PR structure", description="Plan narrative arc, interface verdict, evidence strategy", activeForm="Planning PR")
TaskCreate(subject="Wave 3: Gather evidence", description="Run verification, collect tool outputs", activeForm="Gathering evidence")
TaskCreate(subject="Wave 4: Create PR", description="Write title + body, run gh pr create", activeForm="Creating PR")
```

Update task status as you work:
- `TaskUpdate(taskId="...", status="in_progress")` when starting
- `TaskUpdate(taskId="...", status="completed")` when done

Now proceed through the tasks.
