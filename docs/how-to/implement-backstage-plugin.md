---
id: GUIDE-TPL-BACKSTAGE-PLUGIN-001
doc_type: how-to
title: "Implementing a Backstage Plugin for Platform Integration"
stories: [US-TPL-PLT-001]
requirements: [REQ-TPL-PLATFORM-INTROSPECTION, REQ-TPL-AI-IDP-COMPAT]
acs: [AC-TPL-CLI-JSON-CORE, AC-TPL-CLI-JSON-OUTPUT]
adrs: [ADR-0016]
last_updated: 2025-12-01
---
<!-- doclint:disable orphan-version -->

# Implementing a Backstage Plugin for Platform Integration

This guide walks you through creating a Backstage plugin that integrates with the Rust-as-Spec platform cell's `/platform/*` endpoints to display governance health, documentation status, and AC coverage in your Backstage IDP.

## Prerequisites

Before you begin, ensure you have:

- **Backstage app**: An existing Backstage instance (v1.18+) or ability to create one
- **Node.js and Yarn**: Node 18+ and Yarn 1.22+
- **Platform service running**: The Rust-as-Spec service accessible at a known URL (e.g., `http://localhost:8080`)
- **Basic React/TypeScript knowledge**: Familiarity with React components and TypeScript

## Overview

This guide creates a Backstage plugin with three dashboard cards:

1. **Governance Health Card**: Displays governance status (green/yellow/red) from `/platform/status`
2. **Documentation Health Card**: Shows doc inventory and health metrics from `/platform/docs/index`
3. **AC Coverage Card**: Displays acceptance criteria coverage from `/platform/coverage`

The plugin follows the [IDP Tile Architecture](../adr/0016-idp-tiles-json-contracts.md) and consumes the JSON contracts documented in [json-contracts.md](../explanation/json-contracts.md).

---

## Step 1: Create Plugin Scaffold

If you don't have a Backstage app yet, create one:

```bash
npx @backstage/create-app@latest
# Follow prompts, choose app name (e.g., "my-backstage")
cd my-backstage
```

Create a new frontend plugin for platform governance:

```bash
yarn new --select plugin
# Plugin ID: rust-template-governance
# Owner: your-team
```

This creates a plugin in `plugins/rust-template-governance/`.

**File structure:**
```
plugins/rust-template-governance/
├── src/
│   ├── api/
│   │   └── PlatformClient.ts       # API client (Step 3)
│   ├── components/
│   │   ├── GovernanceHealthCard/   # Health card (Step 4)
│   │   ├── DocsHealthCard/         # Docs card (Step 5)
│   │   └── ExampleComponent/       # Delete this (scaffold default)
│   ├── plugin.ts                   # Plugin definition
│   └── index.ts                    # Public exports
├── dev/
│   └── index.tsx                   # Dev server
└── package.json
```

---

## Step 2: Add Backend Proxy Config

Backstage frontend plugins need a backend proxy to avoid CORS issues when calling external APIs.

**Edit `app-config.yaml` in your Backstage root:**

```yaml
proxy:
  '/rust-template-platform':
    target: 'http://localhost:8080'
    pathRewrite:
      '^/api/proxy/rust-template-platform': '/platform'
    changeOrigin: true
    headers:
      # Optional: Add auth token if your platform requires it
      # Authorization: 'Bearer ${RUST_TEMPLATE_TOKEN}'
```

**Explanation:**
- Requests to `/api/proxy/rust-template-platform/status` proxy to `http://localhost:8080/platform/status`
- `changeOrigin: true` ensures `Host` header matches target
- For production, replace `localhost:8080` with your service URL or use environment variables:

```yaml
proxy:
  '/rust-template-platform':
    target: ${RUST_TEMPLATE_API_URL}  # e.g., https://platform.example.com
```

**Verify proxy works:**

Start Backstage dev server:
```bash
yarn dev
```

In another terminal, test the proxy:
```bash
curl http://localhost:3000/api/proxy/rust-template-platform/status
```

You should see the platform status JSON response.

---

## Step 3: Create API Client

Create a TypeScript client to fetch data from `/platform/*` endpoints.

**Create `plugins/rust-template-governance/src/api/PlatformClient.ts`:**

```typescript
// PlatformClient.ts
export interface GovernanceStatus {
  service: {
    service_id: string;
    template_version: string;
    display_name: string;
    description: string;
  };
  governance: {
    ledger: {
      stories: number;
      requirements: number;
      acs: number;
    };
    devex: {
      commands: number;
      flows: number;
    };
    docs: {
      total: number;
      design: number;
      doc_type_issues: number;
    };
    policies: {
      status: 'passing' | 'failing' | 'unknown';
    };
  };
  config: {
    env: string;
    http_port: number;
  };
}

export interface DocsIndex {
  schema_version: string;
  template_version: string;
  docs: Array<{
    id: string;
    file: string;
    doc_type: string;
    stories: string[];
    requirements: string[];
    acs: string[];
    adrs: string[];
    doc_type_valid: boolean;
    doc_type_issue: string | null;
  }>;
  summary: {
    total: number;
    valid: number;
    with_issues: number;
  };
}

export interface ACCoverageEntry {
  id: string;
  text: string;
  status: 'passing' | 'failing' | 'unknown';
  test_count: number;
  scenarios: string[];
}

export interface ACCoverage {
  template_version: string;
  timestamp: string;
  summary: {
    total: number;
    passing: number;
    failing: number;
    unknown: number;
  };
  acceptance_criteria: ACCoverageEntry[];
}

export class PlatformClient {
  private baseUrl: string;

  constructor(baseUrl: string = '/api/proxy/rust-template-platform') {
    this.baseUrl = baseUrl;
  }

  async getStatus(): Promise<GovernanceStatus> {
    const response = await fetch(`${this.baseUrl}/status`);
    if (!response.ok) {
      throw new Error(`Failed to fetch status: ${response.statusText}`);
    }
    return response.json();
  }

  async getDocsIndex(): Promise<DocsIndex> {
    const response = await fetch(`${this.baseUrl}/docs/index`);
    if (!response.ok) {
      throw new Error(`Failed to fetch docs index: ${response.statusText}`);
    }
    return response.json();
  }

  async getCoverage(): Promise<ACCoverage> {
    const response = await fetch(`${this.baseUrl}/coverage`);
    if (!response.ok) {
      throw new Error(`Failed to fetch coverage: ${response.statusText}`);
    }
    return response.json();
  }
}
```

**Key points:**
- **TypeScript interfaces** match the JSON contracts from ADR-0016
- **Base URL** defaults to Backstage proxy path (`/api/proxy/rust-template-platform`)
- **Error handling** throws on non-2xx responses (Backstage components will handle)

---

## Step 4: Create Governance Health Card Component

This card displays the overall governance health with a green/yellow/red status indicator.

**Create `plugins/rust-template-governance/src/components/GovernanceHealthCard/GovernanceHealthCard.tsx`:**

```tsx
import React from 'react';
import { InfoCard, Progress } from '@backstage/core-components';
import { useAsync } from 'react-use';
import Alert from '@material-ui/lab/Alert';
import { makeStyles, Grid, Typography, Chip } from '@material-ui/core';
import CheckCircleIcon from '@material-ui/icons/CheckCircle';
import ErrorIcon from '@material-ui/icons/Error';
import WarningIcon from '@material-ui/icons/Warning';
import { PlatformClient } from '../../api/PlatformClient';

const useStyles = makeStyles(theme => ({
  statusGood: {
    color: theme.palette.success.main,
  },
  statusBad: {
    color: theme.palette.error.main,
  },
  statusWarning: {
    color: theme.palette.warning.main,
  },
  metricLabel: {
    fontWeight: 600,
    marginRight: theme.spacing(1),
  },
  chipGreen: {
    backgroundColor: theme.palette.success.light,
    color: theme.palette.success.contrastText,
  },
  chipRed: {
    backgroundColor: theme.palette.error.light,
    color: theme.palette.error.contrastText,
  },
  chipYellow: {
    backgroundColor: theme.palette.warning.light,
    color: theme.palette.warning.contrastText,
  },
}));

export const GovernanceHealthCard = () => {
  const classes = useStyles();
  const platformClient = new PlatformClient();

  const { value, loading, error } = useAsync(async () => {
    return await platformClient.getStatus();
  }, []);

  if (loading) {
    return <Progress />;
  }

  if (error) {
    return (
      <Alert severity="error">
        Failed to load governance status: {error.message}
      </Alert>
    );
  }

  if (!value) {
    return <Alert severity="info">No data available</Alert>;
  }

  const { governance, service } = value;
  const isPassing = governance.policies.status === 'passing';
  const hasIssues = governance.docs.doc_type_issues > 0;

  // Determine overall health: green (all good), yellow (issues but passing), red (failing)
  let healthStatus: 'good' | 'warning' | 'bad' = 'good';
  let healthIcon = <CheckCircleIcon className={classes.statusGood} />;
  let healthChipClass = classes.chipGreen;

  if (!isPassing) {
    healthStatus = 'bad';
    healthIcon = <ErrorIcon className={classes.statusBad} />;
    healthChipClass = classes.chipRed;
  } else if (hasIssues) {
    healthStatus = 'warning';
    healthIcon = <WarningIcon className={classes.statusWarning} />;
    healthChipClass = classes.chipYellow;
  }

  return (
    <InfoCard title="Governance Health">
      <Grid container spacing={2}>
        <Grid item xs={12}>
          <Typography variant="h6" gutterBottom>
            {healthIcon} {service.display_name || service.service_id}
          </Typography>
          <Chip
            label={isPassing ? 'Policies Passing' : 'Policies Failing'}
            className={healthChipClass}
            size="small"
          />
          <Typography variant="caption" display="block" color="textSecondary">
            Template: {service.template_version}
          </Typography>
        </Grid>

        <Grid item xs={6}>
          <Typography className={classes.metricLabel} variant="body2">
            Stories:
          </Typography>
          <Typography variant="h6">{governance.ledger.stories}</Typography>
        </Grid>

        <Grid item xs={6}>
          <Typography className={classes.metricLabel} variant="body2">
            Requirements:
          </Typography>
          <Typography variant="h6">{governance.ledger.requirements}</Typography>
        </Grid>

        <Grid item xs={6}>
          <Typography className={classes.metricLabel} variant="body2">
            ACs:
          </Typography>
          <Typography variant="h6">{governance.ledger.acs}</Typography>
        </Grid>

        <Grid item xs={6}>
          <Typography className={classes.metricLabel} variant="body2">
            Commands:
          </Typography>
          <Typography variant="h6">{governance.devex.commands}</Typography>
        </Grid>

        <Grid item xs={12}>
          <Typography className={classes.metricLabel} variant="body2">
            Docs:
          </Typography>
          <Typography variant="body1">
            {governance.docs.total} total
            {hasIssues && (
              <span style={{ color: 'orange' }}>
                {' '}
                ({governance.docs.doc_type_issues} with issues)
              </span>
            )}
          </Typography>
        </Grid>
      </Grid>
    </InfoCard>
  );
};
```

**Key features:**
- **Health indicator**: Green (policies pass, no issues), Yellow (policies pass, doc issues), Red (policies fail)
- **Metrics display**: Shows stories, requirements, ACs, commands, and doc counts
- **Loading/error states**: Backstage standard `Progress` and `Alert` components
- **Material-UI styling**: Follows Backstage design system

---

## Step 5: Create Docs Health Card Component

This card shows documentation inventory and highlights any docs with health issues.

**Create `plugins/rust-template-governance/src/components/DocsHealthCard/DocsHealthCard.tsx`:**

```tsx
import React from 'react';
import { InfoCard, Progress, Table, TableColumn } from '@backstage/core-components';
import { useAsync } from 'react-use';
import Alert from '@material-ui/lab/Alert';
import { makeStyles, Typography, Chip, Box } from '@material-ui/core';
import { PlatformClient } from '../../api/PlatformClient';

const useStyles = makeStyles(theme => ({
  summaryBox: {
    marginBottom: theme.spacing(2),
  },
  chipValid: {
    backgroundColor: theme.palette.success.light,
  },
  chipInvalid: {
    backgroundColor: theme.palette.error.light,
  },
}));

interface DocsTableRow {
  id: string;
  doc_type: string;
  file: string;
  status: string;
  issue?: string;
}

export const DocsHealthCard = () => {
  const classes = useStyles();
  const platformClient = new PlatformClient();

  const { value, loading, error } = useAsync(async () => {
    return await platformClient.getDocsIndex();
  }, []);

  if (loading) {
    return <Progress />;
  }

  if (error) {
    return (
      <Alert severity="error">
        Failed to load docs index: {error.message}
      </Alert>
    );
  }

  if (!value) {
    return <Alert severity="info">No data available</Alert>;
  }

  const { docs, summary } = value;

  // Show only docs with issues, or first 10 docs if all valid
  const docsToShow = summary.with_issues > 0
    ? docs.filter(d => !d.doc_type_valid)
    : docs.slice(0, 10);

  const columns: TableColumn<DocsTableRow>[] = [
    { title: 'ID', field: 'id' },
    { title: 'Type', field: 'doc_type' },
    { title: 'File', field: 'file' },
    { title: 'Status', field: 'status', render: row => (
      <Chip
        label={row.status}
        size="small"
        className={row.status === 'Valid' ? classes.chipValid : classes.chipInvalid}
      />
    )},
    { title: 'Issue', field: 'issue' },
  ];

  const data: DocsTableRow[] = docsToShow.map(doc => ({
    id: doc.id,
    doc_type: doc.doc_type,
    file: doc.file,
    status: doc.doc_type_valid ? 'Valid' : 'Invalid',
    issue: doc.doc_type_issue || '',
  }));

  return (
    <InfoCard title="Documentation Health">
      <Box className={classes.summaryBox}>
        <Typography variant="h6">
          {summary.total} docs ({summary.valid} valid, {summary.with_issues} with issues)
        </Typography>
      </Box>

      <Table
        title={summary.with_issues > 0 ? 'Docs with Issues' : 'Recent Docs'}
        options={{ paging: false, search: false, toolbar: true }}
        columns={columns}
        data={data}
      />
    </InfoCard>
  );
};
```

**Key features:**
- **Summary**: Total docs, valid count, issue count
- **Conditional display**: Shows only docs with issues if any exist, otherwise shows first 10
- **Table view**: Backstage `Table` component with ID, type, file, status, and issue columns
- **Status chips**: Green for valid, red for invalid

---

## Step 6: Register Cards in Backstage App

Now integrate the cards into your Backstage home page or entity page.

**6.1: Export components from plugin**

Edit `plugins/rust-template-governance/src/index.ts`:

```typescript
export { rustTemplateGovernancePlugin } from './plugin';
export { GovernanceHealthCard } from './components/GovernanceHealthCard';
export { DocsHealthCard } from './components/DocsHealthCard';
```

**6.2: Install plugin in app**

Edit `packages/app/package.json`:

```json
{
  "dependencies": {
    "@internal/plugin-rust-template-governance": "^0.1.0"
  }
}
```

Then run:
```bash
yarn install
```

**6.3: Add cards to home page**

Edit `packages/app/src/components/home/HomePage.tsx`:

```tsx
import React from 'react';
import { Content, Page, Header } from '@backstage/core-components';
import { HomePageToolkit } from '@backstage/plugin-home';
import { Grid } from '@material-ui/core';
import { GovernanceHealthCard, DocsHealthCard } from '@internal/plugin-rust-template-governance';

export const HomePage = () => {
  return (
    <Page themeId="home">
      <Header title="Welcome to Backstage" />
      <Content>
        <Grid container spacing={3}>
          <Grid item xs={12} md={6}>
            <GovernanceHealthCard />
          </Grid>
          <Grid item xs={12} md={6}>
            <DocsHealthCard />
          </Grid>
          <Grid item xs={12}>
            <HomePageToolkit
              tools={[
                /* other tools */
              ]}
            />
          </Grid>
        </Grid>
      </Content>
    </Page>
  );
};
```

**6.4: (Optional) Add to entity page**

To show governance cards for specific services, edit `packages/app/src/components/catalog/EntityPage.tsx`:

```tsx
import { GovernanceHealthCard, DocsHealthCard } from '@internal/plugin-rust-template-governance';

// In your serviceEntityPage definition:
const serviceEntityPage = (
  <EntityLayout>
    <EntityLayout.Route path="/" title="Overview">
      <Grid container spacing={3}>
        {/* Existing cards */}
        <Grid item xs={12} md={6}>
          <GovernanceHealthCard />
        </Grid>
        <Grid item xs={12} md={6}>
          <DocsHealthCard />
        </Grid>
      </Grid>
    </EntityLayout.Route>
    {/* Other routes */}
  </EntityLayout>
);
```

---

## Step 7: Testing and Validation

**7.1: Start services**

Terminal 1 (Rust platform):
```bash
cd /path/to/rust-template
cargo run -p app-http
```

Terminal 2 (Backstage):
```bash
cd my-backstage
yarn dev
```

**7.2: Verify proxy**

In browser console or terminal:
```bash
curl http://localhost:3000/api/proxy/rust-template-platform/status
```

Expected: JSON response with governance status.

**7.3: Check Backstage home page**

Navigate to `http://localhost:3000` and verify:
- **Governance Health Card** shows green/yellow/red status
- **Docs Health Card** shows doc counts and any issues
- Cards load without errors in browser console

**7.4: Test error handling**

Stop the Rust platform service and refresh Backstage:
- Cards should show error alerts: "Failed to load governance status"

**7.5: Test with real data**

In Rust platform, trigger governance failures:
```bash
# Intentionally break a test to see red status
cargo test -- --exact some_failing_test
```

Refresh Backstage and verify the health card turns red.

**7.6: Production readiness checklist**

- [ ] Proxy `target` uses environment variable (`${RUST_TEMPLATE_API_URL}`)
- [ ] Authentication header added if platform requires auth
- [ ] CORS properly configured (proxy handles this)
- [ ] Error states display user-friendly messages
- [ ] Loading states show `Progress` component
- [ ] Cards refresh on navigation (Backstage handles via `useAsync`)

---

## Advanced: Adding AC Coverage Card

To display AC coverage metrics, create a third card:

**Create `plugins/rust-template-governance/src/components/ACCoverageCard/ACCoverageCard.tsx`:**

```tsx
import React from 'react';
import { InfoCard, Progress } from '@backstage/core-components';
import { useAsync } from 'react-use';
import Alert from '@material-ui/lab/Alert';
import { makeStyles, Grid, Typography, LinearProgress } from '@material-ui/core';
import { PlatformClient } from '../../api/PlatformClient';

const useStyles = makeStyles(theme => ({
  progressBar: {
    height: 10,
    borderRadius: 5,
  },
  metricLabel: {
    fontWeight: 600,
  },
}));

export const ACCoverageCard = () => {
  const classes = useStyles();
  const platformClient = new PlatformClient();

  const { value, loading, error } = useAsync(async () => {
    return await platformClient.getCoverage();
  }, []);

  if (loading) {
    return <Progress />;
  }

  if (error) {
    return (
      <Alert severity="error">
        Failed to load AC coverage: {error.message}
      </Alert>
    );
  }

  if (!value) {
    return <Alert severity="info">No data available</Alert>;
  }

  const { summary } = value;
  const passingPct = summary.total > 0
    ? Math.round((summary.passing / summary.total) * 100)
    : 0;

  return (
    <InfoCard title="AC Coverage">
      <Grid container spacing={2}>
        <Grid item xs={12}>
          <Typography className={classes.metricLabel} variant="body2">
            Coverage: {passingPct}%
          </Typography>
          <LinearProgress
            variant="determinate"
            value={passingPct}
            className={classes.progressBar}
            color={passingPct >= 80 ? 'primary' : 'secondary'}
          />
        </Grid>

        <Grid item xs={4}>
          <Typography variant="caption" color="textSecondary">
            Total
          </Typography>
          <Typography variant="h6">{summary.total}</Typography>
        </Grid>

        <Grid item xs={4}>
          <Typography variant="caption" color="textSecondary">
            Passing
          </Typography>
          <Typography variant="h6" style={{ color: 'green' }}>
            {summary.passing}
          </Typography>
        </Grid>

        <Grid item xs={4}>
          <Typography variant="caption" color="textSecondary">
            Failing
          </Typography>
          <Typography variant="h6" style={{ color: 'red' }}>
            {summary.failing}
          </Typography>
        </Grid>
      </Grid>
    </InfoCard>
  );
};
```

Export from `src/index.ts` and add to your home page or entity page.

---

## Troubleshooting

### Issue: CORS errors

**Symptom**: Browser console shows `CORS policy: No 'Access-Control-Allow-Origin' header`

**Solution**: Verify backend proxy is configured correctly in `app-config.yaml` and Backstage backend is running.

### Issue: 404 on `/platform/status`

**Symptom**: `Failed to fetch status: Not Found`

**Solution**:
1. Verify Rust platform service is running: `curl http://localhost:8080/platform/status`
2. Check proxy `pathRewrite` is correct (strips `/api/proxy/rust-template-platform` prefix)

### Issue: Cards don't refresh

**Symptom**: Old data persists after platform changes

**Solution**: Backstage caches responses. Hard refresh browser (Ctrl+Shift+R) or add cache-busting headers in proxy config.

### Issue: TypeScript errors on `PlatformClient`

**Symptom**: Type errors when importing `PlatformClient`

**Solution**: Ensure `tsconfig.json` in plugin includes `"esModuleInterop": true` and `"strict": false` (or fix type issues).

---

## Next Steps

1. **Add more cards**: Implement cards for `/platform/tasks`, `/platform/friction`, `/platform/agent/hints`
2. **Entity annotations**: Use Backstage `catalog-info.yaml` annotations to link services to their platform URLs
3. **Authentication**: Add OAuth or API key auth to proxy config
4. **Polling**: Add auto-refresh with `setInterval` or Backstage `useInterval` hook
5. **Filtering**: Add filters to docs/AC cards (by story, requirement, status)
6. **Deep links**: Link card metrics to platform UI (`/ui/graph`, `/ui/coverage`)

---

## Related Documentation

- **JSON Contracts**: [docs/explanation/json-contracts.md](../explanation/json-contracts.md) - Full API contract reference
- **ADR-0016**: [docs/adr/0016-idp-tiles-json-contracts.md](../adr/0016-idp-tiles-json-contracts.md) - IDP tile architecture decision
- **Platform API**: [AGENT_GUIDE.md](../AGENT_GUIDE.md) - Platform API usage guide
- **Backstage Docs**: [backstage.io/docs](https://backstage.io/docs) - Official Backstage documentation

---

## Summary

This guide demonstrated:

1. **Scaffold**: Created a Backstage plugin using `yarn new --select plugin`
2. **Proxy**: Configured backend proxy to avoid CORS issues
3. **API Client**: Built TypeScript client matching JSON contracts from ADR-0016
4. **Cards**: Created Governance Health, Docs Health, and AC Coverage cards
5. **Integration**: Registered cards in Backstage home page and entity pages
6. **Validation**: Tested loading, error handling, and real-time updates

The plugin now provides real-time governance visibility in your IDP, leveraging the platform's stable JSON contracts for reliable integration.
