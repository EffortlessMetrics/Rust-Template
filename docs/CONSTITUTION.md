# Platform Constitution
# Version: 1.0.0

This document defines the non-negotiable invariants and precedence rules for the platform.
All specs, code, and workflows must adhere to these principles.

## 1. Core Invariants

### 1.1 Nix-First
- **Rule**: The Nix devshell is the single source of truth for the development environment.
- **Implication**: No tool may be required that is not provided by `flake.nix`.
- **Enforcement**: `cargo xtask doctor` validates the Nix environment.

### 1.2 Spec-First Behavior
- **Rule**: All functional behavior must be defined in `specs/spec_ledger.yaml` (US/REQ/AC) before implementation.
- **Implication**: No code changes without a corresponding AC or REQ update.
- **Enforcement**: `cargo xtask selftest` validates AC coverage via BDD.

### 1.3 DevEx-as-Spec
- **Rule**: Developer workflows are defined in `specs/devex_flows.yaml`, not just prose.
- **Implication**: `cargo xtask` commands must match the spec.
- **Enforcement**: `cargo xtask selftest` enforces the DevEx contract.

### 1.4 Docs-as-Spec
- **Rule**: Structural documentation (Design, Plans, Requirements) must be registered in `specs/doc_index.yaml`.
- **Implication**: No "orphan" design docs; all must link to REQs/ADRs.
- **Enforcement**: `cargo xtask docs-check` validates the index and front-matter.

### 1.5 Single Gate
- **Rule**: `cargo xtask selftest` is the authoritative gate for correctness.
- **Implication**: If `selftest` passes, the change is valid (behaviorally).
- **Enforcement**: CI runs `selftest` as the primary blocking step.

### 1.6 Security-First
- **Rule**: Dependency health and supply chain security are blocking gates.
- **Implication**: No release with known high-severity vulnerabilities or dirty git tree.
- **Enforcement**: `cargo xtask release-verify` and `audit`.

## 2. Precedence Rules

When artifacts conflict, this hierarchy applies (highest priority first):

1. **Constitution** (this file)
2. **Specs** (`spec_ledger.yaml`, `devex_flows.yaml`, `doc_index.yaml`)
3. **ADRs** (`docs/adr/*.md`)
4. **Code & Tests**
5. **Prose Documentation** (README, CONTRIBUTING)

**Resolution Strategy**:
- If Code conflicts with Spec → **Fix Code**.
- If Spec conflicts with Constitution → **Fix Spec**.
- If Prose conflicts with Spec → **Fix Prose**.

## 3. Change Protocols

### 3.1 Structural Changes
- **Trigger**: New component, major refactor, or platform-level change.
- **Requirement**: Must have `ADR` + `Design Doc` + `REQ` tagged `structural`.

### 3.2 Feature Additions
- **Trigger**: New user-facing functionality.
- **Requirement**: Must have `US/REQ/AC` in ledger + `BDD` scenario.

### 3.3 Dependency Updates
- **Trigger**: `flake.lock` or `Cargo.lock` change.
- **Requirement**: Must pass `cargo xtask audit`.
