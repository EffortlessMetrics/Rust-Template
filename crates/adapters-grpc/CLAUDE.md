# adapters-grpc – CLAUDE.md

**Tier:** Adapter (Layer 4)
**Publish:** Yes
**Dependencies:** tonic, prost, tokio

## Purpose

Tonic-based gRPC adapter. Provides gRPC service implementations for cross-service communication.

## Key Exports

- gRPC service implementations
- Protocol buffer types (generated)
- Client/server utilities

## When to Modify

- Adding new gRPC services
- Updating proto definitions
- Extending service implementations

## When NOT to Modify

- Adding HTTP logic (that goes in http-* crates)

## Architectural Notes

- **Tonic**: Rust gRPC framework
- **Prost**: Protocol buffer code generation
- **Proto files**: Service definitions

## Key Files

- `proto/` for .proto service definitions

## Consumers

Services requiring gRPC communication

## See Also

- `crates/app-http/` for HTTP alternative
