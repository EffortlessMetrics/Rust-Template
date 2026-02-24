# http-metrics

Prometheus metrics middleware and `/metrics` handler for HTTP services.

## Purpose

This crate provides one focused responsibility:

- Record HTTP request count and latency with Prometheus metrics
- Expose collected metrics through a handler suitable for `GET /metrics`

## Public API

- `metrics_middleware` - axum middleware that records request metrics
- `metrics_handler` - axum handler that renders Prometheus text output

## Metric Names

- `http_requests_total` (`method`, `path`, `status`)
- `http_request_duration_seconds` (`method`, `path`, `status_code`)

## License

Internal crate (`publish = false`).
