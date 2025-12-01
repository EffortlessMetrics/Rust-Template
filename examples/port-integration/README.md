# Port.io Integration Example

**EXAMPLE QUALITY - NOT PRODUCTION READY**

This example shows how to integrate a Rust-as-Spec platform cell with Port.io IDP.

## What This Example Shows

1. **Blueprint definition** for modeling platform cells in Port.io
2. **Sync script** (Python) for pushing governance data to Port.io
3. **Scorecard configuration** for governance health metrics
4. **GitHub Actions workflow** for scheduled syncs

## Prerequisites

- Port.io account with API access
- Python 3.11+
- Platform service running (`cargo run -p app-http`)
- Port.io API credentials (`PORT_CLIENT_ID`, `PORT_CLIENT_SECRET`)

## Quick Start

### 1. Create the Blueprint

```bash
# Set credentials
export PORT_CLIENT_ID="your-client-id"
export PORT_CLIENT_SECRET="your-client-secret"

# Create blueprint via Port API
curl -X POST https://api.getport.io/v1/blueprints \
  -H "Authorization: Bearer $(python3 get_token.py)" \
  -H "Content-Type: application/json" \
  -d @blueprint.json
```

### 2. Run the Sync Script

```bash
# Install dependencies
pip install requests

# Set environment
export PLATFORM_URL="http://localhost:8080"
export PORT_CLIENT_ID="your-client-id"
export PORT_CLIENT_SECRET="your-client-secret"

# Run sync
python3 sync_to_port.py
```

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
| `sync_to_port.py` | Python sync script (incremental, idempotent) |
| `scorecard.json` | Governance scorecard configuration |
| `.github/workflows/port-sync.yaml` | GitHub Actions workflow template |

## API Endpoints Used

The sync script consumes these platform endpoints:

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

Then update `sync_to_port.py` to populate the field from platform data.

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
python3 -c "import sync_to_port; print(sync_to_port.get_port_token())"
```

### Entity Not Appearing

Check the sync script output for errors:
```bash
python3 sync_to_port.py --verbose
```

### Stale Data

Force a full sync (ignores cache):
```bash
python3 sync_to_port.py --force
```

## Related Documentation

- [Port.io Integration How-To](../../docs/how-to/implement-port-integration.md)
- [IDP Tile Specifications](../../docs/design/DESIGN-IDP-TILES.md)
- [JSON Contracts](../../docs/explanation/json-contracts.md)
