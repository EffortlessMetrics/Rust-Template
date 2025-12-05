/**
 * Governance Health Card Component
 *
 * EXAMPLE QUALITY - NOT PRODUCTION READY
 *
 * Displays governance health metrics from /platform/status endpoint.
 * Shows ledger counts, policy status, friction, and questions.
 */

import React, { useEffect, useState } from 'react';
import {
  Divider,
  makeStyles,
  Typography,
  Chip,
  Box,
  Grid,
} from '@material-ui/core';
import CheckIcon from '@material-ui/icons/CheckCircle';
import ErrorIcon from '@material-ui/icons/Error';
import WarningIcon from '@material-ui/icons/Warning';
import { InfoCard, Progress } from '@backstage/core-components';
import { PlatformClient, PlatformStatus, PlatformAPIError } from '../api/PlatformClient';

const useStyles = makeStyles(theme => ({
  healthIndicator: {
    display: 'flex',
    alignItems: 'center',
    gap: theme.spacing(1),
    marginBottom: theme.spacing(2),
  },
  metric: {
    display: 'flex',
    justifyContent: 'space-between',
    marginBottom: theme.spacing(1),
  },
  metricBox: {
    textAlign: 'center',
    padding: theme.spacing(1),
  },
  metricValue: {
    fontSize: '1.5rem',
    fontWeight: 'bold',
  },
  metricLabel: {
    color: theme.palette.text.secondary,
    fontSize: '0.75rem',
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
  frictionItem: {
    padding: theme.spacing(1),
    marginBottom: theme.spacing(1),
    borderRadius: 4,
    backgroundColor: 'rgba(0, 0, 0, 0.02)',
  },
}));

/**
 * StatusChip - displays a status badge
 */
const StatusChip: React.FC<{ label: string; status: 'pass' | 'fail' | 'unknown' }> = ({
  label,
  status,
}) => {
  const classes = useStyles();
  const isPass = status === 'pass';
  const isUnknown = status === 'unknown';

  // Unknown status uses warning icon and default color
  if (isUnknown) {
    return (
      <Chip
        size="small"
        label={`${label} (unknown)`}
        icon={<WarningIcon />}
        color="default"
        className={classes.statusChip}
      />
    );
  }

  return (
    <Chip
      size="small"
      label={label}
      icon={isPass ? <CheckIcon /> : <ErrorIcon />}
      color={isPass ? 'primary' : 'secondary'}
      className={classes.statusChip}
    />
  );
};

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
 * - Ledger counts (stories, requirements, ACs)
 * - Policy enforcement status
 * - Friction tracking
 * - Open questions
 * - Auto-refresh capability
 *
 * Example usage:
 *   <GovernanceHealthCard />
 *   <GovernanceHealthCard baseUrl="http://localhost:9090" refreshInterval={60000} />
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
    return undefined;
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
          Unable to fetch governance status. Check that the platform service is
          running and accessible.
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

  const { service, governance } = status;
  const { ledger, policies, friction, questions } = governance;

  // Determine overall health status based on policies and friction
  const isPolicyPassing = policies.status === 'pass';
  const hasCriticalFriction = friction.by_severity.critical > 0;
  const isHealthy = isPolicyPassing && !hasCriticalFriction;

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

      {/* Ledger Counts Section */}
      <Box mt={2}>
        <Typography variant="subtitle2" gutterBottom>
          Governance Ledger
        </Typography>
        <Grid container spacing={2}>
          <Grid item xs={4}>
            <Box className={classes.metricBox}>
              <Typography className={classes.metricValue}>
                {ledger.stories}
              </Typography>
              <Typography className={classes.metricLabel}>Stories</Typography>
            </Box>
          </Grid>
          <Grid item xs={4}>
            <Box className={classes.metricBox}>
              <Typography className={classes.metricValue}>
                {ledger.requirements}
              </Typography>
              <Typography className={classes.metricLabel}>
                Requirements
              </Typography>
            </Box>
          </Grid>
          <Grid item xs={4}>
            <Box className={classes.metricBox}>
              <Typography className={classes.metricValue}>
                {ledger.acs}
              </Typography>
              <Typography className={classes.metricLabel}>ACs</Typography>
            </Box>
          </Grid>
        </Grid>
      </Box>

      <Divider style={{ marginTop: 16, marginBottom: 16 }} />

      {/* Policy Status Section */}
      <Box>
        <Typography variant="subtitle2" gutterBottom>
          Policy Enforcement
        </Typography>
        <Box display="flex" flexWrap="wrap" mt={1}>
          <StatusChip label="Policies" status={policies.status} />
        </Box>
      </Box>

      <Divider style={{ marginTop: 16, marginBottom: 16 }} />

      {/* Friction Summary Section */}
      <Box>
        <Typography variant="subtitle2" gutterBottom>
          Friction ({friction.open} open)
        </Typography>
        <Grid container spacing={1}>
          <Grid item xs={3}>
            <Chip
              size="small"
              label={`${friction.by_severity.critical} critical`}
              color={friction.by_severity.critical > 0 ? 'secondary' : 'default'}
            />
          </Grid>
          <Grid item xs={3}>
            <Chip
              size="small"
              label={`${friction.by_severity.high} high`}
              color={friction.by_severity.high > 0 ? 'secondary' : 'default'}
            />
          </Grid>
          <Grid item xs={3}>
            <Chip
              size="small"
              label={`${friction.by_severity.medium} med`}
              color={friction.by_severity.medium > 0 ? 'primary' : 'default'}
            />
          </Grid>
          <Grid item xs={3}>
            <Chip size="small" label={`${friction.by_severity.low} low`} />
          </Grid>
        </Grid>

        {friction.recent.length > 0 && (
          <Box mt={2}>
            <Typography variant="caption" color="textSecondary">
              Recent friction:
            </Typography>
            {friction.recent.slice(0, 2).map(f => (
              <Box key={f.id} className={classes.frictionItem}>
                <Typography variant="body2">
                  <strong>{f.id}</strong>: {f.summary}
                </Typography>
                <Typography variant="caption" color="textSecondary">
                  {f.severity} - {f.category}
                </Typography>
              </Box>
            ))}
          </Box>
        )}
      </Box>

      <Divider style={{ marginTop: 16, marginBottom: 16 }} />

      {/* Questions Section */}
      <Box>
        <Typography variant="subtitle2" gutterBottom>
          Open Questions ({questions.open})
        </Typography>
        {questions.top_open.length > 0 ? (
          questions.top_open.slice(0, 2).map(q => (
            <Box key={q.id} className={classes.frictionItem}>
              <Typography variant="body2">
                <strong>{q.id}</strong>: {q.summary}
              </Typography>
              <Typography variant="caption" color="textSecondary">
                Flow: {q.flow}
              </Typography>
            </Box>
          ))
        ) : (
          <Typography variant="body2" color="textSecondary">
            No open questions
          </Typography>
        )}
      </Box>

      {/* Metadata Footer */}
      <Box mt={2}>
        <Typography variant="caption" color="textSecondary">
          {service.display_name ?? service.service_id} - Template {service.template_version}
        </Typography>
      </Box>
    </InfoCard>
  );
};
