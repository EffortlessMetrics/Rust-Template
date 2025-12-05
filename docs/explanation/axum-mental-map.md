---
id: EXPLAIN-TPL-AXUM-MAP-001
title: Axum Mental Map for the Rust-as-Spec Platform Cell
doc_type: explanation
status: published
audience: developers
tags: [axum, http, architecture, onboarding]
stories: [US-TPL-001]
requirements: [REQ-PLT-ONBOARDING]
acs: []
adrs: [ADR-0001]
last_updated: 2025-12-05
---

# Axum Mental Map

This repo is a "Rust-as-Spec platform cell": specs, tests, docs, policies, and infra
all agree, and `cargo xtask selftest` + `/platform/*` can prove it.

Under the governance layer there is still a **plain Axum HTTP service**.

This guide is for people who already know Axum and just want to know:

> *"Where do I put my routes and handlers?"*
> *"What's the equivalent of `main.rs`, `Router`, and `State` here?"*

---

## 1. The core pipeline (Axum view)

At a high level, the HTTP side looks like:

```text
crates/app-http (Axum adapter)
  -> business-core (use-cases, domain)
     -> spec-runtime (load & query spec_ledger / platform state)
```

The governance layer (spec ledger, BDD, policies, etc.) sits *behind* this, but you
can treat `crates/app-http` as "the Axum app".

---

## 2. Mapping Axum concepts to this repo

| Axum concept                     | Where it lives here                                          |
| -------------------------------- | ------------------------------------------------------------ |
| `main`, Server startup           | `crates/app-http/src/main.rs`                                |
| `Router`, route tree             | `crates/app-http/src/lib.rs` → `build_router()` function     |
| Handlers (`async fn handler`)    | `crates/app-http/src/lib.rs` (simple) or `platform/*.rs`     |
| Shared state (`State<T>`)        | `AppState` struct in `crates/app-http/src/lib.rs`            |
| JSON types (`derive(Serialize)`) | `crates/model/src/**/*.rs`                                   |
| Error types / `IntoResponse`     | `crates/app-http/src/errors.rs` → `AppError`                 |
| Middleware / layers              | `crates/app-http/src/middleware/*.rs` + router setup         |
| OpenAPI / schema                 | `specs/openapi/openapi.yaml` + `specs/platform_schema.yaml`  |
| Governance / spec graph          | `crates/spec-runtime/src/**/*.rs` + `specs/spec_ledger.yaml` |

> **Quick navigation:** Search for `Router::new()` or `.route("/` in `crates/app-http/src/lib.rs`
> to find the main HTTP entrypoint.

---

## 3. What's "just Axum" vs "governed"

Think of two layers:

### 3.1 Plain Axum layer (you can work here immediately)

You can:

* Add a new `GET /foo` route,
* Define a `struct FooResponse` with `#[derive(Serialize)]`,
* Call into a use-case in `business-core`,

**without touching the spec ledger**.

This is fine for:

* purely internal debugging endpoints,
* experimental admin/debug tools,
* early exploration before you know the right contract.

### 3.2 Governed layer (when it matters)

When the behaviour becomes part of the *platform contract* (e.g. something you
want Backstage or agents to rely on), you move "up" a layer:

* Add a **REQ** and **AC** to `specs/spec_ledger.yaml`,
* Add or update a **BDD scenario** in `specs/features/*.feature`,
* Add/adjust the **OpenAPI schema** in `specs/openapi/openapi.yaml`,
* Ensure the handler uses the shared types from `crates/model`,
* Run `cargo xtask ac-status`, `selftest`, and `idp-check`.

You don't have to understand all of that on day 1. For a junior dev:

> **Step 1:** get your handler working.
> **Step 2:** pair with someone to "govern" it (add REQ/AC, tests, schema).

---

## 4. Typical tasks and where to go

### 4.1 "I want to add a simple GET endpoint"

See [How to add an HTTP endpoint](../how-to/add-http-endpoint.md).

High level:

1. Add a handler function in `crates/app-http/src/lib.rs` (or a new module).
2. Wire it into `build_router()` with `.route("/path", get(handler))`.
3. Add response/request DTOs in the same file or in `crates/model`.
4. Optionally add OpenAPI schema and BDD scenario later.

### 4.2 "I want to expose something on `/platform/*`"

These routes are **part of the platform contract**.

* Look at existing handlers in `crates/app-http/src/platform/*.rs`
  (e.g. `idp.rs`, `status.rs`, `friction.rs`).
* Make sure you:
  * Use the existing types from `crates/model` (or extend them there).
  * Update `specs/openapi/openapi.yaml` and `specs/platform_schema.yaml`.
  * Add / update REQs and ACs in `specs/spec_ledger.yaml`.
  * Extend BDD features under `specs/features/platform_*.feature`.

Then:

```bash
cargo xtask ac-status
cargo xtask selftest
cargo xtask idp-check  # validates OpenAPI + Backstage plugin + TS types
```

### 4.3 "I want to add middleware or global behaviour"

* Look at `crates/app-http/src/middleware/` for existing middleware:
  * `request_id.rs` – adds request ID to all requests
  * `platform_auth.rs` – guards `/platform/*` endpoints
* For things like auth, logging, tracing, CORS:
  * Implement them as Axum layers/middleware,
  * Wire into `build_router()` with `.layer(...)`,
  * Optionally add REQs/ACs if they are part of the governance contract.

### 4.4 "I want to understand the existing routes"

Quick way to see all routes:

```bash
grep -n '\.route(' crates/app-http/src/lib.rs
```

Or start the server and explore:

```bash
cargo run -p app-http
curl http://localhost:8080/health
curl http://localhost:8080/version
curl http://localhost:8080/platform/status
```

---

## 5. Key files at a glance

```text
crates/app-http/
├── src/
│   ├── main.rs          # Entry point, server startup
│   ├── lib.rs           # Router, AppState, core handlers, DTOs
│   ├── errors.rs        # AppError type with IntoResponse
│   ├── metrics.rs       # Prometheus metrics middleware
│   ├── middleware/
│   │   ├── mod.rs
│   │   ├── request_id.rs
│   │   └── platform_auth.rs
│   └── platform/
│       ├── mod.rs       # Platform router
│       ├── status.rs    # GET /platform/status
│       ├── idp.rs       # GET /platform/idp/snapshot
│       ├── friction.rs  # GET /platform/friction
│       ├── questions.rs # GET /platform/questions
│       ├── forks.rs     # GET /platform/forks
│       └── ui.rs        # Platform UI routes
```

---

## 6. If you only remember three things

1. **Axum app lives in `crates/app-http`** – treat it like a normal Axum service.
2. **Router is in `lib.rs` → `build_router()`** – that's where you add routes.
3. **When something becomes "real API"** (used by other teams or tools), that's
   your cue to:
   * Add REQ/AC to `specs/spec_ledger.yaml`,
   * Add a BDD scenario to `specs/features/*.feature`,
   * Update OpenAPI schema,
   * Run `selftest` and `idp-check`.

This keeps junior devs productive ("I can add a handler") while the kernel keeps
its guarantees.

---

## Related Docs

* [How to add an HTTP endpoint](../how-to/add-http-endpoint.md) – step-by-step guide
* [Architecture overview](./architecture.md) – hexagonal architecture explained
* [Template Contracts](./TEMPLATE-CONTRACTS.md) – what the kernel guarantees
