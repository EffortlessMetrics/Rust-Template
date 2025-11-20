---
doc_type: design_doc
id: DESIGN-PLT-001
title: "Platform Developer Experience & Governance"
status: approved
owner: platform-team
stories:
  - US-TPL-PLT-001
requirements:
  - REQ-PLT-ONBOARDING
  - REQ-PLT-DESIGN-SCAFFOLDING
  - REQ-PLT-SECURITY-GOVERNANCE
  - REQ-PLT-DOCS-CONSISTENCY
  - REQ-PLT-RELEASE-SAFETY
  - REQ-PLT-DEVEX-CONTRACT
  - REQ-TPL-PLATFORM-INTROSPECTION
adrs:
  - ADR-0002
  - ADR-0005
  - ADR-0006
  - ADR-0007
---

# Platform Developer Experience & Governance

## Context
The template needs to provide a "pit of success" for developers, ensuring that the right way (spec-first, secure, documented) is the easy way.

## Design
We implement `cargo xtask` commands that orchestrate the entire lifecycle:
1.  **Onboarding**: `doctor`, `check`, `selftest`.
2.  **Design**: `adr-new`, `ac-new`.
3.  **Security**: `audit`, `sbom-local`.
4.  **Release**: `release-prepare`, `release-verify`.

## Governance
All flows are defined in `specs/devex_flows.yaml` and enforced by `selftest`.
Documentation policies are defined in `specs/doc_policies.yaml` and enforced by `docs-check`.
