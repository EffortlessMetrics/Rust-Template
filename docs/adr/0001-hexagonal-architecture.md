# ADR-0001: Hexagonal Architecture via Workspace Crates

**Status**: Accepted
**Date**: 2025-01-18
**Authors**: Steven Zimmerman
**Related ACs**: AC-TPL-001

---

## Context

This template aims to support services that:

- Evolve business logic independently of delivery mechanisms (HTTP, gRPC, CLI)
- Swap infrastructure adapters (databases, message queues) without touching core logic
- Test business rules in isolation from I/O
- Scale teams by allowing parallel work on core vs adapters

Traditional monolithic Rust services often mix business logic with framework code (Axum handlers calling DB repos directly), making testing hard and refactoring expensive.

We need an architecture that:

1. Makes dependencies point inward (business logic depends on nothing external)
2. Keeps ports/adapters as thin translation layers
3. Allows multiple delivery mechanisms to share the same core
4. Is enforceable via project structure and linting

---

## Decision

We adopt **hexagonal architecture** (ports-and-adapters) as the workspace layout:

```
workspace/
├── crates/
│   ├── business-core/       # Domain logic, no external deps
│   ├── app-http/            # HTTP adapter (Axum)
│   ├── adapters-grpc/       # gRPC adapter (optional)
│   ├── adapters-db/         # Database adapter (optional)
│   ├── app-telemetry/       # Cross-cutting telemetry (tracing, metrics)
│   ├── acceptance/          # BDD tests (drive via HTTP or core)
│   └── xtask/               # Build automation
```

**Core principles:**

1. **business-core** contains:
   - Domain models
   - Use cases / service interfaces
   - Pure business rules (no I/O, no frameworks)
   - Only dependencies: `serde`, `thiserror`, basic utilities

2. **app-http** (and other `app-*` crates):
   - Axum routes, handlers, middleware
   - Calls business-core
   - Translates HTTP concerns ↔ domain types
   - Owns OpenAPI spec compliance

3. **adapters-*** crates:
   - Infrastructure implementations (DB repos, message clients)
   - Implement interfaces defined by business-core (via traits)
   - Can be swapped/mocked

4. **Dependency flow**:
   ```
   HTTP/gRPC → business-core ← adapters-db
   ```
   Business-core never imports app-http or adapters-*.

**Enforcement:**

- `cargo check` will fail if business-core tries to depend on app-http
- BDD tests in `acceptance/` can test via HTTP *or* directly via business-core
- Clippy lints (future) could warn about layering violations

---

## Consequences

### Positive

- **Testability**: Business logic testable without spinning up HTTP servers or databases
- **Replaceability**: Swap HTTP for gRPC, Postgres for DynamoDB, without touching core
- **Team scaling**: Frontend/backend/infra teams can work in parallel on separate crates
- **Clarity**: New developers immediately see "core is here, delivery is there"

### Negative

- **Boilerplate**: More crates = more `Cargo.toml` files, more trait definitions for ports
- **Learning curve**: Developers unfamiliar with hexagonal may be confused by indirection
- **Refactoring friction**: Moving logic between layers requires changing crate boundaries

### Neutral

- **Build times**: More crates can parallelize builds, but also add overhead for small changes
- **Monorepo commitment**: This pattern works best in a workspace; extracting single crates later requires care

---

## Compliance

**Automated:**

- Cargo workspace dependencies enforce the graph (business-core cannot depend on app-http)
- `xtask selftest` runs tests for all crates, ensuring boundaries are respected
- BDD scenarios in `acceptance/` validate core behavior via adapters

**Manual:**

- Code review should reject PRs that:
  - Add framework deps to business-core
  - Put business logic in HTTP handlers
  - Leak domain types into adapter implementation details

**Future enforcement:**

- Clippy custom lint to detect layering violations
- Rego policy to validate dependency graph in `Cargo.toml` files

---

## Notes

**Why "business-core" instead of "domain"?**

- "Domain" implies DDD ubiquitous language; this template doesn't mandate DDD
- "Core" is neutral: it's the center of the hexagon, agnostic to modeling approach

**Why workspace instead of single crate with modules?**

- Cargo makes cross-crate dependencies explicit in `Cargo.toml`
- Modules can still be violated via `pub(crate)` or accidental imports
- Workspaces enable independent versioning if crates are published

**Migration from monolith:**

If you have an existing Axum service:

1. Create `business-core/` and move domain models there
2. Create `app-http/` and move Axum setup there
3. Use traits to define ports business-core needs (e.g., `UserRepository`)
4. Implement ports in `adapters-db/`
5. Wire dependencies in `app-http/src/main.rs`

**References:**

- Alistair Cockburn, "Hexagonal Architecture" (2005)
- Robert C. Martin, "Clean Architecture" (2017, Chapter 22)
- [Rust workspace documentation](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
