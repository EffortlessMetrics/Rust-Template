# Explanation: Template Architecture

This document explains the layered architecture of the Rust-as-Spec platform cell.

> **See also:**
> - [Rust-as-Spec Overview](rust-as-spec-overview.md) – The conceptual model and four-phase pipeline
> - [Template Contracts](TEMPLATE-CONTRACTS.md) – Kernel guarantees and extension points

## Rust-as-Spec Pipeline

The core flow of the template can be summarized in one diagram:

```
spec_ledger.yaml → spec-runtime → selftest → /platform/* → Backstage/agents
```

Each stage:

1. **spec_ledger.yaml** - The canonical source of truth: stories, requirements, acceptance criteria
2. **spec-runtime** - Parses specs at startup, builds the governance graph in memory
3. **selftest** - Validates the graph (all REQs have ACs, all ACs have tests, etc.)
4. **`/platform/*`** - Exposes the validated graph as JSON APIs
5. **Backstage/agents** - Consume the APIs for dashboards, tiles, and automated workflows

## Architecture Planes

The template is organized into four planes:

### 1. Specification Plane

Where the "what" lives:

- `specs/spec_ledger.yaml` - Stories, requirements, acceptance criteria
- `specs/features/*.feature` - BDD scenarios tagged with AC IDs
- `specs/tasks.yaml` - Work items and their lifecycle
- `specs/openapi/openapi.yaml` - API contract definitions

### 2. Implementation Plane

Where the "how" lives:

- `crates/core/` - Domain logic (hexagonal architecture)
- `crates/adapters/` - Infrastructure adapters (HTTP, DB, etc.)
- `crates/app-http/` - HTTP application wiring
- `crates/spec-runtime/` - Runtime governance graph and introspection
- `crates/acceptance/` - BDD step definitions

> **Note:** This repo ships only the **platform kernel** (`/platform/*` endpoints) and examples. Domain-specific HTTP APIs (orders, invoices, tasks) live in your forks, not here. See [template-foundation-vs-examples.md](template-foundation-vs-examples.md).

### 3. Policy Plane

Where enforcement lives:

- `policy/*.rego` - OPA/Rego policies for ledger, features, privacy
- `.github/workflows/` - CI workflows enforcing gates
- `cargo xtask selftest` - The 11-step governance validation

### 4. Tooling and LLM Plane

Where developer experience lives:

- `flake.nix` - Reproducible development environment
- `crates/xtask/` - CLI commands for governance workflows
- `.llm/` - Context bundling for LLM agents
- `.claude/` - Skills and agents for Claude Code

## Related Documents

- [Rust-as-Spec Overview](rust-as-spec-overview.md) - Conceptual model
- [Template Contracts](TEMPLATE-CONTRACTS.md) - Kernel guarantees
- [IDP Positioning](idp-positioning.md) - How this fits with platform tools
