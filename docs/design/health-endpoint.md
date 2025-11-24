---
doc_type: design_doc
id: DESIGN-TPL-HEALTH-001
title: "Health Endpoint Design"
stories: ["US-TPL-001"]
requirements: ["REQ-TPL-HEALTH"]
acs: ["AC-TPL-001"]
adrs: ["ADR-0001", "ADR-0003"]
status: accepted
last_reviewed: 2025-11-20
owner: "platform"
---

# Health Endpoint Design

## 1. Context

The health endpoint (`GET /health`) provides a simple mechanism for load balancers and orchestrators to determine if the service instance is healthy and ready to receive traffic.

This implements REQ-TPL-HEALTH from the Service Core Capabilities story and aligns with:
- ADR-0001: Hexagonal Architecture (health check is an adapter-level concern)
- ADR-0003: Spec and BDD as Source of Truth (validated via AC-TPL-001)

## 2. High-Level Design

**Components**:
- HTTP adapter exposes `/health` endpoint
- Returns 200 OK with JSON: `{"status": "ok"}`
- Minimal logic: if server is running, it's healthy

**Future**: Could add dependency checks (database, external services) if needed.

## 3. Edge Cases & Failure Modes

- If service crashes, TCP connection fails -> load balancer removes instance
- No authentication required (public health check)
- Idempotent, safe to call repeatedly

## 4. Tests & Invariants

**Invariants**:
- Health endpoint MUST always return 200 when service is running
- Response MUST include `"status": "ok"` field

**Test Coverage**:
- BDD scenario `@AC-TPL-001` in `specs/features/template_core.feature`
- Integration test validates HTTP 200 + JSON shape

## 5. Open Questions / Future Work

- Add liveness vs readiness distinction if needed for k8s
- Add dependency health checks (DB, cache, etc.) as separate `/ready` endpoint
