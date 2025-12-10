---
id: DESIGN-TPL-AGENT-ERGONOMICS-001
title: "Agent Hints and Bundles Referential Integrity"
doc_type: design_doc
version: 3.3.8
stories: [US-TPL-PLT-001]
requirements: [REQ-TPL-AGENT-ERGONOMICS]
acs:
  - AC-TPL-HINTS-REFERENTIAL-INTEGRITY
  - AC-TPL-HINTS-KERNEL-SIGNALS
  - AC-TPL-BUNDLE-REFERENTIAL-INTEGRITY
adrs: [ADR-0004, ADR-0005]
status: accepted
---

# Agent Hints and Bundles Referential Integrity

## Overview

LLM/AI agents operating in this repository rely on two key interfaces:

1. **Agent hints** (`/platform/agent/hints`, `cargo xtask suggest-next`) - prioritized work suggestions
2. **Bundles** (`cargo xtask bundle <TASK>`) - context packs for focused work

Both interfaces MUST validate that referenced IDs (ACs, REQs, tasks) exist in the
source of truth (`spec_ledger.yaml`, `tasks.yaml`). This prevents agents from
wasting cycles on phantom references and makes governance drift visible.

## Referential Integrity Guarantees

### Agent Hints

When generating hints:

1. All `ac_ids` in task definitions are validated against `spec_ledger.yaml`
2. Invalid references produce a `warnings` array in the response
3. Hints with invalid references may be excluded or flagged

### Kernel AC Signals

When kernel-tagged ACs (those with `must_have_ac: true` and `kernel` tag) are failing:

1. The hints endpoint surfaces a high-priority governance hint
2. `kind: governance`, `priority: high`, `reason.code: KERNEL_AC_FAILING`
3. Agents should prioritize fixing kernel regressions before other work

### Bundle Generation

When generating bundles:

1. All AC IDs in the task's `acs` array are validated
2. Non-existent ACs are logged as warnings
3. Exit code 0 with warnings (not silent success)
4. Optional strict mode via `BUNDLE_STRICT_REFS=1`

## Current Implementation

- **Hints**: `crates/spec-runtime/src/hints.rs` - validation and kernel signal logic
- **Bundle**: `crates/xtask/src/commands/bundle.rs` - referential integrity checks
- **HTTP**: `crates/app-http/src/agent.rs` - exposes hints via HTTP

## Testing

Each AC has corresponding tests:
- `@AC-TPL-HINTS-REFERENTIAL-INTEGRITY` - BDD in `agent_hints.feature`
- `@AC-TPL-HINTS-KERNEL-SIGNALS` - BDD + unit tests in `hints.rs`
- `@AC-TPL-BUNDLE-REFERENTIAL-INTEGRITY` - BDD in `bundles.feature`
