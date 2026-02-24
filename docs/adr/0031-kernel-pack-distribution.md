<!-- doclint:disable orphan-version -->

# ADR-0031: Kernel Pack Distribution

**Status**: Proposed
**Date**: 2026-02-20
**Authors**: Steven Zimmerman
**Related ACs**: (none yet — ACs will be added when implementation begins)
**Relates to**: ADR-0030 (microcrate-architecture), ADR-0001 (hexagonal-architecture)

---

## Context

The Rust-as-Spec platform cell is currently distributed as a monolithic Git repository containing ~49 crates, ~200 governance files (workflows, skills, agents, rules, specs, schemas), and extensive documentation. Downstream teams adopt the template by cloning or forking the entire repository.

### Problems with the Current Model

1. **Update friction**: When the kernel evolves (e.g., new selftest steps, updated workflows, schema changes), downstream forks must manually cherry-pick or copy-paste changes across ~200 files. There is no automated upgrade path.

2. **Monolithic distribution**: Forks carry the full development history and all kernel development tooling, even though most of it is infrastructure they should consume, not modify.

3. **No separation of concerns**: Rust crate code, CI workflows, governance YAML, BDD features, and documentation templates are all interleaved in a single repository. Downstream repos cannot selectively adopt subsets.

4. **Version pinning is manual**: The kernel baseline concept (e.g., `v3.3.9-kernel`) relies on Git tag conventions and human discipline. There is no machine-enforced version contract between the kernel and downstream consumers.

5. **Dependency coordination**: When a downstream repo wants to depend on kernel crates (e.g., `gov-model`, `spec-runtime`), they must use Git path or URL dependencies rather than published crates with semver guarantees.

### Goal

Enable the workflow: "clone a minimal repo, set a kernel version in one config file, get all platform updates via automated PR."

---

## Decision

Distribute the platform cell through three complementary channels, each serving a distinct artifact type:

### 1. Published Crates (Rust code)

Publish workspace crates to crates.io with lockstep versioning matching the template version.

**Tier 1 — Contracts (zero internal deps):**
- `gov-model`: Governance domain types (AC, REQ, Task, etc.)
- `spec-types`: Spec ledger types and serialization
- `platform-contract`: Platform API request/response types
- `http-errors`: Shared HTTP error types

**Tier 2 — Runtime (depends on Tier 1):**
- `spec-runtime`: Spec ledger loading and validation
- `ac-kernel`: AC evaluation engine
- `gov-policy`: Policy evaluation (conftest integration)
- `gov-receipts`: Quality receipt generation

**Tier 3 — Application (depends on Tier 1 + 2):**
- `as-spec-platform`: Re-export crate providing the full HTTP platform

**Strategy:** Use workspace-level `publish = true` and `cargo-release` for coordinated publishing. All published crates share the same version number (e.g., `3.3.15`), matching the template version.

### 2. Kernel Pack (non-Rust assets)

A versioned `tar.zst` archive published as a GitHub release artifact, containing everything that is not a Rust crate:

**Contents:**
- CI workflows (29 files)
- Claude Code skills (5), agents (2), rules (3)
- `flake.nix`, `deny.toml`, `.pre-commit-config.yaml`
- Base specs: `spec_ledger.yaml`, `devex_flows.yaml`, JSON schemas, BDD features
- Documentation templates, guides, ADR scaffold
- `Justfile`, `cliff.toml`, and other tooling configuration

**Integrity:** The pack includes a manifest with SHA-256 checksums for every file. The CLI verifies checksums during `sync` and `upgrade` operations.

**Versioning:** Each pack is tagged with the template version (e.g., `kernel-pack-v3.3.15.tar.zst`). The pack version and crate versions always match.

### 3. `as-spec` CLI (portable tool)

A standalone binary distributed via `cargo install` and GitHub releases:

- `as-spec init` — Materialize a kernel pack into a fresh repository
- `as-spec sync` — Reconcile repo state with the pinned kernel version (idempotent)
- `as-spec upgrade --to X.Y.Z` — Bump pinned version, update Cargo deps, download new pack, open PR
- `as-spec ci` — The single command CI runs (replaces per-repo selftest wiring)

### Configuration

Downstream repos contain an `as_spec.toml` at the repository root:

```toml
[kernel]
version = "3.3.15"
pack_checksum = "sha256:..."

[overlay]
spec_ledger = "specs/spec_ledger.yaml"   # service-specific specs
devex_flows = "specs/devex_flows.yaml"   # service-specific flows (optional)

[crates]
# Override specific crate versions if needed (default: match kernel version)
# gov-model = "3.3.15"
```

### Spec Layering

The kernel pack ships a base `spec_ledger.yaml` containing kernel ACs (those with `must_have_ac: true`). Downstream repos maintain their own `spec_ledger.yaml` as an overlay containing service-specific stories, REQs, and ACs.

At runtime, the spec engine merges these two ledgers with explicit rules:
- Kernel ACs cannot be removed (only demoted via `must_have_ac: false` with ADR justification)
- Service ACs extend the kernel set
- Conflicting AC IDs are rejected at load time
- `as-spec sync` validates the merged ledger is consistent

---

## Consequences

### Positive

- **Minimal downstream repos**: A service repo contains ~10 files (Cargo.toml, `as_spec.toml`, service code, overlay specs) instead of ~200
- **Automated updates**: Dependabot/Renovate can propose kernel pack upgrades as PRs
- **Clear contract boundary**: Published crates provide semver guarantees; kernel pack provides governance infrastructure
- **Existing monorepo preserved**: The kernel development workflow stays the same — this ADR only changes the *distribution* boundary
- **Kernel baseline becomes a published artifact**: No longer a Git tag convention but a versioned, checksummed archive

### Negative

- **Publishing coordination**: All crates and the kernel pack must be released together — adds release complexity
- **Pack staleness risk**: If downstream repos skip upgrades, their governance files diverge from the kernel
- **CLI maintenance burden**: The `as-spec` CLI is a new codebase that must be maintained alongside the kernel

### Neutral

- **Migration path**: Existing forks can adopt gradually — the CLI can detect and migrate from a full-clone layout to a pack-managed layout
- **Template repo remains**: The monorepo continues to exist for kernel development; it just gains publishing and packing steps in the release flow

---

## Alternatives Considered

### Git subtree / submodule

Rejected: Subtrees create merge conflicts on governance files; submodules add cognitive overhead and break when repos move. Neither provides selective file adoption.

### Cargo workspace inheritance only

Rejected: Only covers Rust code. Workflows, skills, agents, and specs (the majority of governance surface area) have no Cargo-based distribution mechanism.

### Template registry (e.g., cargo-generate)

Rejected: Template registries handle initial scaffolding but not ongoing updates. The core problem is *keeping downstream in sync*, not initial creation.

---

## Implementation Phases

This ADR is a design artifact. Implementation will proceed in phases, each gated by its own ACs:

1. **Phase 1**: Publish Tier 1 contract crates to crates.io
2. **Phase 2**: Build kernel pack generation into `release-bundle`
3. **Phase 3**: Build `as-spec init` and `as-spec sync`
4. **Phase 4**: Build `as-spec upgrade` with PR generation
5. **Phase 5**: Validate with a second service fork

Each phase will add ACs to `spec_ledger.yaml` and be gated by selftest.
