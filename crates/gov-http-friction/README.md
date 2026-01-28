# gov-http-friction

Friction log endpoints for tracking development friction.

## Purpose

This crate provides HTTP endpoints for managing friction log entries. Friction entries capture process, tooling, and DevEx issues discovered during development workflows.

## Endpoints

- `GET /friction` - List all friction entries
- `GET /friction/{id}` - Get a specific friction entry by ID

## Dependencies

- `axum` - Web framework
- `http` - HTTP types
- `http-errors` - Error types and mapping (with axum feature)
- `platform-contract` - Platform contract types
- `gov-model` - Governance domain model
- `gov-http-core` - Shared router foundation
- `serde_yaml` - YAML parsing

## Usage

```rust
use gov_http_friction::router;
use axum::Router;
use gov_http_core::PlatformState;

// Compose friction router
let app = Router::new()
    .merge(router::<MyState>())
    .with_state(my_state);
```

## Data Types

### FrictionEntry
Full friction entry with all metadata:
- `id`: Unique identifier (e.g., "FRICTION-001")
- `date`: Discovery date (ISO 8601)
- `category`: Category (e.g., "tooling", "process", "devex")
- `severity`: Severity level (e.g., "low", "medium", "high", "critical")
- `summary`: One-line summary
- `description`: Detailed description
- `expected_behavior`: Optional expected behavior
- `workaround`: Optional workaround
- `impact`: Optional impact description
- `context`: Optional discovery context
- `status`: Status (defaults to "open")
- `resolution`: Optional resolution information
- `refs`: Related references
- `related_items`: Related issues, ADRs, tasks

### FrictionContext
Discovery context for friction entry:
- `discovered_by`: Who discovered the issue
- `flow`: Related development flow
- `phase`: Development phase
- `files_involved`: Files involved
- `commands_involved`: Commands involved

### Resolution
Resolution information:
- `resolved_by`: Who resolved it
- `resolved_at`: Resolution date
- `fix_description`: Optional fix description
- `pr_links`: Related PR links
- `verification`: Optional verification notes

## Friction File Format

Friction entries are stored as YAML files in `friction/` directory:

```yaml
id: FRICTION-EXAMPLE-001
date: "2025-11-26"
category: tooling
severity: medium
summary: "Build takes too long"
description: "The build process takes over 10 minutes"
expected_behavior: "Build should complete in under 2 minutes"
workaround: "Use incremental builds"
impact: "Slows down development iteration"
context:
  discovered_by: "dev-team"
  flow: "bundle"
  phase: "build"
  files_involved:
    - Cargo.toml
    - build.rs
status: open
refs:
  - REQ-001
```
