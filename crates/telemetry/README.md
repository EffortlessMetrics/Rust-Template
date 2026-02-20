# Telemetry Crate

Centralized tracing initialization for the Rust IaC Template.

## Features

- **Console Tracing (Default)**: Structured logging to stdout using `tracing-subscriber`
- **OTLP Export (Optional)**: Send traces to OpenTelemetry collectors via OTLP/gRPC

## Usage

### Basic Console Tracing

```rust
use telemetry;

fn main() {
    telemetry::init_tracing("my-service");

    tracing::info!("Service started");
    tracing::debug!("Debug information");
}
```

Control log levels with the `RUST_LOG` environment variable:

```bash
RUST_LOG=debug cargo run
RUST_LOG=my_service=trace,other_crate=info cargo run
```

### OTLP Tracing (Feature Gated)

To enable OTLP export:

1. **Enable the `otlp` feature** in your `Cargo.toml`:

   ```toml
   [dependencies]
   telemetry = { path = "../telemetry", features = ["otlp"] }
   ```

2. **Set the `OTLP_ENDPOINT` environment variable**:

   ```bash
   OTLP_ENDPOINT=http://localhost:4317 cargo run --features telemetry/otlp
   ```

3. **Traces will be exported** to the OTLP collector at the specified endpoint.

### Graceful Fallback

If OTLP initialization fails (e.g., collector is down), the system automatically falls back to console-only tracing:

```bash
# Collector not running - app still starts
OTLP_ENDPOINT=http://localhost:4317 cargo run --features telemetry/otlp
# Logs: "Failed to initialize OTLP exporter: ..., falling back to console tracing"
```

## Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `otlp`  | Enable OTLP export via gRPC (tonic) | No |

## Environment Variables

| Variable        | Description                           | Example                   |
|-----------------|---------------------------------------|---------------------------|
| `RUST_LOG`      | Log level filter                      | `debug`, `trace`          |
| `OTLP_ENDPOINT` | OTLP collector endpoint (otlp feature only) | `http://localhost:4317` |

## Dependencies

### Core (Always Included)

- `tracing`: Structured logging API
- `tracing-subscriber`: Console formatting and filtering

### OTLP Feature (`otlp`)

- `opentelemetry`: OpenTelemetry API
- `opentelemetry_sdk`: OpenTelemetry SDK
- `opentelemetry-otlp`: OTLP exporter with gRPC support
- `tracing-opentelemetry`: Bridge between `tracing` and OpenTelemetry

## Testing OTLP

See [docs/how-to/test-otlp-tracing.md](../../docs/how-to/test-otlp-tracing.md) for a complete guide on testing OTLP locally with Jaeger.

Quick test:

```bash
# Start Jaeger
docker run -d --name jaeger \
  -e COLLECTOR_OTLP_ENABLED=true \
  -p 16686:16686 -p 4317:4317 \
  jaegertracing/all-in-one:latest

# Run app with OTLP
OTLP_ENDPOINT=http://localhost:4317 \
  cargo run -p app-http --features telemetry/otlp

# Generate traffic
curl http://localhost:8080/health

# View traces at http://localhost:16686
```

## Design Principles

1. **Safe Defaults**: Console tracing always works, OTLP is opt-in
2. **Graceful Degradation**: OTLP failures don't crash the application
3. **Zero Configuration**: Works out of the box for local development
4. **Production Ready**: OTLP feature for production observability

## Implementation Details

The `init_tracing()` function follows this logic:

1. **If `otlp` feature is enabled AND `OTLP_ENDPOINT` is set:**
   - Try to initialize OTLP exporter
   - On success: Set up OTLP + console tracing
   - On failure: Log warning and fall back to console-only

2. **Otherwise:**
   - Initialize console-only tracing

This ensures the application never fails to start due to telemetry issues.

## See Also

- [OpenTelemetry Rust](https://github.com/open-telemetry/opentelemetry-rust)
- [Tracing Documentation](https://docs.rs/tracing/latest/tracing/)
- [OTLP Specification](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/otlp.md)

## Stability

This crate is part of the **rust-as-spec** governance kernel.
Version numbers track the kernel release (currently 3.3.15).
Breaking changes require a major version bump and an ADR.
MSRV: 1.89.0.
