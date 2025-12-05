---
id: DESIGN-TPL-IDP-TILES-001
title: "IDP Tile Specifications for Platform Integration"
author: platform-team
doc_type: design_doc
date: 2025-12-01
status: published
stories: [US-TPL-PLT-001]
requirements: [REQ-TPL-PLATFORM-INTROSPECTION, REQ-TPL-AI-IDP-COMPAT, REQ-TPL-IDP-SNAPSHOT]
tags: [platform, idp, integration, ai]
acs: [AC-TPL-CLI-JSON-CORE, AC-TPL-CLI-JSON-OUTPUT, AC-TPL-AGENT-HINTS-SCHEMA]
adrs: [ADR-0016]
---

# IDP Tile Specifications for Platform Integration

## Purpose

This document specifies the data contracts and integration patterns for surfacing Rust-as-Spec governance in Internal Developer Platforms (Backstage, Port.io, etc.).

## Tiles Overview

### 1. Governance Health Tile

**Primary Endpoint:** `GET /platform/status`

**Key Fields:**
- `governance.policies.status` - Overall policy check status ("passing" or "failing")
- `governance.docs.doc_type_issues` - Count of doc contract violations
- `governance.ledger.acs` - Total AC count

**Display:**
- Traffic light indicator (green/yellow/red)
- "Kernel ACs: 61/61 pass"
- "Selftest: ✓ 11/11"

**Cache TTL:** 5 minutes

### 2. Docs Health Tile

**Primary Endpoint:** `GET /platform/docs/index`

**Response Structure:**
```json
{
  "schema_version": "string",
  "template_version": "string",
  "docs": [...],
  "summary": {
    "total": "number",
    "valid": "number",
    "with_issues": "number"
  }
}
```

**Key Fields:**
- `summary.with_issues` - Docs with contract violations
- `docs[].doc_type_valid` - Per-doc validation status
- `docs[].doc_type_issue` - Specific violation description

**Display:**
- "Docs: N indexed, M with issues"
- Link to detail view for docs with issues

**Cache TTL:** 1 hour

### 3. AC Coverage Tile

**Primary Endpoint:** `GET /platform/coverage`

**Response Structure:**
```json
{
  "summary": {
    "passing": "number",
    "failing": "number",
    "unknown": "number",
    "total": "number"
  },
  "details": [...]
}
```

**Display:**
- Percentage bar: "AC Coverage: 85% (51/60)"
- Breakdown by status

**Cache TTL:** 5 minutes

### 4. Task Hints Tile

**Primary Endpoint:** `GET /platform/agent/hints`

**Key Fields:**
- `hints[].priority` - low/medium/high
- `hints[].status` - open/in_progress/done
- `hints[].requirement_ids` - Linked REQs
- `hints[].ac_ids` - Linked ACs

**Display:**
- Priority-sorted task list
- Links to spec files

**Cache TTL:** 1 minute

## Stability Guarantees

From `docs/explanation/json-contracts.md`:

- **Additive-only** changes across patch releases
- New fields may be added; existing fields will NOT be removed
- Breaking changes require major version bump + migration path
- All endpoints include `template_version` for compatibility checking

## Integration Examples

### Backstage Plugin

```typescript
// Fetch governance health
const status = await fetch(`${serviceUrl}/platform/status`).then(r => r.json());
const health = status.governance.policies.status === 'passing' ? 'HEALTHY' : 'AT_RISK';
```

### Port.io Blueprint

```yaml
identifier: rust-as-spec-service
properties:
  governance_status:
    type: string
    description: "Governance health from /platform/status"
  ac_coverage:
    type: number
    description: "AC pass percentage"
```

## Related Documents

- [JSON Contracts](../explanation/json-contracts.md) - Full schema specifications
- [IDP Positioning](../explanation/idp-positioning.md) - Strategic context
- [Agent Guide](../AGENT_GUIDE.md) - Agent integration patterns
