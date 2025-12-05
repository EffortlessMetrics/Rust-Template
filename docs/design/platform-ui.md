---
id: DESIGN-TPL-PLATFORM-UI-001
title: Platform Web UI
author: governance-system
doc_type: design_doc
date: 2025-11-22
status: draft
stories: [US-TPL-PLT-001]
requirements: [REQ-TPL-PLATFORM-UI]
tags: [platform, devex, ui]
acs: [AC-TPL-PLATFORM-UI-DASHBOARD, AC-TPL-PLATFORM-UI-GRAPH, AC-TPL-PLATFORM-UI-FLOWS, AC-TPL-PLATFORM-UI-CONTRACT]
adrs: [ADR-0004]
---

# Platform Web UI

## Problem

Developers and operators need to visualize governance state (specs, tasks, flows, graph) without learning YAML structures or CLI commands. Current introspection relies on `/platform/*` JSON APIs which are machine-friendly but not human-friendly.

## Solution

Provide a web-based dashboard served by the same binary at `/ui` or `/` that renders governance data from the platform introspection APIs. The UI will display:

1. **Dashboard**: Platform status, governance health metrics, recent activity
2. **Graph View**: Mermaid.js visualization of stories -> requirements -> ACs -> tests
3. **Flows View**: DevEx workflows and available tasks from `devex_flows.yaml`

## Implementation Approach

**Backend**: Add static file serving to `crates/app-http/src/platform.rs` using `axum::routing::ServeDir` or embedded HTML templates.

**Frontend**: Single-page HTML with vanilla JavaScript that fetches from `/platform/status`, `/platform/graph`, `/platform/devex/flows` and renders using:
- Mermaid.js for graph visualization
- Simple HTML/CSS for dashboard and flows tables

**Structure**:
```
static/
+-- index.html       # Dashboard + navigation
+-- graph.html       # Mermaid graph visualization
+-- flows.html       # Flows and tasks view
```

**Benefits**: No build step, no NPM dependencies, works offline after first load.

## UI Contract System

The platform UI is governed by a contract specification that ensures UI surfaces are stable and testable.

### Contract Specification

The UI contract is defined in `specs/ui_contract.yaml`:

```yaml
screens:
  - id: platform_dashboard
    route: "/"
    aliases: ["/ui"]
    regions:
      - id: "dashboard.health"
        kind: "panel"
      - id: "dashboard.nav"
        kind: "navigation"

region_kinds:
  panel: "Grouped content section"
  navigation: "Links to other screens"
```

### DOM Anchors

Each region in the contract maps to a `data-uiid` attribute in the HTML:

```rust
// In Maud templates
div data-uiid="dashboard.health" { /* Health panel content */ }
div data-uiid="dashboard.nav" { /* Navigation content */ }
```

### Validation

The UI contract is validated at multiple levels:

1. **YAML Schema**: `spec_runtime::load_ui_contract()` validates structure and uniqueness
2. **API Endpoint**: `/platform/ui/contract` exposes the contract as JSON
3. **DOM Tests**: Integration tests verify `data-uiid` attributes match the contract
4. **Selftest**: `cargo xtask ui-contract-check` runs as part of step 9

### Benefits

- **Stability**: Agents and external tools can rely on stable UI anchors
- **Testing**: Playwright/E2E tests can reference contract region IDs
- **Discovery**: `/platform/ui/contract` enables programmatic UI introspection
- **Governance**: UI changes require updating the contract, making changes explicit
