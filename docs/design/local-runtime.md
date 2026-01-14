---
id: DESIGN-TPL-LOCAL-RUNTIME-001
title: Local Runtime Sovereignty
author: governance-system
doc_type: design_doc
date: 2025-11-22
status: draft
stories: [US-TPL-PLT-001]
requirements: [REQ-TPL-LOCAL-RUNTIME]
tags: [platform, devex]
acs: [AC-TPL-LOCAL-DOCKER]
adrs: [ADR-0002]
---

# Local Runtime Sovereignty

## Problem

Developers need reproducible local runtime dependencies (database, observability) without manually configuring Postgres credentials or Jaeger endpoints. Current state requires reading application config and manually starting Docker containers, increasing onboarding friction.

## Solution

Provide a `docker-compose.yaml` at repository root that defines all runtime dependencies with sensible defaults matching the application's default configuration for local development.

Services included:
- **Postgres**: Database with default credentials matching `config/default.toml`
- **Jaeger**: OpenTelemetry tracing backend with UI on `localhost:16686`
- **App (placeholder)**: Binds the same HTTP port as `config/local.yaml` so port alignment is tested even if you don't run the app in Docker

## Implementation Approach

Create `docker-compose.yaml`:

```yaml
version: '3.8'
services:
  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: myapp_dev
      POSTGRES_USER: dev
      POSTGRES_PASSWORD: dev
    ports:
      - "5432:5432"

  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"  # UI
      - "4317:4317"    # OTLP gRPC
```

Update `README.md` and `docs/dev-environment.md` with:

```bash
docker-compose up -d
cargo run
```

Document the environment surface in `config/envs.yaml` so compose defaults, config schema, and local dev shells stay in sync:

```yaml
envs:
  dev:
    HTTP_PORT: 8080
    DATABASE_URL: postgres://postgres:postgres@localhost:5432/app
    OTEL_EXPORTER_OTLP_ENDPOINT: http://localhost:4317
    PLATFORM_AUTH_MODE: open
    PLATFORM_AUTH_TOKEN: dev-platform-token
```

This file drives the IaC alignment test (`iac_compose_aligns_with_config`) to keep sample IaC and config/defaults from drifting.

**Benefits**: One command to start all dependencies, matches default config, works on any machine with Docker.
