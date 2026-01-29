<!-- doclint:disable orphan-version -->
<!-- Note: Contains references to historical kernel versions and contract baselines -->

# Contract Inventory

This document catalogs the stable interfaces that must remain versioned and governed across the template. Breaking changes to these interfaces require explicit approval via ADR and version bump.

## Design Principles

1. **Contract crates are few and stable** - Only a small allowlist of crates are designated as contract crates
2. **Fail on breaking changes** - CI fails unless explicitly approved via ADR + manifest bump
3. **Layering enforcement** - Contract crates must not pull in heavy runtime dependencies
4. **Wire contract checks** - Verify `/platform/*` and `xtask --json` outputs

## Contract Surfaces

### 1. HTTP Contract

**Description**: JSON payloads and OpenAPI schema for `/platform/*` endpoints

**Crate**: [`platform-contract`](../../crates/platform-contract/Cargo.toml)

**CI Gate**: `cargo xtask check-openapi-diff`

**Versioning Rule**:
- Breaking changes require ADR documenting the change
- Update `specs/contracts_manifest.yaml` with new contract version
- Update `specs/openapi/openapi.yaml` with new schema

**Key Endpoints**:
- `/platform/status` - Governance health and metrics
- `/platform/graph` - Governance graph structure
- `/platform/devex/flows` - DevEx flows definition
- `/platform/docs/index` - Documentation index
- `/platform/schema` - Platform schemas index
- `/platform/openapi` - OpenAPI specification
- `/platform/tasks` - Task list
- `/platform/tasks/suggest-next` - Task hints
- `/platform/agent/hints` - Agent hints
- `/platform/questions` - Questions list
- `/platform/friction` - Friction log
- `/platform/forks` - Fork registry
- `/platform/issues` - Unified issues endpoint

**Escalation Path**:
1. Create ADR documenting the breaking change
2. Update OpenAPI spec with new schema
3. Update TypeScript consumer tests in `examples/backstage-plugin`
4. Bump template version via `cargo xtask release-prepare`

---

### 2. CLI Contract

**Description**: JSON output schemas and exit codes for xtask commands

**Crate**: [`xtask-contract`](../../crates/xtask-contract/Cargo.toml)

**CI Gate**: `cargo xtask check-json-schemas`

**Versioning Rule**:
- Breaking changes require ADR documenting the change
- Update `specs/contracts_manifest.yaml` with new contract version
- Update golden snapshots in `specs/schemas/`

**Key Commands**:
- `ac-status --json` - AC coverage report
- `friction-list --json` - Friction log entries
- `questions-list --json` - Questions list
- `fork-list --json` - Fork registry
- `issues-search --json` - Unified issues search
- `version --json` - Version information
- `idp-snapshot` - IDP snapshot

**Exit Code Contract**:
- `0`: Success
- `1`: General failure
- Non-zero for specific error conditions

**Escalation Path**:
1. Create ADR documenting the breaking change
2. Update golden snapshots in `specs/schemas/`
3. Update consumer documentation
4. Bump template version via `cargo xtask release-prepare`

---

### 3. Spec Contract

**Description**: Shapes and structure of specification YAML files

**Crate**: [`spec-types`](../../crates/spec-types/Cargo.toml)

**CI Gate**: `cargo xtask check-api-diff` (for spec-types crate)

**Versioning Rule**:
- Breaking changes require ADR documenting the change
- Update `specs/contracts_manifest.yaml` with new contract version
- Update affected spec files with new schema

**Key Spec Files**:
- `spec_ledger.yaml` - Story → Requirement → AC → Test mapping
- `config_schema.yaml` - Configuration schema
- `devex_flows.yaml` - DevEx flows and commands
- `tasks.yaml` - Task definitions
- `contracts_manifest.yaml` - Contract definitions

**Escalation Path**:
1. Create ADR documenting the breaking change
2. Update spec file with new schema
3. Update dependent crates and consumers
4. Bump template version via `cargo xtask release-prepare`

---

### 4. Receipt Contract

**Description**: JSON schemas for governance receipts

**Crate**: [`receipts-core`](../../crates/receipts-core/Cargo.toml)

**CI Gate**: `cargo xtask check-api-diff` (for receipts-core crate)

**Versioning Rule**:
- Breaking changes require ADR documenting the change
- Update `specs/contracts_manifest.yaml` with new contract version
- Update receipt schemas in `specs/schemas/`

**Key Receipts**:
- `gate.json` - Gate execution results
- `economics.json` - DevLT and compute tracking
- `dossier.json` - PR forensics
- `quality.json` - Code quality metrics
- `telemetry.json` - Probe execution results
- `timeline.json` - Development timeline analysis

**Escalation Path**:
1. Create ADR documenting the breaking change
2. Update receipt schemas in `specs/schemas/`
3. Update dependent crates and consumers
4. Bump template version via `cargo xtask release-prepare`

---

### 5. Kernel Closure Contract

**Description**: Stability guarantees for forks pinned to specific kernel versions

**Crate**: N/A (governed by version manifest)

**CI Gate**: `cargo xtask version-check` (validates version consistency)

**Versioning Rule**:
- Breaking changes require major version bump
- Update `specs/spec_ledger.yaml` metadata
- Update `specs/version_manifest.yaml`

<!-- doclint:disable orphan-version -->
**Closure Statement**:
> "A fork pinned to `v3.3.9-kernel` can trust the following contracts to remain stable:
> - Selftest gate structure (12 steps)
> - Kernel AC count and classification rules
> - Platform endpoint schema
> - DevEx flow definitions
> - Receipt schemas"
<!-- doclint:enable orphan-version -->

**Escalation Path**:
1. Create ADR documenting the breaking change
2. Update kernel version in `specs/spec_ledger.yaml`
3. Update `specs/version_manifest.yaml`
4. Run `cargo xtask release-prepare` to propagate version

---

## Contract Crates

The following crates are designated as contract crates and must follow strict dependency rules:

| Crate | Purpose | Dependencies |
|--------|-----------|--------------|
| `platform-contract` | HTTP API types | `serde`, `serde_json`, `chrono`, `uuid`, `thiserror` |
| `xtask-contract` | CLI output types | `serde`, `serde_json`, `chrono`, `uuid`, `thiserror` |
| `receipts-core` | Receipt schemas | `serde`, `serde_json`, `chrono`, `uuid`, `thiserror` |
| `spec-types` | Spec file types | `serde`, `serde_yaml`, `thiserror` |

**Forbidden Dependencies** (for contract crates):
- `axum` - HTTP framework (belongs to app-http)
- `tokio` - Async runtime (belongs to app-http)
- `clap` - CLI parser (belongs to xtask)
- `sqlx` - Database (belongs to adapters-db-sqlx)
- `tonic` - gRPC framework (belongs to adapters-grpc)
- `jsonschema` - Schema validation (belongs to spec-runtime)
- `anyhow` - Error handling (use `thiserror` instead)

---

## CI Gates

### check-api-diff

**Purpose**: Detect breaking changes in public API of contract crates

**Usage**:

```bash
cargo xtask check-api-diff
```

**Behavior**:
- Uses `cargo public-api` or similar tooling
- Compares current public API against baseline (main branch)
- Fails if breaking changes detected
- Allows explicit approval via ADR reference

**Exit Codes**:
- `0`: No breaking changes detected
- `1`: Breaking changes detected (requires ADR)

---

### check-openapi-diff

**Purpose**: Detect breaking changes in `/platform/*` endpoint contracts

**Usage**:

```bash
cargo xtask check-openapi-diff
```

**Behavior**:
- Generates OpenAPI spec deterministically from code
- Compares against baseline (main branch)
- Fails on breaking changes unless contract bump declared
- Validates schema consistency

**Exit Codes**:
- `0`: No breaking changes detected
- `1`: Breaking changes detected (requires ADR)

---

### check-json-schemas

**Purpose**: Detect breaking changes in CLI JSON output contracts

**Usage**:

```bash
cargo xtask check-json-schemas
```

**Behavior**:
- Generates JSON Schema for each command output
- Compares against golden snapshots
- Fails on breaking changes unless versioned
- Validates exit code contract

**Exit Codes**:
- `0`: No breaking changes detected
- `1`: Breaking changes detected (requires ADR)

---

### check-layering

**Purpose**: Enforce dependency rules for contract crates

**Usage**:

```bash
cargo xtask check-layering
```

**Behavior**:
- Uses `cargo metadata` to analyze dependencies
- Validates contract crates don't depend on forbidden packages
- Checks for circular dependencies
- Validates foundation crates have minimal dependencies

**Exit Codes**:
- `0`: All layering rules satisfied
- `1`: Layering violation detected

---

## Approval Process

When a breaking change is required:

1. **Create ADR**: Document the rationale for the breaking change
   - Use `cargo xtask adr-new <title>` to create the ADR
   - Reference the specific contract surface being changed
   - Document migration path for consumers

2. **Update Contract Manifest**: Bump contract version in `specs/contracts_manifest.yaml`
   - Update the `schema_version` field
   - Document the breaking change

3. **Update Schemas**: Update the affected schema files
   - OpenAPI spec for HTTP contract
   - JSON schemas for CLI contract
   - Spec files for spec contract

4. **Version Bump**: Run `cargo xtask release-prepare`
   - This propagates version changes across all files
   - Updates changelog with breaking change notes

5. **Update Consumers**: Update dependent code and documentation
   - TypeScript consumers in `examples/backstage-plugin`
   - Fork documentation
   - Integration tests

---

## CI Integration

These checks should be added to CI workflows:

```yaml
- name: Contract Stability Checks
  run: |
    cargo xtask check-api-diff
    cargo xtask check-openapi-diff
    cargo xtask check-json-schemas
    cargo xtask check-layering
```

For PRs that intentionally break contracts:
- Include ADR reference in PR description
- Add `[contract-bump]` label
- Update `specs/contracts_manifest.yaml` schema version

---

## Maintenance

When adding new contract surfaces:

1. Add entry to this inventory document
2. Create corresponding CI gate
3. Update `specs/contracts_manifest.yaml` with new contract
4. Document versioning rules and escalation path
5. Update consumer documentation

When retiring contract surfaces:

1. Mark as deprecated in this inventory
2. Update CI to skip checks for deprecated contracts
3. Document migration path to new contract
4. Update consumer documentation

---

## References

- [ADR-0001: Hexagonal Architecture](../adr/0001-hexagonal-architecture.md)
- [ADR-0003: Spec and BDD as Source of Truth](../adr/0003-spec-and-bdd-as-source-of-truth.md)
- [ADR-0005: Selftest as Single Gate](../adr/0005-xtask-selftest-single-gate.md)
- [specs/contracts_manifest.yaml](../../specs/contracts_manifest.yaml)
- [specs/spec_ledger.yaml](../../specs/spec_ledger.yaml)
- [specs/devex_flows.yaml](../../specs/devex_flows.yaml)
