# Architecture Decision Records

This repository tracks ADRs under `docs/adr/`. Keep each record stable and reference it from `specs/spec_ledger.yaml` when it applies to your service.

## Status and known warnings

- ADRs `0001`–`0007` are part of the template baseline and referenced today.
- ADRs `0008`–`0016` are template placeholders that are intentionally **unreferenced** during pilot adoption. Tooling may emit warnings about them; either link the ones you adopt into `spec_ledger.yaml` or move the rest to `docs/adr/archive/` once you decide.
- **ADR-0032**: Guard Workflow Architecture Pattern (2026-03-23) — Documents the decision to maintain complementary guard implementations: Rust-Template workflows with Rego policies for governance, and guard repos (depguard, diffguard, covguard) as portable Rust CLI tools with structured output.

## Recent ADRs

| Number | Title | Date | Status |
|--------|-------|------|--------|
| 0032 | Guard Workflow Architecture Pattern | 2026-03-23 | Accepted |
| 0031 | Kernel Pack Distribution | 2025-12-10 | Accepted |
| 0030 | Microcrate Architecture | 2025-12-10 | Accepted |
| 0024 | AC Evidence and Kernel Gate | 2025-12-10 | Accepted |
| 0023 | AC Coverage JSONL as Primary Truth Source | 2025-12-10 | Accepted |
| 0022 | Platform Metadata and Test Isolation | 2025-12-10 | Accepted |
| 0021 | Claude Code Agents Governance | 2025-12-10 | Accepted |
| 0020 | Claude Code Skills Governance | 2025-12-10 | Accepted |
| 0019 | Governance Repository and FS Adapter | 2025-12-10 | Accepted |
| 0017 | Tier1 Selftest Gate | 2025-12-10 | Accepted |
| 0016 | IDP Tiles JSON Contracts | 2025-12-10 | Accepted |
| 0007 | Dependency Security Health | 2025-12-10 | Accepted |
| 0006 | Supply Chain Hardening | 2025-12-10 | Accepted |
| 0005 | Xtask Selftest Single Gate | 2025-12-10 | Accepted |
| 0004 | Policy and LLM Governance | 2025-12-10 | Accepted |
| 0003 | Spec and BDD as Source of Truth | 2025-12-10 | Accepted |
| 0002 | Nix First Dev Env | 2025-12-10 | Accepted |
| 0001 | Hexagonal Architecture | 2025-12-10 | Accepted |
