# http-platform – CLAUDE.md

**Tier:** Application HTTP (Layer 6)
**Publish:** No (internal)
**Dependencies:** http-core, platform-contract, spec-runtime, axum

## Purpose

Platform status, governance, and spec endpoints. Provides `/platform/status`, `/platform/graph`, `/platform/agent/hints` and related endpoints (HTML and JSON).

## Key Endpoints

- `GET /platform/status` – Platform health and metrics
- `GET /platform/graph` – Governance graph
- `GET /platform/agent/hints` – Prioritized work for agents
- `GET /platform/coverage` – AC coverage summary

## When to Modify

- Adding new platform introspection endpoints
- Extending status response data

## When NOT to Modify

- Adding governance domain endpoints (put in gov-http-*)
- Adding task/todo endpoints (put in http-tasks/http-todos)

## Architectural Notes

- **Read-only introspection**: Mostly GET endpoints
- **HTML + JSON**: Supports both response formats
- **Agent-facing**: Provides hints for AI agents

## Consumers

`app-http`

## See Also

- `crates/platform-contract/` for response types
- `crates/spec-runtime/` for data loading
