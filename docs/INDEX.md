# Documentation Index

**Navigation hub for the Rust-as-Spec Platform Cell**

---

## Quick Start

New to this repository? Start here:

1. **[README.md](../README.md)** - Overview, quick start, key features
2. **[Why This Template Exists](why-this-exists.md)** - Problem statement and philosophy  
3. **[Getting Started](#getting-started)** - Step-by-step onboarding

---

## Strategic Documents

### Positioning & Vision
- **[ROADMAP.md](ROADMAP.md)** - Current state (v2.4.0), pilot plan, future direction
- **[Rust-as-Spec Technical Overview](explanation/rust-as-spec-overview.md)** - Architecture deep-dive

### For Humans
- **[CLAUDE.md](../CLAUDE.md)** - Repository constitution and agent oversight

### For Agents
- **[AGENT_GUIDE.md](AGENT_GUIDE.md)** - Operational guide for LLMs driving workflows

---

## Getting Started

### Installation & Setup
- **[Quick Start (Template)](../README.md#quick-start)** - 30-second tour
- **[Windows Development Guide](how-to/windows-development.md)** - Complete Windows setup (WSL2 + native)
- **[Setup Without Nix](how-to/setup-without-nix.md)** - Manual environment setup *(stub)*
- **[Doctor Command Guide](how-to/using-doctor.md)** - Verifying environment health *(stub)*

### First Steps
- **[First AC Change (Tutorial)](tutorials/first-ac-change.md)** - Implement your first acceptance criterion
- **[Understanding Selftest](tutorials/understanding-selftest.md)** - What the 7 steps validate *(stub)*

---

## How-To Guides

### Adoption Patterns
- **[Add Governance to Existing Repo](how-to/add-governance-to-existing-repo.md)** - Brownfield adoption
- **[Create New Service from Template](how-to/new-service-from-template.md)** - Greenfield instantiation *(stub)*
- **[Update Template from Upstream](how-to/template-updates.md)** - Sync with template evolution *(stub)*

### Development Workflows
- **[AC-First Development](how-to/ac-first-workflow.md)** - Spec → BDD → Code → Selftest *(stub)*
- **[Creating Design Docs](how-to/creating-design-docs.md)** - Linking docs to requirements *(stub)*
- **[Writing ADRs](how-to/writing-adrs.md)** - Documenting architecture decisions *(stub)*
- **[Using LLM Context Bundles](how-to/using-bundles.md)** - Bounded context for AI assistance *(stub)*

### Governance & Policies
- **[Understanding Policies](how-to/understanding-policies.md)** - OPA/Rego enforcement *(stub)*
- **[Fixing Policy Violations](how-to/fixing-policy-violations.md)** - Common failures and fixes *(stub)*
- **[Graph Invariants Guide](how-to/graph-invariants.md)** - Structural integrity requirements *(stub)*

### Operations
- **[Running the Service Locally](how-to/running-locally.md)** - HTTP server, database, observability *(stub)*
- **[Deployment Guide](how-to/deployment.md)** - Kubernetes, secrets, ingress *(stub)*
- **[Monitoring & Observability](how-to/observability.md)** - Metrics, traces, logs *(stub)*

---

## Reference Documentation

### Platform & Environment
- **[Platform Support Reference](reference/platform-support.md)** - Complete platform support matrix, Tier 1/Tier 2, Windows guidance, troubleshooting

### APIs & Contracts
- **[Platform API Reference](reference/platform-api.md)** - `/platform/*` HTTP endpoints *(stub)*
- **[xtask Command Reference](reference/xtask-commands.md)** - All CLI commands and options *(stub)*
- **[Spec Schemas](reference/spec-schemas.md)** - YAML structures and validation rules *(stub)*

### Architecture & Design
- **[ADR Index](adr/README.md)** - All architecture decision records
- **[Hexagonal Architecture Guide](explanation/hexagonal-architecture.md)** - Ports & adapters pattern *(stub)*
- **[Governance Graph Explained](explanation/governance-graph.md)** - Node types, edges, invariants *(stub)*

### Comparisons
- **[vs. Backstage](explanation/comparisons.md#backstage)** - When to use each *(stub)*
- **[vs. SpecKit](explanation/comparisons.md#speckit)** - API contracts vs full governance *(stub)*
- **[vs. Zero to Production](explanation/comparisons.md#zero2prod)** - Production patterns vs governance *(stub)*

---

## Tutorials

Step-by-step learning paths:

1. **[First AC Change](tutorials/first-ac-change.md)** - Hello World for this template
2. **[Adding an Endpoint](tutorials/adding-endpoint.md)** - AC → BDD → Handler → Tests *(stub)*
3. **[Implementing Auth](tutorials/implementing-auth.md)** - Cross-cutting concerns *(stub)*
4. **[Release Process](tutorials/release-process.md)** - Changelog → Tag → Deploy *(stub)*

---

## Explanation Documents

Deeper understanding of concepts:

### Core Concepts
- **[Spec-Driven Development](explanation/spec-driven-development.md)** - Why specs are code *(stub)*
- **[Self-Healing Systems](explanation/self-healing.md)** - What it means in practice *(stub)*
- **[Agent-Native Design](explanation/agent-native-design.md)** - Building for LLMs *(stub)*

### Patterns
- **[Adoption Patterns](explanation/adoption-patterns.md)** - Template vs upstream vs generator
- **[Governance Patterns](explanation/governance-patterns.md)** - Must-have-AC, required commands *(stub)*
- **[Testing Patterns](explanation/testing-patterns.md)** - BDD, integration, policy *(stub)*

---

## Policy Documentation

### Policy Reference
- **[Policy Overview](../policy/README.md)** - What policies enforce
- **[Ledger Policy](policy-reference/ledger.md)** - Spec structure validation *(stub)*
- **[Template Core Policy](policy-reference/template-core.md)** - Governance rules *(stub)*
- **[Privacy Policy](policy-reference/privacy.md)** - Secrets and PII *(stub)*
- **[Kubernetes Policy](policy-reference/k8s.md)** - Deployment security *(stub)*

---

## Contributing

- **[CONTRIBUTING.md](../CONTRIBUTING.md)** - How to contribute to this template
- **[FRICTION_LOG.md](../FRICTION_LOG.md)** - Report issues during pilot *(will be created)*
- **[Code of Conduct](../CODE_OF_CONDUCT.md)** - Community standards *(stub)*

---

## Templates & Examples

Scaffolds you can copy:

- **[Friction Log Template](templates/FRICTION_LOG.md)** - Track pain points during development
- **[Service Metadata Template](templates/SERVICE_METADATA.example.yaml)** - Service config
- **[ADR Template](templates/adr-template.md)** - Architecture decision record *(stub)*
- **[Design Doc Template](templates/design-doc-template.md)** - Design document *(stub)*

---

## Status Legend

- ✅ **Complete** - Document exists and is comprehensive
- 🔄 **In Progress** - Document exists but needs expansion
- 📝 **Stub** - Placeholder with "TODO: tracked in ROADMAP"
- ❌ **Missing** - Not yet created

Current status:
- ✅ ROADMAP.md, Technical Overview, AGENT_GUIDE.md, README.md
- 🔄 First AC Change Tutorial, Brownfield Guide
- 📝 Most how-tos, references, and explanations (stubbed for pilot)

**Note:** Stub documents will be filled based on friction log entries from the pilot phase.

---

## Document Conventions

### File Naming
- How-tos: `how-to/{verb}-{noun}.md` (e.g., `how-to/add-governance-to-existing-repo.md`)
- Tutorials: `tutorials/{task-name}.md` (e.g., `tutorials/first-ac-change.md`)
- Explanations: `explanation/{concept}.md` (e.g., `explanation/self-healing.md`)
- Reference: `reference/{topic}.md` (e.g., `reference/platform-api.md`)

### Front Matter
All docs should have YAML front-matter:

```yaml
---
id: unique-doc-id
type: how_to | tutorial | explanation | reference | design_doc
status: draft | active | deprecated
last_reviewed: YYYY-MM-DD
owner: team-name
related_requirements: [REQ-ID-1, REQ-ID-2]
---
```

### Cross-Linking
- Use relative paths: `[Text](../how-to/file.md)`
- Use file:// URIs for code: `[spec_ledger.yaml](file:///path/to/spec_ledger.yaml)`
- Link to specific lines: `[graph.rs:L123](file:///path/to/graph.rs#L123)`

---

## Need Help?

1. **Check this index** for relevant docs
2. **Search** the docs directory: `grep -r "your topic" docs/`
3. **Try the CLI:** `cargo xtask help` or `cargo xtask help-flows`
4. **Open the UI:** `http://localhost:8080/ui` for visual navigation
5. **Open an issue** if documentation is missing or unclear

