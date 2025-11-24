---
id: DESIGN-TPL-PLATFORM-INTROSPECTION-001
title: Platform Introspection API
author: steven
doc_type: design_doc
date: 2025-11-20
status: draft
stories: [US-TPL-PLT-001]
requirements:
  - REQ-TPL-PLATFORM-INTROSPECTION
  - REQ-TPL-PLATFORM-TASKS
  - REQ-TPL-GRAPH-VISUALIZATION
tags: [platform, devex, api]
acs: [AC-TPL-PLATFORM-GRAPH, AC-TPL-PLATFORM-DEVEX, AC-TPL-PLATFORM-DOCS]
adrs: [ADR-0001]
---

# Platform Introspection API

## Context

The Rust-as-Spec platform has built comprehensive governance machinery:
- Spec ledger (stories -> requirements -> ACs)
- DevEx flows (commands and workflows)
- Doc index (design docs, plans, runbooks)
- Policies (doc policies, service policies)

However, this governance state was only accessible via CLI tools (`cargo xtask graph-export`, `cargo xtask docs-check`, etc.). Operators, agents, and platform tools had no way to query the governance graph at runtime without cloning the repository and running xtask commands.

## Decision

Expose the governance state via HTTP endpoints under `/platform/*`:

- `/platform/graph` - Complete governance graph (stories, requirements, ACs, docs, commands, flows)
- `/platform/devex/flows` - DevEx commands and flows from `specs/devex_flows.yaml`
- `/platform/docs/index` - Document index from `specs/doc_index.yaml`
- `/platform/status` - High-level governance health status

## Implementation

### Shared `spec-runtime` Crate

Create `crates/spec-runtime` as a library crate containing:

```
spec-runtime/
+-- src/
|   +-- lib.rs
|   +-- ledger.rs    # Load spec_ledger.yaml
|   +-- devex.rs     # Load devex_flows.yaml
|   +-- docs.rs      # Load doc_index.yaml
|   +-- graph.rs     # build_graph() from all specs
```

This crate is shared between:
- `xtask` (CLI tooling)
- `app-http` (runtime endpoints)

**Benefit:** One source of truth for spec loading, no CLI vs runtime drift.

### HTTP Endpoints

Add `crates/app-http/src/platform.rs`:

```rust
pub fn router() -> Router {
    Router::new()
        .route("/graph", get(get_graph))
        .route("/devex/flows", get(get_devex_flows))
        .route("/docs/index", get(get_docs_index))
        .route("/status", get(get_status))
}
```

Each endpoint:
1. Loads specs using `spec_runtime::load_all_specs()`
2. Returns JSON representation
3. Matches exact schema used by xtask commands

### JSON Schemas

**`/platform/graph`:**
```json
{
  "nodes": [
    { "id": "US-TPL-001", "label": "Service Core", "type": "story" },
    { "id": "REQ-TPL-HEALTH", "label": "Health Check", "type": "requirement" }
  ],
  "edges": [
    { "source": "US-TPL-001", "target": "REQ-TPL-HEALTH", "type": "contains" }
  ]
}
```

**`/platform/devex/flows`:**
```json
{
  "commands": {
    "doctor": { "category": "onboarding", "summary": "...", "required": true }
  },
  "flows": {
    "onboarding": { "name": "Onboarding", "steps": ["doctor", "check"] }
  }
}
```

**`/platform/docs/index`:**
```json
{
  "schema_version": "1.0",
  "docs": [
    {
      "id": "DESIGN-TPL-HEALTH-001",
      "file": "docs/design/health-endpoint.md",
      "requirements": ["REQ-TPL-HEALTH"]
    }
  ]
}
```

## Consequences

### Positive

- **Operators** can introspect service governance state without cloning repo
- **Agents** can query `/platform/graph` to understand service structure
- **Dashboards** (Backstage, OpsLevel, Cortex) can scrape governance data
- **Self-describing**: Runtime exposes the specs it's governed by

### Neutral

- Adds `spec-runtime` dependency to `app-http`
- Specs must be available at runtime (bundled in container)

### Negative

- None identified

## Alternatives Considered

### 1. Keep governance CLI-only
**Rejected:** Limits usefulness to developers with repo access. Agents and operators can't introspect running services.

### 2. Separate runtime endpoints from xtask loaders
**Rejected:** Creates drift risk between CLI and runtime representations.

### 3. GraphQL instead of REST
**Rejected:** REST + JSON is simpler, widely supported by agents and dashboards. GraphQL adds complexity without clear benefit for this use case.

## Follow-up Work

- Add BDD scenarios for `/platform/*` endpoints (tag: `@AC-TPL-PLATFORM-*`)
- Update runbook to document introspection endpoints
- Consider adding `project-health` and `feature-health` endpoints
- Add `--format mermaid` to `graph-export` for visual diagrams
