/**
 * Documentation Health Card Component
 *
 * EXAMPLE QUALITY - NOT PRODUCTION READY
 *
 * Displays documentation inventory metrics from /platform/docs/index endpoint.
 * Shows total docs, health status, and breakdown by document type.
 */

import React, { useEffect, useState } from 'react';
import {
  makeStyles,
  Typography,
  Box,
  Grid,
  List,
  ListItem,
  ListItemText,
  Chip,
} from '@material-ui/core';
import CheckIcon from '@material-ui/icons/CheckCircle';
import WarningIcon from '@material-ui/icons/Warning';
import ErrorIcon from '@material-ui/icons/Error';
import { InfoCard, Progress } from '@backstage/core-components';
import { PlatformClient, DocsIndex, PlatformAPIError } from '../api/PlatformClient';

const useStyles = makeStyles(theme => ({
  metric: {
    textAlign: 'center',
    padding: theme.spacing(2),
  },
  metricValue: {
    fontSize: '2rem',
    fontWeight: 'bold',
  },
  metricLabel: {
    color: theme.palette.text.secondary,
    marginTop: theme.spacing(0.5),
  },
  typeList: {
    padding: 0,
  },
  typeItem: {
    paddingLeft: 0,
    paddingRight: 0,
  },
  healthChip: {
    margin: theme.spacing(0.5),
  },
  errorText: {
    color: theme.palette.error.main,
  },
  warningText: {
    color: theme.palette.warning.main,
  },
  successText: {
    color: theme.palette.success.main,
  },
  icon: {
    marginRight: theme.spacing(1),
  },
}));

/**
 * DocTypeItem - displays a single document type count
 */
const DocTypeItem: React.FC<{ label: string; count: number; icon: string }> = ({
  label,
  count,
  icon,
}) => {
  const classes = useStyles();

  return (
    <ListItem className={classes.typeItem}>
      <ListItemText
        primary={
          <Box display="flex" justifyContent="space-between" alignItems="center">
            <Box display="flex" alignItems="center">
              <span style={{ marginRight: 8 }}>{icon}</span>
              <Typography variant="body2">{label}</Typography>
            </Box>
            <Chip
              label={count}
              size="small"
              color={count > 0 ? 'primary' : 'default'}
            />
          </Box>
        }
      />
    </ListItem>
  );
};

interface DocsHealthCardProps {
  /**
   * Optional custom base URL (defaults to Backstage proxy)
   */
  baseUrl?: string;

  /**
   * Refresh interval in milliseconds (default: 60000 = 60 seconds)
   * Set to 0 to disable auto-refresh
   */
  refreshInterval?: number;
}

/**
 * DocsHealthCard displays documentation inventory metrics
 *
 * Features:
 * - Total document count
 * - Health indicators (valid vs. with issues)
 * - Breakdown by document type (ADRs, design docs, how-tos, etc.)
 * - Auto-refresh capability
 *
 * Example usage:
 *   <DocsHealthCard />
 *   <DocsHealthCard baseUrl="http://localhost:8080" refreshInterval={120000} />
 */
export const DocsHealthCard: React.FC<DocsHealthCardProps> = ({
  baseUrl = '/api/proxy/rust-spec-platform',
  refreshInterval = 60000,
}) => {
  const classes = useStyles();
  const [docs, setDocs] = useState<DocsIndex | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const client = new PlatformClient(baseUrl);

    const fetchDocs = async () => {
      try {
        const data = await client.getDocsIndex();
        setDocs(data);
        setError(null);
      } catch (err) {
        if (err instanceof PlatformAPIError) {
          setError(`API Error: ${err.message}`);
        } else if (err instanceof Error) {
          setError(err.message);
        } else {
          setError('Unknown error occurred');
        }
      } finally {
        setLoading(false);
      }
    };

    // Initial fetch
    fetchDocs();

    // Set up auto-refresh if enabled
    if (refreshInterval > 0) {
      const intervalId = setInterval(fetchDocs, refreshInterval);
      return () => clearInterval(intervalId);
    }
    return undefined;
  }, [baseUrl, refreshInterval]);

  if (loading) {
    return (
      <InfoCard title="Documentation Health">
        <Progress />
      </InfoCard>
    );
  }

  if (error) {
    return (
      <InfoCard title="Documentation Health">
        <Box display="flex" alignItems="center" mb={2}>
          <ErrorIcon color="error" className={classes.icon} />
          <Typography className={classes.errorText}>{error}</Typography>
        </Box>
        <Typography variant="body2" color="textSecondary">
          Unable to fetch documentation index. Check that the platform service is running
          and accessible.
        </Typography>
      </InfoCard>
    );
  }

  if (!docs) {
    return (
      <InfoCard title="Documentation Health">
        <Typography>No data available</Typography>
      </InfoCard>
    );
  }

  const { summary } = docs;
  const healthPercentage = summary.total > 0
    ? ((summary.valid / summary.total) * 100).toFixed(1)
    : '0.0';

  // Determine health status
  const isHealthy = summary.with_issues === 0;
  const healthColor = isHealthy ? classes.successText : classes.warningText;

  return (
    <InfoCard title="Documentation Health">
      {/* Summary Metrics */}
      <Grid container spacing={2}>
        <Grid item xs={4}>
          <Box className={classes.metric}>
            <Typography variant="h4" className={classes.metricValue}>
              {summary.total}
            </Typography>
            <Typography variant="caption" className={classes.metricLabel}>
              Total Docs
            </Typography>
          </Box>
        </Grid>
        <Grid item xs={4}>
          <Box className={classes.metric}>
            <Typography variant="h4" className={`${classes.metricValue} ${classes.successText}`}>
              {summary.valid}
            </Typography>
            <Typography variant="caption" className={classes.metricLabel}>
              Valid
            </Typography>
          </Box>
        </Grid>
        <Grid item xs={4}>
          <Box className={classes.metric}>
            <Typography variant="h4" className={`${classes.metricValue} ${classes.warningText}`}>
              {summary.with_issues}
            </Typography>
            <Typography variant="caption" className={classes.metricLabel}>
              With Issues
            </Typography>
          </Box>
        </Grid>
      </Grid>

      {/* Health Indicator */}
      <Box mt={2} mb={2} display="flex" alignItems="center" justifyContent="center">
        {isHealthy ? (
          <CheckIcon className={classes.successText} />
        ) : (
          <WarningIcon className={classes.warningText} />
        )}
        <Typography variant="body1" style={{ marginLeft: 8 }} className={healthColor}>
          {healthPercentage}% Healthy
        </Typography>
      </Box>

      {/* Document Type Breakdown */}
      <Box mt={3}>
        <Typography variant="subtitle2" gutterBottom>
          By Document Type
        </Typography>
        <List className={classes.typeList}>
          <DocTypeItem
            label="Architecture Decision Records"
            count={summary.by_type.adr}
            icon="📋"
          />
          <DocTypeItem
            label="Design Documents"
            count={summary.by_type.design}
            icon="🎨"
          />
          <DocTypeItem
            label="How-To Guides"
            count={summary.by_type['how-to']}
            icon="📖"
          />
          <DocTypeItem
            label="Explanation"
            count={summary.by_type.explanation}
            icon="💡"
          />
          <DocTypeItem
            label="Reference"
            count={summary.by_type.reference}
            icon="📚"
          />
        </List>
      </Box>

      {/* Issues Summary */}
      {summary.with_issues > 0 && (
        <Box mt={2} p={2} bgcolor="rgba(255, 152, 0, 0.1)" borderRadius={4}>
          <Box display="flex" alignItems="center">
            <WarningIcon className={classes.warningText} />
            <Typography variant="body2" style={{ marginLeft: 8 }}>
              {summary.with_issues} document{summary.with_issues !== 1 ? 's' : ''} with issues
            </Typography>
          </Box>
          <Typography variant="caption" color="textSecondary" style={{ marginTop: 4 }}>
            Check documentation inventory for details
          </Typography>
        </Box>
      )}

      {/* Perfect Health Message */}
      {summary.with_issues === 0 && summary.total > 0 && (
        <Box mt={2} p={2} bgcolor="rgba(76, 175, 80, 0.1)" borderRadius={4}>
          <Box display="flex" alignItems="center">
            <CheckIcon className={classes.successText} />
            <Typography variant="body2" style={{ marginLeft: 8 }}>
              All documentation is valid
            </Typography>
          </Box>
        </Box>
      )}
    </InfoCard>
  );
};
