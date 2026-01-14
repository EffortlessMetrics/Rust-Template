---
doc_type: design_doc
id: DESIGN-TPL-ERRORS
title: "Standardized Error Handling"
status: approved
owner: service-team
stories:
  - US-TPL-001
requirements:
  - REQ-TPL-ERROR-HANDLING
adrs:
  - ADR-0003
---

# Standardized Error Handling

## Context

Services need a consistent error response format for clients to reliably handle failures.

## Design

All 4xx/5xx responses return a JSON envelope:

```json
{
  "error": {
    "code": "INVALID_INPUT",
    "message": "Field 'foo' is required",
    "request_id": "req-123"
  }
}
```

## Implementation

Middleware captures errors and formats them. `X-Request-ID` is propagated.
