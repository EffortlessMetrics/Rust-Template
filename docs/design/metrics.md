---
doc_type: design_doc
id: DESIGN-TPL-METRICS
title: "Prometheus Metrics Strategy"
status: approved
owner: service-team
stories:
  - US-TPL-001
requirements:
  - REQ-TPL-METRICS
adrs:
  - ADR-0003
---

# Prometheus Metrics Strategy

## Context

Observability requires standard metrics for all services.

## Design

Expose `/metrics` endpoint compatible with Prometheus.
Standard metrics:
- `http_requests_total`
- `http_request_duration_seconds`
- `process_cpu_seconds_total`

## Implementation

Use `prometheus` crate and `axum` middleware.
