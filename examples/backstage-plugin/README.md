# Backstage Plugin Example

**EXAMPLE QUALITY - NOT PRODUCTION READY**

This is a minimal reference implementation showing how to integrate the Rust-as-Spec platform with Spotify Backstage.

## What This Example Shows

This plugin demonstrates:

1. **API Integration**: How to fetch from `/platform/status` and `/platform/docs/index` endpoints
2. **Component Structure**: React components for displaying governance health
3. **TypeScript Client**: Type-safe API client for platform endpoints
4. **Backstage Patterns**: Plugin registration and card-based UI components

## Features

### Governance Health Card

Displays real-time governance metrics:
- AC coverage percentage
- Policy enforcement status
- Selftest health
- Authentication mode

### Docs Health Card

Shows documentation inventory:
- Total documents count
- Valid documents
- Documents with issues
- Breakdown by type (ADRs, design docs, how-tos)

## Installation

### Prerequisites

- Node.js 18+ and npm/yarn
- Running Backstage instance
- Rust-as-Spec service running on `http://localhost:8080`

### Setup

1. **Copy plugin to your Backstage monorepo**:

```bash
cp -r examples/backstage-plugin packages/rust-spec-platform-plugin
cd packages/rust-spec-platform-plugin
```

2. **Install dependencies**:

```bash
yarn install
```

3. **Configure backend proxy** (see `app-config.yaml.example`):

```yaml
proxy:
  '/rust-spec-platform':
    target: 'http://localhost:8080'
    changeOrigin: true
    pathRewrite:
      '^/api/proxy/rust-spec-platform': '/'
```

4. **Add to your Backstage app** (`packages/app/src/components/catalog/EntityPage.tsx`):

```typescript
import { GovernanceHealthCard, DocsHealthCard } from 'rust-spec-platform-plugin';

const overviewContent = (
  <Grid container spacing={3}>
    {/* Other cards */}
    <Grid item md={6}>
      <GovernanceHealthCard />
    </Grid>
    <Grid item md={6}>
      <DocsHealthCard />
    </Grid>
  </Grid>
);
```

## Architecture

```
backstage-plugin/
├── src/
│   ├── plugin.ts              # Plugin registration
│   ├── api/
│   │   └── PlatformClient.ts  # API client with types
│   └── components/
│       ├── GovernanceHealthCard.tsx  # Governance metrics
│       └── DocsHealthCard.tsx        # Documentation metrics
├── package.json
└── README.md
```

## API Client

The `PlatformClient` provides typed access to platform endpoints:

```typescript
const client = new PlatformClient('http://localhost:8080');

// Get governance status
const status = await client.getStatus();
console.log(`AC Coverage: ${status.governance.ac_coverage.percentage}%`);

// Get documentation inventory
const docs = await client.getDocsIndex();
console.log(`Total docs: ${docs.summary.total}`);
```

## Components

### GovernanceHealthCard

Fetches from `/platform/status` and displays:
- Overall health indicator
- AC coverage with progress bar
- Policy status badges
- Selftest gate status

### DocsHealthCard

Fetches from `/platform/docs/index` and displays:
- Documentation count summary
- Health indicators (valid vs. issues)
- Breakdown by document type
- Links to full documentation

## Customization

### Extending the Client

Add new endpoints to `PlatformClient.ts`:

```typescript
async getGraph(): Promise<GraphResponse> {
  return this.fetch('/platform/graph');
}

async getTasks(status?: string): Promise<TasksResponse> {
  const params = status ? `?status=${status}` : '';
  return this.fetch(`/platform/tasks${params}`);
}
```

### Creating New Cards

Follow the pattern in existing cards:

```typescript
export const MyCustomCard = () => {
  const [data, setData] = useState<MyData | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const client = new PlatformClient('/api/proxy/rust-spec-platform');
    client.getMyEndpoint()
      .then(setData)
      .finally(() => setLoading(false));
  }, []);

  // Render logic
};
```

## Deployment Considerations

### Production Checklist

Before using in production:

1. **Security**:
   - Add authentication/authorization
   - Use HTTPS for all connections
   - Validate and sanitize API responses
   - Implement rate limiting

2. **Reliability**:
   - Add error boundaries
   - Implement retry logic
   - Add loading states and timeouts
   - Handle offline scenarios

3. **Performance**:
   - Add caching layer
   - Implement data polling strategy
   - Optimize bundle size
   - Add performance monitoring

4. **Testing**:
   - Add unit tests for components
   - Add integration tests for API client
   - Add E2E tests for user flows
   - Test error scenarios

### Configuration Management

Use Backstage's config system:

```yaml
rustSpecPlatform:
  backend:
    baseUrl: ${RUST_SPEC_PLATFORM_URL}
  refreshInterval: 30000  # 30 seconds
  timeout: 5000  # 5 seconds
```

Access in code:

```typescript
import { useApi, configApiRef } from '@backstage/core-plugin-api';

const config = useApi(configApiRef);
const baseUrl = config.getString('rustSpecPlatform.backend.baseUrl');
```

## Platform API Reference

### Endpoints Used

- `GET /platform/status` - Governance health and policy status
- `GET /platform/docs/index` - Documentation inventory

### Available Endpoints

See full API documentation:

- `/platform/graph` - Complete governance graph
- `/platform/tasks` - Task list with filtering
- `/platform/agent/hints` - AI agent work suggestions
- `/platform/friction` - DevEx friction log
- `/platform/coverage` - AC coverage details

## Troubleshooting

### "Failed to fetch" errors

1. Check that the Rust service is running: `curl http://localhost:8080/platform/status`
2. Verify proxy configuration in `app-config.yaml`
3. Check CORS settings if accessing directly
4. Inspect browser network tab for details

### Component not rendering

1. Check browser console for errors
2. Verify plugin is registered in `packages/app/src/App.tsx`
3. Check that dependencies are installed
4. Verify import paths are correct

### Stale data

1. Check refresh interval in component
2. Verify API responses are not cached
3. Clear browser cache
4. Check backend service logs

## Contract Reference

This plugin is a **consumer** of the Rust-as-Spec kernel's platform API contract. The source of truth is:

- **[`docs/reference/platform_api_contract.md`](../../docs/reference/platform_api_contract.md)** – Full contract documentation
- **[`specs/openapi/openapi.yaml`](../../specs/openapi/openapi.yaml)** – OpenAPI schema with `PlatformStatus` and `DocsIndex` definitions

TypeScript types in `src/api/PlatformClient.ts` are aligned with these schemas. When the kernel updates its contract, the plugin types should be updated to match.

## Further Reading

- [Backstage Plugin Development](https://backstage.io/docs/plugins/)
- [Platform API Contract](../../docs/reference/platform_api_contract.md) – Source of truth for API shapes
- [Rust-as-Spec Platform APIs](../../docs/AGENT_GUIDE.md)
- [Governance Model](../../docs/explanation/TEMPLATE-CONTRACTS.md)

## License

This example is provided as-is for reference purposes. Follow your organization's licensing requirements.

---

**Remember**: This is example quality code. Review, test, and harden before production use.
