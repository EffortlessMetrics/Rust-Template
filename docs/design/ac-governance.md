---
id: DESIGN-PLT-AC-GOVERNANCE-001
title: AC Governance Framework
author: governance-system
doc_type: design_doc
date: 2025-11-28
status: published
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-AC-GOVERNANCE]
tags: [platform, governance]
acs: [AC-PLT-002, AC-PLT-004]
adrs: [ADR-0001, ADR-0020, ADR-0021]
---

# AC Governance Framework

## Problem

Without formal acceptance criteria governance, feature coverage becomes unclear and system behaviour drifts from specification. There needs to be a way to:
- Define acceptance criteria tied to requirements
- Automatically validate AC coverage
- Track AC health across stories and requirements
- Ensure each AC has concrete test evidence

## Solution

Implement a structured AC governance layer in `specs/spec_ledger.yaml` where:
1. Each Requirement lists acceptance criteria with unique IDs (e.g., `AC-PLT-002`)
2. Each AC has a clear description, test list, and status
3. `cargo xtask ac-status` generates health reports
4. CI enforces AC completeness before release

## Implementation Approach

- **Spec ledger**: `specs/spec_ledger.yaml` houses all ACs under their parent requirements
- **AC-status command**: Reads ledger, checks test results, generates `docs/feature_status.md`
- **Validation gates**:
  - Each AC must have at least one test in the `tests` array
  - Tests must map to scenarios in feature files (e.g., `specs/features/*.feature`)
  - `cargo xtask selftest` enforces via Rego policy
- **Doc links**: Each AC can reference relevant ADRs for design rationale

## Notes

- AC health is the primary view of "done-ness" for features
- Spec ledger is source-of-truth; generated reports (feature_status.md) are views
- See `docs/feature_status.md` for current AC health summary
