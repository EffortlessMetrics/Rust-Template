---
id: DESIGN-TPL-SKILLS-GOVERNANCE-001
title: Skills Governance Framework
author: governance-system
doc_type: design_doc
date: 2025-11-28
status: published
stories: [US-TPL-PLT-001]
requirements: [REQ-TPL-SKILLS-GOVERNANCE]
tags: [platform, agent, governance, devex]
acs: [AC-TPL-SKILLS-LINT, AC-TPL-SKILLS-FRONTMATTER]
adrs: [ADR-0004, ADR-0020, ADR-0021]
---

# Skills Governance Framework

## Problem

Skills (high-level workflows in `.claude/skills/`) need structural consistency and safety guardrails. Without governance, skills may have inconsistent formats, missing documentation, or expose system state unsafely to agents.

## Solution

Implement a lint framework that validates:
1. **Frontmatter structure**: All skills must have valid YAML front-matter (name, description, trigger, etc.)
2. **Enum validation**: Required fields use well-defined enums (e.g., trigger modes, permission levels)
3. **Secret detection**: Skills don't contain hardcoded API keys or credentials
4. **Reference integrity**: All referenced skills or tools exist and are properly documented

## Implementation Approach

- **Lint tool**: `cargo xtask skills-lint` runs structural checks
- **Integration**:
  - Runs in `cargo xtask selftest`
  - Runs in `cargo xtask precommit` when skills change
  - Integrated into CI via `.github/workflows/ci-skills.yml`
- **Error levels**: Hard errors (security, frontmatter) vs. warnings (style, quality hints)
- **Gradual adoption**: Only hard, actionable checks in lint; style guidance in docs/SKILLS_GOVERNANCE.md

## Notes

- Skills are the agent's window into safe workflows; governance is critical for safety
- See `.claude/skills/*/SKILL.md` for examples
- Lint rules are conservative and only enforce constraints that affect safety or consistency
