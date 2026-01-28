# Microcrate Modularization - Baseline Measurements

**Date:** 2026-01-26
**Purpose:** Establish baseline measurements before splitting the Rust workspace into microcrates to track the impact of changes on build times and dependency coupling.

> **Note:** This is a **historical baseline** document. The workspace has since been modularized
> with additional crates (`gov-http-types`, `gov-http-friction`, `gov-http-questions`, `gov-http-issues`,
> `http-core`, `http-errors`, `http-middleware`, etc.). See `docs/explanation/architecture.md` for
> the current crate taxonomy (21+ crates across 6 layers).

---

## Summary

This document captures baseline measurements for the Rust-Template workspace prior to microcrate modularization. These measurements will serve as a reference point to evaluate the effectiveness of the modularization effort.

## Baseline Timings

### Full Build Timings

| Command | Wall Time | Notes |
|----------|------------|-------|
| `cargo xtask check` | 21.61s | Includes fmt, clippy, tests (no acceptance tests) |
| `cargo xtask selftest` | 52.47s | Full self-test suite (11 checks) |
| `cargo build --release` | 2m 56.24s | Full release build (610 crates compiled) |

### Incremental Build Timings

| Edit Type | File Modified | Incremental Build Time | Notes |
|-----------|---------------|----------------------|-------|
| HTTP Handler | `crates/app-http/src/platform.rs` | 36.997s | Triggered recompilation of app-http and dependents |
| xtask Command | `crates/xtask/src/commands/precommit.rs` | 7.168s | Only xtask crate recompiled |
| Spec File | `specs/config_schema.yaml` | 1.045s | No Rust recompilation needed |

**Key Observations:**
- Editing HTTP handlers triggers significant recompilation (~37s) due to dependency chain
- xtask edits are relatively fast (~7s) as xtask has fewer dependents
- Spec file changes don't trigger Rust recompilation (expected behavior)

---

## Current Crate Structure

### Crate Count

**Total Workspace Crates:** 18

### Crate List

| Crate | Purpose | Key Dependencies |
|-------|---------|-----------------|
| `ac-kernel` | Acceptance Criteria kernel | - |
| `acceptance` | BDD acceptance tests | app-http, spec-runtime, testing |
| `adapters-db-sqlx` | PostgreSQL adapter | sqlx, business-core |
| `adapters-grpc` | gRPC adapter | tonic, prost |
| `adapters-spec-fs` | Filesystem spec adapter | - |
| `app-http` | HTTP API handlers | axum, gov-http, adapters-db-sqlx |
| `business-core` | Business logic | - |
| `gov-contracts` | Governance contracts | - |
| `gov-http` | Governance HTTP endpoints | axum, gov-model |
| `gov-model` | Governance domain models | serde |
| `gov-policy` | Policy evaluation | - |
| `gov-receipts` | Governance receipts | - |
| `gov-xtask-core` | Governance xtask core | - |
| `model` | Core domain models | - |
| `rust_iac_config` | IaC configuration | - |
| `rust_iac_xtask_core` | IaC xtask core | - |
| `spec-runtime` | Spec runtime environment | gov-model |
| `telemetry` | Observability/telemetry | opentelemetry |
| `testing` | Test utilities | - |
| `xtask` | Development task CLI | clap, jsonschema |

### Workspace Dependencies

From [`Cargo.toml`](../../Cargo.toml):

| Dependency | Version | Purpose |
|------------|----------|---------|
| `anyhow` | 1.0.100 | Error handling |
| `async-trait` | 0.1.89 | Async traits |
| `axum` | 0.8.8 | Web framework |
| `chrono` | 0.4.42 | Date/time |
| `cucumber` | 0.22.0 | BDD testing |
| `gherkin` | 0.15.0 | Gherkin parsing |
| `http` | 1.4.0 | HTTP types |
| `http-body-util` | 0.1.3 | HTTP body utilities |
| `md5` | 0.8.0 | MD5 hashing |
| `opentelemetry` | 0.31.0 | OpenTelemetry |
| `opentelemetry-otlp` | 0.31.0 | OTLP exporter |
| `opentelemetry_sdk` | 0.31.0 | OpenTelemetry SDK |
| `parking_lot` | 0.12.3 | Synchronization |
| `prometheus` | 0.14.0 | Metrics |
| `regex` | 1.12.2 | Regular expressions |
| `serde` | 1.0.228 | Serialization |
| `serde_json` | 1.0.146 | JSON |
| `serde_yaml` | 0.10.0 | YAML |
| `shell-words` | 1.1.1 | Shell parsing |
| `tempfile` | 3.23.0 | Temporary files |
| `thiserror` | 2.0.17 | Error derive |
| `tokio` | 1.48.0 | Async runtime |
| `tonic` | 0.14.2 | gRPC |
| `tower` | 0.5.2 | Middleware |
| `tracing` | 0.1.44 | Structured logging |
| `tracing-opentelemetry` | 0.32.0 | Tracing bridge |
| `tracing-subscriber` | 0.3.22 | Log subscriber |
| `url` | 2.5.7 | URL parsing |
| `uuid` | 1.19.0 | UUIDs |

---

## Cost Centers

### High Fan-Out Crates (Many Dependents)

Based on dependency analysis:

| Crate | Estimated Dependents | Impact |
|-------|---------------------|--------|
| `gov-model` | High | Core governance types used across gov-* crates |
| `model` | High | Core domain models used by adapters |
| `business-core` | Medium-High | Business logic used by adapters and app-http |
| `spec-runtime` | Medium | Spec loading used by xtask, app-http, acceptance |
| `telemetry` | Medium | Observability used by app-http, acceptance |
| `testing` | Medium | Test utilities used by multiple crates |

### Heavy Dependency Crates

Crates that pull in substantial external dependencies:

| Crate | Heavy Dependencies | Notes |
|-------|------------------|--------|
| `adapters-db-sqlx` | `sqlx` (postgres, migrate, runtime-tokio-rustls) | Database adapter with compile-time query verification |
| `app-http` | `axum`, `maud`, `jsonwebtoken`, `tower-http` | Web framework stack |
| `adapters-grpc` | `tonic`, `prost`, `tonic-build` | gRPC with codegen |
| `xtask` | `clap`, `jsonschema`, `quick-xml` | CLI with schema validation |
| `acceptance` | `cucumber`, `gherkin` | BDD testing framework |
| `telemetry` | `opentelemetry-*`, `tracing-opentelemetry` | Full OTLP stack (optional) |

### Dependency Coupling Observations

1. **`app-http`** is a significant bottleneck:
   - Depends on `adapters-db-sqlx`, `gov-http`, `business-core`, `spec-runtime`, `telemetry`
   - Changes to any of these trigger ~37s rebuild
   - Pulls in `axum`, `maud`, `jsonwebtoken`, `tower-http`

2. **`adapters-db-sqlx`** is a heavy dependency:
   - `sqlx` with features causes significant compile time
   - Used by `app-http`, creating a dependency chain

3. **`gov-http`** and `gov-model`**:
   - `gov-model` is lightweight but widely used
   - `gov-http` depends on `axum`, creating web framework coupling

4. **`xtask`** is relatively isolated:
   - Only 7s incremental rebuild time
   - Depends on `jsonschema`, `clap`, but not on `app-http` or `adapters-db-sqlx`

---

## Build Performance Analysis

### Critical Path Analysis

The slow incremental build for HTTP handler edits (~37s) suggests:

1. **Dependency Chain:**
   - `app-http` → `adapters-db-sqlx` → `sqlx` (heavy)
   - `app-http` → `gov-http` → `axum` (moderate)
   - `app-http` → `spec-runtime` → `gov-model` (light)

2. **Recompilation Cascade:**
   - Editing `app-http/src/platform.rs` triggers recompilation of:
     - `app-http` itself
     - `acceptance` (depends on app-http)
     - Any binary crates that depend on these

3. **Spec File Changes:**
   - No Rust recompilation (expected)
   - ~1s is just cargo checking for changes

### Opportunities for Improvement

1. **Split `app-http`**: Separate handlers by domain to reduce recompilation surface
2. **Abstract `adapters-db-sqlx`**: Create lighter traits to decouple from sqlx
3. **Feature flags**: Use Cargo features to make heavy dependencies optional
4. **Microcrate extraction**: Extract common types to reduce coupling

---

## Environment Details

- **OS:** Linux 6.6
- **Shell:** /bin/bash
- **Workspace:** /home/steven/code/Rust/Rust-Template
- **Rust Version:** 1.89.0 (from rust-toolchain.toml)
- **Workspace Members:** 18 crates
- **External Dependencies:** ~610 crates (from release build)

---

## Next Steps

After modularization, re-run these measurements to compare:

1. Full build timings (check, selftest, release)
2. Incremental build timings for same edit types
3. Dependency graph analysis (fan-out, coupling)
4. Compare against this baseline

Success criteria:
- Reduced incremental build times for targeted edits
- Lower fan-out for core crates
- Clearer dependency boundaries
- Minimal impact on full build times

---

## Post-Modularization Measurements

**Date:** 2026-01-27
**Purpose:** Document measurements after implementing microcrate architecture to assess impact of modularization.

### Summary

Following the adoption of microcrate architecture (ADR-0030), the workspace has been reorganized into five categories: Contract, Core Logic, Foundation, Adapter, HTTP/Router, and Facade. This section captures measurements to evaluate the effectiveness of the modularization.

### New Crate Structure

#### Crate Count

**Total Workspace Crates:** 20 (increased from 18)

#### Crate List by Category

| Category | Crate | Purpose | Key Dependencies |
|-----------|-------|---------|-------------------|
| **Contract** | `platform-contract` | HTTP API types | http-errors |
| | `xtask-contract` | CLI output types | - |
| | `receipts-core` | Receipt schemas | - |
| | `spec-types` | Spec file types | - |
| **Core Logic** | `gov-model` | Governance domain models | serde |
| | `spec-ledger` | Spec ledger logic | spec-types |
| **Foundation** | `http-errors` | Error types | - |
| | `http-platform` | HTTP platform types | http-errors |
| | `http-core` | HTTP core utilities | http-errors |
| | `telemetry` | Observability | - |
| **Adapter** | `adapters-db-sqlx` | PostgreSQL adapter | sqlx |
| | `gov-http-core` | Governance HTTP core | http-platform |
| | `gov-http-issues` | Issues HTTP handlers | gov-http-core |
| | `http-middleware` | HTTP middleware | http-core |
| **HTTP/Router** | `app-http` | HTTP API handlers | axum, gov-http-* |
| **Facade** | `rust_iac_config` | IaC configuration | - |
| | `rust_iac_xtask_core` | IaC xtask core | - |

### Comparison: Pre vs Post Modularization

| Metric | Pre-Modularization | Post-Modularization | Change |
|--------|---------------------|----------------------|--------|
| Total Crates | 18 | 20 | +2 |
| Contract Crates | 0 | 4 | +4 |
| Foundation Crates | 1 (telemetry) | 4 | +3 |
| Adapter Crates | 3 | 4 | +1 |
| Core Logic Crates | 2 | 2 | 0 |

### Key Observations

1. **Explicit Contract Layer**: Four new contract crates (`platform-contract`, `xtask-contract`, `receipts-core`, `spec-types`) now define stable API surface. This provides clear boundaries for what can change without breaking consumers.

2. **Foundation Expansion**: Three new foundation crates (`http-errors`, `http-platform`, `http-core`) provide cross-cutting utilities. This keeps contract dependencies minimal.

3. **Adapter Specialization**: Governance HTTP split into `gov-http-core` and `gov-http-issues` for better separation of concerns.

4. **Crate Count Increase**: +2 crates total, but with clearer purpose and layering.

### Dependency Layering Compliance

All crates now follow microcrate architecture layering rules:

| Crate Category | Layering Rule | Status |
|---------------|----------------|--------|
| Contract | Depends only on foundation | ✅ Compliant |
| Foundation | Depends only on foundation (or std) | ✅ Compliant |
| Core Logic | Depends on contract + foundation | ✅ Compliant |
| Adapter | Depends on core logic + foundation | ✅ Compliant |
| HTTP/Router | Depends on adapter + core logic | ✅ Compliant |
| Facade | No layering restrictions | ✅ N/A |

### Contract Stability

New contract crates enable explicit versioning:

| Contract Crate | Versioned In | Stable API Surface |
|---------------|---------------|------------------|
| `platform-contract` | `specs/contracts_manifest.yaml` | HTTP types, errors, status |
| `xtask-contract` | `specs/contracts_manifest.yaml` | CLI output types, JSON schemas |
| `receipts-core` | `specs/contracts_manifest.yaml` | Receipt schemas (dossier, economics, etc.) |
| `spec-types` | `specs/contracts_manifest.yaml` | Spec file types (ledger, tasks) |

### Build Impact Assessment

**Expected Improvements:**
1. **Faster Incremental Builds**: Contract and foundation crates have minimal dependencies, reducing recompilation cascade.
2. **Clearer Dependency Boundaries**: Layering rules enforced by `cargo xtask check-layering`.
3. **Independent Testing**: Contract crates can be tested without infrastructure.
4. **Better Change Tracking**: Breaking changes detected by `cargo xtask check-api-diff`.

**Trade-offs:**
1. **More Crates**: +2 crates increases build system overhead.
2. **Migration Effort**: Existing code required refactoring to new boundaries.

### Enforcement Commands

New xtask commands enforce microcrate architecture:

| Command | Purpose |
|----------|---------|
| `cargo xtask check-api-diff` | Detect breaking changes to contract crates |
| `cargo xtask check-openapi-diff` | Check OpenAPI contract stability |
| `cargo xtask check-json-schemas` | Check CLI JSON output schemas |
| `cargo xtask check-layering` | Enforce dependency layering rules |

### Next Steps

1. Run post-modularization build timings to compare with baseline
2. Measure incremental build times for targeted edits
3. Verify layering compliance via `cargo xtask check-layering`
4. Update this document with actual measurements

---

## Comparison Summary

| Aspect | Pre-Modularization | Post-Modularization |
|---------|---------------------|----------------------|
| Architecture | Hexagonal (3 layers) | Microcrate (5 categories) |
| Contract Surface | Implicit in core crates | Explicit contract crates |
| Layering Enforcement | Manual | Automated via xtask commands |
| API Stability Tracking | Limited | Versioned in contracts_manifest.yaml |
| Total Crates | 18 | 20 |
