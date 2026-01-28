//! Economics receipt types for DevLT and compute tracking.
//!
//! The economics receipt captures developer time, compute spend, iteration
//! counts, and value delivered. It allows unknowns via confidence levels.

use serde::{Deserialize, Serialize};

/// Economics receipt for tracking DevLT and compute spend.
///
/// This receipt captures the economic cost of a PR including developer time,
/// compute resources, iteration counts, and value delivered.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EconomicsReceipt {
    /// Schema version for forward compatibility.
    pub schema_version: String,

    /// PR number this receipt is associated with.
    pub pr: u64,

    /// Run ID for correlation with gate receipts.
    pub run_id: String,

    /// Developer time tracking.
    pub devlt_minutes: DevLtMinutes,

    /// Compute spend tracking.
    pub compute: ComputeSpend,

    /// Iteration tracking.
    pub iterations: Iterations,

    /// Value delivered by this PR.
    pub value_delivered: ValueDelivered,
}

/// Confidence level for measurements.
///
/// Use these consistently to distinguish between actual measurements
/// and estimates. Never fake precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    /// Actual measurement from timer, counter, etc.
    Measured,
    /// Reasonable guess based on available evidence.
    Estimated,
    /// No basis for estimate.
    Unknown,
}

/// Developer time tracking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DevLtMinutes {
    /// Author time in minutes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<u32>,

    /// Confidence level for author time.
    pub author_confidence: Confidence,

    /// Review time in minutes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review: Option<u32>,

    /// Confidence level for review time.
    pub review_confidence: Confidence,

    /// Number of human interventions required.
    #[serde(default)]
    pub interventions: u32,

    /// Additional notes about time tracking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Compute spend tracking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComputeSpend {
    /// Estimated cost in USD for token usage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_usd: Option<f64>,

    /// Confidence level for cost estimate.
    pub confidence: Confidence,

    /// Number of CI/gate runs.
    #[serde(default)]
    pub runs: u32,

    /// Additional notes about compute spend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Iteration tracking for fix loops.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Iterations {
    /// Number of failed gate runs before success.
    #[serde(default)]
    pub failed_gates: u32,

    /// Number of fix-and-retry loops.
    #[serde(default)]
    pub fix_loops: u32,

    /// Additional notes about iterations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Value delivered by the PR.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValueDelivered {
    /// Description of uncertainty reduced by this PR.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uncertainty_reduced: Option<String>,

    /// Description of rework prevented by this PR.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rework_prevented: Option<String>,
}

impl Default for DevLtMinutes {
    fn default() -> Self {
        Self {
            author: None,
            author_confidence: Confidence::Unknown,
            review: None,
            review_confidence: Confidence::Unknown,
            interventions: 0,
            notes: None,
        }
    }
}

impl Default for ComputeSpend {
    fn default() -> Self {
        Self { tokens_usd: None, confidence: Confidence::Unknown, runs: 0, notes: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_serde() {
        assert_eq!(serde_json::to_string(&Confidence::Measured).unwrap(), r#""measured""#);
        assert_eq!(serde_json::to_string(&Confidence::Estimated).unwrap(), r#""estimated""#);
        assert_eq!(serde_json::to_string(&Confidence::Unknown).unwrap(), r#""unknown""#);

        assert_eq!(
            serde_json::from_str::<Confidence>(r#""measured""#).unwrap(),
            Confidence::Measured
        );
        assert_eq!(
            serde_json::from_str::<Confidence>(r#""estimated""#).unwrap(),
            Confidence::Estimated
        );
        assert_eq!(
            serde_json::from_str::<Confidence>(r#""unknown""#).unwrap(),
            Confidence::Unknown
        );
    }

    #[test]
    fn test_economics_receipt_roundtrip() {
        let receipt = EconomicsReceipt {
            schema_version: "1.0".to_string(),
            pr: 209,
            run_id: "2026-01-07T14:32Z-pr209".to_string(),
            devlt_minutes: DevLtMinutes {
                author: Some(25),
                author_confidence: Confidence::Estimated,
                review: None,
                review_confidence: Confidence::Unknown,
                interventions: 2,
                notes: Some("Two fix-loops after initial clippy failures".to_string()),
            },
            compute: ComputeSpend {
                tokens_usd: Some(4.20),
                confidence: Confidence::Estimated,
                runs: 3,
                notes: Some("Three selftest runs during iteration".to_string()),
            },
            iterations: Iterations {
                failed_gates: 2,
                fix_loops: 2,
                notes: Some("Clippy warnings, then test failure".to_string()),
            },
            value_delivered: ValueDelivered {
                uncertainty_reduced: Some("Confirmed BDD scenarios cover error paths".to_string()),
                rework_prevented: Some("Caught missing 400 handler before merge".to_string()),
            },
        };

        let json = serde_json::to_string_pretty(&receipt).unwrap();
        let parsed: EconomicsReceipt = serde_json::from_str(&json).unwrap();

        assert_eq!(receipt, parsed);
    }

    #[test]
    fn test_optional_fields_skipped() {
        let receipt = EconomicsReceipt {
            schema_version: "1.0".to_string(),
            pr: 123,
            run_id: "test-run".to_string(),
            devlt_minutes: DevLtMinutes::default(),
            compute: ComputeSpend::default(),
            iterations: Iterations::default(),
            value_delivered: ValueDelivered::default(),
        };

        let json = serde_json::to_string(&receipt).unwrap();

        // Optional fields with None should not appear in JSON
        assert!(!json.contains("tokens_usd"));
        assert!(!json.contains("notes"));
        assert!(!json.contains("uncertainty_reduced"));
        assert!(!json.contains("rework_prevented"));
    }

    #[test]
    fn test_default_values() {
        let devlt = DevLtMinutes::default();
        assert_eq!(devlt.author, None);
        assert_eq!(devlt.author_confidence, Confidence::Unknown);
        assert_eq!(devlt.interventions, 0);

        let compute = ComputeSpend::default();
        assert_eq!(compute.tokens_usd, None);
        assert_eq!(compute.confidence, Confidence::Unknown);
        assert_eq!(compute.runs, 0);

        let iterations = Iterations::default();
        assert_eq!(iterations.failed_gates, 0);
        assert_eq!(iterations.fix_loops, 0);

        let value = ValueDelivered::default();
        assert_eq!(value.uncertainty_reduced, None);
        assert_eq!(value.rework_prevented, None);
    }
}
