# Feature Flag Test Matrix

**Version:** 3.3.6
**Last Updated:** 2025-12-01

## Overview

This document describes the Cargo feature flags available in the Rust-as-Spec template and their testing coverage in CI.

## Available Cargo Features

### 1. `adapters-grpc` Crate

**Location:** `/crates/adapters-grpc/Cargo.toml`

| Feature | Description | Default | Test Coverage |
|---------|-------------|---------|---------------|
| `default` | No features enabled by default | ✗ | ✓ (tier1-selftest) |
| `integration-grpc` | Enable gRPC integration tests | ✗ | ✓ (manual, not in CI) |

**Usage:**
```bash
# Run with gRPC integration tests
cargo test -p adapters-grpc --features integration-grpc
```

**CI Coverage:**
- **Tier-1 Selftest:** Builds without features (default)
- **Integration Tests:** Not currently automated in CI (requires external gRPC service)

### 2. `adapters-db-sqlx` Crate

**Location:** `/crates/adapters-db-sqlx/Cargo.toml`

| Feature | Description | Default | Test Coverage |
|---------|-------------|---------|---------------|
| `default` | No features enabled by default | ✗ | ✓ (tier1-selftest) |
| `integration-db` | Enable database integration tests | ✗ | ✓ (manual, not in CI) |

**Usage:**
```bash
# Run with database integration tests
cargo test -p adapters-db-sqlx --features integration-db
```

**CI Coverage:**
- **Tier-1 Selftest:** Builds without features (default)
- **Integration Tests:** Not currently automated in CI (requires live database)
- **DB Schema Validation:** Covered by `ci-db.yml` (Atlas migrations)

### 3. `telemetry` Crate

**Location:** `/crates/telemetry/Cargo.toml`

| Feature | Description | Default | Test Coverage |
|---------|-------------|---------|---------------|
| `default` | Basic telemetry without OTLP | ✓ | ✓ (tier1-selftest) |
| `otlp` | Enable OpenTelemetry OTLP export (gRPC) | ✗ | ✓ (tier1-selftest) |

**Feature Dependencies (otlp):**
- `opentelemetry`
- `opentelemetry_sdk` (with `rt-tokio`)
- `opentelemetry-otlp` (with `tonic`, `trace`)
- `tracing-opentelemetry`

**Usage:**
```bash
# Build with OTLP support
cargo build -p telemetry --features otlp

# Build without OTLP (default)
cargo build -p telemetry
```

**CI Coverage:**
- **Tier-1 Selftest:** Builds with default features (no OTLP)
- **OTLP Feature:** Not currently tested in CI (optional telemetry backend)

## CI Test Matrix

### Current Coverage

| Workflow | Features Tested | Notes |
|----------|-----------------|-------|
| `tier1-selftest.yml` | Default (no features) | Full workspace build + selftest |
| `ci-db.yml` | N/A | Schema validation only (Atlas) |
| `ci-proto.yml` | N/A | Protobuf linting (buf) |
| `ci-coverage.yml` | Default (no features) | Code coverage (tarpaulin) |

### Feature Combinations NOT Currently Tested in CI

The following feature combinations are **not** currently tested in CI but are valid for manual testing:

1. **adapters-grpc + integration-grpc**
   - Requires: External gRPC service running
   - Manual test: `cargo test -p adapters-grpc --features integration-grpc`

2. **adapters-db-sqlx + integration-db**
   - Requires: Live SQLite/Postgres database
   - Manual test: `cargo test -p adapters-db-sqlx --features integration-db`

3. **telemetry + otlp**
   - Requires: OTLP collector endpoint (optional)
   - Manual test: `cargo build -p telemetry --features otlp`

4. **Full feature matrix (all optional features)**
   - Manual test: `cargo test --workspace --exclude acceptance --exclude xtask --all-features`

## Recommended CI Enhancements

To improve feature flag coverage in CI, consider adding:

### 1. Feature Matrix Job (`.github/workflows/ci-feature-matrix.yml`)

```yaml
name: Feature Matrix
on:
  pull_request:
    paths: ['crates/adapters-*/**', 'crates/telemetry/**']

jobs:
  feature-matrix:
    strategy:
      matrix:
        features:
          - ""                          # Default (no features)
          - "telemetry/otlp"           # OTLP telemetry
          - "adapters-grpc/default"    # gRPC adapter (no integration)
          - "adapters-db-sqlx/default" # DB adapter (no integration)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: cachix/install-nix-action@v27
      - run: nix develop -c cargo build --workspace --features ${{ matrix.features }}
      - run: nix develop -c cargo test --workspace --exclude acceptance --features ${{ matrix.features }}
```

### 2. Integration Test Job (Manual Trigger)

```yaml
name: Integration Tests
on:
  workflow_dispatch:  # Manual trigger only

jobs:
  integration-grpc:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: cachix/install-nix-action@v27
      # TODO: Start gRPC test service
      - run: nix develop -c cargo test -p adapters-grpc --features integration-grpc

  integration-db:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: cachix/install-nix-action@v27
      - run: nix develop -c cargo test -p adapters-db-sqlx --features integration-db
        env:
          DATABASE_URL: "sqlite::memory:"
```

## Feature Flag Best Practices

1. **Default Features:** Keep `default` minimal for fast builds
2. **Integration Features:** Gate integration tests behind feature flags (e.g., `integration-grpc`)
3. **Optional Dependencies:** Use `optional = true` for heavy dependencies
4. **CI Coverage:** Test default features in tier1-selftest, optional features in separate jobs
5. **Documentation:** Keep this matrix up-to-date when adding new features

## Related Documents

- `docs/TROUBLESHOOTING.md` - Environment setup and common issues
- `.github/workflows/tier1-selftest.yml` - Main CI workflow
- `crates/*/Cargo.toml` - Feature definitions

## Changelog

| Date | Version | Changes |
|------|---------|---------|
| 2025-12-01 | 3.3.6 | Initial feature flag test matrix documentation |
