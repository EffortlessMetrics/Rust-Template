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
acs: [AC-TPL-PLATFORM-UI-DASHBOARD, AC-TPL-PLATFORM-UI-GRAPH, AC-TPL-PLATFORM-UI-FLOWS]
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
