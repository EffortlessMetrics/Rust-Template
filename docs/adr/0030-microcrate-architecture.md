# ADR-0030: Microcrate Architecture for Contract Stability

**Status**: Accepted
**Date**: 2026-01-27
**Authors**: Steven Zimmerman
**Related ACs**: AC-TPL-MICROCRATE-001, AC-TPL-MICROCRATE-002, AC-TPL-MICROCRATE-003, AC-TPL-MICROCRATE-004
**Relates to**: ADR-0001 (hexagonal-architecture)

---

## Context

The original hexagonal architecture (ADR-0001) established a clean separation between core logic and adapters. However, as the template evolved, several issues emerged:

### Problems with Original Structure

1. **Monolithic Core**: `business-core` and `model` crates accumulated diverse concerns (domain models, governance types, receipt schemas, CLI output types), making it difficult to reason about stability boundaries.

2. **Unclear Contract Surface**: No clear distinction between "stable API surface" (contracts) and internal implementation details. This led to accidental breaking changes when modifying core types.

3. **Dependency Violations**: Adapters occasionally depended on framework-specific types from `app-http`, creating circular dependencies and violating hexagonal principles.

4. **No Versioned Contracts**: While `contracts_manifest.yaml` tracked HTTP API versions, there was no systematic tracking of Rust API stability across crate boundaries.

5. **Testing Challenges**: Contract crates couldn't be tested independently due to mixed responsibilities (e.g., governance types mixed with domain logic).

### Need

We need an architecture that:

1. **Separates contract from implementation**: Clearly distinguish between stable APIs (contracts) and internal implementation details.
2. **Enforces dependency inversion**: Higher layers depend on lower layers; contracts depend on nothing external.
3. **Provides stability guarantees**: Contract crates have minimal dependencies and explicit versioning.
4. **Enables independent testing**: Contract crates can be tested without spinning up infrastructure.
5. **Supports gradual adoption**: Existing crates can be migrated incrementally.

---

## Decision

We adopt **microcrate architecture** with explicit contract crates and strict layering rules.

### 1. Crate Taxonomy

Crates are organized into five categories:

| Category | Purpose | Examples |
|----------|---------|----------|
| **Contract** | Stable API surface, versioned, minimal deps | `platform-contract`, `xtask-contract`, `receipts-core`, `spec-types` |
| **Core Logic** | Business rules, domain models, governance logic | `gov-model`, `spec-ledger` |
| **Foundation** | Cross-cutting utilities, lightweight infrastructure | `http-errors`, `http-platform`, `http-core`, `telemetry` |
| **Adapter** | Infrastructure implementations (DB, HTTP, messaging) | `adapters-db-sqlx`, `gov-http-*`, `http-middleware` |
| **HTTP/Router** | Axum application, routing, middleware wiring | `app-http` |
| **Facade** | Build-time tools, configuration, IaC | `rust_iac_config`, `rust_iac_xtask_core` |

### 2. Layering Rules

#### Contract Crates

Contract crates define the **stable API surface** of the platform. They must:

- **Depend only on foundation crates** (or standard library)
- **Forbidden dependencies**: `axum`, `tokio`, `clap`, `sqlx`, `tonic`, `jsonschema`
- **Have minimal dependencies** (typically < 5)
- **Be versioned** in `specs/contracts_manifest.yaml`

**Rationale**: Contract crates are consumed by adapters, HTTP handlers, and external services. They must remain stable across releases.

#### Foundation Crates

Foundation crates provide cross-cutting utilities. They must:

- **Have minimal dependencies** (max 10)
- **Depend only on other foundation crates** or standard library
- **Be framework-agnostic** (no Axum, Tokio, etc.)

**Rationale**: Foundation crates are shared by contracts, core logic, and adapters. Lightweight foundation ensures contracts stay minimal.

#### Core Logic Crates

Core logic crates contain business rules and domain models. They may:

- **Depend on contract crates** for type definitions
- **Depend on foundation crates** for utilities
- **Define traits** that adapters implement

**Rationale**: Core logic is the "inner circle" of hexagonal architecture. It should be testable without infrastructure.

#### Adapter Crates

Adapter crates implement infrastructure (DB, HTTP, messaging). They may:

- **Depend on core logic crates** for domain types
- **Depend on foundation crates** for utilities
- **Implement traits** defined by core logic

**Rationale**: Adapters are the "outer circle" of hexagonal architecture. They translate between external systems and core logic.

#### HTTP/Router Crate

`app-http` is the HTTP application layer. It may:

- **Depend on adapter crates** for infrastructure
- **Depend on core logic crates** for domain types
- **Wire dependencies** in `main.rs`

**Rationale**: `app-http` is the delivery mechanism. It should be thin and delegate to adapters and core logic.

#### Facade Crates

Facade crates provide build-time utilities and configuration. They may:

- **Depend on any crate** (they are not part of runtime layering)
- **Provide CLI tools** for development and operations

**Rationale**: Facade crates are build-time concerns (xtask, IaC). They are not subject to runtime layering rules.

### 3. Dependency Graph

```
┌─────────────────────────────────────────────────────────────────┐
│                    HTTP/Router Layer                        │
│                      app-http                              │
└─────────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Adapter Layer                            │
│  adapters-db-sqlx  │  gov-http-*  │  http-middleware    │
└─────────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Core Logic Layer                          │
│                gov-model  │  spec-ledger                     │
└─────────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Contract Layer                             │
│  platform-contract │ xtask-contract │ receipts-core │ spec-types │
└─────────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Foundation Layer                             │
│  http-errors │ http-platform │ http-core │ telemetry          │
└─────────────────────────────────────────────────────────────────┘
```

**Key rules**:
- Dependencies point **downward** (higher layers depend on lower layers)
- Contract crates depend only on foundation (or std)
- Foundation crates depend only on other foundation (or std)
- No circular dependencies

### 4. Contract Inventory

| Contract Crate | Purpose | Versioned In |
|----------------|---------|---------------|
| `platform-contract` | HTTP API types, error types, status types | `specs/contracts_manifest.yaml` |
| `xtask-contract` | CLI output types, JSON schemas | `specs/contracts_manifest.yaml` |
| `receipts-core` | Receipt schemas (dossier, economics, gate, etc.) | `specs/contracts_manifest.yaml` |
| `spec-types` | Spec file types (ledger, tasks, features) | `specs/contracts_manifest.yaml` |

### 5. Guidelines for Adding New Crates

#### When to Create a Contract Crate

Create a contract crate when:

1. You need to expose a **stable API surface** across crate boundaries
2. Multiple adapters need to **share common types**
3. You want to **version the API** explicitly

**Example**: If you add a new HTTP endpoint, define request/response types in `platform-contract`.

#### When to Create a Core Logic Crate

Create a core logic crate when:

1. You have **business rules** that are independent of infrastructure
2. You need to **test logic** without spinning up databases or HTTP servers
3. You want to **define traits** for adapters to implement

**Example**: If you add a new governance policy, create `gov-policy` crate with policy evaluation logic.

#### When to Create a Foundation Crate

Create a foundation crate when:

1. You have **cross-cutting utilities** used by multiple crates
2. The utilities are **framework-agnostic** (no Axum, Tokio, etc.)
3. You want to **keep contract dependencies minimal**

**Example**: If you add a new error type used across multiple crates, add it to `http-errors`.

#### When to Create an Adapter Crate

Create an adapter crate when:

1. You need to **implement infrastructure** (DB, HTTP client, message queue)
2. You want to **swap implementations** (e.g., Postgres → DynamoDB)
3. You need to **translate between external systems and core logic**

**Example**: If you add a new database backend, create `adapters-db-dynamodb`.

### 6. Enforcement via xtask Commands

Four new xtask commands enforce microcrate architecture:

#### `cargo xtask check-api-diff`

Checks contract crates for breaking API changes:

- Uses `cargo-public-api` to detect removed functions, type changes
- Validates contract crates don't depend on forbidden packages
- Requires ADR approval for breaking changes

#### `cargo xtask check-openapi-diff`

Checks OpenAPI contract for breaking changes:

- Validates `/platform/*` endpoints are present
- Compares against baseline list of expected endpoints
- Warns if contract version not tracked

#### `cargo xtask check-json-schemas`

Checks CLI JSON output schemas for breaking changes:

- Compares current output against golden snapshots in `specs/schemas/`
- Detects removed fields, type changes
- Supports `--generate` flag for initial setup

#### `cargo xtask check-layering`

Enforces dependency layering rules:

- Validates contract crates don't depend on forbidden packages
- Checks foundation crates have minimal dependencies (max 10)
- Detects circular dependencies using DFS algorithm

---

## Consequences

### Positive

1. **Clear Stability Boundaries**: Contract crates are explicitly versioned, making it easy to understand what can change without breaking consumers.

2. **Independent Testing**: Contract and core logic crates can be tested without infrastructure, improving test speed and reliability.

3. **Reduced Coupling**: Strict layering rules prevent accidental dependencies between layers, making refactoring safer.

4. **Better Documentation**: Crate taxonomy makes it immediately clear what each crate is responsible for.

5. **Automated Enforcement**: xtask commands catch layering violations and breaking changes before they land in main.

6. **Gradual Adoption**: Existing crates can be migrated incrementally to the new structure.

### Negative

1. **More Crates**: Microcrate architecture increases the number of crates, which adds overhead (more `Cargo.toml` files, more dependency management).

2. **Learning Curve**: Developers need to understand the five-layer taxonomy and layering rules.

3. **Refactoring Friction**: Moving logic between layers requires changing crate boundaries, which can be tedious.

4. **Initial Migration Effort**: Existing code must be migrated to the new structure, which requires careful planning.

### Neutral

1. **Build Times**: More crates can parallelize builds, but also add overhead for small changes. Net effect depends on project size.

2. **Monorepo Commitment**: This pattern works best in a workspace; extracting single crates later requires care.

3. **Tooling Overhead**: New xtask commands add complexity to the build system.

---

## Compliance

### Automated

1. **`cargo xtask check-api-diff`**: Detects breaking changes to contract crates.
2. **`cargo xtask check-openapi-diff`**: Validates OpenAPI contract stability.
3. **`cargo xtask check-json-schemas`**: Checks JSON output schema stability.
4. **`cargo xtask check-layering`**: Enforces dependency layering rules.
5. **CI Gates**: All four commands run in CI; PRs fail if violations detected.
6. **`xtask selftest`**: Includes contract stability checks as part of comprehensive validation.

### Manual

1. **Code Review**: Reviewers should reject PRs that:
   - Add forbidden dependencies to contract crates
   - Put business logic in HTTP handlers
   - Leak framework types into contract crates
   - Violate layering rules

2. **ADR Process**: Breaking changes to contract crates require an ADR documenting the change and migration path.

### Future Enforcement

1. **Clippy Custom Lints**: Detect layering violations at compile time.
2. **Rego Policy**: Validate dependency graph in `Cargo.toml` files.
3. **Automated Migration Tools**: Suggest crate boundaries based on dependency analysis.

---

## Notes

### Why "Microcrate" Instead of "Module"?

Cargo crates enforce boundaries at the build system level:
- `Cargo.toml` makes dependencies explicit
- Modules can still be violated via `pub(crate)` or accidental imports
- Workspaces enable independent versioning if crates are published

### Why Five Layers Instead of Three?

Original hexagonal architecture had three layers (core, adapters, delivery). Five layers provide:
- **Explicit contract layer**: Distinguishes stable APIs from implementation
- **Foundation layer**: Keeps contract dependencies minimal
- **Clearer separation**: Each layer has a well-defined purpose

### How Does This Relate to ADR-0001?

ADR-0001 established hexagonal architecture principles (ports and adapters). ADR-0030 refines those principles with:
- **Explicit contract crates**: Stable API surface is now a first-class concept
- **Five-layer taxonomy**: More granular layering than original three-layer model
- **Automated enforcement**: xtask commands catch violations

### What About Existing Crates?

Existing crates are grandfathered in:
- `business-core` and `model` can be gradually migrated to new structure
- No immediate refactoring required
- New code should follow microcrate architecture
- Legacy code updated opportunistically

### How to Handle Breaking Changes?

1. **Create an ADR** documenting the breaking change and migration path.
2. **Update `specs/contracts_manifest.yaml`** with new contract version.
3. **Update dependent crates** to use new API.
4. **Run `cargo xtask check-api-diff --adr <path>`** to approve change.
5. **Release with new version** following semver semantics.

---

## References

- ADR-0001: Hexagonal Architecture via Workspace Crates
- `docs/explanation/architecture.md` — Microcrate architecture documentation
- `docs/reference/xtask-commands.md` — Contract stability commands reference
- `specs/contracts_manifest.yaml` — Contract version tracking
- Robert C. Martin, "Clean Architecture" (2017, Chapter 22)
- Alistair Cockburn, "Hexagonal Architecture" (2005)
