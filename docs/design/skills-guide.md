---
id: DESIGN-TPL-SKILLS-GUIDE-001
title: Agent Skills Documentation and Governance
author: governance-system
doc_type: design_doc
date: 2025-11-22
status: draft
stories: [US-TPL-PLT-001]
requirements: [REQ-TPL-SKILLS-GUIDE]
tags: [platform, agent, docs]
acs: [AC-TPL-SKILLS-GUIDE-001, AC-TPL-SKILLS-ALIGN-001]
adrs: [ADR-0004]
---

# Agent Skills Documentation and Governance

## Problem

Agent Skills exist in `.claude/skills/` but lack standardized structure, documentation of best practices, and alignment verification with DevEx flows. Without guidance, Skills can drift from intended workflows or duplicate functionality.

## Solution

Create `docs/AGENT_SKILLS.md` documenting the Skills architecture, templates, and best practices. Ensure existing Skills align with documented workflows (onboarding, feature dev, maintenance, release) as defined in `specs/devex_flows.yaml`.

## Implementation Approach

**Documentation**: Create `docs/AGENT_SKILLS.md` with sections:
1. **Overview**: What Skills are, when to use them vs direct commands
2. **Skill Templates**: Frontmatter structure, required sections, examples
3. **Workflow Mapping**: How each Skill maps to DevEx flows
4. **Best Practices**: Naming conventions, trigger conditions, validation gates
5. **Maintenance**: How to update Skills when flows change

**Alignment Verification**: Create manual review checklist ensuring:
- Each Skill references appropriate xtask commands from `devex_flows.yaml`
- Skills use platform APIs (`/platform/*`) for discovery not file parsing
- Skills enforce selftest and ac-coverage as validation gates
- No duplicate workflows across Skills

**Required Skills**:
- `bootstrap-dev-env` -> onboarding flow
- `governed-feature-dev` -> AC-first development flow
- `governed-maintenance` -> doctor + audit + docs flow
- `governed-release` -> release preparation flow
- `governed-governance-debug` -> selftest failure recovery flow

**Benefits**: Standardized Skill structure, agents follow correct workflows, easier to maintain and update Skills.
