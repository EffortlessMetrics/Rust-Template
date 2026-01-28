---
id: API-UI-CONTRACT-001
title: Platform UI Contract API
doc_type: reference
status: published
audience: developers, idp-operators, integration-developers
tags: [api, ui-contract, platform]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-PLATFORM-APIS]
acs: [AC-PLT-015]
adrs: [ADR-0005]
last_updated: 2025-12-27
---

<!-- doclint:disable orphan-version -->
<!-- Note: JSON examples contain version strings that are intentionally not tied to template version -->

# Platform UI Contract API

The UI Contract API provides the governed UI surface definitions with screens, regions, and stable `data-uiid` identifiers for tests and automation.

## Endpoints

### GET /platform/ui/contract

Returns the UI contract specification defining screens, regions, and stable `data-uiid` identifiers.

#### Request Example

```bash
curl http://localhost:8080/platform/ui/contract
```

#### Response

<!-- doclint:disable orphan-version -->
```json
{
  "schema_version": "1.0",
  "template_version": "3.3.9",
  "screens": [
    {
      "id": "dashboard",
      "name": "Platform Dashboard",
      "description": "Main dashboard showing health metrics and quick links",
      "url_path": "/",
      "regions": [
        {
          "id": "dashboard.health",
          "name": "Health Metrics",
          "description": "Governance health metrics display",
          "selector": "[data-uiid='dashboard.health']",
          "elements": [
            {
              "uiid": "metric-value",
              "description": "Individual metric value display",
              "type": "text"
            }
          ]
        },
        {
          "id": "dashboard.ac_coverage",
          "name": "AC Coverage",
          "description": "Acceptance criteria coverage summary",
          "selector": "[data-uiid='dashboard.ac_coverage']",
          "elements": []
        }
      ]
    },
    {
      "id": "graph",
      "name": "Governance Graph",
      "description": "Visual governance graph showing relationships",
      "url_path": "/ui/graph",
      "regions": [
        {
          "id": "graph.diagram",
          "name": "Graph Diagram",
          "description": "Mermaid diagram container",
          "selector": "[data-uiid='graph.diagram']",
          "elements": []
        }
      ]
    },
    {
      "id": "flows",
      "name": "Flows & Tasks",
      "description": "Developer flows and task guidance",
      "url_path": "/ui/flows",
      "regions": [
        {
          "id": "flows.devex",
          "name": "DevEx Flows",
          "description": "Developer experience flows display",
          "selector": "[data-uiid='flows.devex']",
          "elements": []
        },
        {
          "id": "flows.tasks",
          "name": "Tasks",
          "description": "Task list display",
          "selector": "[data-uiid='flows.tasks']",
          "elements": []
        }
      ]
    },
    {
      "id": "coverage",
      "name": "AC Coverage",
      "description": "Interactive AC coverage table with filtering",
      "url_path": "/ui/coverage",
      "regions": [
        {
          "id": "coverage.summary",
          "name": "Coverage Summary",
          "description": "Summary metrics display",
          "selector": "[data-uiid='coverage.summary']",
          "elements": []
        },
        {
          "id": "coverage.filters",
          "name": "Filter Controls",
          "description": "Filter buttons and search box",
          "selector": "[data-uiid='coverage.filters']",
          "elements": [
            {
              "uiid": "filter-all",
              "description": "All filter button",
              "type": "button"
            },
            {
              "uiid": "filter-passing",
              "description": "Passing filter button",
              "type": "button"
            },
            {
              "uiid": "filter-failing",
              "description": "Failing filter button",
              "type": "button"
            },
            {
              "uiid": "filter-unknown",
              "description": "Unknown filter button",
              "type": "button"
            },
            {
              "uiid": "search-box",
              "description": "Search input field",
              "type": "input"
            }
          ]
        },
        {
          "id": "coverage.table",
          "name": "Coverage Table",
          "description": "AC coverage data table",
          "selector": "[data-uiid='coverage.table']",
          "elements": []
        }
      ]
    }
  ]
}
```
<!-- doclint:enable orphan-version -->

#### Response Schema

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `schema_version` | string | Yes | UI contract schema version |
| `template_version` | string | Yes | Template version this contract applies to |
| `screens` | array | Yes | Array of screen definitions |
| `screens[].id` | string | Yes | Screen identifier (used for data-uiid prefix) |
| `screens[].name` | string | Yes | Human-readable screen name |
| `screens[].description` | string | Yes | Screen description |
| `screens[].url_path` | string | Yes | URL path to access this screen |
| `screens[].regions` | array | Yes | Array of region definitions within the screen |
| `screens[].regions[].id` | string | Yes | Region identifier (used for data-uiid) |
| `screens[].regions[].name` | string | Yes | Human-readable region name |
| `screens[].regions[].description` | string | Yes | Region description |
| `screens[].regions[].selector` | string | Yes | CSS selector for this region |
| `screens[].regions[].elements` | array | Yes | Array of element definitions (may be empty) |
| `screens[].regions[].elements[].uiid` | string | Yes | Element data-uiid identifier |
| `screens[].regions[].elements[].description` | string | Yes | Element description |
| `screens[].regions[].elements[].type` | string | Yes | Element type (button, input, text, etc.) |

---

## data-uiid Naming Convention

The `data-uiid` attribute provides stable identifiers for UI elements used in tests and automation. The format is:

```
data-uiid="{screen_id}.{region_id}.{element_id}"
```

### Examples

| UI Element | data-uiid Value |
|-------------|------------------|
| Dashboard health metrics | `dashboard.health` |
| AC coverage summary | `dashboard.ac_coverage` |
| Graph diagram | `graph.diagram` |
| DevEx flows section | `flows.devex` |
| Tasks section | `flows.tasks` |
| Coverage summary metrics | `coverage.summary` |
| Filter controls | `coverage.filters` |
| All filter button | `filter-all` |
| Search input | `search-box` |
| Coverage table | `coverage.table` |

---

## Stable Identifiers

The UI contract provides **stable identifiers** that:

1. **Persist across UI changes** - Identifiers remain stable even when layout changes
2. **Support automated testing** - Tests can locate elements without fragile selectors
3. **Enable integration** - External tools can reliably interact with UI elements

### Stability Guarantees

- Screen IDs are stable across minor versions
- Region IDs are stable across minor versions
- Element IDs are stable across minor versions
- Breaking changes require major version bump

---

## Usage in Tests

### Example: Using data-uiid in Playwright

```typescript
// Find the AC coverage summary region
const summaryRegion = page.locator('[data-uiid="coverage.summary"]');

// Find a specific filter button
const passingFilter = page.locator('[data-uiid="filter-passing"]');

// Find the search input
const searchBox = page.locator('[data-uiid="search-box"]');
```

### Example: Using data-uiid in CSS Selectors

```css
/* Target specific region */
[data-uiid="dashboard.health"] {
  border: 1px solid #ccc;
}

/* Target specific element */
[data-uiid="filter-all"] {
  background-color: #667eea;
}
```

---

## Contract Evolution

### Version 1.0

Initial UI contract schema with:
- Screen definitions with regions
- data-uiid identifiers
- Element type classification

### Future Versions

Planned additions:
- Accessibility attributes
- Responsive breakpoints
- Internationalization support

---

## Related Endpoints

- [`/platform/status`](./status.md) - Platform status with UI metadata
- [`/platform/graph`](./overview/index.md) - Governance graph endpoint

---

## Related Documentation

- [Platform API Overview](./overview/index.md)
- [Platform API Reference](../reference/platform-api-endpoints.md)
- [UI Contract Design](../design/platform-api-contract.md)
- [IDP Cell Contract](../IDP_CELL_CONTRACT.md)
