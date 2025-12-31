# Platform API Contract

This document defines the contract between the Rust kernel and its consumers (e.g., Backstage plugin, other developer portal integrations).

## Contract Hierarchy

```
Rust Kernel (source of truth)
    │
    ├── specs/openapi/openapi.yaml (formal contract)
    │
    └── /platform/* HTTP endpoints (implementation)
            │
            └── Backstage Plugin (reference consumer)
                    │
                    ├── PlatformClient.ts (typed client)
                    └── Components (UI rendering)
```

**Authority flows downward:**
1. Rust structs define the canonical data model
2. OpenAPI documents the HTTP contract
3. TypeScript types are derived from (and must match) the OpenAPI/Rust contract
4. Components render whatever the client provides

## Primary Endpoints

This section documents the primary `/platform/*` endpoints. For the complete endpoint reference, see `docs/reference/platform-api-endpoints.md`.

### GET /platform/status

Returns comprehensive governance status.

**Response:** `PlatformStatus`

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `service` | `ServiceInfo` | Yes | Service metadata |
| `governance` | `GovernanceInfo` | Yes | Governance metrics |
| `config` | `ConfigInfo` | No | Runtime config (omitted if not loaded) |
| `errors` | `ErrorSummary` | Yes | Error tracking summary (AC-TPL-ERROR-MAPPING) |

**ServiceInfo fields:**

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `service_id` | string | Yes | Unique identifier |
| `template_version` | string | Yes | Template version |
| `display_name` | string | No | Human-readable name |
| `description` | string | No | Service description |
| `links` | object | Yes | May be empty `{}` |
| `tags` | array | Yes | May be empty `[]` |

**GovernanceInfo fields:**

| Field | Type | Description |
|-------|------|-------------|
| `ledger` | `LedgerCounts` | Stories, requirements, ACs counts |
| `devex` | `DevExCounts` | Commands, flows counts |
| `docs` | `DocsCounts` | Document counts and issues |
| `tasks` | `TasksCounts` | Task counts |
| `questions` | `QuestionsInfo` | Open/answered/resolved questions |
| `friction` | `FrictionInfo` | Friction log tracking |
| `forks` | `ForksInfo` | Fork information |
| `policies` | `PoliciesInfo` | Policy enforcement status |

**PoliciesInfo.status values:**
- `"pass"` - All policies pass
- `"fail"` - One or more policies fail
- `"unknown"` - Policy status not yet evaluated (policy-test hasn't run)

**ErrorSummary fields:**

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `has_recent_errors` | boolean | Yes | Whether any errors have occurred since service start |
| `last_error` | `LastErrorSummary` | No | The most recent error (if any) |
| `stats` | `ErrorStats` | Yes | Aggregated error statistics |

**LastErrorSummary fields:**

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `category` | string | Yes | Error category (e.g., `task_not_found`, `invalid_transition`, `internal`) |
| `message` | string | Yes | Human-readable error message |
| `status_code` | integer | Yes | HTTP status code returned (4xx or 5xx) |
| `occurred_at` | string | Yes | ISO 8601 timestamp |
| `request_id` | string | No | X-Request-ID for correlation |

**ErrorStats fields:**

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `total_errors` | integer | Yes | Total errors since service start |
| `client_errors` | integer | Yes | Number of 4xx client errors |
| `server_errors` | integer | Yes | Number of 5xx server errors |

**Error category values:**
- `task_not_found` - Task ID not found in tasks.yaml
- `invalid_transition` - Invalid task status transition
- `validation` - Request validation failed
- `internal` - Internal server error

### GET /platform/docs/index

Returns documentation inventory with health validation.

**Response:** `DocsIndex`

| Field | Type | Description |
|-------|------|-------------|
| `schema_version` | string | Schema version |
| `template_version` | string | Template version |
| `docs` | `DocumentEntry[]` | All documents |
| `summary` | `DocsSummary` | Health summary |

**DocumentEntry fields:**

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | string | Yes | Document ID |
| `file` | string | Yes | File path |
| `doc_type` | string | Yes | Document type (see enum) |
| `stories` | string[] | Yes | May be empty |
| `requirements` | string[] | Yes | May be empty |
| `acs` | string[] | Yes | May be empty |
| `adrs` | string[] | Yes | May be empty |
| `doc_type_valid` | boolean | Yes | Passes type-contract validation |
| `doc_type_issue` | string | No | Issue description if invalid |

**doc_type enum:**
`adr`, `design_doc`, `impl_plan`, `requirements_doc`, `guide`, `how-to`, `how_to`, `explanation`, `reference`, `ci_workflow`, `status`

### Additional Platform Endpoints

The following endpoints are also available (see `docs/reference/platform-api-endpoints.md` for full details):

| Endpoint | Description |
|----------|-------------|
| `/platform/graph` | Full governance graph (stories -> REQs -> ACs -> tests -> docs) |
| `/platform/openapi` | OpenAPI specification (YAML format) |
| `/platform/openapi.yaml` | OpenAPI specification (alias with .yaml suffix) |
| `/platform/schema` | JSON Schema definitions for all platform data files |
| `/platform/schema/{name}` | Get specific schema by name (spec_ledger, tasks, etc.) |
| `/platform/devex/flows` | Developer flows and available xtask commands |
| `/platform/coverage` | AC coverage summary with BDD test results |
| `/platform/tasks` | Task list with optional filtering by status/requirement |
| `/platform/tasks/suggest-next` | Recommended next work for a given task |
| `/platform/tasks/graph` | Task dependency graph (JSON or Mermaid format) |
| `/platform/agent/hints` | Prioritized work suggestions for AI agents |
| `/platform/friction` | Development friction log entries |
| `/platform/questions` | Design questions and ambiguities |
| `/platform/forks` | Registered template forks/branches |
| `/platform/issues` | Unified issues aggregation (friction + questions + tasks) |
| `/platform/ui/contract` | Governed UI contract (screens, regions, data-uiid anchors) |
| `/platform/idp/snapshot` | IDP snapshot with governance health for agents |
| `/platform/debug/info` | Basic kernel and template version info |

## Stability Policy

### Versioning

The platform API follows semantic versioning principles:

- **Non-breaking additions** (new optional fields, new endpoints) may be added in minor versions
- **Breaking changes** (removing fields, changing types, changing required status) require major version bump
- The `x-template-version` in OpenAPI tracks the template version

### Optional Fields

Fields marked as optional in the Rust kernel use `#[serde(skip_serializing_if = "Option::is_none")]` or similar. This means:

1. The field may be completely absent from the JSON response
2. Consumers MUST handle the field being `undefined`/missing
3. Consumers SHOULD NOT assume a default value unless documented

### Arrays and Maps

Empty arrays and maps may be:
- Present as `[]` or `{}`
- Completely omitted (if using `skip_serializing_if = "Vec::is_empty"`)

Consumers should treat missing arrays/maps as empty.

## Ownership Model

### Rust Kernel Team Owns

- Adding/changing `/platform/*` endpoint logic
- Semantics of governance, DevEx, docs, ledger concepts
- OpenAPI definitions in `specs/openapi/openapi.yaml`
- Acceptance criteria and BDD scenarios

### Backstage/TS Team Owns

- Layout and UX of cards/pages
- Additional visualizations using existing fields
- Styling and theming
- Integration with specific Backstage instances

### Contract Change Workflow

1. **Feature request:** Backstage team files issue requesting new field/endpoint
2. **Evaluation:** Rust team evaluates:
   - If kernel concern → add to Rust, update OpenAPI, update TS types
   - If display-only → derive in TS from existing JSON
3. **Implementation:** Changes flow: Rust → OpenAPI → TS types → Components
4. **Validation:** Both sides must pass their respective tests

## TypeScript Client Contract

The `PlatformClient` class in the Backstage plugin:

### Public API (stable)

```typescript
class PlatformClient {
  getStatus(): Promise<PlatformStatus>;
  getDocsIndex(): Promise<DocsIndex>;

  // Convenience helpers
  getACCount(): Promise<number>;
  getPolicyStatus(): Promise<'pass' | 'fail' | 'unknown'>;
  getTemplateVersion(): Promise<string>;
  getOpenFrictionCount(): Promise<number>;
  isReachable(): Promise<boolean>;
}
```

### Exported Types (stable)

- `PlatformStatus`, `ServiceInfo`, `GovernanceInfo`, `ConfigInfo`
- `DocsIndex`, `DocumentEntry`, `DocsSummary`
- `ErrorSummary`, `LastErrorSummary`, `ErrorStats`
- All nested types (`LedgerCounts`, `FrictionInfo`, etc.)
- `PlatformAPIError` for error handling

### Breaking Changes

Breaking changes to the client's public API require:
1. Version bump
2. Matching OpenAPI/Rust kernel change
3. Migration notes

## Testing the Contract

### Rust Side

```bash
# Full governance validation including API
cargo xtask selftest
```

### TypeScript Side

```bash
cd examples/backstage-plugin
pnpm test  # Includes PlatformClient.test.ts
```

### Manual Validation

```bash
# Start the kernel
cargo run -p app-http &

# Test /platform/status
curl http://localhost:9090/platform/status | jq

# Test /platform/docs/index
curl http://localhost:9090/platform/docs/index | jq
```

## See Also

- `specs/openapi/openapi.yaml` - Formal OpenAPI specification
- `crates/app-http/src/platform.rs` - Rust implementation (platform endpoints)
- `crates/app-http/src/errors.rs` - Error tracking implementation (AC-TPL-ERROR-MAPPING)
- `examples/backstage-plugin/src/api/PlatformClient.ts` - TypeScript client
- `examples/backstage-plugin/src/api/PlatformClient.test.ts` - Client tests
