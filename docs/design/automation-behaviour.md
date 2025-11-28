---
id: DESIGN-TPL-AUTOMATION-BEHAVIOUR-001
title: Automation and Workflow Behaviour
author: governance-system
doc_type: design_doc
date: 2025-11-28
status: published
stories: [US-TPL-PLT-001]
requirements: [REQ-TPL-AUTOMATION-BEHAVIOUR]
tags: [platform, devex, ci, ai]
acs: [AC-TPL-XTASK-NONINTERACTIVE, AC-TPL-FLOW-IDEMPOTENT]
adrs: [ADR-0005, ADR-0017]
---

# Automation and Workflow Behaviour

## Problem

Workflows must be deterministic and idempotent so that both humans and agents can run them safely without unexpected side effects. Xtask commands should not require interactive input in CI contexts, and flows should be replayable.

## Solution

Define and enforce automation constraints:
1. **Non-interactive mode**: All xtask commands accept `--non-interactive` flag or detect CI context automatically
2. **Idempotency**: Running a command twice produces the same result (no state corruption)
3. **Flow contracts**: Each skill documents what it will and won't do, including cleanup obligations
4. **Error recovery**: Workflows provide rollback paths or can be safely re-run after failure

## Implementation Approach

- **xtask design**: Commands write state atomically; use `.bak` files and restore-on-cleanup patterns
- **CI automation**: GitHub Actions workflows run with `--non-interactive` by default
- **Skill contracts**: Each SKILL.md documents idempotency guarantees
- **Testing**: Acceptance tests verify that running a flow twice is safe (see `service-init` acceptance scenarios)

## Notes

- Idempotency is enforced through acceptance test scenarios (e.g., @AC-PLT-021)
- CI gates reject changes that introduce interactive prompts
- See `specs/features/*.feature` for explicit idempotency assertions
