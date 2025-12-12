//! Platform status contract types.
use serde::{Deserialize, Serialize};

/// Platform governance status response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformStatus {
    pub governance: GovernanceStatus,
    pub config: ConfigSummary,
}

/// Governance health summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceStatus {
    pub ledger: LedgerCounts,
    pub ac_coverage: CoverageSummary,
}

/// Spec ledger counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerCounts {
    pub stories: usize,
    pub requirements: usize,
    pub acs: usize,
}

/// AC coverage summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageSummary {
    pub total: usize,
    pub passing: usize,
    pub failing: usize,
    pub unknown: usize,
}

/// Configuration summary (redacted).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSummary {
    pub env: Option<String>,
    pub http_port: u16,
}
