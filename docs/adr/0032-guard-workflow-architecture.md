# ADR-0032: Guard Workflow Architecture Pattern

**Status**: Accepted
**Date**: 2026-03-23
**Authors**: Cross-Repo Pattern Analyzer, Steven Zimmerman
**Related ACs**: AC-TPL-CI-GUARD, AC-SUPPLY-CHAIN-001

---

## Context

What is the issue we're facing that motivates this decision?

The Rust-Template project needs a consistent strategy for implementing "guard" functionality—automated checks that enforce code quality, security, coverage, and policy compliance in CI/CD workflows. Two distinct implementation patterns have emerged:

1. **Rust-Template's existing approach**: GitHub Actions workflows + Rego policy files (policy-as-code)
   - Examples: `ci-scope-guard.yml`, `ci-coverage.yml`, `ci-policy-verify.yml`
   - Policy logic in `policy/*.rego` files (scope.rego, flags.rego, k8s.rego, etc.)
   - Enforcement via `conftest test` (Open Policy Agent)

2. **Existing EffortlessMetrics guard repositories**: Multi-crate Rust workspaces with CLI + library interfaces
   - `depguard` (8 crates): Dependency manifest hygiene enforcement
   - `diffguard` (6 crates): Diff-aware policy enforcement
   - `covguard` (~20 crates): Coverage-aware guard orchestration

The question: Should Rust-Template consolidate guard implementations into a single pattern, or maintain both approaches with clear separation of concerns?

### Constraints

- Rust-Template serves as a **template repository** for governed SDLC infrastructure
- Guard repos (`depguard`, `diffguard`, `covguard`) are already structured as production-ready, publishable crates
- Agent workflows require **structured receipts** (schema-validated JSON, SARIF) for automated decision-making
- CI workflows need to be **fork-friendly** and easily customizable by template consumers
- Some guard logic is **governance policy** (declarative rules), while other logic is **analysis tooling** (complex algorithms)

---

## Decision

What is the change that we're proposing and/or doing?

**Maintain two complementary guard implementation patterns with clear separation:**

### 1. Rust-Template: Workflow Orchestration + Policy-as-Code

**Purpose**: GitHub-native workflow orchestration and governance policy enforcement

**Implementation**:
- GitHub Actions YAML workflows (`.github/workflows/ci-*.yml`)
- Rego policy files (`policy/*.rego`) for declarative rules
- Shell scripts for orchestration and JSON input preparation
- Enforcement via `conftest test -p policy/*.rego`

**Scope**:
- PR scope classification (`ci-scope-guard.yml`)
- Feature flag test matrix enforcement
- Kubernetes configuration validation
- Privacy/PII detection policies
- Template-specific governance rules
- Workflow-level advisory warnings vs. hard failures

**Example**:
```yaml
- name: Run scope policy check
  run: |
    jq -n \
      --argjson files "$CHANGED_FILES" \
      --arg scope_type "$SCOPE_TYPE" \
      '{files: $files, scope_type: $scope_type}' > /tmp/scope_input.json
    conftest test -p policy/scope.rego /tmp/scope_input.json
```

### 2. Guard Repos: Portable Analysis Tools with Structured Output

**Purpose**: Reusable, framework-agnostic analysis tools with library + CLI interfaces

**Implementation**:
- Multi-crate Rust workspaces with clean architecture
- Separation: `-types`, `-domain`, `-core`, `-adapters-*`, `-cli`, `-render`
- Property-based testing, fuzz testing, snapshot testing
- Schema-first output (JSON Schema contracts, SARIF support)

**Scope**:
- Dependency manifest analysis (`depguard`)
- Diff parsing and policy enforcement (`diffguard`)
- Coverage report aggregation and threshold enforcement (`covguard`)
- Multi-CI support (GitHub Actions, GitLab CI, Azure Pipelines)
- Local developer feedback (CLI invocation)
- Agent workflow integration (structured receipts)

**Example**:
```bash
# CLI invocation
diffguard check --preset strict --output sarif.json

# Library embedding
use diffguard_core::{DiffChecker, Config};
let result = DiffChecker::new(config).check(&diff)?;
```

### 3. Integration Strategy: Hybrid Architecture

**Future Enhancement**: Rust-Template workflows will invoke guard CLIs for enforcement:

```yaml
- name: Run depguard check
  if: steps.scope.outputs.declared_type == 'mechanical'
  run: |
    cargo install depguard-cli
    depguard check --format sarif > depguard.sarif

- name: Run diffguard check
  if: steps.scope.outputs.declared_type == 'behavior'
  run: |
    diffguard check --preset strict --output diffguard.json
```

**Rationale for this option**:
- Leverages **declarative policies** (Rego) for governance rules
- Uses **Rust analysis tools** for complex algorithmic checks
- Maintains **portability** (guard CLIs work in any CI system)
- Provides **structured output** (SARIF, JSON receipts) for agent consumption
- Enables **local developer feedback** (CLI invocation outside CI)

---

## Consequences

What becomes easier or more difficult to do because of this change?

### Positive

- **Clear separation of concerns**: Policy-as-code for governance, Rust tools for analysis
- **Template flexibility**: Fork consumers can customize workflows without recompiling Rust
- **Agentic-friendly**: Guard repos provide schema-validated receipts for agent workflows
- **Multi-CI support**: Guard CLIs can be invoked from GitHub, GitLab, Azure Pipelines, or locally
- **Publishable crates**: Guard repos are already structured for crates.io publication
- **Declarative governance**: Rego policies are auditable, versionable, and testable in isolation
- **Composability**: Guard CLIs can be chained (`diffguard | covguard`) or embedded in other tools

### Negative

- **Dual maintenance**: Two implementation patterns require documentation and onboarding
- **Build overhead**: Guard repos require Rust toolchain compilation in CI
- **Complexity**: Multi-crate workspaces have steeper learning curve than Rego policies
- **Version drift**: Risk of workflow logic diverging from guard CLI capabilities
- **Integration effort**: Requires wiring guard CLIs into Rust-Template workflows (future work)

### Neutral

- **Policy changes**: Rego policies can still be updated independently of guard CLIs
- **Workflow changes**: GitHub Actions can be modified without touching Rust code
- **Monitoring needs**: Both patterns require health checks and failure mode documentation

---

## Compliance

How do we ensure this decision is followed?

### Automated Checks

- `xtask selftest` validates workflow syntax and policy structure
- `cargo test` in guard repos enforces code quality and test coverage
- CI workflows invoke both Rego policies and guard CLIs (once integrated)
- SARIF output from guard CLIs integrates with GitHub code scanning

### Manual Review Patterns

- Architecture reviews verify new guards follow the appropriate pattern:
  - Governance policy → Rego + workflow
  - Analysis tooling → Rust multi-crate workspace
- ADR updates required when guard architecture evolves
- Documentation in `docs/audit/` tracks guard invocation patterns

### Drift Detection

- Periodic cross-repo analysis (this agent's mandate) identifies divergence
- Guard CLI version pinning in workflows prevents silent behavior changes
- Receipt provenance tracking (`docs/receipts/`) documents guard execution history

---

## Notes

### Related Issues and PRs

- Analysis performed: Cross-Repo Pattern Analyzer run, 2026-03-23
- Related repositories: `EffortlessMetrics/depguard`, `EffortlessMetrics/diffguard`, `EffortlessMetrics/covguard`
- Template repository: `EffortlessMetrics/Rust-Template`

### Migration Path (Future)

If Rust-Template workflows adopt guard CLIs:

1. Pin guard CLI versions in workflows (e.g., `depguard-cli@0.3.0`)
2. Replace shell-based analysis with CLI invocations
3. Standardize output formats (SARIF, JSON receipts)
4. Update `ci-scope-guard.yml` to invoke `diffguard` for behavior changes
5. Update `ci-coverage.yml` to invoke `covguard` for threshold enforcement

### Examples

**Rego Policy (Rust-Template)**:
```rego
package scope

deny[msg] {
    input.scope_type == "behavior"
    not input.filesContainBehaviorChange
    msg := "Behavior-declared PR lacks code changes"
}

warn[msg] {
    input.fileCount > 50
    msg := sprintf("Large PR (%d files); consider splitting", [input.fileCount])
}
```

**Rust Guard CLI (Guard Repos)**:
```rust
// diffguard-core/src/check.rs
pub struct Finding {
    pub code: String,           // e.g., "DIFF-001"
    pub severity: Severity,     // Error, Warning, Info
    pub location: Location,     // File, line, column
    pub message: String,
    pub suppression: Option<Suppression>,
}

pub fn check(diff: &UnifiedDiff, config: &Config) -> Vec<Finding> {
    // Complex analysis logic with property tests
}
```

### References

- Open Policy Agent: https://www.openpolicyagent.org/
- Rego Language: https://www.openpolicyagent.org/docs/latest/policy-language/
- SARIF Format: https://sarifweb.azurewebsites.net/
- conf test: https://github.com/open-policy-agent/conftest
- Multi-crate workspaces: https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html
