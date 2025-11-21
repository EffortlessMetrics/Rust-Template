# How to Test OTLP Tracing

This guide explains how to manually test OTLP (OpenTelemetry Protocol) tracing with a local collector.

## Prerequisites

- Docker or Podman installed
- Rust toolchain
- `telemetry` crate built with `otlp` feature

## Quick Start with Jaeger

Jaeger provides an all-in-one container with OTLP support and a web UI for viewing traces.

### 1. Start Jaeger Collector

```bash
docker run -d \
  --name jaeger \
  -e COLLECTOR_OTLP_ENABLED=true \
  -p 16686:16686 \
  -p 4317:4317 \
  -p 4318:4318 \
  jaegertracing/all-in-one:latest
```

**Ports:**
- `16686`: Jaeger UI (web interface)
- `4317`: OTLP gRPC endpoint
- `4318`: OTLP HTTP endpoint

### 2. Run Application with OTLP

Build and run `app-http` with OTLP enabled:

```bash
OTLP_ENDPOINT=http://localhost:4317 \
  cargo run -p app-http --features telemetry/otlp
```

### 3. Generate Some Traffic

```bash
# Health check
curl http://localhost:3000/health

# Version endpoint
curl http://localhost:3000/version

# Metrics endpoint
curl http://localhost:3000/metrics
```

### 4. View Traces in Jaeger UI

1. Open your browser to http://localhost:16686
2. Select **app-http** from the "Service" dropdown
3. Click "Find Traces"
4. You should see traces for the HTTP requests you made

### 5. Cleanup

```bash
docker stop jaeger
docker rm jaeger
```

## Alternative: Using OpenTelemetry Collector

If you want more control or need to send traces to multiple backends:

### 1. Create Collector Config

Create `otel-collector-config.yaml`:

```yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

exporters:
  logging:
    loglevel: debug
  jaeger:
    endpoint: jaeger:14250
    tls:
      insecure: true

processors:
  batch:

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch]
      exporters: [logging, jaeger]
```

### 2. Start with Docker Compose

Create `docker-compose.yml`:

```yaml
version: "3.8"

services:
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"  # Jaeger UI
      - "14250:14250"  # Jaeger gRPC
    environment:
      - COLLECTOR_OTLP_ENABLED=true

  otel-collector:
    image: otel/opentelemetry-collector:latest
    command: ["--config=/etc/otel-collector-config.yaml"]
    volumes:
      - ./otel-collector-config.yaml:/etc/otel-collector-config.yaml
    ports:
      - "4317:4317"   # OTLP gRPC
      - "4318:4318"   # OTLP HTTP
    depends_on:
      - jaeger
```

### 3. Start Services

```bash
docker-compose up -d
```

### 4. Run Application

```bash
OTLP_ENDPOINT=http://localhost:4317 \
  cargo run -p app-http --features telemetry/otlp
```

### 5. Cleanup

```bash
docker-compose down
```

## Fallback Behavior

The `telemetry` crate is designed to gracefully handle OTLP failures:

### Test Fallback Without Collector

```bash
# OTLP endpoint is set but collector is not running
OTLP_ENDPOINT=http://localhost:4317 \
  cargo run -p app-http --features telemetry/otlp
```

**Expected behavior:**
- Application starts successfully
- Warning printed to stderr: `Failed to initialize OTLP exporter: <error>, falling back to console tracing`
- Console tracing works normally

### Test Without OTLP Feature

```bash
# No OTLP feature, environment variable is ignored
OTLP_ENDPOINT=http://localhost:4317 \
  cargo run -p app-http
```

**Expected behavior:**
- Application starts successfully
- Only console tracing is active
- No OTLP initialization attempted

## Verification Checklist

Use this checklist to validate OTLP integration:

- [ ] **Application starts with OTLP enabled**
  ```bash
  OTLP_ENDPOINT=http://localhost:4317 cargo run -p app-http --features telemetry/otlp
  ```

- [ ] **Traces appear in Jaeger UI**
  - Open http://localhost:16686
  - Select "app-http" service
  - See traces for HTTP requests

- [ ] **Application falls back gracefully when collector is down**
  ```bash
  # Stop Jaeger, then run app
  docker stop jaeger
  OTLP_ENDPOINT=http://localhost:4317 cargo run -p app-http --features telemetry/otlp
  # App should start and log warning
  ```

- [ ] **Application works without OTLP feature**
  ```bash
  cargo run -p app-http
  # Should work with console tracing only
  ```

- [ ] **All tests pass with and without OTLP feature**
  ```bash
  cargo test --workspace
  cargo test --workspace --features telemetry/otlp
  ```

## Troubleshooting

### "Failed to initialize OTLP exporter: connection refused"

**Cause:** OTLP collector is not running or not reachable.

**Solution:**
1. Verify Jaeger/collector is running: `docker ps | grep jaeger`
2. Check port 4317 is exposed: `docker port jaeger 4317`
3. Try explicit localhost: `OTLP_ENDPOINT=http://127.0.0.1:4317`

### Traces not appearing in Jaeger UI

**Possible causes:**
1. Wrong service name in Jaeger UI dropdown
2. Traces not yet flushed (wait a few seconds)
3. OTLP endpoint misconfigured

**Debug steps:**
1. Check app logs for "Initialized OTLP tracing" message
2. Check Jaeger logs: `docker logs jaeger`
3. Verify traffic with `docker exec jaeger netstat -an | grep 4317`

### Application crashes on startup with OTLP

**This should not happen** - if it does, it's a bug. OTLP failures should always fall back to console tracing.

**Please report:**
- Full error message
- Rust version: `rustc --version`
- OpenTelemetry versions: `cargo tree | grep opentelemetry`

## Environment Variables Reference

| Variable        | Purpose                           | Example                      | Required |
|-----------------|-----------------------------------|------------------------------|----------|
| `OTLP_ENDPOINT` | OTLP collector gRPC endpoint      | `http://localhost:4317`      | No       |
| `RUST_LOG`      | Log level filter                  | `debug`, `app_http=trace`    | No       |

## See Also

- [OpenTelemetry Rust Documentation](https://github.com/open-telemetry/opentelemetry-rust)
- [Jaeger Documentation](https://www.jaegertracing.io/docs/)
- [OTLP Specification](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/otlp.md)
