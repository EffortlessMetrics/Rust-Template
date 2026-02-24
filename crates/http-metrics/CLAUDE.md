# http-metrics - CLAUDE.md

**Tier:** HTTP Foundation (Layer 4)
**Publish:** No (internal)
**Dependencies:** axum, once_cell, prometheus, tracing

## Purpose

Provide a single SRP module for HTTP Prometheus instrumentation:

- Request metrics middleware (`http_requests_total`, `http_request_duration_seconds`)
- Metrics endpoint handler (`/metrics`)

## When to Modify

- Adding new HTTP metrics labels or buckets
- Adjusting Prometheus output behavior

## When NOT to Modify

- Business/domain metrics (keep in domain crates)
- Routing composition (keep in `app-http`)

## Consumers

- `app-http`
