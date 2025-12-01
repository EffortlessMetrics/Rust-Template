/**
 * Governance Health Card Component
 *
 * EXAMPLE QUALITY - NOT PRODUCTION READY
 *
 * Displays governance health metrics from /platform/status endpoint.
 * Shows AC coverage, policy status, and selftest gate health.
 */

import React, { useEffect, useState } from 'react';
import {
  Card,
  CardContent,
  CardHeader,
  Divider,
  LinearProgress,
  makeStyles,
  Typography,
  Chip,
  Box,
  Grid,
} from '@material-ui/core';
import {
  CheckCircle as CheckIcon,
  Error as ErrorIcon,
  Warning as WarningIcon,
} from '@material-ui/icons';
import { InfoCard, Progress } from '@backstage/core-components';
import { PlatformClient, PlatformStatus, PlatformAPIError } from '../api/PlatformClient';

const useStyles = makeStyles(theme => ({
  healthIndicator: {
    display: 'flex',
    alignItems: 'center',
    gap: theme.spacing(1),
    marginBottom: theme.spacing(2),
  },
  coverageBar: {
    height: 10,
    borderRadius: 5,
    marginTop: theme.spacing(1),
    marginBottom: theme.spacing(0.5),
  },
  metric: {
    display: 'flex',
    justifyContent: 'space-between',
    marginBottom: theme.spacing(1),
  },
  statusChip: {
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
}));

interface GovernanceHealthCardProps {
  /**
   * Optional custom base URL (defaults to Backstage proxy)
   */
  baseUrl?: string;

  /**
   * Refresh interval in milliseconds (default: 30000 = 30 seconds)
   * Set to 0 to disable auto-refresh
   */
  refreshInterval?: number;
}

/**
 * GovernanceHealthCard displays real-time governance metrics
 *
 * Features:
 * - AC coverage with progress bar
 * - Policy enforcement status badges
 * - Selftest gate indicator
 * - Auto-refresh capability
 *
 * Example usage:
 *   <GovernanceHealthCard />
 *   <GovernanceHealthCard baseUrl="http://localhost:8080" refreshInterval={60000} />
 */
export const GovernanceHealthCard: React.FC<GovernanceHealthCardProps> = ({
  baseUrl = '/api/proxy/rust-spec-platform',
  refreshInterval = 30000,
}) => {
  const classes = useStyles();
  const [status, setStatus] = useState<PlatformStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const client = new PlatformClient(baseUrl);

    const fetchStatus = async () => {
      try {
        const data = await client.getStatus();
        setStatus(data);
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
    fetchStatus();

    // Set up auto-refresh if enabled
    if (refreshInterval > 0) {
      const intervalId = setInterval(fetchStatus, refreshInterval);
      return () => clearInterval(intervalId);
    }
  }, [baseUrl, refreshInterval]);

  if (loading) {
    return (
      <InfoCard title="Governance Health">
        <Progress />
      </InfoCard>
    );
  }

  if (error) {
    return (
      <InfoCard title="Governance Health">
        <Box className={classes.healthIndicator}>
          <ErrorIcon color="error" />
          <Typography className={classes.errorText}>{error}</Typography>
        </Box>
        <Typography variant="body2" color="textSecondary">
          Unable to fetch governance status. Check that the platform service is running
          and accessible.
        </Typography>
      </InfoCard>
    );
  }

  if (!status) {
    return (
      <InfoCard title="Governance Health">
        <Typography>No data available</Typography>
      </InfoCard>
    );
  }

  const { governance } = status;
  const { ac_coverage, policy_status, selftest_passing } = governance;

  // Determine overall health status
  const isHealthy = selftest_passing && ac_coverage.percentage >= 80;
  const healthIcon = isHealthy ? (
    <CheckIcon className={classes.successText} />
  ) : (
    <WarningIcon className={classes.warningText} />
  );
  const healthText = isHealthy ? 'Healthy' : 'Needs Attention';
  const healthColor = isHealthy ? classes.successText : classes.warningText;

  return (
    <InfoCard title="Governance Health">
      {/* Overall Health Indicator */}
      <Box className={classes.healthIndicator}>
        {healthIcon}
        <Typography variant="h6" className={healthColor}>
          {healthText}
        </Typography>
      </Box>

      <Divider />

      {/* AC Coverage Section */}
      <Box mt={2}>
        <Typography variant="subtitle2" gutterBottom>
          Acceptance Criteria Coverage
        </Typography>
        <LinearProgress
          variant="determinate"
          value={ac_coverage.percentage}
          className={classes.coverageBar}
          color={ac_coverage.percentage >= 80 ? 'primary' : 'secondary'}
        />
        <Typography variant="caption" color="textSecondary">
          {ac_coverage.percentage.toFixed(1)}% ({ac_coverage.passing}/{ac_coverage.total} passing)
        </Typography>

        <Box mt={1}>
          <Grid container spacing={1}>
            <Grid item xs={4}>
              <Typography variant="caption" color="textSecondary">
                Passing
              </Typography>
              <Typography variant="body2" className={classes.successText}>
                {ac_coverage.passing}
              </Typography>
            </Grid>
            <Grid item xs={4}>
              <Typography variant="caption" color="textSecondary">
                Failing
              </Typography>
              <Typography variant="body2" className={classes.errorText}>
                {ac_coverage.failing}
              </Typography>
            </Grid>
            <Grid item xs={4}>
              <Typography variant="caption" color="textSecondary">
                Pending
              </Typography>
              <Typography variant="body2" className={classes.warningText}>
                {ac_coverage.pending}
              </Typography>
            </Grid>
          </Grid>
        </Box>
      </Box>

      <Divider style={{ marginTop: 16, marginBottom: 16 }} />

      {/* Policy Status Section */}
      <Box>
        <Typography variant="subtitle2" gutterBottom>
          Policy Enforcement
        </Typography>
        <Box display="flex" flexWrap="wrap" mt={1}>
          <PolicyChip label="Skills" valid={policy_status.skills_valid} />
          <PolicyChip label="Agents" valid={policy_status.agents_valid} />
          <PolicyChip label="ADRs" valid={policy_status.adrs_valid} />
          <PolicyChip label="Specs" valid={policy_status.specs_valid} />
          <PolicyChip label="BDD" valid={policy_status.bdd_valid} />
        </Box>
      </Box>

      <Divider style={{ marginTop: 16, marginBottom: 16 }} />

      {/* Selftest Gate Section */}
      <Box>
        <Typography variant="subtitle2" gutterBottom>
          Selftest Gate
        </Typography>
        <Box display="flex" alignItems="center" mt={1}>
          {selftest_passing ? (
            <>
              <CheckIcon fontSize="small" className={classes.successText} />
              <Typography variant="body2" style={{ marginLeft: 8 }}>
                Passing
              </Typography>
            </>
          ) : (
            <>
              <ErrorIcon fontSize="small" className={classes.errorText} />
              <Typography variant="body2" style={{ marginLeft: 8 }}>
                Failing
              </Typography>
            </>
          )}
        </Box>
      </Box>

      {/* Metadata Footer */}
      <Box mt={2}>
        <Typography variant="caption" color="textSecondary">
          Template v{status.metadata.template_version} • Last updated:{' '}
          {new Date(governance.last_validated).toLocaleString()}
        </Typography>
      </Box>
    </InfoCard>
  );
};

/**
 * PolicyChip - displays a policy validation status badge
 */
const PolicyChip: React.FC<{ label: string; valid: boolean }> = ({ label, valid }) => {
  const classes = useStyles();
  return (
    <Chip
      size="small"
      label={label}
      icon={valid ? <CheckIcon /> : <ErrorIcon />}
      color={valid ? 'primary' : 'secondary'}
      className={classes.statusChip}
    />
  );
};
