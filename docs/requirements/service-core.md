---
doc_type: requirements_doc
id: REQS-TPL-CORE-001
title: "Service Core Requirements"
stories: ["US-TPL-001"]
requirements:
  - "REQ-TPL-HEALTH"
  - "REQ-TPL-VERSION"
  - "REQ-TPL-ERROR-HANDLING"
  - "REQ-TPL-METRICS"
acs: []
adrs: ["ADR-0001", "ADR-0003", "ADR-0005"]
status: accepted
last_reviewed: 2025-11-20
owner: "product"
---

# Service Core Requirements

## 1. Problem Statement

Every service needs foundational capabilities before domain logic:
- Health checks for orchestration
- Version information for debugging/deployment
- Consistent error handling
- Observability via metrics

These are template-provided "batteries included" features that downstream services inherit and extend.

## 2. Requirements

### REQ-TPL-HEALTH: Health Check Endpoint
- **Priority**: High
- **Description**: Service MUST expose `GET /health` returning 200 OK when healthy
- **Rationale**: Load balancers and orchestrators need a standard health check endpoint
- **Acceptance Criteria**: AC-TPL-001

### REQ-TPL-VERSION: Version Information Endpoint
- **Priority**: High
- **Description**: Service MUST expose `GET /version` with build info (version, git SHA)
- **Rationale**: Operations teams need clear version identification for deployments
- **Acceptance Criteria**: AC-TPL-002

### REQ-TPL-ERROR-HANDLING: Error Response Envelope
- **Priority**: High
- **Description**: All HTTP errors MUST return consistent JSON envelope with error code, message, request ID
- **Rationale**: Clients need predictable error format for debugging and logging
- **Acceptance Criteria**: AC-TPL-003, AC-TPL-004

### REQ-TPL-METRICS: Prometheus Metrics Endpoint
- **Priority**: High
- **Description**: Service MUST expose `/metrics` in Prometheus format with at least `http_requests_total`
- **Rationale**: Observability is non-negotiable for production services
- **Acceptance Criteria**: AC-TPL-007

## 3. Non-Requirements

- **Not** providing application-level business metrics (that's domain-specific)
- **Not** providing distributed tracing (can be added later)
- **Not** providing advanced health check dependencies (v1 is simple liveness)

## 4. Dependencies

- Hexagonal architecture (ADR-0001) enables clean separation of these cross-cutting concerns
- Spec and BDD (ADR-0003) ensures all requirements have executable validation
- Selftest gate (ADR-0005) enforces these requirements in CI

## 5. References

- `specs/spec_ledger.yaml` - US-TPL-001 story with all REQs linked
- `specs/features/template_core.feature` - BDD scenarios for all ACs
- Related ADRs: 0001, 0003, 0005
