---
doc_type: explanation
id: EXPLANATION-TPL-IDP-POSITIONING-001
title: "IDP Positioning: Integrating Platform Cells with Internal Developer Portals"
stories: [US-TPL-PLT-001]
requirements:
  - REQ-TPL-PLATFORM-INTROSPECTION
  - REQ-TPL-METADATA-CONSISTENT
  - REQ-TPL-PLATFORM-SCHEMA
  - REQ-TPL-REL-BUNDLE
acs:
  - AC-TPL-PLATFORM-GRAPH
  - AC-TPL-PLATFORM-DEVEX
  - AC-TPL-PLATFORM-DOCS
  - AC-TPL-PLATFORM-SCHEMA
  - AC-TPL-METADATA-COMPLETE
  - AC-TPL-REL-EVIDENCE
adrs: [ADR-0001, ADR-0004, ADR-0005]
status: published
last_updated: 2025-11-26
---

# IDP Positioning: Integrating Platform Cells with Internal Developer Portals

This document explains how Internal Developer Portals (IDPs) like Backstage, Port, Cortex, or OpsLevel should integrate with Rust-as-Spec platform cells.

---

## Executive Summary

**Platform cells are self-describing services.** Integrate via runtime APIs or static artifacts—no manual catalog maintenance required.

**Runtime APIs:** `/platform/status` (governance health), `/platform/graph` (traceability), `/platform/tasks` (work items), `/platform/agent/hints` (task suggestions), `/platform/schema` (OpenAPI contract), `/platform/devex/flows` (commands), `/platform/docs/index` (doc inventory)

**Evidence Files:** `service_metadata.yaml` (team/tier/links), `feature_status.md` (AC status), `release_evidence/v*.md` (governance bundles)

**Example Widgets:**
```yaml
# Governance Health Card
Service: rust-template | Selftest: ✅ PASS | AC Coverage: 93% (56/60) | Tasks: 2 todo, 1 in_progress

# Compliance Traceability
SOC2 Access Control → AC-PLT-006 (RBAC) ✅ PASS | AC-PLT-007 (Audit) ✅ PASS
Evidence: specs/features/platform_security.feature
```

**Integration Strategy:** API scraping for live state, repository scraping for history. Cache: `/platform/status` (5min), `/platform/graph` (1hr), `/platform/tasks` (1min). Link to `/ui` for exploration.

**See below for:** API schemas, integration patterns (Backstage, Port, OpsLevel), use cases, and best practices.

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Overview](#overview)
3. [Platform Cell Contract](#platform-cell-contract)
4. [Integration Patterns](#integration-patterns)
5. [Platform API Endpoints](#platform-api-endpoints)
6. [Metadata and Documentation](#metadata-and-documentation)
7. [IDP Use Cases](#idp-use-cases)
8. [Integration Examples](#integration-examples)
9. [Best Practices](#best-practices)

---

## Overview

### What is a Platform Cell?

A **platform cell** is a self-governing service instance built from this template. Each cell:

- Exposes its governance state via runtime HTTP APIs (`/platform/*`)
- Maintains structured metadata (`specs/service_metadata.yaml`)
- Auto-generates feature status (`docs/feature_status.md`)
- Produces release evidence (`release_evidence/*.md`)
- Enforces governance contracts via `cargo xtask selftest`

### IDP Integration Philosophy

**Platform cells are self-describing.** They expose everything an IDP needs:

- Service metadata (team, tier, criticality, links)
- Governance graph (stories → requirements → ACs → tests)
- Feature status (what's implemented, what's tested)
- Release evidence (what changed, why, with traceability)
- Runtime health and platform schema

**IDPs should consume, not duplicate.** Rather than maintaining separate service catalogs, IDPs should scrape platform cell APIs to get the single source of truth.

---

## Platform Cell Contract

Every platform cell provides:

### Runtime APIs

| Endpoint | Purpose |
|----------|---------|
| `/platform/status` | High-level governance health summary |
| `/platform/graph` | Complete governance graph (stories, requirements, ACs, docs, commands) |
| `/platform/devex/flows` | DevEx commands and workflows |
| `/platform/docs/index` | Document index with traceability to requirements |
| `/platform/schema` | Machine-readable platform contract (OpenAPI/JSON Schema) |
| `/platform/tasks` | Current tasks (CLI/HTTP for task management) |
| `/platform/agent/hints` | Prioritized task suggestions for agents |
| `/ui` | Web UI for exploring governance, flows, and graphs |

### Static Artifacts

| Artifact | Location | Purpose |
|----------|----------|---------|
| Service Metadata | `specs/service_metadata.yaml` | Ownership, tier, criticality, links |
| Feature Status | `docs/feature_status.md` | Auto-generated AC/test status |
| Release Evidence | `release_evidence/v*.md` | Per-version changelog with governance traceability |
| Spec Ledger | `specs/spec_ledger.yaml` | Stories → Requirements → ACs → tests |
| Config Schema | `specs/config_schema.yaml` | Service configuration contract |

---

## Integration Patterns

### Pattern 1: API Scraping (Recommended)

**IDP periodically queries platform cell runtime APIs.**

**Pros:**
- Always up-to-date (queries live service)
- No file parsing or Git cloning required
- Service is self-describing at runtime

**Cons:**
- Requires service to be running
- Network dependency

**Implementation:**

```python
# Example: Backstage catalog processor
import requests

def scrape_platform_cell(service_url):
    """Fetch governance metadata from platform cell."""

    # Get service metadata
    status = requests.get(f"{service_url}/platform/status").json()

    # Get governance graph
    graph = requests.get(f"{service_url}/platform/graph").json()

    # Get docs index
    docs = requests.get(f"{service_url}/platform/docs/index").json()

    # Get platform schema
    schema = requests.get(f"{service_url}/platform/schema").json()

    return {
        "service_id": status.get("service_id"),
        "health": status.get("governance_health"),
        "governance_graph": graph,
        "documentation": docs,
        "schema": schema,
    }
```

### Pattern 2: Repository Scraping

**IDP reads static artifacts from Git repository.**

**Pros:**
- No runtime dependency
- Can be used in CI/CD pipelines
- Works for services not yet deployed

**Cons:**
- Requires Git access and cloning
- Slightly stale (last commit, not live state)

**Implementation:**

```python
# Example: Port.io ingestion
import yaml
from pathlib import Path

def scrape_from_repo(repo_path):
    """Extract metadata from Git repository."""

    # Read service metadata
    with open(f"{repo_path}/specs/service_metadata.yaml") as f:
        metadata = yaml.safe_load(f)

    # Read feature status
    with open(f"{repo_path}/docs/feature_status.md") as f:
        feature_status = f.read()

    # Read spec ledger
    with open(f"{repo_path}/specs/spec_ledger.yaml") as f:
        spec_ledger = yaml.safe_load(f)

    # Find latest release evidence
    release_evidence = sorted(
        Path(f"{repo_path}/release_evidence").glob("v*.md"),
        reverse=True
    )[0].read_text()

    return {
        "service_id": metadata["service_id"],
        "team": metadata["ownership"]["team"],
        "tier": metadata["lifecycle"]["tier"],
        "feature_status": feature_status,
        "latest_release": release_evidence,
        "requirements": spec_ledger["requirements"],
    }
```

### Pattern 3: Hybrid (Best of Both)

**IDP uses API scraping for runtime state, repository scraping for historical data.**

**Use API scraping for:**
- Current governance health (`/platform/status`)
- Live task status (`/platform/tasks`)
- Real-time feature status (via `/platform/graph`)

**Use repository scraping for:**
- Historical release evidence (`release_evidence/*.md`)
- Full spec ledger and BDD scenarios (too large for API responses)
- CI/CD integration (pre-deployment checks)

---

## Platform API Endpoints

### `/platform/status`

**Purpose:** High-level governance health summary

**Response:**

```json
{
  "service_id": "rust-template",
  "display_name": "Rust-as-Spec Platform Cell",
  "template_version": "3.3.1",
  "governance_health": "healthy",
  "selftest_status": "PASS",
  "policy_status": "ok",
  "ac_coverage": {
    "total": 60,
    "passing": 56,
    "unknown": 4,
    "failing": 0
  },
  "tasks_summary": {
    "todo": 2,
    "in_progress": 1,
    "done": 45
  }
}
```

**IDP Use:** Service health dashboard, compliance scorecards

---

### `/platform/graph`

**Purpose:** Complete governance graph (stories → requirements → ACs → tests → docs)

**Response:**

```json
{
  "nodes": [
    { "id": "US-TPL-001", "label": "Service Core", "type": "story" },
    { "id": "REQ-TPL-HEALTH", "label": "Health Check", "type": "requirement" },
    { "id": "AC-TPL-001", "label": "Health endpoint returns 200", "type": "ac" },
    { "id": "test-health", "label": "health_check.feature", "type": "test" }
  ],
  "edges": [
    { "source": "US-TPL-001", "target": "REQ-TPL-HEALTH", "type": "contains" },
    { "source": "REQ-TPL-HEALTH", "target": "AC-TPL-001", "type": "satisfies" },
    { "source": "AC-TPL-001", "target": "test-health", "type": "tested_by" }
  ]
}
```

**IDP Use:**
- Traceability explorer (requirement → AC → test)
- Compliance mapping (audit requirement → code implementation)
- Dependency analysis (which ACs depend on which requirements)

---

### `/platform/devex/flows`

**Purpose:** DevEx commands and workflows

**Response:**

```json
{
  "commands": {
    "doctor": {
      "category": "onboarding",
      "summary": "Run full environment health check",
      "required": true,
      "command": "cargo xtask doctor"
    },
    "selftest": {
      "category": "validation",
      "summary": "Run full governance selftest",
      "required": true,
      "command": "cargo xtask selftest"
    }
  },
  "flows": {
    "onboarding": {
      "name": "Developer Onboarding",
      "steps": ["doctor", "check", "test"],
      "description": "First-run workflow for new developers"
    },
    "feature_development": {
      "name": "Governed Feature Development",
      "steps": ["bundle", "implement", "test-ac", "selftest"],
      "description": "Implement a new AC with full governance"
    }
  }
}
```

**IDP Use:**
- Developer onboarding guides
- Workflow automation (trigger flows from IDP)
- DevEx analytics (which commands are most used)

---

### `/platform/docs/index`

**Purpose:** Document index with traceability to requirements

**Response:**

```json
{
  "schema_version": "1.0",
  "docs": [
    {
      "id": "DESIGN-TPL-HEALTH-001",
      "file": "docs/design/health-endpoint.md",
      "doc_type": "design_doc",
      "stories": ["US-TPL-001"],
      "requirements": ["REQ-TPL-HEALTH"],
      "acs": ["AC-TPL-001"],
      "adrs": ["ADR-0001", "ADR-0003"]
    }
  ]
}
```

**IDP Use:**
- Documentation portal (link docs to features)
- Compliance evidence (which docs cover which requirements)
- Onboarding (show relevant docs for a user story)

---

### `/platform/schema`

**Purpose:** Machine-readable platform contract (OpenAPI/JSON Schema)

**Response:**

```json
{
  "openapi": "3.0.0",
  "info": {
    "title": "Rust-as-Spec Platform API",
    "version": "3.3.1"
  },
  "paths": {
    "/platform/status": { "get": { "responses": { "200": { ... } } } },
    "/platform/graph": { "get": { "responses": { "200": { ... } } } }
  }
}
```

**IDP Use:**
- API documentation
- Contract testing (validate IDP scraper against schema)
- Schema evolution tracking

---

### `/platform/tasks`

**Purpose:** Current tasks (CLI/HTTP for task management)

**Response:**

```json
{
  "tasks": [
    {
      "id": "TASK-TPL-STATUS-CLI-001",
      "title": "Implement CLI governance status dashboard",
      "status": "InProgress",
      "requirement": "REQ-PLT-STATUS-CLI",
      "acs": ["AC-PLT-017"],
      "owner": "agent"
    }
  ]
}
```

**IDP Use:**
- Task tracking dashboard
- Team workload visibility
- Sprint planning integration

---

### `/platform/agent/hints`

**Purpose:** Prioritized task suggestions for agents

**Response:**

```json
{
  "hints": [
    {
      "task_id": "TASK-TPL-FIX-AUDIT-001",
      "title": "Fix cargo-audit findings",
      "priority": "high",
      "status": "Todo",
      "requirement": "REQ-PLT-SECURITY",
      "acs": ["AC-PLT-006", "AC-PLT-007"],
      "suggested_commands": [
        "cargo xtask bundle implement_ac AC-PLT-006",
        "cargo xtask test-ac AC-PLT-006",
        "cargo xtask selftest"
      ]
    }
  ]
}
```

**IDP Use:**
- AI assistant integration (surface tasks to agents)
- Automated remediation (trigger suggested commands)
- Prioritized backlog display

---

## Metadata and Documentation

### `specs/service_metadata.yaml`

**Structure:**

```yaml
service_id: rust-template
display_name: "Rust-as-Spec Platform Cell"
description: >
  Governed Rust service template with selftest, platform APIs, and agent support.
template_version: "3.3.1"

ownership:
  team: plat-governance
  email: platform-governance@example.com
  slack: "#team-platform"

lifecycle:
  tier: 1                          # 1 (critical), 2 (important), 3 (best-effort)
  data_class: internal             # public, internal, confidential, restricted
  criticality: platform            # platform, customer-facing, internal-tool
  languages: [rust]
  runtime: [kubernetes]

links:
  roadmap: "docs/ROADMAP.md"
  kernel_contract: "docs/runbooks/platform-kernel.md"
  agent_guide: "docs/AGENT_GUIDE.md"
  feature_status: "docs/feature_status.md"
  support: "docs/reference/platform-support.md"
  ui: "http://localhost:8080/ui"
  status: "http://localhost:8080/platform/status"
  repo: "https://github.com/EffortlessMetrics/Rust-Template"

tags:
  - rust
  - idp
  - governance
  - template
```

**IDP Mapping:**

| Field | IDP Use |
|-------|---------|
| `service_id` | Unique identifier for catalog entity |
| `display_name` | Human-readable service name |
| `ownership.team` | Owner in IDP (team entity reference) |
| `ownership.email` | Contact email |
| `ownership.slack` | Support channel |
| `lifecycle.tier` | Service tier/SLA classification |
| `lifecycle.data_class` | Data sensitivity label |
| `lifecycle.criticality` | Business criticality |
| `links.*` | Deep links to docs, roadmap, status |
| `tags` | Service taxonomy tags |

**Example: Backstage catalog-info.yaml generation**

```yaml
# Generated from service_metadata.yaml
apiVersion: backstage.io/v1alpha1
kind: Component
metadata:
  name: rust-template
  description: Governed Rust service template with selftest, platform APIs, and agent support.
  annotations:
    github.com/project-slug: EffortlessMetrics/Rust-Template
  tags:
    - rust
    - idp
    - governance
    - template
  links:
    - url: http://localhost:8080/ui
      title: Platform UI
    - url: http://localhost:8080/platform/status
      title: Governance Status
    - url: docs/ROADMAP.md
      title: Roadmap
spec:
  type: service
  lifecycle: production
  owner: plat-governance
  system: platform
```

---

### `docs/feature_status.md`

**Auto-generated by:** `cargo xtask ac-status`

**Purpose:** Real-time AC/test status report

**Structure:**

```markdown
# Feature Status

Auto-generated AC status from acceptance (BDD) and unit tests.

## AC Status Summary

| AC ID | Story | Requirement | Status | Tests (executed/total) |
|-------|-------|-------------|--------|------------------------|
| AC-TPL-001 | US-TPL-001 | REQ-TPL-HEALTH | [PASS] pass | 1 / 1 |
| AC-TPL-002 | US-TPL-001 | REQ-TPL-VERSION | [PASS] pass | 1 / 1 |
| AC-TPL-FLOW-IDEMPOTENT | US-TPL-PLT-001 | REQ-TPL-FLOW-IDEMPOTENCY | [UNKNOWN] unknown | 0 / 1 |

## Unmapped ACs

ACs with no mapped or executed tests:

- AC-TPL-FLOW-IDEMPOTENT: Running cargo xtask selftest produces stable outputs.
```

**IDP Use:**
- Feature completeness dashboard
- Compliance scorecard (% ACs with tests)
- Quality gate (block deployment if ACs are failing)

---

### `release_evidence/v*.md`

**Generated by:** `cargo xtask release-bundle <version>`

**Purpose:** Per-release governance evidence bundle

**Structure:**

```markdown
# Release Evidence v3.1.0
Generated at: 2025-01-01T00:00:00Z

---

## Tasks Completed
- TASK-TPL-STATUS-CLI-001 (REQ-PLT-STATUS-CLI) [AC-PLT-017]

## Acceptance Criteria & Requirements
- REQ-TPL-HEALTH: Health check endpoint implementation
- REQ-TPL-VERSION: Version endpoint implementation

## Architecture Decisions
- ADR-0001: Platform introspection via HTTP endpoints
- ADR-0005: Selftest as governance contract

## Git Changelog
- previous tag: v3.0.0
- commit: feat(platform): add /platform/status endpoint
- commit: feat(tasks): lock CLI/HTTP semantics and align with BDD

## Governance Status
- Selftest: PASS
- Policy Status: ok
- AC Coverage: 56/60 passing

## Resolved Friction
- Fixed cargo-audit findings (TASK-TPL-FIX-AUDIT-001)
```

**IDP Use:**
- Release notes portal
- Compliance audit trail (what changed, why, with traceability)
- Rollback analysis (which ACs were affected by a release)

---

## IDP Use Cases

### Use Case 1: Service Catalog

**Goal:** Maintain an up-to-date catalog of all services

**Implementation:**

1. **Scrape service metadata** from `specs/service_metadata.yaml` or `/platform/status`
2. **Index services** by `service_id`, team, tier, tags
3. **Link to platform UI** via `links.ui` field
4. **Show governance health** from `/platform/status` (selftest status, AC coverage)

**Example: Backstage Component**

```yaml
apiVersion: backstage.io/v1alpha1
kind: Component
metadata:
  name: rust-template
  annotations:
    platform.governance/status: "healthy"
    platform.governance/ac-coverage: "93%"
    platform.governance/selftest: "PASS"
spec:
  type: service
  lifecycle: production
  owner: plat-governance
```

---

### Use Case 2: Compliance Dashboard

**Goal:** Show which services meet compliance requirements

**Implementation:**

1. **Scrape governance graph** from `/platform/graph`
2. **Map compliance requirements** to ACs (e.g., "SOC2 controls" → AC-PLT-006)
3. **Display AC status** from `feature_status.md`
4. **Show coverage gaps** (unmapped ACs)

**Example Dashboard:**

| Service | Compliance Req | AC | Status | Evidence |
|---------|---------------|-----|--------|----------|
| rust-template | SOC2: Access Control | AC-PLT-006 | ✅ PASS | [BDD scenario](specs/features/platform_security.feature) |
| rust-template | SOC2: Audit Logging | AC-PLT-007 | ✅ PASS | [BDD scenario](specs/features/platform_security.feature) |
| rust-template | SOC2: Data Retention | AC-PLT-008 | ✅ PASS | [BDD scenario](specs/features/platform_security.feature) |

---

### Use Case 3: Developer Onboarding

**Goal:** Help new developers get started quickly

**Implementation:**

1. **Scrape DevEx flows** from `/platform/devex/flows`
2. **Generate onboarding checklist** from `flows.onboarding.steps`
3. **Link to docs** from `/platform/docs/index`
4. **Show agent guide** from `links.agent_guide`

**Example Onboarding Page:**

```markdown
# Onboarding: rust-template

## Setup Steps (from DevEx flow: "onboarding")
- [ ] Run `cargo xtask doctor` (health check)
- [ ] Run `cargo xtask check` (linting)
- [ ] Run `cargo xtask test` (unit tests)

## Key Documentation
- [Agent Guide](docs/AGENT_GUIDE.md)
- [Roadmap](docs/ROADMAP.md)
- [Platform Kernel](docs/runbooks/platform-kernel.md)

## Get Help
- Team: plat-governance
- Slack: #team-platform
- Email: platform-governance@example.com
```

---

### Use Case 4: Release Management

**Goal:** Track releases and their governance evidence

**Implementation:**

1. **Scrape release evidence** from `release_evidence/*.md`
2. **Display changelog** with task/AC traceability
3. **Show selftest status** per release
4. **Link to ADRs** (architecture decisions)

**Example Release View:**

```markdown
# Release v3.1.0 (2025-01-01)

## Governance Status
✅ Selftest: PASS
✅ Policy Status: ok
✅ AC Coverage: 93% (56/60)

## Tasks Completed
- TASK-TPL-STATUS-CLI-001: Implement CLI governance status dashboard

## Requirements & ACs
- REQ-PLT-STATUS-CLI → AC-PLT-017 (✅ PASS)

## Architecture Decisions
- ADR-0001: Platform introspection via HTTP endpoints
- ADR-0005: Selftest as governance contract

## Changelog
- feat(platform): add /platform/status endpoint
- feat(tasks): lock CLI/HTTP semantics and align with BDD
```

---

### Use Case 5: AI Assistant Integration

**Goal:** Enable AI agents to discover and execute work

**Implementation:**

1. **Scrape task hints** from `/platform/agent/hints`
2. **Surface prioritized tasks** to AI assistant (e.g., Claude, GPT-4)
3. **Provide suggested commands** from `suggested_commands` field
4. **Track task completion** via `/platform/tasks` API

**Example: Slack Bot Integration**

```python
# Slack bot that suggests tasks to developers
def suggest_task_for_service(service_url):
    hints = requests.get(f"{service_url}/platform/agent/hints").json()

    if not hints["hints"]:
        return "No pending tasks! 🎉"

    task = hints["hints"][0]  # Highest priority task

    return f"""
🤖 Suggested Task for {task['task_id']}:

**{task['title']}**
- Status: {task['status']}
- Requirement: {task['requirement']}
- ACs: {', '.join(task['acs'])}

**Recommended commands:**
```
{chr(10).join(task['suggested_commands'])}
```
    """
```

---

## Integration Examples

### Example 1: Backstage Plugin

**Goal:** Display platform cell governance in Backstage

**Implementation:**

```typescript
// backstage-plugin-platform-governance/src/components/GovernanceCard.tsx
import React from 'react';
import { InfoCard } from '@backstage/core-components';
import { useApi } from '@backstage/core-plugin-api';

export const GovernanceCard = ({ entity }: { entity: Entity }) => {
  const [status, setStatus] = React.useState<any>(null);

  React.useEffect(() => {
    const platformUrl = entity.metadata.annotations?.['platform.governance/url'];
    if (platformUrl) {
      fetch(`${platformUrl}/platform/status`)
        .then(res => res.json())
        .then(setStatus);
    }
  }, [entity]);

  if (!status) return <InfoCard title="Governance">Loading...</InfoCard>;

  return (
    <InfoCard title="Governance Status">
      <dl>
        <dt>Selftest:</dt>
        <dd>{status.selftest_status === 'PASS' ? '✅ PASS' : '❌ FAIL'}</dd>

        <dt>AC Coverage:</dt>
        <dd>{status.ac_coverage.passing} / {status.ac_coverage.total}</dd>

        <dt>Policy Status:</dt>
        <dd>{status.policy_status === 'ok' ? '✅ OK' : '⚠️ Issues'}</dd>
      </dl>
    </InfoCard>
  );
};
```

---

### Example 2: Port.io Ingestion Script

**Goal:** Sync platform cells into Port.io service catalog

**Implementation:**

```python
# port-sync.py
import requests
import yaml

PORT_API_URL = "https://api.getport.io/v1"
PORT_CLIENT_ID = "your-client-id"
PORT_CLIENT_SECRET = "your-client-secret"

def sync_platform_cell_to_port(repo_path):
    """Sync platform cell metadata to Port.io."""

    # Read service metadata
    with open(f"{repo_path}/specs/service_metadata.yaml") as f:
        metadata = yaml.safe_load(f)

    # Read feature status
    with open(f"{repo_path}/docs/feature_status.md") as f:
        feature_status = f.read()
        ac_coverage = parse_ac_coverage(feature_status)

    # Create Port entity
    entity = {
        "identifier": metadata["service_id"],
        "title": metadata["display_name"],
        "properties": {
            "team": metadata["ownership"]["team"],
            "tier": metadata["lifecycle"]["tier"],
            "ac_coverage": ac_coverage,
            "links": metadata["links"],
            "tags": metadata["tags"],
        }
    }

    # Upsert to Port
    headers = {"Authorization": f"Bearer {get_port_token()}"}
    requests.post(
        f"{PORT_API_URL}/blueprints/service/entities",
        json=entity,
        headers=headers
    )

def parse_ac_coverage(feature_status_md):
    """Parse AC coverage from feature_status.md."""
    # Example: "| AC-TPL-001 | ... | [PASS] pass | 1 / 1 |"
    # Returns: {"passing": 56, "total": 60, "percentage": 93}
    # Implementation left as exercise
    pass
```

---

### Example 3: OpsLevel Integration

**Goal:** Sync governance checks to OpsLevel scorecards

**Implementation:**

```ruby
# opslevel-sync.rb
require 'opslevel'
require 'yaml'

client = OpsLevel::Client.new(api_token: ENV['OPSLEVEL_API_TOKEN'])

def sync_platform_cell(repo_path)
  # Read service metadata
  metadata = YAML.load_file("#{repo_path}/specs/service_metadata.yaml")

  # Get service in OpsLevel
  service = client.services.find_by(alias: metadata['service_id'])

  # Read feature status
  feature_status = File.read("#{repo_path}/docs/feature_status.md")
  ac_coverage = parse_ac_coverage(feature_status)

  # Update OpsLevel checks
  client.checks.create(
    service: service,
    name: "AC Coverage",
    type: "generic",
    status: ac_coverage[:percentage] >= 90 ? "passing" : "failing",
    message: "#{ac_coverage[:passing]}/#{ac_coverage[:total]} ACs passing"
  )

  client.checks.create(
    service: service,
    name: "Selftest Status",
    type: "generic",
    status: selftest_passing?(repo_path) ? "passing" : "failing",
    message: "Governance selftest status"
  )
end
```

---

## Best Practices

### 1. Prefer API Scraping Over File Parsing

**Why:** APIs are versioned, stable, and self-describing. File parsing is brittle and depends on format stability.

**Exception:** Use file parsing for historical data (release evidence, changelogs) or when service is not running (CI/CD).

---

### 2. Use `/platform/schema` for Contract Validation

**Before deploying an IDP scraper:**

1. Fetch `/platform/schema` from a platform cell
2. Generate client code or validation schemas
3. Test your scraper against the schema
4. Subscribe to schema updates (breaking changes should be versioned)

---

### 3. Cache API Responses

**Platform cell APIs are read-heavy.** Cache responses aggressively:

- `/platform/status`: 5-minute TTL (changes infrequently)
- `/platform/graph`: 1-hour TTL (only changes on spec updates)
- `/platform/docs/index`: 1-hour TTL (only changes on doc updates)
- `/platform/tasks`: 1-minute TTL (changes frequently during active work)

---

### 4. Handle Service Unavailability Gracefully

**Platform cells may not always be running** (local dev, deployments, outages).

**Fallback strategy:**

1. Try API scraping first
2. If API fails, fall back to repository scraping
3. If both fail, display cached data with staleness warning

---

### 5. Respect Platform Cell Auth

**Production platform cells may require authentication** for `/platform/*` endpoints.

**See:** `specs/service_metadata.yaml` → `links.status` for auth requirements.

**Default:** Basic auth or API key (check `/platform/schema` for details).

---

### 6. Link to Platform UI for Deep Exploration

**IDP dashboards should be entry points, not replacements.**

**Good practice:**

- Show governance summary in IDP (selftest status, AC coverage)
- Link to `/ui` for interactive graph exploration
- Link to specific docs from `/platform/docs/index`

**Why:** Platform cells have rich UIs optimized for governance exploration. Don't duplicate that in your IDP.

---

### 7. Use Release Evidence for Audit Trails

**For compliance/audit purposes:**

- Scrape `release_evidence/v*.md` into IDP
- Index by release version
- Provide search/filter by REQ/AC/ADR
- Link to Git tags for full diff

**This gives auditors:**
- What changed (tasks, ACs)
- Why it changed (requirements, ADRs)
- How it was validated (selftest, tests)
- When it was deployed (release timestamp)

---

### 8. Integrate with Slack/Teams for Notifications

**Example: Notify when governance degrades**

```python
# Monitor /platform/status and alert on failures
def monitor_governance_health(service_url, slack_webhook):
    status = requests.get(f"{service_url}/platform/status").json()

    if status["selftest_status"] != "PASS":
        requests.post(slack_webhook, json={
            "text": f"🚨 Governance failure in {status['service_id']}: selftest status is {status['selftest_status']}"
        })

    if status["ac_coverage"]["failing"] > 0:
        requests.post(slack_webhook, json={
            "text": f"⚠️ {status['ac_coverage']['failing']} ACs failing in {status['service_id']}"
        })
```

---

## Summary

**Platform cells are self-describing services** that expose governance state via:

1. **Runtime APIs** (`/platform/*`) for live status
2. **Static artifacts** (`service_metadata.yaml`, `feature_status.md`, `release_evidence/*.md`) for metadata and history

**IDPs should consume, not duplicate:**

- Use `/platform/status` for governance health
- Use `/platform/graph` for traceability
- Use `/platform/docs/index` for documentation
- Use `service_metadata.yaml` for ownership/tier/tags
- Use `release_evidence/*.md` for audit trails

**Integration patterns:**

- **API scraping** (recommended): Always up-to-date, requires service running
- **Repository scraping**: Works offline, slightly stale
- **Hybrid**: API for runtime, repo for history

**Best practices:**

- Prefer APIs over file parsing
- Cache aggressively
- Handle unavailability gracefully
- Link to platform UI for deep exploration
- Use release evidence for audit trails

---

## See Also

- **[Platform Introspection Design](../design/platform-introspection.md)** - API design and rationale
- **[Architecture Overview](architecture.md)** - Platform cell architecture
- **[Agent Guide](../AGENT_GUIDE.md)** - How agents interact with platform cells
- **[Template Adoption Patterns](adoption-patterns.md)** - How to adopt this template in your organization
