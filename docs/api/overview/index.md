---
id: API-OVERVIEW-001
title: Platform API Overview
doc_type: reference
status: published
audience: developers, idp-operators, integration-developers
tags: [api, platform, endpoints]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-PLATFORM-APIS]
acs: [AC-PLT-015]
adrs: [ADR-0005]
last_updated: 2025-12-27
---

# Platform API Overview

The Platform API provides comprehensive endpoints for governance, task management, and developer experience operations. All endpoints are mounted under the `/platform/*` path prefix.

## Base URL

```
http://localhost:8080/platform
```

The default HTTP port is `8080`, configurable via `config/local.yaml`.

## Authentication

Most `/platform/*` endpoints support authentication based on the `platform.auth_mode` configuration:

| Mode | Description | Required Headers |
|-------|-------------|------------------|
| `disabled` | No authentication required (development mode) | None |
| `basic` | Bearer token authentication | `Authorization: Bearer <token>` |

### Example with Authentication

```bash
curl http://localhost:8080/platform/status \
  -H "Authorization: Bearer your-token-here"
```

## API Categories

### Core Platform Endpoints

| Endpoint | Description | Method |
|----------|-------------|---------|
| `/status` | Platform health and governance metrics | GET |
| `/graph` | Full governance graph | GET |
| `/schema` | All platform schemas | GET |
| `/schema/{name}` | Specific schema by name | GET |
| `/openapi` | OpenAPI specification (YAML) | GET |
| `/openapi.yaml` | OpenAPI spec alias | GET |
| `/devex/flows` | DevEx flows and commands | GET |
| `/docs/index` | Documentation inventory | GET |
| `/coverage` | AC coverage summary | GET |
| `/debug/info` | Debug version info | GET |

### Work & Tasks

| Endpoint | Description | Method |
|----------|-------------|---------|
| `/tasks` | List all tasks with filters | GET |
| `/tasks/suggest-next` | Suggested task sequence | GET |
| `/tasks/graph` | Task dependency graph | GET |
| `/tasks/{id}/status` | Update task status | POST |

### Metadata & Issues

| Endpoint | Description | Method |
|----------|-------------|---------|
| `/friction` | All friction entries | GET |
| `/friction/{id}` | Specific friction entry | GET |
| `/questions` | All questions with filters | GET |
| `/questions/{id}` | Specific question | GET |
| `/forks` | All template forks | GET |
| `/forks/{name}` | Specific fork details | GET |
| `/issues` | Unified issues aggregation | GET |

### Agent Support

| Endpoint | Description | Method |
|----------|-------------|---------|
| `/agent/hints` | Prioritized work hints | GET |
| `/idp/snapshot` | IDP snapshot for agents | GET |

### UI Contract

| Endpoint | Description | Method |
|----------|-------------|---------|
| `/ui/contract` | UI contract specification | GET |

## Response Formats

All API endpoints return JSON responses unless otherwise noted (e.g., OpenAPI YAML).

### Common Response Structure

Most responses follow this pattern:

```json
{
  "field": "value",
  "nested": {
    "item": "value"
  }
}
```

### Empty Collections

Empty arrays are represented as `[]`, not `null`.

### Optional Fields

Fields marked as optional may be:
- Completely absent from the JSON response
- Present with `null` value

Consumers should handle both cases.

## Error Handling

### HTTP Status Codes

| Code | Description |
|-------|-------------|
| `200` | Success |
| `204` | Success with no content (e.g., DELETE, status updates) |
| `400` | Bad request (validation error) |
| `401` | Unauthorized (authentication required) |
| `404` | Resource not found |
| `500` | Internal server error |

### Error Response Format

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": "Additional context (optional)"
  }
}
```

## Rate Limiting

Rate limiting is not currently enforced. Future versions may include rate limiting headers.

## CORS

CORS configuration is managed via the HTTP server settings. Refer to `config/local.yaml` for CORS configuration.

## Versioning

The Platform API follows semantic versioning principles:

- **Non-breaking additions** (new optional fields, new endpoints) may be added in minor versions
- **Breaking changes** (removing fields, changing types, changing required status) require major version bump
- The `template_version` field in responses tracks the current template version

## Related Documentation

- [Platform API Reference](../../reference/platform-api-endpoints.md) - Complete endpoint reference
- [Platform API Contract](../../reference/platform_api_contract.md) - Contract specification
- [OpenAPI Specification](../../../specs/openapi/openapi.yaml) - Machine-readable API contract
- [IDP Cell Contract](../../IDP_CELL_CONTRACT.md) - IDP integration datasheet
