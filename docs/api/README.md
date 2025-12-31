---
id: API-TPL-INDEX-001
title: API Documentation Index
doc_type: reference
status: published
audience: developers, idp-operators, integration-developers
tags: [api, openapi, platform, endpoints]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-PLATFORM-APIS, REQ-TPL-IDP-SNAPSHOT]
acs: [AC-PLT-015, AC-TPL-PLATFORM-SCHEMA]
adrs: [ADR-0005]
last_updated: 2025-12-27
---

# API Documentation

API documentation is generated from specifications, not maintained as separate prose.

## Canonical Sources

| Source              | Location                              | Description                          |
| ------------------- | ------------------------------------- | ------------------------------------ |
| **OpenAPI Spec**    | `specs/openapi/openapi.yaml`          | Machine-readable API contract        |
| **Runtime Schema**  | `/platform/schema`                    | JSON Schema index and endpoint list  |
| **Runtime OpenAPI** | `/platform/openapi`                   | OpenAPI spec from the running service |
| **Platform Ref**    | `docs/reference/platform_api_contract.md` | Human-readable reference         |

## Quick Access

Start the service and query endpoints:

```bash
cargo run -p app-http &
curl http://localhost:8080/platform/openapi  # OpenAPI spec
curl http://localhost:8080/platform/schema   # Schema index
curl http://localhost:8080/platform/status   # Governance health
```

## Platform Endpoints

| Endpoint              | Description                           |
| --------------------- | ------------------------------------- |
| `/platform/status`    | Governance health and AC coverage     |
| `/platform/graph`     | REQ/AC/test/doc relationships         |
| `/platform/docs/index`| Documentation inventory               |
| `/platform/schema`    | Schema index (JSON Schema + endpoints) |
| `/platform/openapi`   | OpenAPI specification                 |
| `/platform/tasks`     | Task list with filters                |
| `/platform/agent/hints` | Prioritized task suggestions        |
| `/platform/friction`  | Friction log entries                  |
| `/platform/questions` | Question artifacts                    |
| `/platform/forks`     | Fork registry                         |

## Related Documentation

- [`docs/reference/platform_api_contract.md`](../reference/platform_api_contract.md) - Full API reference
- [`docs/IDP_CELL_CONTRACT.md`](../IDP_CELL_CONTRACT.md) - IDP integration datasheet
- [`specs/openapi/openapi.yaml`](../../specs/openapi/openapi.yaml) - OpenAPI spec
