# Port.io Integration Example

**EXAMPLE QUALITY - NOT PRODUCTION READY**

This example shows how to integrate a Rust-as-Spec platform cell with Port.io IDP.

## What This Example Shows

1. **Blueprint definition** for modeling platform cells in Port.io
2. **Sync binary** (Rust) for pushing governance data to Port.io
3. **Scorecard configuration** for governance health metrics
4. **GitHub Actions workflow** for scheduled syncs

## Prerequisites

- Port.io account with API access
- Rust toolchain
- Platform service running (`cargo run -p app-http`)
- Port.io API credentials (`PORT_CLIENT_ID`, `PORT_CLIENT_SECRET`)

## Quick Start

### 1. Create the Blueprint

```bash
# Set credentials
export PORT_CLIENT_ID="your-client-id"
export PORT_CLIENT_SECRET="your-client-secret"

# Create the blueprint via the Port UI or API using blueprint.json
```

### 2. Run the Sync Binary

```bash
# Set environment
export PLATFORM_URL="http://localhost:8080"
export PORT_CLIENT_ID="your-client-id"
export PORT_CLIENT_SECRET="your-client-secret"

# Run sync
cargo run --manifest-path examples/port-integration/Cargo.toml --
```

### 2b. Offline Testing (No Port Credentials)

To test the IDP integration without Port.io credentials:

```bash
# Start the platform service
cargo run -p app-http &

# Run in dump-only mode (no Port.io auth required)
PLATFORM_URL="http://localhost:8080" cargo run --manifest-path examples/port-integration/Cargo.toml -- --dump-only
```

This will:
1. Fetch governance data from `/platform/idp/snapshot` and `/platform/status`
2. Transform it to the Port.io entity format
3. Print the entity JSON to stdout (status messages go to stderr)

### 2c. Capturing JSON for Parsing

The `--dump-only` flag outputs pure JSON to stdout, making it easy to pipe to `jq` or other tools:

```bash
# Extract pure JSON to file
cargo run --manifest-path examples/port-integration/Cargo.toml -- --dump-only 2>/dev/null > entity.json

# Parse with jq
cargo run --manifest-path examples/port-integration/Cargo.toml -- --dump-only 2>/dev/null | jq '.properties'

# Get specific fields
cargo run --manifest-path examples/port-integration/Cargo.toml -- --dump-only 2>/dev/null | jq '.properties.ac_coverage_percent'

# See status messages alongside (both streams visible)
cargo run --manifest-path examples/port-integration/Cargo.toml -- --dump-only
```

**Note:** Status messages like "Fetching from..." go to stderr, so `2>/dev/null` silences them for pure JSON output.

### 3. View in Port.io

Navigate to your Port.io catalog to see the synced service entity with:
- Governance health status
- AC coverage percentage
- Documentation metrics
- Task counts

## Files

| File | Description |
|------|-------------|
| `blueprint.json` | Port.io blueprint schema for platform cells |
| `Cargo.toml`, `src/main.rs` | Rust sync binary (incremental, idempotent) |
| `scorecard.json` | Governance scorecard configuration |
| `.github/workflows/port-sync.yaml` | GitHub Actions workflow template |

## API Endpoints Used

The sync binary consumes these platform endpoints:

- `GET /platform/status` - Governance health and policies
- `GET /platform/idp/snapshot` - Consolidated IDP payload
- `GET /platform/docs/index` - Documentation inventory

See [IDP Integration Guide](../../docs/how-to/integrate-idp-or-agent.md) for full API documentation.

## Customization

### Adding Custom Properties

Edit `blueprint.json` to add properties:

```json
{
  "my_custom_field": {
    "title": "Custom Field",
    "type": "string",
    "description": "My custom governance metric"
  }
}
```

Then update `src/main.rs` to populate the field from platform data.

### Sync Frequency

The default GitHub Actions workflow runs hourly. Adjust in `.github/workflows/port-sync.yaml`:

```yaml
schedule:
  - cron: '*/15 * * * *'  # Every 15 minutes
```

## Troubleshooting

### Authentication Errors

Ensure your Port.io credentials are correct:

```bash
cargo run --manifest-path examples/port-integration/Cargo.toml -- --verbose
```

### Entity Not Appearing

Check the sync binary output for errors:

```bash
cargo run --manifest-path examples/port-integration/Cargo.toml -- --verbose
```

### Stale Data

Force a full sync (ignores cache):

```bash
cargo run --manifest-path examples/port-integration/Cargo.toml -- --force
```

## Related Documentation

- [Port.io Integration How-To](../../docs/how-to/implement-port-integration.md)
- [IDP Tile Specifications](../../docs/design/DESIGN-IDP-TILES.md)
- [JSON Contracts](../../docs/explanation/json-contracts.md)
