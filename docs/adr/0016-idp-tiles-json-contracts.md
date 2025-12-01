---
doc_type: adr
id: ADR-0016
title: "IDP Tile Architecture and JSON Contract Strategy"
status: Accepted
date: 2025-12-01
authors: Platform Team
stories: [US-TPL-PLT-001]
requirements: [REQ-TPL-PLATFORM-INTROSPECTION, REQ-TPL-AI-IDP-COMPAT]
acs: [AC-TPL-CLI-JSON-CORE, AC-TPL-CLI-JSON-OUTPUT, AC-TPL-AGENT-HINTS-SCHEMA]
adrs: []
---
<!-- doclint:disable orphan-version -->
<!-- ADR: This document contains historical version references as part of the decision record. -->

# ADR-0016: IDP Tile Architecture and JSON Contract Strategy

**Status**: Accepted
**Date**: 2025-12-01
**Authors**: Platform Team
**Related Requirements**: REQ-TPL-PLATFORM-INTROSPECTION, REQ-TPL-AI-IDP-COMPAT
**Related ADRs**: ADR-0003 (Spec as Source of Truth), ADR-0005 (Selftest as Single Gate)
**Related Design Docs**: docs/design/DESIGN-IDP-TILES.md

---

## Context

The platform exposes governance data via `/platform/*` HTTP endpoints for multiple consumers:

1. **Internal Developer Platforms (IDPs)**: Backstage, Port.io, custom dashboards need stable, documented contracts
2. **AI Agents**: Claude Code agents and automation scripts require predictable JSON structures
3. **Human developers**: CLI and UI consumers need readable, well-structured data
4. **Third-party integrations**: External tools need stable APIs with clear versioning

### Current State

The platform has 15+ `/platform/*` endpoints serving JSON data:

- **Governance**: `/platform/status`, `/platform/graph`, `/platform/coverage`
- **Documentation**: `/platform/docs/index`, `/platform/docs/<id>`
- **Tasks**: `/platform/tasks`, `/platform/tasks/suggest-next`, `/platform/tasks/graph`
- **Feedback**: `/platform/friction`, `/platform/questions`, `/platform/agent/hints`
- **DevEx**: `/platform/devex/flows`, `/platform/schema`

However, these endpoints lack:

1. **Formal contract documentation**: No JSON schemas, no versioning strategy
2. **Stability guarantees**: What changes are breaking? When do we bump versions?
3. **IDP integration patterns**: No guidance on which endpoints to use for common IDP tiles
4. **Migration paths**: No plan for evolving contracts without breaking consumers

### Problem Statement

Without formal contracts and stability guarantees:

- **IDP teams can't rely on our data**: Fear of breaking changes blocks integration
- **Agents are brittle**: Automation scripts break when field names change
- **Version ambiguity**: No clear way to communicate breaking vs. non-breaking changes
- **Discovery friction**: Consumers don't know which endpoints power which use cases

---

## Decision

We adopt an **IDP Tile + JSON Contract** architecture with explicit stability guarantees.

### 1. Primary IDP Tiles

We define 4 primary IDP tiles as first-class integration points:

#### Tile 1: Governance Health
- **Purpose**: High-level governance status for dashboard cards
- **Endpoint**: `/platform/status`
- **Schema**: `GovernanceStatus` (documented in docs/explanation/json-contracts.md)
- **Use case**: IDP health check, compliance dashboard, CI badge
- **Key fields**: `governance.all_passing`, `ac_coverage.{passing,total}`, `policy_health`

#### Tile 2: Documentation Health
- **Purpose**: Documentation inventory and health metrics
- **Endpoint**: `/platform/docs/index`
- **Schema**: `DocsIndex` (documented in docs/explanation/json-contracts.md)
- **Use case**: Doc catalog, stale doc detection, coverage tracking
- **Key fields**: `docs[].{id, title, doc_type, health}`, `summary.{total, by_type, health_dist}`

#### Tile 3: AC Coverage
- **Purpose**: Acceptance criteria test mapping and coverage
- **Endpoint**: `/platform/coverage`
- **Schema**: `ACCoverage` (documented in docs/explanation/json-contracts.md)
- **Use case**: Quality dashboard, spec-to-test traceability, CI reporting
- **Key fields**: `acceptance_criteria[].{id, status, test_count}`, `summary.{total, passing, failing}`

#### Tile 4: Task Hints (Agent Guidance)
- **Purpose**: Prioritized work recommendations for agents and developers
- **Endpoint**: `/platform/agent/hints`
- **Schema**: `AgentHints` (documented in docs/explanation/json-contracts.md)
- **Use case**: Work queue, agent task selection, developer dashboard
- **Key fields**: `tasks[].{id, priority, status, linked_acs, linked_reqs}`

### 2. JSON Contract Model

All platform endpoints follow an **additive-only evolution model**:

#### Contract Principles

1. **Additive changes only (within MINOR)**: New fields can be added without version bump
2. **No breaking changes in PATCH**: Field removal, renaming, type changes require MAJOR version bump
3. **Explicit versioning**: Template version (e.g., v3.3.5) governs all contracts
4. **Schema documentation**: All contracts documented in `docs/explanation/json-contracts.md`
5. **Deprecation policy**: Deprecated fields marked with `deprecated: true` + sunset date

#### Breaking vs. Non-Breaking Changes

**Non-breaking (safe in MINOR/PATCH):**
- Adding new optional fields
- Adding new endpoints
- Adding enum values (if client uses fallback)
- Relaxing validation (e.g., max_length 100 → 200)
- Changing field documentation/descriptions

**Breaking (requires MAJOR bump):**
- Removing fields
- Renaming fields
- Changing field types (e.g., string → array)
- Removing enum values
- Tightening validation (e.g., max_length 200 → 100)
- Changing response status codes for existing endpoints
- Changing endpoint URLs

#### Versioning Strategy

**Version source of truth**: Template version in `Cargo.toml` (e.g., `3.3.5`)

**Version communication:**
- HTTP header: `X-Template-Version: 3.3.5`
- JSON field: `/platform/status` includes `template_version: "3.3.5"`
- Schema docs: `docs/explanation/json-contracts.md` versioned alongside template

**Compatibility guarantee:**
- Within MAJOR version (e.g., v3.x.x): No breaking changes to JSON contracts
- MAJOR bump (e.g., v3 → v4): Breaking changes allowed, migration guide required
- MINOR bump (e.g., v3.3 → v3.4): Additive changes only
- PATCH bump (e.g., v3.3.5 → v3.3.6): Bug fixes, no schema changes

#### Deprecation Process

When a field must be removed:

1. **Mark deprecated** (MINOR release): Add `deprecated: true` + `sunset_date: "2026-01-01"`
2. **Add replacement field** (same release): New field with recommended migration
3. **Document migration** (docs/explanation/json-contracts.md): Before/after examples
4. **Warn in responses** (optional): Add `X-Deprecated-Fields: old_field_name` header
5. **Remove deprecated field** (MAJOR release): Actually remove, update schema docs

### 3. Schema Documentation

All contracts documented in `docs/explanation/json-contracts.md` with:

**For each endpoint:**
- **Purpose**: What problem does this solve?
- **Use cases**: IDP tiles, agent automation, developer workflows
- **Schema**: JSON structure with field types, constraints, examples
- **Stability**: Breaking/non-breaking change examples
- **Versioning**: When was this schema introduced? Last breaking change?
- **Deprecations**: Any sunset fields?

**Format:**
```markdown
## `/platform/status` (Governance Health)

**Purpose**: High-level governance status for dashboard tiles.

**Schema**:
```json
{
  "template_version": "3.3.5",          // string, semver
  "governance": {
    "all_passing": true,                // boolean
    "failed_phases": []                 // array<string>
  },
  "ac_coverage": {
    "total": 42,                        // integer, ≥0
    "passing": 40,                      // integer, ≥0
    "failing": 2                        // integer, ≥0
  }
}
```

**Introduced**: v3.3.0
**Last breaking change**: v3.0.0
**Stability**: Stable (no planned breaking changes)
```

### 4. Integration Guidance

`docs/explanation/json-contracts.md` includes **IDP integration recipes**:

**Example: Backstage plugin**
```yaml
# backstage/catalog-info.yaml
apiVersion: backstage.io/v1alpha1
kind: Component
metadata:
  name: my-service
  annotations:
    rust-template.dev/status-url: https://api.example.com/platform/status
    rust-template.dev/docs-url: https://api.example.com/platform/docs/index
spec:
  type: service
  lifecycle: production
```

**Example: Port.io integration**
```json
{
  "identifier": "governance-health",
  "title": "Governance Health",
  "blueprint": "rust-template-service",
  "properties": {
    "template_version": "3.3.5",
    "all_passing": true,
    "ac_coverage_pct": 95.2
  },
  "relations": {}
}
```

**Example: Agent automation**
```python
# Agent script: Fetch prioritized work
response = requests.get("http://localhost:8080/platform/agent/hints")
hints = response.json()

for task in hints["tasks"]:
    if task["status"] == "Todo" and task["priority"] == "high":
        print(f"Next task: {task['id']} - {task['title']}")
```

---

## Consequences

### Positive

1. **IDP teams can integrate safely**: Documented contracts + stability guarantees enable confident integration
2. **Agents are resilient**: Additive-only changes mean scripts don't break on MINOR/PATCH upgrades
3. **Clear versioning**: Template version communicates compatibility at a glance
4. **Discoverable**: `docs/explanation/json-contracts.md` is single source of truth for all contracts
5. **Migration-friendly**: Deprecation process gives consumers time to adapt
6. **Backward-compatible**: Within MAJOR version, all consumers continue to work

### Negative

1. **Maintenance overhead**: Must document all schemas and update on changes
2. **Design constraints**: Additive-only model limits refactoring options (can't easily rename bad field names)
3. **Deprecation lag**: Deprecated fields clutter responses until MAJOR bump
4. **Version coordination**: Breaking changes blocked until MAJOR release window

### Mitigation

- **Maintenance**: Automate schema generation where possible (Rust typeshare, JSON Schema export)
- **Design constraints**: Upfront design review for new fields (avoid naming mistakes)
- **Deprecation lag**: Use HTTP headers to warn consumers, minimize response bloat
- **Version coordination**: Plan MAJOR bumps carefully, batch breaking changes

---

## Rationale

### Why 4 Primary Tiles?

These 4 tiles cover 90% of IDP/agent use cases:

1. **Governance Health**: High-level "is this service healthy?" check
2. **Documentation Health**: "What docs exist? Are they up-to-date?"
3. **AC Coverage**: "What is tested? What is failing?"
4. **Task Hints**: "What should I work on next?"

Additional endpoints (`/platform/graph`, `/platform/friction`, etc.) serve specialized needs but aren't required for basic IDP integration.

### Why Additive-Only?

**Alternatives considered:**

1. **Versioned URLs (e.g., `/v1/platform/status`)**: Forces consumers to migrate explicitly
   - **Rejected**: Doubles maintenance burden, splits ecosystem, complicates discovery

2. **GraphQL**: Clients request only needed fields, schema evolution easier
   - **Rejected**: Adds complexity (GraphQL server, schema language), overkill for simple read-only data

3. **Breaking changes anytime**: Fast iteration, no constraints
   - **Rejected**: Destroys trust, blocks IDP adoption, breaks agents on every release

**Additive-only chosen because:**
- Simple to implement (just don't remove/rename fields)
- Compatible with SemVer (MAJOR for breaking, MINOR for additive)
- Industry standard (AWS, Stripe, GitHub all use additive-only)
- Enables long-term stability within MAJOR version

### Why Template Version (Not API Version)?

**Alternatives:**

1. **Separate API version** (e.g., `/api/v2/platform/status`)
   - **Rejected**: Disconnected from template evolution, confusing for consumers

2. **Per-endpoint versioning** (e.g., `/platform/status?version=2`)
   - **Rejected**: Fragments contract, makes compatibility tracking impossible

3. **Template version** (current decision)
   - **Chosen**: Single version for entire template, clear SemVer semantics, already tracked in `Cargo.toml`

Consumers can rely on template version to determine compatibility:
- v3.3.x → v3.4.x: Safe (additive only)
- v3.x.x → v4.x.x: Review migration guide (breaking changes)

---

## Implementation

### Phase 1: Document Existing Contracts (Immediate)

1. **Create `docs/explanation/json-contracts.md`**
   - Document all 4 primary tiles (schemas, use cases, examples)
   - Document all 15+ platform endpoints (brief schema, stability notes)
   - Add IDP integration recipes (Backstage, Port.io, agent scripts)

2. **Add version headers**
   - All `/platform/*` endpoints return `X-Template-Version: <version>` header
   - `/platform/status` includes `template_version` in JSON body

3. **Update `/platform/schema`**
   - Export JSON Schema for primary tiles (if not already present)
   - Link to `docs/explanation/json-contracts.md` for human-readable docs

**Deliverables:**
- `docs/explanation/json-contracts.md` (comprehensive schema docs)
- HTTP header `X-Template-Version` on all endpoints
- Updated `/platform/status` with `template_version` field

### Phase 2: Governance Integration (v3.4.x)

1. **Add contract validation to selftest**
   - Selftest step: Validate `/platform/status` response matches documented schema
   - Fail if required fields missing or types wrong

2. **Add to docs-check**
   - `cargo xtask docs-check` verifies `docs/explanation/json-contracts.md` exists
   - Warns if undocumented endpoints detected

3. **Add to precommit**
   - Pre-commit hook runs lightweight schema validation (if `/platform/*` endpoints changed)

**Deliverables:**
- Selftest validates contract compliance
- docs-check detects missing contract docs
- Pre-commit catches schema regressions

### Phase 3: Automated Schema Generation (v3.5.x)

1. **Rust typeshare integration**
   - Annotate response types with `#[typeshare]`
   - Generate TypeScript/JSON Schema from Rust types

2. **Schema export endpoint**
   - `/platform/schema/<endpoint>` returns JSON Schema for that endpoint
   - Example: `/platform/schema/status` → JSON Schema for `/platform/status`

3. **Documentation generation**
   - Auto-generate portions of `docs/explanation/json-contracts.md` from Rust types
   - Manual review required, but reduces drift

**Deliverables:**
- JSON Schema export for all primary tiles
- Automated schema generation (reduce manual drift)

---

## Testing Strategy

1. **Unit tests**: Rust response types → JSON serialization matches documented schema
2. **Integration tests**: HTTP requests to `/platform/*` validate response structure
3. **BDD scenarios**: Acceptance tests for contract stability (no breaking changes in MINOR)
4. **Selftest**: Contract validation step (Phase 2)
5. **CI checks**: Schema drift detection (Phase 2)

---

## Related Documentation

- **Design Doc**: `docs/design/DESIGN-IDP-TILES.md` (detailed tile specifications)
- **Contract Reference**: `docs/explanation/json-contracts.md` (all schemas, examples, recipes)
- **Agent Guide**: `docs/AGENT_GUIDE.md` (using `/platform/*` endpoints for automation)
- **Platform API Reference**: Existing docs in `docs/reference/` (endpoint catalog)

---

## Related Decisions

- **ADR-0003**: Spec as source of truth (contracts align with spec_ledger.yaml)
- **ADR-0005**: Selftest as single gate (contract validation in selftest)
- **ADR-0017**: Tier-1 selftest enforces contract stability

---

## Questions & Answers

**Q: What if I need to rename a badly-named field?**
A: Add new field (MINOR), deprecate old field, remove in next MAJOR. Example: `old_name` → `new_name` (both present for 6+ months).

**Q: What if a third-party IDP needs a custom field?**
A: File GitHub issue requesting field addition. If generally useful, add in MINOR release. If niche, recommend custom endpoint.

**Q: How do I know if a change is breaking?**
A: Check "Breaking vs. Non-Breaking Changes" section above. If unsure, treat as breaking (safer).

**Q: What if I'm experimenting with a new endpoint?**
A: Mark as experimental in docs (`Stability: Experimental`). No contract guarantees until marked `Stable`.

**Q: How do I migrate to a new MAJOR version?**
A: Read migration guide in `docs/explanation/json-contracts.md` (published with MAJOR release). Update client code, test, deploy.

---

## Approval & Sign-Off

- **Decision Owner**: Platform Team
- **Approved**: 2025-12-01
- **Implementation**: Phase 1 (v3.3.5), Phase 2 (v3.4.x), Phase 3 (v3.5.x)

---

**Version**: 1.0.0
**Last Updated**: 2025-12-01
