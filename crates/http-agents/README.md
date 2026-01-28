# http-agents

HTTP handlers for `/agents/*` endpoints.

## Purpose

This crate implements agent hints API including:

- **List agent hints** (`GET /platform/agent/hints`) - Get task and governance hints
- **Filter hints** - Filter by owner, label, requirement, or kind (task/governance)

## Design Philosophy

- **Agent-focused**: Only agent-related handlers
- **Async-safe**: Uses `spawn_blocking` for blocking I/O operations
- **Hint-based**: Uses spec-runtime hint engine for intelligent task suggestions
- **Referential integrity**: Validates AC and REQ references

## Dependencies

- `axum` - HTTP web framework
- `http` - HTTP types
- `http-errors` - Error types with axum feature
- `platform-contract` - Platform contract types
- `http-core` - Core HTTP types
- `spec-runtime` - Spec loading and hint engine
- `adapters-spec-fs` - Task definitions loading
- `gov-model` - Governance model types
- `business-core` - Task operations via TaskService
- `serde`, `serde_json`, `serde_yaml` - Serialization
- `tracing` - Structured logging
- `tokio` - Async runtime

## Usage

```rust
use http_agents::{router, AgentsState, HintsFilters};

let app = Router::new()
    .merge(router(state));
```

## Public API

### Traits

- `AgentsState` - Agents state trait for handlers

### Types

- `AgentHint` - Agent hint with full schema and convenience fields
- `AgentHintsResponse` - Response with hints and warnings
- `RecommendedStep` - Recommended command sequence step
- `AgentHintReason` - Hint reason (wire format)
- `HintsFilters` - Query filters for hints

### Functions

- `router()` - Create the agents router
- `agent_hints()` - Get agent hints endpoint handler

## License

Internal crate (publish = false)
