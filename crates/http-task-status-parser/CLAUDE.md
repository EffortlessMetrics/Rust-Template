# http-task-status-parser – CLAUDE.md

Focused request parser for task-status update payloads.

## Responsibilities

- Parse task status payloads from JSON/form bodies
- Accept raw bytes plus optional content type
- Return typed request or deterministic parse errors

## Non-responsibilities

- HTTP routing, middleware, auth, or response formatting
- Task transition validation or persistence
