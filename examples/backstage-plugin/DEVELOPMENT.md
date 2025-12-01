# Development Guide

**EXAMPLE QUALITY - NOT PRODUCTION READY**

This guide is for developers extending or customizing the Backstage plugin.

## Project Structure

```
backstage-plugin/
├── src/
│   ├── api/
│   │   └── PlatformClient.ts       # API client with TypeScript types
│   ├── components/
│   │   ├── GovernanceHealthCard.tsx # AC coverage and policy status
│   │   └── DocsHealthCard.tsx       # Documentation inventory
│   ├── plugin.ts                    # Plugin registration
│   └── index.ts                     # Public exports
├── package.json                     # Dependencies and scripts
├── tsconfig.json                    # TypeScript configuration
├── app-config.yaml.example          # Backstage configuration
├── catalog-info.yaml.example        # Catalog entity definition
└── README.md                        # User-facing documentation
```

## Development Setup

### Prerequisites

- Node.js 18+ and npm/yarn
- Backstage instance (development or existing)
- Rust-as-Spec platform service running

### Local Development

1. **Copy to Backstage monorepo**:

```bash
# From Backstage root
cp -r /path/to/examples/backstage-plugin packages/rust-spec-platform-plugin
cd packages/rust-spec-platform-plugin
```

2. **Install dependencies**:

```bash
yarn install
```

3. **Add proxy configuration** (in Backstage root `app-config.yaml`):

```yaml
proxy:
  '/rust-spec-platform':
    target: 'http://localhost:8080'
    changeOrigin: true
    pathRewrite:
      '^/api/proxy/rust-spec-platform': '/'
```

4. **Import plugin in your Backstage app** (`packages/app/src/App.tsx`):

```typescript
import { GovernanceHealthCard, DocsHealthCard } from 'rust-spec-platform-plugin';
```

5. **Start development server**:

```bash
# From Backstage root
yarn dev
```

## Adding New Components

### Creating a New Card

Follow this pattern:

```typescript
// src/components/MyCard.tsx
import React, { useEffect, useState } from 'react';
import { InfoCard, Progress } from '@backstage/core-components';
import { PlatformClient } from '../api/PlatformClient';

interface MyCardProps {
  baseUrl?: string;
  refreshInterval?: number;
}

export const MyCard: React.FC<MyCardProps> = ({
  baseUrl = '/api/proxy/rust-spec-platform',
  refreshInterval = 30000,
}) => {
  const [data, setData] = useState<MyData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const client = new PlatformClient(baseUrl);

    const fetchData = async () => {
      try {
        const result = await client.getMyEndpoint();
        setData(result);
        setError(null);
      } catch (err) {
        setError(err.message);
      } finally {
        setLoading(false);
      }
    };

    fetchData();

    if (refreshInterval > 0) {
      const intervalId = setInterval(fetchData, refreshInterval);
      return () => clearInterval(intervalId);
    }
  }, [baseUrl, refreshInterval]);

  if (loading) return <InfoCard title="My Card"><Progress /></InfoCard>;
  if (error) return <InfoCard title="My Card"><div>Error: {error}</div></InfoCard>;

  return (
    <InfoCard title="My Card">
      {/* Your UI here */}
    </InfoCard>
  );
};
```

### Export in `src/index.ts`:

```typescript
export { MyCard } from './components/MyCard';
```

## Extending the API Client

### Adding New Endpoints

```typescript
// src/api/PlatformClient.ts

// 1. Add response type
export interface MyEndpointResponse {
  field1: string;
  field2: number;
}

// 2. Add method to PlatformClient class
export class PlatformClient {
  // ... existing methods ...

  /**
   * Get data from /platform/my-endpoint
   */
  async getMyEndpoint(): Promise<MyEndpointResponse> {
    return this.fetch<MyEndpointResponse>('/platform/my-endpoint');
  }
}
```

### Adding Query Parameters

```typescript
async getTasks(status?: string): Promise<TasksResponse> {
  const params = new URLSearchParams();
  if (status) params.append('status', status);

  const query = params.toString();
  const endpoint = `/platform/tasks${query ? `?${query}` : ''}`;

  return this.fetch<TasksResponse>(endpoint);
}
```

## Styling Components

### Using Material-UI Theme

```typescript
import { makeStyles } from '@material-ui/core';

const useStyles = makeStyles(theme => ({
  myClass: {
    color: theme.palette.primary.main,
    padding: theme.spacing(2),
    [theme.breakpoints.down('sm')]: {
      padding: theme.spacing(1),
    },
  },
}));

export const MyComponent = () => {
  const classes = useStyles();
  return <div className={classes.myClass}>Content</div>;
};
```

### Responsive Design

```typescript
import { useMediaQuery, useTheme } from '@material-ui/core';

export const MyComponent = () => {
  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('sm'));

  return (
    <Grid container spacing={isMobile ? 1 : 3}>
      {/* Content */}
    </Grid>
  );
};
```

## Testing

### Unit Testing Components

```typescript
// src/components/__tests__/MyCard.test.tsx
import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import { MyCard } from '../MyCard';

// Mock the API client
jest.mock('../../api/PlatformClient');

describe('MyCard', () => {
  it('renders loading state', () => {
    render(<MyCard />);
    expect(screen.getByText(/loading/i)).toBeInTheDocument();
  });

  it('renders data after loading', async () => {
    render(<MyCard />);
    await waitFor(() => {
      expect(screen.getByText(/expected text/i)).toBeInTheDocument();
    });
  });

  it('handles errors gracefully', async () => {
    // Mock API error
    render(<MyCard />);
    await waitFor(() => {
      expect(screen.getByText(/error/i)).toBeInTheDocument();
    });
  });
});
```

### Testing API Client

```typescript
// src/api/__tests__/PlatformClient.test.ts
import { PlatformClient } from '../PlatformClient';

describe('PlatformClient', () => {
  let client: PlatformClient;

  beforeEach(() => {
    client = new PlatformClient('http://localhost:8080');
  });

  it('fetches status successfully', async () => {
    global.fetch = jest.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ status: 'ok' }),
    });

    const status = await client.getStatus();
    expect(status.status).toBe('ok');
  });

  it('handles network errors', async () => {
    global.fetch = jest.fn().mockRejectedValue(new Error('Network error'));

    await expect(client.getStatus()).rejects.toThrow();
  });
});
```

### Running Tests

```bash
yarn test
yarn test --watch          # Watch mode
yarn test --coverage       # With coverage
```

## Building for Production

### Build Plugin

```bash
yarn build
```

### Type Checking

```bash
yarn tsc --noEmit
```

### Linting

```bash
yarn lint
yarn lint --fix  # Auto-fix issues
```

## Debugging

### Browser DevTools

1. Open browser console
2. Check Network tab for API calls
3. Check Console for errors
4. Use React DevTools for component inspection

### API Client Debugging

Add logging to `PlatformClient`:

```typescript
private async fetch<T>(endpoint: string): Promise<T> {
  console.log(`[PlatformClient] Fetching: ${endpoint}`);

  try {
    const response = await fetch(url);
    console.log(`[PlatformClient] Response:`, response.status);
    // ... rest of implementation
  } catch (error) {
    console.error(`[PlatformClient] Error:`, error);
    throw error;
  }
}
```

### Component State Debugging

```typescript
useEffect(() => {
  console.log('[MyCard] State:', { data, loading, error });
}, [data, loading, error]);
```

## Common Patterns

### Error Boundaries

```typescript
import { ErrorBoundary } from '@backstage/core-components';

<ErrorBoundary>
  <MyCard />
</ErrorBoundary>
```

### Configuration from Backstage Config

```typescript
import { useApi, configApiRef } from '@backstage/core-plugin-api';

export const MyCard = () => {
  const config = useApi(configApiRef);
  const baseUrl = config.getString('rustSpecPlatform.backend.baseUrl');
  const refreshInterval = config.getNumber('rustSpecPlatform.frontend.refreshIntervals.governance');

  // Use configuration
};
```

### Conditional Rendering

```typescript
export const MyCard = () => {
  if (!data) return null;

  return data.items.length > 0 ? (
    <List>
      {data.items.map(item => <ListItem key={item.id}>{item.name}</ListItem>)}
    </List>
  ) : (
    <EmptyState
      title="No items"
      description="There are no items to display"
    />
  );
};
```

## Performance Optimization

### Memoization

```typescript
import { useMemo } from 'react';

const expensiveValue = useMemo(() => {
  return computeExpensiveValue(data);
}, [data]);
```

### Lazy Loading

```typescript
import React, { lazy, Suspense } from 'react';
import { Progress } from '@backstage/core-components';

const MyCard = lazy(() => import('./components/MyCard'));

export const MyPage = () => (
  <Suspense fallback={<Progress />}>
    <MyCard />
  </Suspense>
);
```

## Troubleshooting

### Plugin not loading

- Check that plugin is registered in `App.tsx`
- Verify imports are correct
- Check browser console for errors
- Ensure dependencies are installed

### API calls failing

- Verify proxy configuration in `app-config.yaml`
- Check that platform service is running
- Inspect Network tab in browser DevTools
- Test API directly: `curl http://localhost:8080/platform/status`

### TypeScript errors

- Run `yarn tsc --noEmit` to check types
- Ensure all types are exported from `PlatformClient.ts`
- Check import paths are correct

### Styling issues

- Verify Material-UI theme is available
- Check responsive breakpoints
- Use browser inspector to debug CSS

## Further Resources

- [Backstage Plugin Development](https://backstage.io/docs/plugins/)
- [Material-UI Documentation](https://v4.mui.com/)
- [React Hooks Guide](https://react.dev/reference/react)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/)

---

**Remember**: This is example quality code. Add comprehensive testing, error handling, and security before production use.
