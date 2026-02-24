---
id: REF-IGNORED-TESTS-001
title: Ignored Tests Reference
doc_type: reference
status: published
audience: developers, maintainers
tags: [testing, reference, infrastructure]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-DEVEX-CONTRACT]
acs: []
adrs: []
last_updated: 2026-01-28
---

# Ignored Tests Reference

This document catalogs all tests marked with `#[ignore]` in the codebase, explaining why they're ignored and how to run them.

## Summary

| Crate | Test | Reason | Prerequisites |
|-------|------|--------|---------------|
| adapters-db-sqlx | `test_postgres_repository_roundtrip` | Requires Docker | Docker daemon running |
| adapters-grpc | `test_grpc_service_create_task` | Network/port binding | None |
| rust_iac_config | `test_kustomize_required_but_missing` | Mutates global state | Run in isolation |
| rust_iac_config | `test_environment_names` | Mutates global state | Run in isolation |
| xtask | `test_validate_xtask_commands` | Infrastructure incomplete | Planned for v1.2 |
| xtask | `test_extract_commands_from_enum` | Infrastructure incomplete | Planned for v1.2 |
| xtask-lib | `test_validate_xtask_commands` | Infrastructure incomplete | Planned for v1.2 |

---

## Database Adapter Integration Test

**Crate:** `adapters-db-sqlx`
**Test:** `test_postgres_repository_roundtrip`
**File:** `crates/adapters-db-sqlx/tests/integration.rs`

### Why Ignored

Requires Docker to be running to spin up a PostgreSQL container via testcontainers. This keeps CI fast and avoids requiring Docker in all development environments.

### Prerequisites

- Docker daemon must be running
- No specific Docker configuration needed (uses `postgres:16-alpine` image)

### How to Run

```bash
# Run just this test
cargo test -p adapters-db-sqlx test_postgres_repository_roundtrip -- --ignored

# Run all ignored tests in this crate
cargo test -p adapters-db-sqlx -- --ignored
```

### What It Tests

- Connects to ephemeral PostgreSQL container
- Creates tasks table via migration
- Tests full CRUD cycle: save, find_by_id, update_status, find_all

---

## gRPC Adapter Smoke Test

**Crate:** `adapters-grpc`
**Test:** `test_grpc_service_create_task`
**File:** `crates/adapters-grpc/tests/smoke.rs`

### Why Ignored

Starts a gRPC server on a random port and makes network calls. Kept ignored to avoid port-binding issues in parallel test runs.

### Prerequisites

None (uses in-memory repository, no external dependencies)

### How to Run

```bash
# Run just this test
cargo test -p adapters-grpc test_grpc_service_create_task -- --ignored

# Run all ignored tests in this crate
cargo test -p adapters-grpc --test smoke -- --ignored
```

### What It Tests

- Starts TaskServiceImpl gRPC server on random port
- Uses tonic client to call CreateTask RPC
- Verifies task creation succeeds end-to-end

---

## rust_iac_config Global State Tests

**Crate:** `rust_iac_config`
**Tests:** `test_kustomize_required_but_missing`, `test_environment_names`
**File:** `crates/rust_iac_config/tests/integration_tests.rs`

### Why Ignored

These tests call `std::env::set_current_dir()` which mutates global process state. When run in parallel with other tests, this causes non-deterministic failures due to race conditions.

### Prerequisites

Must be run in isolation (single-threaded or as the only test)

### How to Run

```bash
# Run with single thread to avoid race conditions
cargo test -p rust_iac_config test_kustomize_required_but_missing -- --ignored --test-threads=1

cargo test -p rust_iac_config test_environment_names -- --ignored --test-threads=1

# Run all ignored tests in isolation
cargo test -p rust_iac_config -- --ignored --test-threads=1
```

### What They Test

- `test_kustomize_required_but_missing`: Tests error handling when kustomize binary is required but not available
- `test_environment_names`: Tests environment name resolution in Kubernetes manifests

---

## xtask Validation Infrastructure Tests

**Crate:** `xtask`, `xtask-lib`
**Tests:** `test_validate_xtask_commands`, `test_extract_commands_from_enum`
**Files:** `crates/xtask/src/validation.rs`, `crates/xtask-lib/src/validation.rs`

### Why Ignored

These tests are part of validation infrastructure that isn't fully integrated yet. They're planned for v1.2 release.

### Prerequisites

Requires completion of validation infrastructure work

### How to Run

```bash
# These tests will fail until infrastructure is complete
cargo test -p xtask test_validate_xtask_commands -- --ignored
cargo test -p xtask test_extract_commands_from_enum -- --ignored
cargo test -p xtask-lib test_validate_xtask_commands -- --ignored
```

### What They Test

- Validates that the Commands enum matches the spec
- Ensures xtask command names are synchronized with devex_flows.yaml

---

## Running All Ignored Tests

To run all ignored tests in the workspace (useful for comprehensive local validation):

```bash
# Run all ignored tests (may fail if Docker is not running)
cargo test --workspace -- --ignored

# Run with single thread to avoid global state conflicts
cargo test --workspace -- --ignored --test-threads=1
```

## CI Considerations

- The `tier1-selftest.yml` workflow does **not** run ignored tests by default
- Integration tests requiring Docker should be run in a separate CI job with Docker support
- The `--ignored` flag is intentionally not used in standard CI to keep builds fast

## Adding New Ignored Tests

When marking a test as ignored:

1. Always provide a reason in the ignore attribute:

   ```rust
   #[ignore = "Requires Docker for testcontainers"]
   ```

2. Document the test in this file with:
   - Why it's ignored
   - Prerequisites to run it
   - How to run it
   - What it tests

3. Ensure the test can actually pass when run with the right prerequisites
