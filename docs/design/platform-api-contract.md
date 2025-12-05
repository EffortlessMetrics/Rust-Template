---
id: DESIGN-TPL-PLATFORM-API-CONTRACT-001
title: Platform API Contract
author: platform-team
doc_type: design_doc
date: 2025-12-04
status: published
stories: [US-TPL-PLT-001]
requirements:
  - REQ-TPL-PLATFORM-CONTRACT
tags: [platform, api, idp, contract]
acs:
  - AC-TPL-PLATFORM-STATUS-CONTRACT
  - AC-TPL-DOCS-INDEX-CONTRACT
adrs: [ADR-0001, ADR-0005]
---
<!-- doclint:disable orphan-version -->

# Platform API Contract

## Problem

The platform endpoints (`/platform/status`, `/platform/docs/index`) are consumed by both human developers and automated systems (IDP dashboards, Backstage plugins, LLM agents). Without a stable, documented contract, consumers cannot rely on response shapes remaining consistent across template versions.

## Decision

We treat `/platform/status` and `/platform/docs/index` as **kernel-level contracts**:

1. **Schema-first**: Response shapes are defined in `specs/openapi/openapi.yaml` before implementation
2. **Type-safe chain**: Rust structs derive from the schema; TypeScript clients consume the same OpenAPI
3. **Versioned stability**: Breaking changes require major version bumps and migration guides

## Contract Chain

```
spec_ledger.yaml → spec-runtime → /platform/* → openapi.yaml → TypeScript client
```

The chain ensures:

- **Rust structs** in `crates/platform/src/` serialize to JSON matching the OpenAPI schema
- **OpenAPI document** at `/platform/schema` is generated from the same runtime graph
- **TypeScript consumers** (like the Backstage plugin) import types generated from OpenAPI

## Covered Endpoints

| Endpoint | Schema | Purpose |
|----------|--------|---------|
| `/platform/status` | `PlatformStatus` | Governance health, version, auth mode |
| `/platform/docs/index` | `DocsIndex` | Documentation inventory with front-matter |
| `/platform/graph` | `GovernanceGraph` | Full REQ → AC → test → doc linkage |
| `/platform/schema` | OpenAPI 3.0.3 | Machine-readable API specification |

## Verification

- **Unit tests**: Snapshot tests ensure response shapes match schemas
- **BDD scenarios**: `@AC-TPL-PLATFORM-STATUS-CONTRACT` and `@AC-TPL-DOCS-INDEX-CONTRACT`
- **OpenAPI lint**: Redocly validates the schema structure
- **TypeScript build**: Backstage plugin type-checks against generated types

## Implementation Notes

- `crates/platform/src/handlers.rs` implements the endpoints
- `crates/spec-runtime/src/openapi.rs` generates the OpenAPI document
- `examples/backstage-plugin/` consumes the contract via `PlatformClient`
- `scripts/validate-ts-config.sh` ensures TypeScript config is modern and enforceable

## Related Documents

- `docs/reference/platform_api_contract.md` - Reference documentation for consumers
- `specs/openapi/openapi.yaml` - The OpenAPI specification
- `docs/how-to/integrate-idp-or-agent.md` - Integration guide
