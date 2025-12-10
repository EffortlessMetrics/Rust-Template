---
id: DESIGN-TPL-INTROSPECTION-PARITY-001
title: "CLI/HTTP Introspection Parity"
doc_type: design_doc
version: 3.3.8
stories: [US-TPL-PLT-001]
requirements: [REQ-TPL-INTROSPECTION-PARITY]
acs:
  - AC-TPL-STATUS-PARITY-CLI-HTTP
  - AC-TPL-STATUS-AC-COVERAGE
  - AC-TPL-STATUS-TASK-BREAKDOWN
adrs: [ADR-0005]
status: accepted
---

# CLI/HTTP Introspection Parity

## Overview

The Rust-as-Spec platform provides two access methods for governance introspection:

1. **CLI** (`cargo xtask status`) - for local development and CI
2. **HTTP** (`/platform/status`) - for dashboards, IDPs, and external tooling

Both surfaces MUST expose the same key governance metrics so operators and agents
get consistent information regardless of how they query the system.

## Key Fields

The following fields are guaranteed to be present in both:

| Field | Description |
|-------|-------------|
| `stories` | Total story count |
| `requirements` | Total requirement count |
| `acceptance_criteria` | Total AC count |
| `ac_coverage.total` | Total ACs tracked for coverage |
| `ac_coverage.passing` | ACs with passing tests |
| `ac_coverage.failing` | ACs with failing tests |
| `ac_coverage.unknown` | ACs with no test results |
| `task_status.todo` | Tasks in Todo state |
| `task_status.in_progress` | Tasks in InProgress state |
| `task_status.review` | Tasks in Review state |
| `task_status.done` | Tasks in Done state |

## Current Implementation

- **CLI**: `crates/xtask/src/commands/status.rs` - reads from spec_ledger and coverage files
- **HTTP**: `crates/app-http/src/platform.rs` - exposes the same data via JSON endpoint
- **Shared runtime**: `crates/spec-runtime/src/lib.rs` - provides common data loading

## Testing

Each AC has corresponding BDD scenarios in `specs/features/platform_introspection.feature`:
- `@AC-TPL-STATUS-PARITY-CLI-HTTP` - verifies parity between CLI and HTTP
- `@AC-TPL-STATUS-AC-COVERAGE` - verifies AC coverage fields
- `@AC-TPL-STATUS-TASK-BREAKDOWN` - verifies task status breakdown
