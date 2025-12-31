---
id: DESIGN-TPL-RUNTIME-CONTRACT-001
title: Platform Runtime Contract
author: platform-team
doc_type: design_doc
date: 2025-11-25
status: draft
stories: [US-TPL-PLT-001]
requirements:
  - REQ-TPL-PLATFORM-SCHEMA
  - REQ-TPL-METADATA-CONSISTENT
  - REQ-TPL-PLATFORM-AUTH
  - REQ-TPL-LOG-HYGIENE
  - REQ-TPL-QUESTIONS-AS-ARTIFACTS
  - REQ-TPL-FLOW-IDEMPOTENCY
  - REQ-TPL-CONFIG-INTEGRITY
  - REQ-TPL-IAC-ALIGNMENT
tags: [platform, structural, security]
acs:
  - AC-TPL-PLATFORM-SCHEMA
  - AC-TPL-METADATA-COMPLETE
  - AC-TPL-PLATFORM-AUTH-BASIC
  - AC-TPL-LOG-NO-SECRETS
  - AC-TPL-QUESTIONS-LOGGED
  - AC-TPL-FLOW-IDEMPOTENT
  - AC-TPL-CONFIG-VALIDATION
  - AC-TPL-IAC-COMPOSE-ALIGN
adrs: [ADR-0001, ADR-0005]
---

# Platform Runtime Contract

## Problem

The platform endpoints, config surface, and devex flows already exist, but without a single contract that explains how schema exposure, metadata, auth, logging hygiene, and IaC alignment fit together. That gap made it easy for Windows-native development or Docker packaging to drift from the governance guarantees enforced in Tier-1.

## Decisions

1) **Machine-readable schema**
- `/platform/schema` returns the schema index (JSON Schema + endpoint list) emitted from the same spec-runtime graph used by xtask. `/platform/openapi` serves the OpenAPI document for codegen and validation.
- The schema includes template version and build SHA for change tracking and is bundled in the Docker image so dashboards/agents can fetch it without local specs.

2) **Metadata coherence**
- `config/service_metadata.yaml` is the single source for service_id, template_version, runbook/roadmap URLs, and tags. `/platform/status` and the UI render the same identifiers; tests assert the fields remain in sync.
- When metadata is missing or malformed, `/platform/status` responds with redacted placeholders and logs a warning without leaking secrets.

3) **Platform auth modes**
- `PLATFORM_AUTH_MODE` supports `open`, `basic`, and `basic-without-token` (warn-only). All POST/PUT/PATCH/DELETE under `/platform/*` are guarded; GET/HEAD/OPTIONS stay open by default.
- `PLATFORM_AUTH_TOKEN` is the only credential; middleware redacts it in logs and returns structured denial responses for invalid or missing tokens.

4) **Log hygiene**
- Sensitive config keys are redacted before status serialization and tracing output. Only key names appear; values are replaced with `***`.
- HTTP trace spans include request_id and method/URI only; auth headers and config payloads are excluded.

5) **Questions as artifacts**
- Ambiguous flows (e.g., missing AC context or unclear task) emit a structured question artifact recorded alongside other run outputs instead of blocking. On Windows this writes to `target/questions/*.json` to avoid file locks in `target/debug`.
- `cargo xtask selftest` and `suggest-next` surface pending questions in their summaries so agents/users can resolve them.

6) **Flow idempotency**
- `cargo xtask selftest` and `suggest-next` avoid generating duplicate artifacts by writing deterministic filenames and overwriting prior runs. Logs note when an existing artifact is reused.
- `test-changed` uses git diff to avoid rerunning unrelated suites; reruns with the same diff produce identical plans.

7) **Config integrity**
- `config/local.yaml` is validated against `specs/config_schema.yaml` at service startup and during `cargo xtask docs-check`. Invalid configs cause a non-zero exit with a human-readable path to the failing field.
- Docker/compose samples mount the validated config and fail fast if the schema check fails, keeping local runs honest.

8) **IaC alignment**
- `docker-compose.yaml` uses ports and environment keys defined in `envs.yaml` and `config_schema.yaml`; tests (`iac_compose_aligns_with_config`) guard drift.
- Future Kubernetes/Terraform samples must reuse the same keys; adding new required config fields requires updating IaC examples and re-running the alignment test.

## Implementation Notes

- `spec_runtime::validate_config` enforces schema conformance at startup and is used by both `app-http` and xtask commands.
- The platform router applies `middleware::platform_auth_guard` once at the `/platform` mount, so new write endpoints inherit auth automatically.
- Log redaction is performed in `platform::status_response` before serialization and traced fields are limited to non-sensitive data.
- `target/questions` and `target/xtask` are the preferred write locations on Windows to minimize file-lock conflicts with cargo.

## Verification

- **Schema:** BDD `@AC-TPL-PLATFORM-SCHEMA` plus unit snapshot of the OpenAPI document.
- **Metadata:** BDD `@AC-TPL-METADATA-COMPLETE`; unit `platform::tests::log_hygiene_redacts_secrets`.
- **Auth:** BDD `@AC-TPL-PLATFORM-AUTH-BASIC`; unit coverage in `middleware::platform_auth`.
- **Config/IaC:** Unit `config_validation_rejects_invalid` and `iac_compose_aligns_with_config`; BDD `@AC-TPL-CONFIG-VALIDATION`.
- **Idempotency/Questions:** Unit coverage in xtask planners; manual check that reruns reuse artifacts and surface questions in summaries.
