# ADR-0004: Policy-as-Code and LLM Governance

**Status**: Accepted
**Date**: 2025-01-18
**Authors**: Steven Zimmerman
**Related ACs**: AC-TPL-003 (if policy testing AC exists)

---

## Context

Governance requirements often live in:

- Security runbooks (rarely consulted)
- Compliance checklists (checked once, then ignored)
- Code review tribal knowledge ("we don't do that here")
- Post-incident retros ("we should have caught this")

Traditional enforcement:

1. **Manual review**: slow, inconsistent, doesn't scale
2. **Linters**: catch syntax, not policy (e.g., "no hardcoded secrets")
3. **Runtime gates**: too late (policy violated before deploy)
4. **Documentation**: ignored unless enforced

With LLM-native development:

- Agents can generate compliant code *or* violate policies at scale
- Need machine-readable policies that both CI and LLMs can check
- Policy drift (doc says X, CI enforces Y) breaks trust

We need:

- Policies as executable code (not prose)
- Validated in CI, visible to LLMs
- Covering infrastructure (k8s), data (privacy), and LLM workflows

---

## Decision

We adopt **policy-as-code** via Open Policy Agent (OPA) Rego:

### 1. Policy Directory (`policy/`)

Policies organized by domain:

```
policy/
├── k8s_standards.rego         # Kubernetes manifest rules
├── privacy_compliance.rego    # PII/sensitive data rules
├── flags_rollout.rego         # Feature flag governance
├── llm.rego                   # LLM bundle validation
└── testdata/                  # Test fixtures for each policy
    ├── k8s/
    ├── privacy/
    └── llm/
```

Each policy:

- Written in Rego (OPA's policy language)
- Tested via `conftest verify` against testdata
- Enforced in CI via `conftest test`

### 2. Policy Categories

#### a. Kubernetes Standards (`k8s_standards.rego`)

Enforce:

- Containers run as non-root (`runAsNonRoot: true`)
- No `hostPath` volumes
- Resource limits defined (`requests`, `limits`)
- Liveness/readiness probes present
- No privileged containers

Example violation:

```yaml
# k8s/deployment.yaml
spec:
  template:
    spec:
      containers:
        - name: app
          # FAIL: missing securityContext.runAsNonRoot
```

#### b. Privacy Compliance (`privacy_compliance.rego`)

Enforce:

- No PII in logs or error messages (email, SSN, credit card patterns)
- Sensitive data encrypted at rest (check k8s Secrets, DB configs)
- Data retention policies documented

Example violation:

```rust
// FAIL: logging user email directly
tracing::info!("User logged in: {}", user.email);
```

#### c. Feature Flags (`flags_rollout.rego`, `flags_warn.rego`)

Enforce:

- No `enabled: true` for experimental flags in prod
- Flags must have owner and expiry date
- Deprecated flags trigger warnings

#### d. LLM Governance (`llm.rego`)

Enforce:

- Contextpack bundles don't exceed size limits
- Bundle metadata includes template version
- No sensitive files in LLM context (`.env`, `credentials.json`)

### 3. Enforcement

**CI (mandatory):**

```yaml
# .github/workflows/policy-tests.yml
- name: Run policy tests
  run: nix develop -c conftest test -p policy/ <targets>
```

**Local (optional but encouraged):**

```bash
# Inside nix develop (provides conftest)
cargo run -p xtask -- selftest  # includes policy tests
```

If `conftest` not installed locally:

- `xtask selftest` shows warning: `⚠ Policy tests skipped (conftest not found)`
- Still passes locally, but fails in CI

**LLM integration:**

- Policies visible in `policy/` (LLMs can read them)
- `CLAUDE.md` references policies as constraints
- LLM bundles include policy context for relevant domains

---

## Consequences

### Positive

- **Automated enforcement**: Policies run in CI, no manual gate-keeping
- **Explicit**: Rules are code, not tribal knowledge
- **Testable**: Rego policies tested via `conftest verify` with fixtures
- **LLM-friendly**: Agents can read Rego and generate compliant manifests
- **Auditability**: Policy violations visible in CI logs, traceable to commits

### Negative

- **Learning curve**: Rego is unfamiliar to most developers
- **Maintenance**: Policies can become stale if not reviewed regularly
- **False positives**: Overly strict policies can block valid use cases
- **Tooling dependency**: Requires OPA/conftest in CI and locally (via Nix)

### Neutral

- **Not a replacement for runtime enforcement**: Policies catch static violations, not runtime behavior
- **Complements linters**: Rego for policy, Clippy for code quality

---

## Compliance

**Automated:**

- `xtask selftest` step 5: policy tests
  - Locally: warns if conftest missing
  - CI: fails if conftest missing or policies fail
- `conftest test` runs against:
  - `infra/k8s/**/*.yaml` (k8s policies)
  - `specs/spec_ledger.yaml` (ledger policies, future)
  - `.llm/bundle/*.md` (LLM policies)

**Manual:**

- Code review: reject PRs that disable policy checks without justification
- Quarterly review: audit policies for relevance and coverage

**Detection:**

- CI logs show policy violations with file/line context
- `conftest test --trace` for debugging policy logic

---

## Notes

**Why Rego instead of custom linters?**

- Rego is declarative: expresses "what" not "how"
- OPA ecosystem: tooling (conftest), testing, IDE support
- Portable: same policies work for k8s, JSON, YAML, etc.

**Why allow local skip?**

- Some developers can't/won't install conftest (org restrictions, resistance)
- Failing selftest locally hurts adoption
- CI still enforces, so violations are caught before merge

**Why not runtime policy enforcement (e.g., k8s admission controllers)?**

- We do both: Rego at CI time (shift-left), admission controllers at runtime (defense-in-depth)
- This ADR focuses on CI policies; runtime enforcement is orthogonal

**Adding a new policy:**

1. Create `policy/<domain>.rego` (e.g., `database_security.rego`)
2. Add testdata: `policy/testdata/<domain>/pass/` and `fail/`
3. Run `conftest verify -p policy/` to validate tests
4. Update `xtask selftest` to run `conftest test` on relevant targets
5. Document in `docs/explanation/controls-as-code.md`

**Migration from prose policies:**

If you have existing security/compliance docs:

1. Identify enforceable rules (e.g., "no root containers")
2. Write Rego policy to detect violations
3. Test against current codebase (expect failures)
4. Fix violations or add exceptions (with justification)
5. Enable policy in CI
6. Archive prose doc or convert to explanatory context

**LLM workflow integration:**

When using LLM bundles:

1. Bundle includes relevant policies (e.g., k8s standards for infra work)
2. CLAUDE.md references policies as constraints
3. LLM generates code respecting policies
4. `xtask selftest` catches violations before commit

**References:**

- [Open Policy Agent](https://www.openpolicyagent.org/)
- [Conftest](https://www.conftest.dev/)
- [Rego documentation](https://www.openpolicyagent.org/docs/latest/policy-language/)
- [Policy testing best practices](https://www.openpolicyagent.org/docs/latest/policy-testing/)
