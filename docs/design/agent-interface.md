---
id: DESIGN-TPL-AGENT-INTERFACE-001
title: Agent-Native Interface
author: governance-system
doc_type: design_doc
date: 2025-11-22
status: draft
stories: [US-TPL-PLT-001]
requirements: [REQ-TPL-AGENT-INTERFACE]
tags: [platform, ai, structural]
acs: [AC-TPL-AGENT-SKILLS, AC-TPL-AGENT-HINTS]
adrs: [ADR-0004]
---

# Agent-Native Interface

## Problem

AI agents need structured workflows to navigate this repository's governance contracts without guessing. Current state requires agents to read CLAUDE.md and reverse-engineer workflows from CLI commands, leading to incorrect assumptions about allowed operations.

## Solution

Provide formal Skill definitions in `.claude/skills/` that map to high-level workflows (not individual commands). Each Skill encodes:
- When to use it (trigger conditions)
- What steps to execute (xtask commands, platform API calls)
- What validation gates to check (selftest, ac-coverage)

**Core Skills**:
1. **bootstrap-dev-env**: Environment setup and health validation
2. **governed-feature-dev**: AC -> BDD -> Code -> Selftest workflow
3. **governed-release**: Prepare -> Verify -> Tag -> CI workflow
4. **governed-maintenance**: Doctor -> Audit -> Docs -> Graph -> Selftest

## Implementation Approach

**Skills Structure**:
```
.claude/skills/
+-- bootstrap-dev-env/SKILL.md
+-- governed-feature-dev/SKILL.md
+-- governed-release/SKILL.md
+-- governed-maintenance/SKILL.md
```

Each `SKILL.md` contains frontmatter (name, description, trigger) and markdown sections (steps, validation, recovery).

**Agent Hints API**: Add `GET /platform/agent/hints` endpoint returning prioritized task suggestions:
```json
{
  "suggested_tasks": [
    {"id": "TASK-001", "status": "Todo", "priority": "high", "requirement": "REQ-TPL-HEALTH"}
  ]
}
```

**Benefits**: Agents follow correct workflows, governance gates enforced, reduces agent confusion and invalid operations.
