<!-- doclint:disable orphan-version -->
# Controls as Code

**Version**: v3.3.6
**Last Updated**: 2025-01-18

This template implements **controls-as-code**: governance and compliance requirements expressed as executable policies, validated automatically, and enforced in CI.

---

## What Are "Controls"?

**Controls** are requirements that protect against risk:

- **Security**: "Containers must not run as root"
- **Privacy**: "Logs must not contain PII"
- **Operational**: "All deployments must have resource limits"
- **Architectural**: "Business logic must not depend on HTTP framework"

Traditional approaches:

1. **Manual checklists**: error-prone, inconsistent
2. **Runbook documentation**: ignored until post-incident
3. **Code review**: tribal knowledge, not scalable
4. **Penetration testing**: finds issues too late

**Controls-as-code** treats these requirements as:

- **Executable code** (policies in Rego)
- **Testable** (with fixtures and conftest)
- **Versioned** (in git, like any code)
- **Enforced** (in CI, blocks merge if violated)

---

## Control Categories in This Template

This template ships with four control categories:

### 1. Infrastructure Security (`policy/k8s_standards.rego`)

**Purpose**: Ensure Kubernetes manifests follow security best practices.

**Rules enforced:**

- Containers must run as non-root (`securityContext.runAsNonRoot: true`)
- No `hostPath` volumes (prevents container escape)
- Resource requests and limits defined (prevents resource exhaustion)
- Liveness and readiness probes present (ensures observability)
- No privileged containers (prevents elevated access)

**Example violation:**

```yaml
# infra/k8s/deployment.yaml
spec:
  template:
    spec:
      containers:
        - name: app
          image: my-app:latest
          # FAIL: missing securityContext.runAsNonRoot
```

**How it's enforced:**

```bash
# Locally (via Nix)
nix develop -c conftest test -p policy/k8s_standards.rego infra/k8s/

# In CI (automated)
cargo run -p xtask -- policy-test
cargo run -p xtask -- selftest  # includes policy tests in step 5
```

**Rationale**: Security hardening at build time (shift-left), before deployment.

---

### 2. Privacy Compliance (`policy/privacy_compliance.rego`)

**Purpose**: Prevent accidental exposure of Personally Identifiable Information (PII).

**Rules enforced:**

- No email addresses in log messages
- No Social Security Numbers (SSN) in code or config
- No credit card numbers in code or config
- Sensitive data encrypted at rest (checks k8s Secrets)

**Example violation:**

```rust
// FAIL: logging user email directly
tracing::info!("User logged in: {}", user.email);
```

**Correct approach:**

```rust
// OK: log user ID instead of email
tracing::info!("User logged in: user_id={}", user.id);
```

**How it's enforced:**

```bash
# Locally
nix develop -c conftest test -p policy/privacy_compliance.rego --data specs/

# In CI
cargo run -p xtask -- selftest  # step 5
```

**Rationale**: Compliance with GDPR, CCPA, and other privacy regulations. Prevents data leaks via logs or error messages.

---

### 3. Feature Flags Governance (`policy/flags_rollout.rego`, `policy/flags_warn.rego`)

**Purpose**: Ensure feature flags are managed safely and intentionally.

**Rules enforced:**

- No experimental flags enabled in production (`enabled: true` → FAIL)
- Flags must have an owner (team or individual)
- Flags must have an expiry date (prevents zombie flags)
- Deprecated flags trigger warnings (cleanup reminder)

**Example violation:**

```yaml
# flags/experimental.yaml
flags:
  - id: new-algorithm
    enabled: true  # FAIL: experimental flags must be off in prod
    owner: null    # FAIL: must have an owner
```

**Correct approach:**

```yaml
flags:
  - id: new-algorithm
    enabled: false  # OK: off by default
    owner: "team-platform"
    expiry: "2025-06-01"
    rollout_percentage: 0
```

**How it's enforced:**

```bash
# Locally
nix develop -c conftest test -p policy/flags_rollout.rego flags/

# In CI
cargo run -p xtask -- selftest
```

**Rationale**: Prevents accidental full rollout of untested features. Ensures flags are owned and cleaned up.

---

### 4. LLM Governance (`policy/llm.rego`)

**Purpose**: Validate LLM context bundles for safety and compliance.

**Rules enforced:**

- Bundle size must not exceed `max_bytes` limit (prevents token overflow)
- Bundle metadata must include template version (traceability)
- No sensitive files in bundles (`.env`, `credentials.json`, etc.)
- Bundle structure matches expected schema

**Example violation:**

```
.llm/bundle/implement_ac.md  (300KB)  # FAIL: exceeds 250KB limit
```

**How it's enforced:**

```bash
# Locally (after generating a bundle)
nix develop -c conftest test -p policy/llm.rego .llm/bundle/

# In CI
cargo run -p xtask -- bundle implement_ac  # includes validation
cargo run -p xtask -- selftest  # step 4 tests bundler
```

**Rationale**: Ensures LLM context doesn't leak secrets, stays within token limits, and remains traceable.

---

## How Controls Are Tested

Each policy includes **test fixtures** under `policy/testdata/`:

```
policy/testdata/
├── k8s/
│   ├── pass/
│   │   └── valid-deployment.yaml
│   └── fail/
│       └── missing-security-context.yaml
├── privacy/
│   ├── pass/
│   │   └── safe-logging.rs
│   └── fail/
│       └── logs-email.rs
└── llm/
    ├── pass/
    │   └── valid-bundle.md
    └── fail/
        └── oversized-bundle.md
```

**Testing policies:**

```bash
# Validate that policies work as expected
nix develop -c conftest verify -p policy/

# Output:
# PASS - policy/k8s_standards.rego - 8 tests
# PASS - policy/privacy_compliance.rego - 6 tests
```

This ensures:

- Policies correctly detect violations (fail/ fixtures)
- Policies don't false-positive on valid code (pass/ fixtures)

---

## Adding a New Control

If you need to enforce a new requirement:

### 1. Write the Rego policy

Create `policy/<domain>.rego`:

```rego
# policy/database_security.rego
package database_security

# Deny if database password is hardcoded
deny[msg] {
    input.kind == "Secret"
    input.data.password == "admin123"
    msg := "Database password must not be hardcoded"
}

# Require TLS for database connections
deny[msg] {
    input.kind == "ConfigMap"
    contains(input.data.DATABASE_URL, "sslmode=disable")
    msg := "Database connections must use TLS (sslmode=require)"
}
```

### 2. Add test fixtures

Create `policy/testdata/database_security/`:

```
pass/
  valid-secret.yaml      # password from env var, not hardcoded
  valid-connection.yaml  # DATABASE_URL with sslmode=require

fail/
  hardcoded-password.yaml  # password = "admin123"
  no-tls.yaml              # sslmode=disable
```

### 3. Test the policy

```bash
nix develop -c conftest verify -p policy/database_security.rego
```

Fix policy until all tests pass.

### 4. Wire into selftest

Update `crates/xtask/src/commands/policy_test.rs` to run the new policy:

```rust
let targets = vec![
    "infra/k8s/",
    "config/",  // add new target for DB configs
];
```

### 5. Document the control

Add to this file (`controls-as-code.md`):

- Category name and purpose
- Rules enforced
- Example violation and fix
- Rationale

---

## Local vs CI Behavior

**Locally:**

- If `conftest` is not installed, `xtask selftest` shows:

  ```
  ⚠ Policy tests skipped (conftest not found)
  💡 Hint: Run nix develop -c cargo run -p xtask -- selftest for full validation
  ```

- Selftest still passes (allows native Rust workflow without Nix).

**In CI:**

- `conftest` is provided via Nix flake.
- Policy tests are **mandatory**.
- If policies fail, CI blocks merge.

**Why allow local skip?**

- Some developers can't/won't install Nix due to org policy.
- Failing selftest locally hurts adoption.
- CI still enforces, so violations are caught before merge.

**Recommended local setup:**

```bash
# Option 1: Use Nix (provides conftest + all tools)
nix develop
cargo run -p xtask -- selftest  # full validation

# Option 2: Install conftest manually
# See docs/dev-environment.md for installation steps
```

---

## Compliance Reporting

**For auditors / compliance teams:**

This template provides:

1. **Policy inventory**: All controls documented in `policy/*.rego`
2. **Test coverage**: Every control has test fixtures in `policy/testdata/`
3. **Enforcement proof**: CI logs show policy test results for every commit
4. **Traceability**: Policies are versioned in git, tied to ADRs

**Example audit trail:**

```bash
# Show all policies
ls policy/*.rego

# Show test coverage
conftest verify -p policy/

# Show CI enforcement
# (Check .github/workflows/ci.yml for policy-test job)

# Show policy history
git log -- policy/
```

**Compliance mapping:**

| Requirement | Policy | Enforced By | Evidence |
|-------------|--------|-------------|----------|
| CIS k8s 5.2.1: Run as non-root | `k8s_standards.rego` | CI (conftest) | CI logs, policy tests |
| GDPR Art. 32: Data minimization | `privacy_compliance.rego` | CI (conftest) | CI logs, policy tests |
| SOC2 CC6.1: Logical access controls | `k8s_standards.rego` | CI (conftest) | CI logs, policy tests |

---

## Controls vs Other Checks

This template has multiple validation layers. Here's how controls fit in:

| Layer | Tool | Purpose | Scope |
|-------|------|---------|-------|
| **Code Quality** | `cargo fmt`, `cargo clippy` | Style, linting | Rust code |
| **Business Logic** | `cargo test` | Unit/integration tests | Business rules |
| **Behavior Contracts** | Cucumber (BDD) | Acceptance criteria | User stories |
| **AC Mapping** | `xtask ac-status` | Spec traceability | Ledger ↔ features |
| **Controls** | `conftest` (Rego) | Governance, compliance | Infra, policies |
| **Bundler** | `xtask bundle` | LLM context generation | Governance metadata |

**When to use controls (Rego) vs lints (Clippy):**

- **Clippy**: Code-level patterns (e.g., "don't use `unwrap()`")
- **Rego**: Policy-level patterns (e.g., "no PII in logs", "no root containers")

Rego is for **cross-cutting concerns** that span code, config, and infrastructure.

---

## Future Enhancements

**Possible additions:**

1. **Runtime enforcement**: Deploy OPA as a Kubernetes admission controller
   - Policies run at deploy time, block non-compliant manifests
   - Complements CI enforcement (defense-in-depth)

2. **Policy versioning**: Track policy changes with ADRs
   - Example: `ADR-0010: Require TLS for all DB connections`
   - Link policies to ADRs in `specs/spec_ledger.yaml`

3. **Custom lint for Rust code**: Clippy plugin to enforce template patterns
   - Example: "Business-core must not import Axum"
   - Would replace manual code review for some rules

4. **Automated remediation**: Generate fixes for common violations
   - Example: Add `securityContext.runAsNonRoot: true` to deployments
   - Risky, but possible for well-understood patterns

---

## References

- [Open Policy Agent (OPA)](https://www.openpolicyagent.org/)
- [Conftest](https://www.conftest.dev/)
- [Rego Language Reference](https://www.openpolicyagent.org/docs/latest/policy-language/)
- [Policy Testing Best Practices](https://www.openpolicyagent.org/docs/latest/policy-testing/)
- [ADR-0004: Policy and LLM Governance](../adr/0004-policy-and-llm-governance.md)

---

**Summary:**

Controls-as-code in this template means:

- Governance requirements are **executable** (Rego policies)
- Validated **automatically** (conftest in CI)
- **Tested** like any code (test fixtures)
- **Versioned** and **traceable** (git, ADRs)

This shifts compliance from "manual checklist before deploy" to "automated gate before merge."
