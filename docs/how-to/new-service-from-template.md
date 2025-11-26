---
id: GUIDE-TPL-NEW-SERVICE-001
title: Create a New Service from Template
doc_type: how-to
status: published
audience: developers, platform-engineers
tags: [onboarding, setup, fork, greenfield, customization]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-ONBOARDING, REQ-PLT-DOC-TEMPLATES]
acs: [AC-PLT-001, AC-PLT-008]
adrs: [ADR-0005]
last_updated: 2025-11-26
---

# Create a New Service from Template

This guide walks you through forking the Rust-as-Spec template to launch a new governed service (e.g., Knowledge Hub, Order Service, Audit Log).

> **Quick Reference:** See [examples/fork-customization](../../examples/fork-customization/) for sample files showing what to customize.

---

## Prerequisites

- **Rust 1.70+** and **Cargo**
- **Nix** (optional but recommended for full selftest)
- **Git** and a GitHub account
- Basic familiarity with **Rust**, **YAML**, and **BDD** (Gherkin/Cucumber)

---

## Step 1: Fork and Rename

### 1.1 Create repo from template

```bash
# On GitHub: Use "Use this template" → Create new repository
# Or via CLI:
gh repo create myservice --template your-org/Rust-Template --public
cd myservice
```

### 1.2 Update core metadata

Edit these files with your service identity:

**`service_metadata.yaml`** (create if missing):

```yaml
service_id: "knowledge-hub"
name: "Knowledge Hub"
description: "Central repository for domain knowledge and entity definitions"
tags: [platform, knowledge-base, core]
version: "0.1.0"
owner_team: "platform"
```

**`README.md`** – Update header and service description:

```markdown
# Knowledge Hub (v0.1.0)

Central repository for domain knowledge and entity definitions.

A governed Rust service where specs, tests, docs, and infra all agree.
```

**`CLAUDE.md`** – Update service-specific guidance (optional but recommended):

```markdown
# Knowledge Hub Development

Service ID: `knowledge-hub`
Task prefix: `TASK-KH-`
Requirement prefix: `REQ-KH-`
AC prefix: `AC-KH-`

Domain routes:
- `POST /api/entities` – register a new entity type
- `GET /api/entities/{id}` – retrieve entity definition
- `GET /health`, `/version`, `/metrics` – platform endpoints (inherited from template)

Entity types: Document, Glossary, Relationship, MediaAsset

See `TEMPLATE-CONTRACTS.md` for inherited patterns.
```

---

## Step 2: Define Your Domain

### 2.1 Add first user story to `specs/spec_ledger.yaml`

Under `stories:`, add your domain story alongside the inherited template core:

```yaml
  - id: US-KH-001
    title: "Entity Registration"
    description: >
      As a domain expert, I want to register new entity types (Document, Glossary)
      so the platform can catalog and link them.
    adr: ADR-0001  # Hexagonal architecture
    requirements:
      - id: REQ-KH-ENTITY-REGISTER
        title: "Entity Type Registration API"
        tags: [core, api]
        must_have_ac: true
        acceptance_criteria:
          - id: AC-KH-001
            text: "POST /api/entities registers a new entity type and returns 201 with entity ID"
            tags: [core]
            must_have_ac: true
            tests: [ { type: bdd, tag: "@AC-KH-001", file: "specs/features/entity_registration.feature" } ]

          - id: AC-KH-002
            text: "POST /api/entities validates schema: name (required), description (optional), tags (array)"
            tags: [core]
            must_have_ac: true
            tests: [ { type: bdd, tag: "@AC-KH-002", file: "specs/features/entity_registration.feature" } ]
```

**Keep template core stories** (`US-TPL-001`, `US-TPL-PLT-001`) – you inherit those requirements.

### 2.2 Create BDD feature file

Create `specs/features/entity_registration.feature`:

```gherkin
# Entity Type Registration
# Schema: spec_ledger.yaml v1.0
# Last Updated: 2025-11-26

Feature: Entity Registration
  As a domain expert
  I want to register new entity types
  So the platform can catalog and link knowledge artifacts

  @AC-KH-001 @smoke
  Scenario: Register a new entity type
    When I POST /api/entities with:
      | name        | Document      |
      | description | Knowledge doc |
      | tags        | knowledge,api |
    Then I receive 201 with "entityId" and "name" = "Document"

  @AC-KH-002
  Scenario: Reject entity registration without name
    When I POST /api/entities with:
      | description | Missing name |
    Then I receive 400 with error code "VALIDATION_ERROR"
```

---

## Step 3: Seed Your Tasks

### 3.1 Add initial tasks to `specs/tasks.yaml`

```yaml
tasks:
  - id: TASK-KH-BOOTSTRAP-001
    title: "Bootstrap Entity Registration Service"
    requirement: REQ-KH-ENTITY-REGISTER
    acs: [AC-KH-001, AC-KH-002]
    status: open
    owner: team-platform
    labels: [core, api, ac_first]
    summary: "Implement entity registration endpoint with validation"
    recommended_flows: [ac_first]
    docs:
      design: []
      plan: []

  - id: TASK-KH-ENTITY-QUERY-001
    title: "Implement Entity Retrieval"
    requirement: REQ-KH-ENTITY-QUERY
    acs: [AC-KH-003]
    status: open
    owner: team-platform
    labels: [core, api]
    summary: "GET /api/entities/{id} returns entity definition"
    recommended_flows: [ac_first]
    depends_on:
      - TASK-KH-BOOTSTRAP-001
    docs:
      design: []
      plan: []

  - id: TASK-KH-PERSISTENCE-001
    title: "Add Entity Store (SQLite/Postgres)"
    requirement: REQ-KH-PERSISTENCE
    acs: [AC-KH-004]
    status: open
    owner: team-platform
    labels: [infrastructure, storage]
    summary: "Implement persistent storage for entity definitions"
    recommended_flows: [ac_first]
    depends_on:
      - TASK-KH-BOOTSTRAP-001
    docs:
      design: []
      plan: []
```

---

## Step 4: Validate the Baseline

### 4.1 Check environment and inherit template ACs

```bash
cd myservice
cargo xtask doctor
```

You should see:
- Template core ACs listed (health, version, errors, metrics)
- Your new ACs listed (entity registration, etc.)

### 4.2 Run selftest to baseline

```bash
cargo xtask selftest
```

**Expect**: Template core ACs pass (inherited), your ACs fail (not implemented yet). This is OK.

### 4.3 Confirm CI is set up

Check `.github/workflows/ci.yml` runs `cargo xtask selftest` on push/PR. Update branch protection to require it passing.

---

## Step 5: Make It Agent-Native

### 5.1 Verify CLAUDE.md is discoverable

Ensure `CLAUDE.md` contains:
- Service ID and task/requirement/AC prefixes
- Domain routes and key entities
- Link to inherited template patterns

### 5.2 Test AC bundle generation

```bash
cargo xtask bundle implement_ac
```

You should see your BDD features and ACs listed in the bundle.

### 5.3 Test next-step suggestions

```bash
cargo xtask ac-status
```

Should show your AC tasks and their status.

---

## Step 6: Implement Your First Vertical Slice

### 6.1 Pick one AC (e.g., AC-KH-001: basic entity registration)

```bash
# Implement code and tests for entity registration
cargo xtask test-ac AC-KH-001
```

### 6.2 Validate locally

```bash
cargo xtask check
cargo xtask test-changed
```

### 6.3 Get to green

```bash
cargo xtask selftest
```

Once selftest passes for your AC, you're ready for review.

---

## Step 7: Next Steps

### 7.1 Iterative development

For each new feature:
1. Add REQ + ACs to `spec_ledger.yaml`
2. Write BDD scenarios in `specs/features/*.feature`
3. Add tasks to `specs/tasks.yaml`
4. Implement code + tests
5. Run `cargo xtask selftest` to validate

### 7.2 Backport template improvements

Periodically pull in updates from the template repo:

```bash
git remote add template https://github.com/your-org/Rust-Template.git
git fetch template main
git merge template/main --no-ff -m "chore: backport template improvements"
```

### 7.3 Maintain alignment

- Keep `spec_ledger.yaml` in sync with code and tests
- Use ADRs for architectural decisions
- Document tradeoffs in `FRICTION_LOG.md` if guidance is unclear

---

## Checklist: "Ready for Team"

- [ ] Service metadata (ID, name, description) updated
- [ ] First user story + 2-3 requirements + ACs added to spec ledger
- [ ] BDD feature file created and scenarios tagged with AC IDs
- [ ] Tasks created and linked to ACs
- [ ] `cargo xtask selftest` runs without fatal errors (ACs may be red, that's OK)
- [ ] CLAUDE.md has service-specific guidance
- [ ] README updated with service description
- [ ] CI workflow enforces selftest
- [ ] First AC implemented and passing (optional but recommended)

You're now ready to share with your team and start iterating!
