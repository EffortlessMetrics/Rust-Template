# http-agents – CLAUDE.md

**Tier:** Application HTTP (Layer 6)
**Publish:** No (internal)
**Dependencies:** http-core, spec-runtime, axum

## Purpose

Agent hint endpoints for task recommendations. Provides AI agents with prioritized work suggestions.

## Key Endpoints

- `GET /agents/hints` – Get prioritized hints for agents
- `GET /agents/context` – Get agent context

## When to Modify

- Adding new agent-facing endpoints
- Extending hint algorithms

## Architectural Notes

- **Agent-facing**: Designed for AI agent consumption
- **Prioritization**: Uses HintEngine from spec-runtime

## Consumers

`app-http`, AI agents

## See Also

- `crates/spec-runtime/` for HintEngine
- `crates/http-platform/` for `/platform/agent/hints`
