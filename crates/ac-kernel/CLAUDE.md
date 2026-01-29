# ac-kernel – CLAUDE.md

**Tier:** Kernel (Layer 2)
**Publish:** Yes
**Dependencies:** gov-model, serde, anyhow

## Purpose

Core AC governance logic. Owns the AC data model, coverage parsing, history analysis, and the `AcKernel` facade. This is the authoritative source for AC-related types and semantics.

## Key Exports

- `AcKernel` – Main facade for loading AC status and history
- `AcStatus` – Pass/Fail/Unknown enum
- `AcEvidence` – Unified test evidence model (ADR-0024)
- `SpecLayout` – Path conventions for AC artifacts
- `parse_ledger_with_metadata()` – Ledger parsing
- `parse_ac_coverage()` – Coverage.jsonl parsing
- `AcHistoryReport` – Historical trend analysis

## When to Modify

- Changing AC status semantics
- Adding new evidence sources
- Extending the `AcKernel` facade

## When NOT to Modify

- Adding CLI output (that goes in xtask)
- Adding HTTP endpoints (that goes in gov-http)
- Adding CI/SLO thresholds (those go in workflows)

## Architectural Notes

- **Standalone**: No CLI, no colors, no shell assumptions
- **Versioned schemas**: `AC_STATUS_SCHEMA_VERSION`, `AC_HISTORY_SCHEMA_VERSION`
- **Shape-lock tests**: Schema stability is enforced by tests

## Key Files

- `specs/spec_ledger.yaml` – Source of AC definitions
- `target/ac/coverage.jsonl` – BDD coverage output
- `artifacts/ac-status/` – Historical snapshots

## Consumers

`xtask` (ac-status, ac-coverage commands), `acceptance`, `gov-http`

## See Also

- `README.md` in this crate for full API documentation
- `docs/reference/ac-status-json-schema.md` for output schema
- ADR-0024 for evidence model specification
