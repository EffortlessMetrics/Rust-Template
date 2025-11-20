---
doc_type: design_doc
id: DESIGN-TPL-VERSION
title: "Version Information Strategy"
status: approved
owner: service-team
stories:
  - US-TPL-001
requirements:
  - REQ-TPL-VERSION
adrs:
  - ADR-0003
---

# Version Information Strategy

## Context
Deployment verification requires knowing exactly what version is running.

## Design
Expose `/version` endpoint returning JSON:
```json
{
  "version": "1.2.3",
  "git_sha": "abcdef...",
  "build_date": "2025-01-01..."
}
```

## Implementation
Inject build-time variables via `build.rs` or environment variables.
