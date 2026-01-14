---
id: HOWTO-AI-FIRST-HOUR-001
title: "AI Agent First Hour: Autonomous Onboarding"
doc_type: how_to
status: published
audience: ai-agents, llm-tools
tags: [ai, agent, onboarding, quickstart, autonomous]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-ONBOARDING, REQ-PLT-DEVEX-CONTRACT]
acs: [AC-PLT-ENV-CHECK, AC-PLT-009, AC-TPL-AGENT-HINTS]
adrs: [ADR-0002, ADR-0005]
last_updated: 2025-12-27
---

# AI Agent First Hour: Autonomous Onboarding

> **For AI Agents/LLMs:** This guide is your entry point. It shows how to bootstrap,
> orient, discover work, and validate changes using the platform's structured APIs
> rather than browsing files.

**Kernel Version:** v3.3.9-kernel | **Template Version:** v3.3.14

---

## 1. Immediate Context Acquisition (2 minutes)

Before doing anything else, query the platform APIs to understand the current state:

```bash
# Start the service (if not already running)
cargo run -p app-http &
sleep 3

# Get governance health and counts
curl -s http://localhost:8080/platform/status | jq '.governance'

# Get prioritized work suggestions
curl -s http://localhost:8080/platform/agent/hints | jq '.hints[:3]'
```

**Key fields in `/platform/status`:**
- `governance.ac_coverage` - How many ACs are passing/failing/unknown
- `governance.tasks.by_status` - Work item counts by status
- `governance.friction.open` - Number of open DevEx issues

---

## 2. Environment Bootstrap (5 minutes)

Use the one-command setup - don't run individual doctor steps:

```bash
# One-command environment setup
cargo xtask dev-up
```

This runs:
1. `doctor` - Validates Rust, Nix, git, conftest
2. `install-hooks` - Sets up pre-commit hooks
3. `kernel-smoke` - Quick baseline validation
4. `ac-status --summary` - Current AC coverage

**If dev-up fails:**

```bash
# Run doctor alone to see what's missing
cargo xtask doctor
```

---

## 3. Understanding Current State (5 minutes)

Query the full governance graph and task list:

```bash
# Full governance graph (stories → REQs → ACs → tests → docs)
curl -s http://localhost:8080/platform/graph | jq '.governance | {stories: .stories | length, requirements: .requirements | length, acs: .acceptance_criteria | length}'

# All tasks with optional filtering
curl -s "http://localhost:8080/platform/tasks?status=Todo" | jq '.tasks[:5]'

# Documentation inventory
curl -s http://localhost:8080/platform/docs/index | jq '.docs | length'
```

**CLI equivalents:**

```bash
cargo xtask ac-status --summary      # AC coverage summary
cargo xtask tasks-list               # List all tasks
cargo xtask help-flows               # Available workflows
```

---

## 4. Finding Work (3 minutes)

The platform provides prioritized work suggestions:

```bash
# Prioritized hints (Todo/InProgress tasks with AC/REQ IDs)
curl -s http://localhost:8080/platform/agent/hints | jq '.hints[] | {id: .id, title: .title, priority: .priority, ac_ids: .ac_ids}'

# Suggest next step given current task
curl -s "http://localhost:8080/platform/tasks/suggest-next?task_id=TASK-001" | jq
```

**CLI equivalent:**

```bash
cargo xtask suggest-next TASK-001
```

**Priority interpretation:**
- `high` - Blocking work, do first
- `medium` - Standard priority
- `low` - Can defer

---

## 5. Generating Context Bundles (5 minutes)

Before implementing, generate a focused context bundle:

```bash
# List available bundle tasks
cargo xtask bundle --list

# Generate bundle for AC implementation
cargo xtask bundle implement_ac

# Bundle output location
ls bundle/implement_ac/
# - bundle.yaml   (manifest with task_id, requirement_ids, ac_ids)
# - context.md    (bundled files in markdown format)
```

**Working with bundles:**
1. Read `bundle.yaml` to understand scope and dependencies
2. Use `context.md` as your primary working context
3. Stay within the bundle - don't scan the entire repo
4. Bundles are ephemeral (not versioned); validate via tests

---

## 6. Validation Loop (5 minutes)

After making changes, validate using the governance ladder:

```bash
# Fast checks (format, clippy, unit tests)
cargo xtask check

# Only changed code
cargo xtask test-changed

# AC-specific tests
cargo xtask test-ac AC-TPL-001

# AC health mapping
cargo xtask ac-status

# Full governance gate (before PR)
cargo xtask selftest
```

**Validation order matters:**
1. `check` first (fast feedback)
2. `test-changed` or `test-ac` for focused testing
3. `selftest` only when ready for review

---

## 7. Decision Capture

When you encounter ambiguity or make non-trivial decisions:

**For architectural decisions:**

```bash
cargo xtask adr-new "Title of Decision"
# Edit docs/adr/ADR-NNNN-title-of-decision.md
```

**For DevEx friction:**

```bash
# Add to friction log
cargo xtask friction-list --status open  # View current issues
# Create friction/FRICTION-XXXX.yaml for new issues
```

**For design questions:**
- File GitHub issue with REQ/AC references
- Query existing questions: `curl http://localhost:8080/platform/questions`

---

## 8. Key Differences from Human Workflow

| Human Approach | Agent Approach |
|----------------|----------------|
| Browse files to understand codebase | Query `/platform/graph` and `/platform/status` |
| Read README and docs | Use `cargo xtask bundle` for focused context |
| Manually track work | Poll `/platform/agent/hints` for prioritized tasks |
| Run tests ad-hoc | Use validation ladder: `check` → `test-changed` → `selftest` |
| Edit files directly | Generate bundles, implement, validate |

**Agent advantages:**
- Structured JSON APIs vs. text parsing
- Prioritized work suggestions
- Pre-computed AC→test→doc mappings
- Deterministic validation gates

---

## Quick Reference Commands

```bash
# Bootstrap
cargo xtask dev-up

# Orient
cargo xtask kernel-status
curl http://localhost:8080/platform/status | jq

# Find work
curl http://localhost:8080/platform/agent/hints | jq
cargo xtask suggest-next TASK-ID

# Generate context
cargo xtask bundle implement_ac

# Validate
cargo xtask check
cargo xtask selftest
```

---

## See Also

- [CLAUDE.md](../../CLAUDE.md) - Full agent operating constitution
- [AGENT_GUIDE.md](../AGENT_GUIDE.md) - Comprehensive agent reference
- [first-hour.md](./first-hour.md) - Human-focused first hour guide
- [Platform API Reference](../reference/platform_api_contract.md) - API documentation
