---
id: REF-CONFIG-SCHEMA-001
title: Configuration Schema Reference
doc_type: reference
status: published
audience: developers, maintainers
tags: [reference, schema, configuration]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-DEVEX-CONTRACT]
acs: []
adrs: [ADR-0003]
last_updated: 2026-01-30
---

# Reference: Configuration Schema

This document defines the YAML schema for `specs/config_schema.yaml`, the configuration schema that validates runtime configuration files.

**Related:**
- `crates/spec-runtime/README.md` - Runtime library for config validation
- `docs/reference/spec-ledger-schema.md` - Spec ledger schema
- `config/local.yaml` - Example configuration file

---

## Schema Overview

The configuration schema defines three categories of configuration:

```
schema_version
envs[]        # Environment definitions
settings[]    # Application settings
secrets[]     # Sensitive credentials
```

---

## Root Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `schema_version` | integer | Yes | Schema version number (e.g., `1`) |
| `envs` | `Environment[]` | No | Environment definitions |
| `settings` | `Setting[]` | No | Application settings |
| `secrets` | `Secret[]` | No | Sensitive configuration |

**Example:**

```yaml
schema_version: 1

envs:
  - name: dev
    required: false
  - name: prod
    required: true

settings:
  - key: http.port
    type: int
    default: 8080

secrets:
  - key: db.url
    type: string
    required: true
```

---

## Environment

Defines deployment environments and their requirements.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Environment name (e.g., `dev`, `staging`, `prod`) |
| `required` | boolean | No | Whether this environment must be configured (default: `false`) |

**Common environments:**
- `dev` - Local development
- `staging` - Pre-production testing
- `prod` - Production deployment

**Example:**

```yaml
envs:
  - name: dev
    required: false
  - name: staging
    required: true
  - name: prod
    required: true
```

---

## Setting

Defines non-sensitive application configuration.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `key` | string | Yes | Dot-separated configuration key |
| `type` | string | Yes | Value type (see Types section) |
| `description` | string | No | Human-readable description |
| `default` | any | No | Default value if not specified |
| `required` | boolean | No | Whether the setting must be provided (default: `false`) |

**Key naming convention:** Use dot notation to indicate hierarchy (e.g., `http.port`, `telemetry.otlp_endpoint`, `platform.auth_mode`).

**Example:**

```yaml
settings:
  - key: http.port
    type: int
    default: 8080
    description: "HTTP listen port"

  - key: telemetry.otlp_endpoint
    type: string
    description: "OTLP collector URL"
    required: false

  - key: platform.auth_mode
    type: string
    description: "Authentication mode for platform endpoints (open|basic|jwt)"
    default: "open"

  - key: platform.redact_secrets
    type: bool
    description: "Whether to redact secrets from status/UI output"
    default: true

  - key: database.auto_migrate
    type: bool
    description: "Whether to automatically run database migrations on startup"
    default: false
```

---

## Secret

Defines sensitive credentials and keys.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `key` | string | Yes | Dot-separated configuration key |
| `type` | string | Yes | Value type (typically `string`) |
| `description` | string | No | Human-readable description |
| `required` | boolean | No | Whether the secret must be provided (default: `false`) |

**Note:** Secrets never have default values. They must be provided via environment variables or secure configuration sources.

**Example:**

```yaml
secrets:
  - key: db.url
    type: string
    description: "Postgres connection string"
    required: true

  - key: auth.jwt_signing_key
    type: string
    required: true

  - key: platform.auth_token
    type: string
    description: "Shared secret for platform basic auth mode"
    required: false

  - key: platform.jwt_secret
    type: string
    description: "JWT secret key for platform JWT auth mode"
    required: false
```

---

## Types

Supported value types for settings and secrets:

| Type | Description | Example Values |
|------|-------------|----------------|
| `string` | Text value | `"http://localhost:4317"` |
| `int` | Integer number | `8080`, `3600` |
| `bool` | Boolean flag | `true`, `false` |
| `float` | Floating point number | `0.5`, `3.14` |

---

## Configuration Files

Configuration files (`config/*.yaml`) are validated against this schema.

### File Structure

```yaml
# config/local.yaml
env: dev

http:
  port: 8080

telemetry:
  otlp_endpoint: "http://localhost:4317"

platform:
  auth_mode: "open"
  redact_secrets: true

database:
  auto_migrate: true
```

### Environment Variable Override

Settings and secrets can be overridden via environment variables:

| Schema Key | Environment Variable |
|------------|---------------------|
| `http.port` | `HTTP_PORT` |
| `db.url` | `DB_URL` |
| `platform.auth_mode` | `PLATFORM_AUTH_MODE` |

**Convention:** Replace dots with underscores and convert to uppercase.

---

## Validation

Configuration is validated at startup using `spec-runtime`:

```rust
use spec_runtime::{validate_config, ValidatedConfig};

let schema_path = Path::new("specs/config_schema.yaml");
let config_path = Path::new("config/local.yaml");

let config: ValidatedConfig = validate_config(schema_path, config_path)?;

println!("Environment: {:?}", config.env);
println!("HTTP Port: {}", config.http_port);
```

### Validation Rules

1. **Type checking** - Values must match declared types
2. **Required fields** - Required settings/secrets must be present
3. **Environment validation** - Config env must be in declared envs list
4. **No unknown keys** - Undeclared keys are warnings (or errors in strict mode)

---

## Platform Authentication Modes

The `platform.auth_mode` setting controls access to `/platform/*` endpoints:

| Mode | Description | Required Secrets |
|------|-------------|-----------------|
| `open` | No authentication (development) | None |
| `basic` | HTTP Basic Auth | `platform.auth_token` |
| `jwt` | JWT Bearer tokens | `platform.jwt_secret` |

**Example configuration for basic auth:**

```yaml
# config/staging.yaml
env: staging

platform:
  auth_mode: "basic"

# Secrets via environment
# PLATFORM_AUTH_TOKEN=your-secret-token
```

---

## Schema Evolution

The config schema is versioned via `schema_version`. When adding new settings:

1. Add to `specs/config_schema.yaml` with appropriate defaults
2. Update this documentation
3. Ensure backward compatibility (new settings should have defaults or be optional)

Breaking changes (removing required fields, changing types) require:
- Major schema version bump
- Migration guidance in release notes
- ADR documenting the change

---

## See Also

- `crates/spec-runtime/README.md` - Config validation API
- `docs/reference/spec-ledger-schema.md` - Spec ledger schema
- `docs/reference/environment.md` - Environment setup
- `config/local.yaml` - Example configuration
