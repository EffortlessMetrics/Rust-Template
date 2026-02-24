# http-task-status-parser

Single-responsibility parser for `POST /tasks/{id}/status` request payloads.

## Scope

- Parse `UpdateTaskStatusRequest` from `application/json`
- Parse `UpdateTaskStatusRequest` from `application/x-www-form-urlencoded`
- Provide deterministic parse errors without framework coupling

## Why this crate exists

`http-tasks` previously mixed request parsing with endpoint orchestration.
This crate isolates payload parsing so it can be reused, fuzzed, and property-tested
independently from HTTP routing and repository I/O.
