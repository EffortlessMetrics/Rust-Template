# Guard Implementation Analysis

**Date**: 2026-03-23
**Author**: Cross-Repo Pattern Analyzer

This document compares the independent guard implementations between Rust-Template's workflow-based approach and the standalone guard repositories (depguard, diffguard, covguard).

## Executive Summary

Two distinct guard implementation patterns have emerged across the EffortlessMetrics ecosystem:

1. **Rust-Template**: GitHub Actions workflows + Rego policy files (policy-as-code)
2. **Standalone Guards**: Multi-crate Rust workspaces with CLI + library interfaces

**Recommendation**: Maintain both approaches with clear separation of concerns and a defined integration strategy. Do not converge—hybridize.

---

## Side-by-Side Technical Comparison

### Detection Mechanisms

| Aspect | Rust-Template (Workflow + Rego) | Standalone Guards (Rust CLI) |
|--------|--------------------------------|------------------------------|
| **Core Technology** | Open Policy Agent (OPA) + Rego DSL | Rust crates with domain-specific logic |
| **Analysis Scope** | File-level metadata, PR body parsing, JSON/YAML validation | Deep semantic analysis (diff parsing, coverage aggregation, dependency graphs) |
| **Detection Logic** | Declarative rules (`deny[msg] { ... }`) | Algorithmic analysis with property-based testing |
| **Input** | Changed files list, PR metadata, config file paths | Unified diffs, LCOV reports, Cargo.toml manifests |
| **Performance** | Fast (policy evaluation only, no compilation) | Slower (requires Rust build, but highly optimized) |
| **Example** | `is_danger_zone(f) { danger_zone_paths[_] == f }` | `DiffChecker::new(config).check(&diff)?` |

### Enforcement Strategies

| Aspect | Rust-Template | Standalone Guards |
|--------|--------------|-------------------|
| **Primary Mechanism** | `conftest test -p policy/*.rego` in CI workflows | CLI exit codes (0/2/3), GitHub annotations |
| **Failure Modes** | Hard failure (deny) or warning (warn) | Configurable: `fail_on = "error" | "warn" | "never"` |
| **PR Integration** | PR comments via workflow outputs, status checks | Inline annotations (`::error file=X,line=Y`), SARIF upload |
| **Scope Enforcement** | Pre-merge: analyzes PR body + changed files | Pre-merge: diff-scoped analysis only |
| **Advisory Mode** | Yes (warn rules never block) | Yes (`--fail-on never`, profiles) |

### Output Formats

| Aspect | Rust-Template | Standalone Guards |
|--------|--------------|-------------------|
| **Primary Output** | CI logs, conftest stdout/stderr | JSON receipts, Markdown PR comments |
| **Structured Data** | Limited (conftest output, custom JSON prep) | Schema-first (JSON Schema contracts, versions) |
| **SARIF Support** | No | Yes (diffguard, covguard) |
| **GitHub Annotations** | Manual (`::error` in shell) | Built-in (`--github-annotations` flag) |
| **Artifact Stability** | Ad-hoc (workflow-specific) | Versioned schemas (`sensor.report.v1`, `depguard.report.v2`) |
| **Agent Integration** | Requires parsing logs/outputs | Direct JSON consumption with fix hints |

### Portability

| Aspect | Rust-Template | Standalone Guards |
|--------|--------------|-------------------|
| **CI Platform** | GitHub Actions only | GitHub Actions, Azure Pipelines, GitLab CI, local CLI |
| **Dependencies** | Nix (conftest, jq, OPA) | Rust toolchain (cargo) |
| **Local Developer Use** | Limited (requires conftest setup) | First-class (`depguard check`, `diffguard check`) |
| **Cross-Repo Reuse** | Copy workflows + policy directory | `cargo install` or embed as library |
| **Version Pinning** | Workflow action pins | Cargo.lock, crates.io releases |

### Agent Integration

| Aspect | Rust-Template | Standalone Guards |
|--------|--------------|-------------------|
| **Structured Receipts** | No (ad-hoc JSON, workflow outputs) | Yes (schema-validated, versioned) |
| **Fix Hints** | Manual (policy messages only) | Yes (`fix_action` tokens in findings) |
| **Cockpit Compatibility** | Not designed for it | First-class (sensor.report.v1 contract) |
| **Bot Automation** | Requires parsing | Direct JSON consumption |
| **Determinism** | Yes (policy evaluation) | Yes (sorted findings, bounded outputs) |

---

## Why They Were Developed Independently

### Rust-Template Guard Workflows

**Context**: Created as part of the governed SDLC template infrastructure.

**Drivers**:
- **GitHub-native**: Leverage platform features (Actions, PR comments, status checks)
- **Declarative governance**: Policy logic as versionable, auditable Rego files
- **Fork-friendly**: Easy to customize workflows without recompiling Rust
- **Fast iteration**: Policy changes don't require crate releases
- **Template baseline**: Provide "batteries included" governance out of the box

**Use Cases**:
- PR scope classification (mechanical vs. behavior changes)
- Feature flag test matrix enforcement
- Kubernetes configuration validation
- Privacy/PII detection policies
- Template-specific governance rules

### Standalone Guard Repos

**Context**: Built as production-ready, publishable analysis tools.

**Drivers**:
- **Portability**: Run anywhere Rust compiles (not GitHub-locked)
- **Deep analysis**: Complex algorithms (diff parsing, coverage aggregation) better suited to Rust than Rego
- **Structured outputs**: Schema-first design for agent/automation consumption
- **Library embedding**: Can be embedded in other Rust tools, not just CI
- **Multi-CI support**: Azure Pipelines examples included (diffguard)

**Use Cases**:
- Dependency manifest hygiene (depguard)
- Diff-aware policy enforcement (diffguard)
- Coverage-aware gating (covguard)
- Local developer feedback (CLI invocation)
- Agent workflow integration (structured receipts)

---

## Trade-Offs

### Rust-Template Approach

**Strengths**:
- ✅ **Declarative**: Rego policies are auditable, testable in isolation
- ✅ **Fast deployment**: Policy changes via PR merge, no build artifacts
- ✅ **GitHub integration**: Native workflow outputs, comments, status checks
- ✅ **Template flexibility**: Consumers customize workflows without Rust knowledge
- ✅ **Low barrier**: Shell + conftest easier than multi-crate workspace

**Weaknesses**:
- ❌ **GitHub-locked**: Cannot run outside GitHub Actions without significant adaptation
- ❌ **Limited analysis**: Rego not suited for complex algorithmic checks (diff parsing, coverage math)
- ❌ **Ad-hoc outputs**: No schema-first design, harder for agent consumption
- ❌ **Shell orchestration**: PR body parsing, JSON prep in bash scripts
- ❌ **No local feedback**: Developers need conftest setup for local testing

### Standalone Guard Approach

**Strengths**:
- ✅ **Portable**: Run in any CI system, or locally via `cargo install`
- ✅ **Deep analysis**: Complex algorithms (diff parsing, aggregation) in Rust
- ✅ **Structured outputs**: Schema-validated JSON, SARIF support, fix hints
- ✅ **Agent-ready**: Designed for Cockpit/automation consumption
- ✅ **Library + CLI**: Can embed in tools or run standalone
- ✅ **Deterministic**: Sorted findings, bounded state, reproducible outputs

**Weaknesses**:
- ❌ **Build overhead**: Requires Rust toolchain, crate compilation in CI
- ❌ **Version drift**: CLI behavior changes require crate updates
- ❌ **Complexity**: Multi-crate workspaces steeper learning curve than Rego
- ❌ **Release cycle**: Policy changes require crate releases (slower iteration)
- ❌ **Less GitHub-integrated**: Must manually invoke for comments/annotations

---

## Architectural Direction: Hybrid Approach

### Decision

**Maintain two complementary guard implementation patterns with clear separation:**

#### 1. Rust-Template: Workflow Orchestration + Policy-as-Code

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

#### 2. Standalone Guards: Portable Analysis Tools

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

---

## Integration Strategy

### Future Enhancement: Rust-Template Workflows Invoke Guard CLIs

**Rationale**: Combine declarative governance (Rego) with powerful analysis (Rust).

**Implementation**:
```yaml
- name: Run depguard check
  if: steps.scope.outputs.declared_type == 'mechanical'
  run: |
    cargo install depguard-cli@0.3.0
    depguard check --format sarif > depguard.sarif

- name: Run diffguard check
  if: steps.scope.outputs.declared_type == 'behavior'
  run: |
    diffguard check --preset strict --output diffguard.json
```

**Wiring Plan**:

| Rust-Template Workflow | Guard CLI to Invoke | Trigger Condition |
|------------------------|---------------------|-------------------|
| `ci-scope-guard.yml` | `diffguard` | Behavior-declared PRs |
| `ci-coverage.yml` | `covguard` | Threshold enforcement |
| `ci-supply-chain.yml` | `depguard` | Dependency changes |
| `ci-lints.yml` | `diffguard` | Code quality rules |

**Benefits**:
- Leverages **declarative policies** (Rego) for governance rules
- Uses **Rust analysis tools** for complex algorithmic checks
- Maintains **portability** (guard CLIs work in any CI system)
- Provides **structured output** (SARIF, JSON receipts) for agent consumption
- Enables **local developer feedback** (CLI invocation outside CI)

---

## Compliance

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

- Periodic cross-repo analysis identifies divergence
- Guard CLI version pinning in workflows prevents silent behavior changes
- Receipt provenance tracking (`docs/receipts/`) documents guard execution history

---

## Migration Path (If Convergence Desired)

### Option A: Rust-Template Workflows Adopt Guard CLIs

**Phase 1** (Immediate):
1. Pin guard CLI versions in `ci-supply-chain.yml`, `ci-coverage.yml`
2. Replace shell-based analysis with CLI invocations
3. Standardize output formats (SARIF, JSON receipts)

**Phase 2** (Q2 2026):
4. Update `ci-scope-guard.yml` to invoke `diffguard` for behavior changes
5. Update `ci-coverage.yml` to invoke `covguard` for threshold enforcement
6. Document migration in `docs/guard-migration.md`

**Phase 3** (Q3 2026):
7. Deprecate duplicate Rego policies (where guard CLIs provide equivalent checks)
8. Keep Rego for governance-only rules (scope, PII, k8s validation)

### Option B: Guard Repos Adopt Rego for Policy Logic

**Not Recommended**:
- Rego not suited for algorithmic analysis (diff parsing, coverage math)
- Would require embedding OPA in Rust (increased complexity)
- Loses portability advantage (OPA dependency everywhere)
- No clear benefit over current architecture

---

## Related Analysis

- Analysis performed: Cross-Repo Pattern Analyzer run, 2026-03-23
- Related repositories: `EffortlessMetrics/depguard`, `EffortlessMetrics/diffguard`, `EffortlessMetrics/covguard`
- Template repository: `EffortlessMetrics/Rust-Template`

## References

- Open Policy Agent: https://www.openpolicyagent.org/
- Rego Language: https://www.openpolicyagent.org/docs/latest/policy-language/
- SARIF Format: https://sarifweb.azurewebsites.net/
- conftest: https://github.com/open-policy-agent/conftest
- Multi-crate workspaces: https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html
