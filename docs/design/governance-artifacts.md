---
id: DESIGN-TPL-GOV-ARTIFACTS-001
title: Governance Artifacts and Decision Capture
author: governance-system
doc_type: design_doc
date: 2025-11-28
status: published
stories: [US-TPL-PLT-001]
requirements: [REQ-TPL-GOV-ARTIFACTS]
tags: [platform, governance, ai, devex]
acs: [AC-TPL-QUESTIONS-LOGGED, AC-TPL-ADR-STRUCTURE]
adrs: [ADR-0005, ADR-0020, ADR-0021, ADR-0022]
---

# Governance Artifacts and Decision Capture

## Problem

Without systematic decision capture, the system's design rationale becomes implicit and hard to trace. New developers and agents don't understand why certain choices were made, leading to regressions or re-opening settled decisions.

## Solution

Establish three artifact types for capturing decisions:
1. **ADRs** (Architectural Decision Records): Major design decisions with alternatives evaluated
2. **Issues**: Tracking questions, blockers, and spec clarifications
3. **Friction logs**: Process/tooling friction points for retrospectives

Each type has a clear purpose and integration point in the governance system.

## Implementation Approach

- **ADR storage**: `docs/adr/ADR-*.md` with standard front-matter and structure
- **Linking**: Specs link to relevant ADRs via `adr: [ADR-00xx]` in requirement definitions
- **Issues**: GitHub issues reference REQ/AC IDs and link to relevant ADRs
- **Friction logs**: `FRICTION_LOG.md` appended during development for post-mortems
- **Validation**: `cargo xtask adr-check` verifies all ADR references are valid

## Notes

- ADRs are not optional; non-trivial design choices must be captured
- See `docs/adr/` directory for examples and template
- Spec ledger can reference ADRs to explain trade-offs in requirements
