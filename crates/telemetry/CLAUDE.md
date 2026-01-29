# telemetry – CLAUDE.md

**Tier:** Support (Layer 8)
**Publish:** Yes
**Dependencies:** tracing, tracing-subscriber, opentelemetry (feature-gated)

## Purpose

Optional OpenTelemetry configuration. Provides tracing, metrics, and logging setup for the platform.

## Key Exports

- Telemetry initialization
- Tracing subscriber setup
- OpenTelemetry configuration (optional)

## Features

- `otel` – Enable OpenTelemetry integration
- `jaeger` – Enable Jaeger exporter
- `otlp` – Enable OTLP exporter

## When to Modify

- Adding new telemetry exporters
- Changing tracing configuration
- Adding custom spans/metrics

## When NOT to Modify

- Adding application logic

## Architectural Notes

- **Feature-gated**: OTel is optional to reduce dependencies
- **Configurable**: Multiple exporter options
- **Non-blocking**: Async telemetry export

## Consumers

`app-http`, any crate needing tracing

## See Also

- OpenTelemetry documentation
- `tracing` crate documentation
