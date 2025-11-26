# Template Contracts: What You Must Keep vs. What You Can Change

This document defines the **template contracts** – the core APIs, behaviors, and structures that MUST NOT be removed or broken – versus the **customization surface** where you're free to extend, modify, or replace.

## Why This Matters

The Rust Template is designed to provide:
1. **A stable foundation** – contracts that policies, tools, and automation depend on
2. **Flexibility** – clear extension points for domain-specific features
3. **Governance** – automated checks (via Rego policies) that prevent accidental contract violations

**Rule of thumb:** If a policy checks for it, it's a contract. If it's not checked by policy, you're free to customize it.

---

## 🔒 Template Core Contracts (DO NOT REMOVE)

These are the **non-negotiable** elements that must exist in every service built from this template. Removing or breaking these will cause policy failures, broken tooling, or incompatibility with the template ecosystem.

### 1. HTTP Endpoints

**Contract:**
- `GET /health` → Returns 200 OK with `{"status": "ok"}`
- `GET /version` → Returns 200 OK with `{"version": "..."}`
- `POST /api/echo` → Returns 400 with standard error envelope (used for error testing)

**Why:**
- Health checks are required for Kubernetes liveness/readiness probes
- Version endpoint is needed for deployment verification and diagnostics
- Echo endpoint demonstrates the error envelope contract (AC-TPL-003)

**Checked by:** `policy/template_core.rego`

**How to maintain:**
- Keep these routes in `crates/app-http/src/routes/*.rs`
- Ensure acceptance tests for AC-TPL-001, AC-TPL-002, AC-TPL-003 pass
- Do NOT remove or rename these endpoints

### 2. Error Response Envelope

**Contract:**
```json
{
  "error": {
    "code": "SOME_ERROR_CODE",
    "message": "Human-readable error message",
    "details": { /* optional context */ }
  }
}
```

**Why:**
- Provides consistent error handling across all API endpoints
- Enables structured error logging and monitoring
- Client libraries can parse errors reliably

**Checked by:** `policy/template_core.rego` (AC-TPL-003)

**How to maintain:**
- Use `ErrorResponse` type from `crates/model/src/lib.rs`
- All HTTP error responses must use this envelope
- Add new error codes as needed for domain errors

### 3. Request ID Propagation

**Contract:**
- Accept `X-Request-ID` header from clients
- Generate a unique request ID if not provided
- Return the same `X-Request-ID` in responses
- Log the request ID with all log entries for that request

**Why:**
- Enables distributed tracing across services
- Critical for debugging production issues
- Standard practice for observability

**Checked by:** `policy/template_core.rego` (AC-TPL-004)

**How to maintain:**
- Middleware in `crates/app-http/src/middleware/request_id.rs` must remain
- All logging should include the request ID
- Response must echo the request ID header

### 4. Acceptance Criteria (ACs)

**Contract:**
- Template-core ACs must exist in `specs/spec_ledger.yaml`:
  - `AC-TPL-001`: Health endpoint
  - `AC-TPL-002`: Version endpoint
  - `AC-TPL-003`: Error envelope
  - `AC-TPL-004`: Request ID propagation

**Why:**
- These define the behavioral contracts of the template
- BDD tests verify these contracts
- `cargo run -p xtask -- ac-status` checks coverage

**Checked by:** `policy/ledger.rego`, `policy/template_core.rego`

**How to maintain:**
- Do NOT remove these ACs from `specs/spec_ledger.yaml`
- Keep corresponding `.feature` files in `features/template-core/`
- Ensure all template-core ACs have `status: green`

### 5. Rego Policies

**Contract:**
- Policy files must exist and pass:
  - `policy/template_core.rego` – checks template contracts
  - `policy/ledger.rego` – validates spec_ledger.yaml structure
  - `policy/features.rego` – validates .feature files
  - `policy/k8s.rego` – validates K8s manifests
  - `policy/llm.rego` – validates LLM contextpack bundles

**Why:**
- These are the governance layer enforcing contracts
- They prevent accidental regressions
- They document what's actually required vs. optional

**Checked by:** `cargo run -p xtask -- policy-test`

**How to maintain:**
- Do NOT delete these policy files
- Extend them for new contracts if needed
- Run `cargo run -p xtask -- policy-test` before committing

### 6. xtask Control Plane

**Contract:**
- Commands must exist:
  - `cargo run -p xtask -- check` – runs all checks
  - `cargo run -p xtask -- bdd` – runs BDD tests
  - `cargo run -p xtask -- ac-status` – shows AC coverage
  - `cargo run -p xtask -- bundle` – generates LLM context packs
  - `cargo run -p xtask -- policy-test` – validates policies
  - `cargo run -p xtask -- deploy --env {dev,staging,prod}` – deployment instructions
  - `cargo run -p xtask -- selftest` – full template validation

**Why:**
- Provides consistent DevEx across all services
- CI/CD pipelines depend on these commands
- LLM tools (like Claude Code) use these for automation

**Checked by:** `cargo run -p xtask -- selftest`

**How to maintain:**
- Keep `crates/xtask/` working
- Do NOT remove or rename these commands
- Extend with new commands as needed

### 7. Multi-Environment K8s Structure

**Contract:**
- K8s manifests must exist:
  - `infra/k8s/dev/` – base manifests with kustomization.yaml
  - `infra/k8s/staging/` – overlay with 2+ replicas
  - `infra/k8s/prod/` – overlay with 3+ replicas, HA, zero-downtime config

**Why:**
- Enforces production-readiness from day one
- Policies check environment-specific constraints
- Deployment automation depends on this structure

**Checked by:** `policy/k8s.rego`

**How to maintain:**
- Keep the dev/staging/prod directory structure
- Ensure prod has minimum 3 replicas, anti-affinity, probes, team/cost-center labels
- Use Kustomize for environment-specific overrides

---

## ✏️ Customization Surface (SAFE TO CHANGE)

These are the **extension points** where you're expected to customize for your domain.

### 1. Domain Model (`crates/model/`)

**You can:**
- Add new domain types (structs, enums)
- Add new error codes to `ErrorCode` enum
- Create domain-specific DTOs and value objects

**You must:**
- Keep `ErrorResponse` type and its structure
- Keep `Health` and `Version` types for template endpoints

**Example:**
```rust
// SAFE: Adding new domain types
pub struct Task { ... }
pub enum TaskStatus { ... }

// SAFE: Adding new error codes
pub enum ErrorCode {
    #[serde(rename = "VALIDATION_ERROR")]
    ValidationError,
    // Your domain-specific codes:
    #[serde(rename = "TASK_NOT_FOUND")]
    TaskNotFound,
}
```

### 2. Core Business Logic (`crates/core/`)

**You can:**
- Add new domain services and use cases
- Implement new business logic
- Add new traits and implementations

**You must:**
- Keep health and version handlers (or equivalent)
- Maintain the error handling pattern using `ErrorResponse`

**Example:**
```rust
// SAFE: Adding new use cases
pub mod create_task;
pub mod list_tasks;
pub mod update_task;
```

### 3. HTTP Routes (`crates/app-http/src/routes/`)

**You can:**
- Add new API endpoints for your domain
- Create new route modules
- Add custom middleware (auth, rate limiting, etc.)

**You must:**
- Keep `/health`, `/version`, `/api/echo` endpoints
- Use `ErrorResponse` for all error responses
- Maintain request ID middleware

**Example:**
```rust
// SAFE: Adding new routes
pub mod tasks;  // New module for task endpoints

// In main.rs:
.route("/api/tasks", post(routes::tasks::create_task))
.route("/api/tasks/:id", get(routes::tasks::get_task))
```

### 4. Acceptance Criteria and Features

**You can:**
- Add new ACs to `specs/spec_ledger.yaml` for your features
- Create new `.feature` files in `features/` (outside `template-core/`)
- Write new step definitions for your domain scenarios

**You must:**
- Keep template-core ACs (AC-TPL-001/002/003/004)
- Follow the AC structure defined by `policy/ledger.rego`
- Ensure new features have valid Gherkin syntax (checked by `policy/features.rego`)

**Example:**
```yaml
# SAFE: Adding domain-specific ACs
- id: AC-TASK-001
  story_id: S-TASK-001
  description: "User can create a task with title and description"
  status: green
  feature_file: features/tasks/create-task.feature
  scenarios:
    - Create task with valid input
    - Reject task with empty title
```

### 5. K8s Manifests (Environment-Specific)

**You can:**
- Customize resource limits, replicas (within policy bounds)
- Add ConfigMaps, Secrets, Services, Ingress rules
- Add new environment variables
- Customize namespaces (e.g., `your-service-prod`)

**You must:**
- Maintain dev/staging/prod overlay structure
- Meet minimum replica counts (staging: 2, prod: 3)
- Include liveness/readiness probes in prod
- Have team and cost-center labels in prod
- Use zero-downtime deployments in prod (maxUnavailable: 0)

**Example:**
```yaml
# SAFE: Adding domain-specific config
configMapGenerator:
  - name: app-config
    literals:
      - ENVIRONMENT=staging
      - DATABASE_URL=postgres://...
      - TASK_QUEUE_URL=redis://...
```

### 6. Dependencies (`Cargo.toml`)

**You can:**
- Add new crates for your domain (e.g., `sqlx`, `redis`, `kafka`)
- Update versions of existing dependencies
- Add workspace members for new crates

**You must:**
- Keep core dependencies: `axum`, `tower`, `serde`, `tokio`
- Keep testing dependencies: `cucumber`, `httpc-test`

**Example:**
```toml
# SAFE: Adding domain-specific dependencies
[dependencies]
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio"] }
redis = "0.24"
```

### 7. LLM Contextpack Bundles

**You can:**
- Add new bundle configurations in `.llm/config.yaml`
- Customize which files are included in bundles
- Adjust max_bytes limits per bundle

**You must:**
- Keep the `implement_ac` and `add_feature` bundles (or equivalent)
- Ensure bundles meet structure and size policies (`policy/llm.rego`)

**Example:**
```yaml
# SAFE: Adding domain-specific bundles
- name: debug_task_queue
  description: Context for debugging task queue issues
  max_bytes: 100000
  include:
    - specs/spec_ledger.yaml
    - crates/core/src/tasks/**
    - features/tasks/**
```

---

## 🔬 How Contracts Are Enforced

### Automated Checks

1. **Policy Tests** – `cargo run -p xtask -- policy-test`
   - Runs conftest on all policies
   - Fails if template-core contracts are violated

2. **BDD Tests** – `cargo run -p xtask -- bdd`
   - Runs Cucumber tests for all `.feature` files
   - Ensures template-core ACs pass

3. **Acceptance Tests** – `cargo run -p xtask -- ac-status`
   - Checks AC coverage and status
   - Warns if template-core ACs are missing or red

4. **Self-Test** – `cargo run -p xtask -- selftest`
   - Full validation: checks, BDD, policies, bundler
   - CI runs this to ensure template integrity

### CI Enforcement

GitHub Actions runs:
- Template Self-Test (includes all checks above)
- Lints (clippy, rustfmt)
- MSRV check (minimum supported Rust version)
- Nix Flake Check

If any fail, PRs cannot merge (when branch protection is enabled).

---

## 📚 Reference

- **Template Foundation vs. Examples**: `docs/explanation/template-foundation-vs-examples.md`
- **Policy Reference**: `policy/*.rego` files
- **AC Ledger**: `specs/spec_ledger.yaml`
- **BDD Features**: `features/template-core/*.feature`

---

## ❓ FAQ

**Q: Can I rename the `/health` endpoint to `/healthz`?**
A: No. Policies and K8s manifests reference `/health`. Renaming will break contracts.

**Q: Can I remove the `ErrorResponse` type and use my own error format?**
A: No. This is checked by policy (AC-TPL-003). Extend it instead.

**Q: Can I delete `features/template-core/` if I'm not using BDD?**
A: No. Template-core scenarios must exist and pass. You can skip writing NEW features, but don't delete template-core.

**Q: Can I add a new xtask command like `cargo run -p xtask -- migrate`?**
A: Yes! Adding new commands is safe. Just don't remove existing ones.

**Q: Can I remove the `prod` K8s overlay if I only deploy to dev/staging?**
A: No. The template enforces a multi-env structure. Keep the directory structure, even if you don't currently use prod.

**Q: Can I change from Kustomize to Helm?**
A: Yes, but you'll need to update policies and deploy command to match. This is advanced.

**Q: What if I find a contract too restrictive?**
A: Open an issue in the template repo. We can discuss relaxing it in a future version. Don't silently break it.

---

## Summary

**Contracts (must keep):**
- `/health`, `/version`, `/api/echo` endpoints
- `ErrorResponse` envelope
- Request ID propagation
- Template-core ACs (AC-TPL-001/002/003/004)
- Rego policies
- xtask commands
- Multi-env K8s structure

**Customization (safe to change):**
- Domain model, core logic, HTTP routes
- New ACs and features
- K8s config (within policy bounds)
- Dependencies and bundles

**When in doubt:** Run `cargo run -p xtask -- selftest`. If it passes, you haven't broken contracts.
