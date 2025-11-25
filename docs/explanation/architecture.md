# Architecture Overview

This document explains the architectural decisions, design patterns, and philosophy behind the Rust Template.

## Table of Contents

1. [Design Philosophy](#design-philosophy)
2. [Crate Structure](#crate-structure)
3. [Platform Cell Surfaces](#platform-cell-surfaces)
4. [Hexagonal Architecture](#hexagonal-architecture)
5. [Governance Kernel & Selftest](#governance-kernel--selftest)
6. [Governance Model](#governance-model)
7. [Observability Strategy](#observability-strategy)
8. [Environment Model](#environment-model)
9. [Security & Access](#security--access)
10. [Why These Choices](#why-these-choices)

---

## Design Philosophy

The template is built around three core principles:

### 1. Specification-First Development

**Philosophy:** Code should implement specifications, not the other way around.

- Specifications live in `specs/spec_ledger.yaml`
- User Stories → Requirements → Acceptance Criteria form a hierarchy
- Every AC has mapped tests (enforced by policy)
- Changes start with spec updates, then tests, then code

**Why:** This creates traceability from business requirements to running code. When auditors ask "how do you know feature X works?", you can point to AC-123 → BDD scenario → passing test.

### 2. Policy-as-Code Governance

**Philosophy:** Governance rules should be automated, not manual checklists.

- OPA/Rego policies encode organizational rules
- Examples: "ACs must have tests", "flags must have owners", "PII must have retention"
- Policies are tested like code (fixtures + conftest)
- CI enforces policies automatically

**Why:** Manual governance doesn't scale. Policies-as-code means rules are consistent, versioned, and impossible to skip.

### 3. LLM-Native Development

**Philosophy:** LLMs are powerful coding assistants when given proper context.

- Context bundler generates focused, bounded input for LLMs
- Tasks define relevant globs + size limits
- Output validates against specs and policies
- LLMs augment developers, don't replace them

**Why:** Generic "paste random code" LLM usage creates technical debt. Curated context + validation creates maintainable code.

---

## Crate Structure

The template uses a workspace with clear separation of concerns:

```
crates/
├── app-http/       → HTTP adapter (ports)
├── core/           → Domain logic (hexagon center)
├── model/          → Domain entities
├── telemetry/      → Cross-cutting: observability
├── acceptance/     → BDD tests (outside-in)
└── xtask/          → Dev/CI tooling
```

### app-http: HTTP Adapter

**Role:** Translate HTTP requests into domain operations.

**Responsibilities:**
- Route definitions (Axum)
- Request/response DTOs
- HTTP-specific error handling
- Calling core domain functions

**Anti-pattern:** Business logic in handlers. Keep handlers thin - they translate and delegate.

**Example:**
```rust
async fn create_refund(Json(req): Json<CreateRefundRequest>) -> Result<...> {
    // ✓ Validate HTTP inputs
    if req.amount_cents == 0 {
        return Err(AppError::BadRequest(...));
    }

    // ✓ Call domain
    let refund = core::refunds::create(req.order_id, req.amount_cents)?;

    // ✓ Translate to HTTP response
    Ok(Json(CreateRefundResponse::from(refund)))
}
```

### core: Domain Logic

**Role:** Encode business rules, independent of HTTP/DB/etc.

**Responsibilities:**
- Business validations
- State transitions
- Domain events
- Pure functions (no I/O unless via traits)

**Anti-pattern:** Depending on app-http, databases, or any infrastructure.

**Example:**
```rust
pub fn refund_ok() -> bool {
    // Business rule: refunds always allowed for now
    true
}

pub fn create_refund(order_id: &str, amount: u64) -> Result<Refund> {
    // Business validations
    if amount == 0 {
        return Err(DomainError::InvalidAmount);
    }
    // Create domain entity
    Ok(Refund::new(order_id, amount))
}
```

### model: Domain Entities

**Role:** Define domain concepts as Rust types.

**Responsibilities:**
- Value objects (OrderId, Amount)
- Entities (Refund, Order)
- Domain enums (RefundStatus)
- Serde derives for serialization

**Anti-pattern:** Putting business logic here. Entities are data, core/ has logic.

### telemetry: Observability

**Role:** Centralize tracing/logging setup.

**Responsibilities:**
- Initialize `tracing_subscriber`
- Configure log filtering (RUST_LOG)
- Provide test helpers

**Why separate:** Every crate needs observability. Centralizing avoids duplication and ensures consistency.

### acceptance: BDD Tests

**Role:** Outside-in behavioral tests.

**Responsibilities:**
- Cucumber scenarios in Gherkin
- Step definitions calling app-http endpoints
- JUnit XML output for CI

**Why BDD:** Scenarios are readable by non-developers, map directly to ACs, and test the whole system.

### xtask: Development Tooling

**Role:** Single Rust-native CLI for all dev/CI operations.

**Responsibilities:**
- `check`: fmt, clippy, tests
- `bdd`: Run acceptance tests
- `bundle`: Generate LLM context
- `selftest`: Comprehensive validation

**Why Rust:** Cross-platform, type-safe, integrates with Cargo ecosystem. No bash script fragmentation.

---

## Platform Cell Surfaces

The template exposes three governed faces that share the same runtime and specs:

- **CLI (`cargo xtask`)**: kernel interface for doctor, dev-up, check, selftest, suggest-next, policy-test, release-bundle, and tasks commands.
- **HTTP (`/platform/*`)**: introspection and control-plane APIs for status, graph, tasks, agent hints, docs index, and flows.
- **Web UI (`/ui`)**: thin console backed by the same runtime as `/platform/*`, surfacing status, graph, flows, tasks, and key documentation links.

---

## Hexagonal Architecture

The template follows hexagonal (ports & adapters) architecture:

```
       ┌──────────────────────┐
       │    app-http (Port)   │
       │  HTTP requests/resp  │
       └──────────┬───────────┘
                  │
                  ▼
       ┌──────────────────────┐
       │   core (Hexagon)     │
       │  Business Logic      │
       │  Domain Rules        │
       └──────────────────────┘
```

### Key Pattern: Dependency Inversion

**Rule:** Dependencies point INWARD to the domain.

✓ **Correct:**
```rust
// app-http/src/main.rs
use core::refunds;

async fn create_refund(...) {
    let refund = core::refunds::create(...)?;  // ✓ Adapter calls domain
    Ok(Json(refund))
}
```

✗ **Wrong:**
```rust
// core/src/refunds.rs
use app_http::handlers;  // ✗ Domain depends on adapter!

pub fn create() {
    handlers::send_response(...);  // ✗ Never do this!
}
```

### Why Hexagonal?

1. **Testability:** Core logic tested without HTTP/DB
2. **Flexibility:** Swap HTTP for gRPC without changing core
3. **Clarity:** Business rules isolated from infrastructure
4. **Maintainability:** Changes to adapters don't break domain

### Ports & Adapters in Practice

**Ports** (interfaces the hexagon exposes):
- Function signatures in `core/`
- Traits for external dependencies (future: Database, Queue traits)

**Adapters** (implementations for specific technologies):
- `app-http`: HTTP adapter using Axum
- Future: `app-grpc`, `infra-postgres`, etc.

**Adapters for persistence and external systems:**
```
          (HTTP)       (gRPC)       (Jobs)
        app-http     app-grpc     app-worker
            \           |            /
             \          |           /
              \         |          /
               +-------------------+
               |       core        |
               |   domain rules    |
               +-------------------+
              /         |          \
             /          |           \
            /           |            \
    adapters-db   adapters-queue  adapters-external
```
Persistence, queues, and external APIs live in adapter crates; core only sees traits.

---

## Governance Kernel & Selftest

- `crates/spec-runtime` loads the ledger, builds the REQ/AC/test/task graph, and enforces invariants.
- `cargo xtask selftest` is the canonical gate: fmt, clippy, unit/integration/BDD, AC/test mapping, policy tests, graph invariants, and regeneration of feature status + bundles.
- `/platform/status` and `/ui` surface this kernel state so humans and agents see the same truth.

---

## Governance Model

The template encodes three governance layers:

### Layer 1: Specification Ledger

**File:** `specs/spec_ledger.yaml`

**Structure:**
```yaml
stories:
  - id: US-001
    title: "Refund Processing"
    requirements:
      - id: REQ-001
        acceptance_criteria:
          - id: AC-123
            text: "Customer can create refund"
            tests:
              - type: bdd
                tag: "@AC-123"
```

**Governance:** Every AC must have tests (enforced by `policy/ledger.rego`).

### Layer 2: Policy-as-Code

**Files:** `policy/*.rego`

**Examples:**
- `ledger.rego`: ACs have tests
- `features.rego`: Features reference valid ACs
- `flags.rego`: Flags have owners + valid rollouts
- `privacy.rego`: PII fields have owner + retention

**Governance:** Policies run in CI. Breaking a policy fails the build.

### Layer 3: CI Enforcement

**Mechanism:** GitHub Actions with branch protection.

**Profiles:**
- **Minimal:** Basic checks (fmt, clippy, tests)
- **Standard:** + BDD, AC status, some policies
- **Strict:** + All policies, no warnings, no skipped checks

**Governance:** Branch protection prevents merging if required checks fail.

### Why Three Layers?

- **Ledger:** Human-readable source of truth
- **Policies:** Machine-enforced rules
- **CI:** Automatic gating

This creates "governance by default" - doing the wrong thing is harder than doing the right thing.

---

## Observability Strategy

The template bakes in observability from day 0:

### Tracing with `tracing` crate

**Setup:** `telemetry::init()` in `app-http/main.rs`

**Filtering:** RUST_LOG environment variable
```bash
RUST_LOG=info cargo run              # Default
RUST_LOG=debug cargo run             # Verbose
RUST_LOG=app_http=trace cargo run    # Specific crate
```

### Structured Logging

**Pattern:** Use `#[instrument]` on handlers
```rust
#[instrument(skip(payload))]
async fn create_refund(Json(payload): Json<CreateRefundRequest>) -> Result<...> {
    info!(order_id = %payload.order_id, amount = payload.amount_cents, "Creating refund");
    // ...
}
```

**Output:**
```
INFO app_http::create_refund{order_id="ORD-123" amount=5000}: Creating refund
```

### Request Tracing

**Mechanism:** `TraceLayer` from `tower-http`

```rust
let app = Router::new()
    .route("/refunds", post(create_refund))
    .layer(TraceLayer::new_for_http());  // ← Adds span per request
```

**Benefit:** Every request gets a span, making distributed tracing possible.

### Why Structured Logging?

- **Queryable:** `order_id="ORD-123"` is machine-parseable
- **Contextual:** Spans provide nesting (request → handler → domain call)
- **Standardized:** `tracing` is Rust ecosystem standard

---

## Environment Model

- Tier-1 (canonical): Nix dev shell on Linux/macOS/WSL2; selftest and kernel ACs are guaranteed here.
- Tier-1b: Non-Nix Linux/macOS hosts use the same xtask commands with slightly less enforcement on toolchain versions.
- Tier-2: Native Windows uses the low-resource path (check, test-changed, selftest with skips) for constrained contributors.
- Docker: `docker-compose.yaml` provides Postgres + Jaeger as convenience, covered by a non-kernel AC instead of a hard gate.

---

## Security & Access

- Local/dev: `/platform/*` and `/ui` are open by default for fast iteration.
- Prod hook: `PLATFORM_AUTH_MODE` enables a basic auth guard on write endpoints; adopters can swap in OAuth/OIDC/mTLS while keeping the same contract.
- Log hygiene: `/platform/status`, `/ui`, and selftest output must redact secret values; secrets belong in config, not logs.

---

## Why These Choices

This template is intended to be cloned as an IDP cell: `service_metadata.yaml` describes the cell, `/platform/*` exposes its runtime state, and `cargo xtask release-bundle` generates evidence per version so fleet tooling can consume it without bespoke scraping.

### Why Rust-native tooling (xtask)?

**Alternatives:** Makefiles, bash scripts, just, cargo-make

**Choice:** `xtask` - Rust binary in workspace

**Reasoning:**
- ✅ Cross-platform (Windows, Mac, Linux)
- ✅ Type-safe (no string quoting hell)
- ✅ Single language (Rust developers already know it)
- ✅ Cargo integration (runs via `cargo run -p xtask`)
- ✅ Testable (xtask itself can have tests)

**Trade-off:** More verbose than bash, but more maintainable.

### Why BDD with Cucumber?

**Alternatives:** Property tests, unit tests only, integration tests

**Choice:** Cucumber-rs for acceptance tests

**Reasoning:**
- ✅ Readable by non-developers (Gherkin syntax)
- ✅ Maps directly to ACs (via `@AC-####` tags)
- ✅ Tests from user perspective (not implementation)
- ✅ JUnit XML output (CI integration)

**Trade-off:** Slower than unit tests, but tests whole system behavior.

### Why OPA/Rego for policies?

**Alternatives:** Custom validators, linters, manual review

**Choice:** OPA (Open Policy Agent) with Rego language

**Reasoning:**
- ✅ Domain-specific language for policies
- ✅ Declarative (describe rules, not implementation)
- ✅ Industry standard (Kubernetes, Terraform use it)
- ✅ Testable (conftest validates against fixtures)

**Trade-off:** Learning curve for Rego, but policies are clearer.

### Why Axum for HTTP?

**Alternatives:** Actix-web, Rocket, warp

**Choice:** Axum

**Reasoning:**
- ✅ Built on tokio (ecosystem standard)
- ✅ Type-safe extractors
- ✅ Tower middleware (mature ecosystem)
- ✅ Async/await native
- ✅ Good performance

**Trade-off:** Newer than Actix, but cleaner API.

### Why Nix for dev environment?

**Alternatives:** Docker, asdf, manual installation

**Choice:** Nix flake + DevContainer wrapper

**Reasoning:**
- ✅ Reproducible (same versions everywhere)
- ✅ Declarative (`flake.nix` is version-controlled)
- ✅ Fast (no containers for local dev)
- ✅ Composable (overlays, overrides)

**Trade-off:** Nix learning curve, but pays off for teams.

---

## Summary

The template's architecture is designed for:

1. **Traceability:** Specs → ACs → Tests → Code
2. **Governance:** Policies enforce rules automatically
3. **Maintainability:** Hexagonal architecture isolates concerns
4. **Observability:** Tracing baked in from day 0
5. **Ergonomics:** Rust-native tooling, clear patterns

These aren't arbitrary choices - they solve real problems teams face:
- "How do we know feature X works?" → AC-first with BDD
- "How do we enforce standards?" → Policy-as-code
- "How do we change HTTP framework?" → Hexagonal architecture
- "How do we debug production?" → Structured tracing
- "How do we onboard developers?" → Nix + xtask

The architecture is optimized for **governed, observable, maintainable services** - not just "get something running quickly."
