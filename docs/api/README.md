# API Documentation

API documentation is generated from specifications, not maintained as separate prose.

## Authoritative Sources

- **OpenAPI Specification**: [`specs/openapi/openapi.yaml`](../../specs/openapi/openapi.yaml)
- **Runtime Schema**: `/platform/schema` endpoint (machine-readable, available when service is running)

## Usage

Start the service and query the schema endpoint:

```bash
cargo run -p app-http &
curl http://localhost:8080/platform/schema
```

For the full governance API surface, see `/platform/status` and related endpoints documented in [`CLAUDE.md`](../../CLAUDE.md).
