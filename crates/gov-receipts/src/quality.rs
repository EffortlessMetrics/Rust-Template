//! Quality receipt types for tracking code quality metrics.
//!
//! The quality receipt captures contract changes, boundary integrity,
//! verification depth, and risk indicators for a PR or change.

use crate::meta::ReceiptMeta;
use serde::{Deserialize, Serialize};

/// Quality receipt for tracking code quality metrics.
///
/// This receipt captures contract surface changes, boundary integrity,
/// verification depth, and risk indicators.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualityReceipt {
    /// Schema version for forward compatibility.
    pub schema_version: String,

    /// PR number, if this receipt is associated with a pull request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr: Option<u64>,

    /// Run ID for correlation with other receipts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,

    /// Quality metrics.
    pub quality: Quality,

    /// Meta provenance for re-analysis and method versioning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ReceiptMeta>,
}

/// Quality metrics for a PR or change.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Quality {
    /// Contract surface changes.
    pub contract: Contract,

    /// Boundary integrity metrics.
    pub boundaries: Boundaries,

    /// Verification and test coverage metrics.
    pub verification: Verification,

    /// Risk indicators.
    pub risks: Risks,
}

/// Contract surface changes (public API, schema, CLI).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contract {
    /// Public API contract changes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_api: Option<ContractChange>,

    /// Schema contract changes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<ContractChange>,

    /// CLI contract changes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cli: Option<ContractChange>,
}

/// Details of a contract change.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractChange {
    /// Whether this contract surface changed.
    #[serde(default)]
    pub changed: bool,

    /// Whether the change is breaking.
    #[serde(default)]
    pub breaking: bool,

    /// Evidence pointers supporting the assessment.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<String>,
}

/// Boundary integrity and change surface metrics.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Boundaries {
    /// Number of distinct modules touched.
    #[serde(default)]
    pub modules_touched: u32,

    /// Files or modules with high churn.
    #[serde(default)]
    pub hotspots: Vec<String>,

    /// Optional LLM assessment of boundary integrity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_assessment: Option<LlmBoundaryAssessment>,
}

/// LLM-generated assessment of boundary integrity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmBoundaryAssessment {
    /// Overall boundary integrity rating.
    pub rating: BoundaryRating,

    /// Observations about design alignment, coupling, and cohesion.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,

    /// Confidence level of the assessment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<LlmConfidence>,

    /// Evidence pointers supporting the assessment.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<String>,
}

/// Rating of how a change affects boundary integrity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BoundaryRating {
    /// Boundary integrity improved.
    Improved,
    /// Boundary integrity unchanged.
    Neutral,
    /// Boundary integrity degraded.
    Degraded,
}

/// Verification and test coverage metrics.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Verification {
    /// Lines of test code added.
    #[serde(default)]
    pub tests_added_loc: u32,

    /// Lines of implementation code added.
    #[serde(default)]
    pub impl_added_loc: u32,

    /// Change in test density ratio.
    #[serde(default)]
    pub test_density_delta: f64,

    /// Optional LLM assessment of test depth.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_test_depth: Option<LlmTestDepthAssessment>,
}

/// LLM-generated assessment of test depth and quality.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmTestDepthAssessment {
    /// Overall test depth rating.
    pub rating: TestDepthRating,

    /// Observations about test quality.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,

    /// Confidence level of the assessment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<LlmConfidence>,

    /// Evidence pointers supporting the assessment.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<String>,
}

/// Rating of test depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TestDepthRating {
    /// Robust, behavior-asserting tests.
    Hardened,
    /// Some gaps in coverage.
    Mixed,
    /// Presence-only or minimal tests.
    Shallow,
}

/// Risk indicators and safety markers.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Risks {
    /// Changes to unsafe code usage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unsafe_delta: Option<UnsafeDelta>,

    /// New dependencies added.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub deps_added: Vec<String>,

    /// New concurrency primitives introduced.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub concurrency_primitives_added: Vec<String>,

    /// LLM-generated risk notes.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub llm_risk_notes: Vec<String>,
}

/// Change in unsafe code usage.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnsafeDelta {
    /// Number of unsafe blocks/functions added.
    #[serde(default)]
    pub added: u32,

    /// Number of unsafe blocks/functions removed.
    #[serde(default)]
    pub removed: u32,
}

/// Confidence level for LLM assessments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LlmConfidence {
    /// Strong evidence supports the assessment.
    High,
    /// Reasonable inference.
    Medium,
    /// Limited evidence.
    Low,
}

impl QualityReceipt {
    /// Create a new quality receipt builder.
    pub fn builder() -> QualityReceiptBuilder {
        QualityReceiptBuilder::default()
    }
}

/// Builder for constructing `QualityReceipt` instances.
#[derive(Debug, Default)]
pub struct QualityReceiptBuilder {
    schema_version: Option<String>,
    pr: Option<u64>,
    run_id: Option<String>,
    quality: Quality,
    meta: Option<ReceiptMeta>,
}

impl QualityReceiptBuilder {
    /// Set the schema version.
    pub fn schema_version(mut self, version: impl Into<String>) -> Self {
        self.schema_version = Some(version.into());
        self
    }

    /// Set the PR number.
    pub fn pr(mut self, pr: u64) -> Self {
        self.pr = Some(pr);
        self
    }

    /// Set the run ID.
    pub fn run_id(mut self, id: impl Into<String>) -> Self {
        self.run_id = Some(id.into());
        self
    }

    /// Set the quality metrics.
    pub fn quality(mut self, quality: Quality) -> Self {
        self.quality = quality;
        self
    }

    /// Set the meta provenance.
    pub fn meta(mut self, meta: ReceiptMeta) -> Self {
        self.meta = Some(meta);
        self
    }

    /// Build the quality receipt.
    pub fn build(self) -> QualityReceipt {
        QualityReceipt {
            schema_version: self.schema_version.unwrap_or_else(|| "1.0".to_string()),
            pr: self.pr,
            run_id: self.run_id,
            quality: self.quality,
            meta: self.meta,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_receipt_roundtrip() {
        let receipt = QualityReceipt {
            schema_version: "1.0".to_string(),
            pr: Some(123),
            run_id: Some("test-run".to_string()),
            quality: Quality {
                contract: Contract {
                    public_api: Some(ContractChange {
                        changed: true,
                        breaking: false,
                        evidence: vec![],
                    }),
                    ..Default::default()
                },
                boundaries: Boundaries {
                    modules_touched: 3,
                    hotspots: vec!["lib.rs".to_string()],
                    ..Default::default()
                },
                verification: Verification {
                    tests_added_loc: 50,
                    impl_added_loc: 100,
                    test_density_delta: 0.5,
                    ..Default::default()
                },
                risks: Risks {
                    unsafe_delta: Some(UnsafeDelta { added: 1, removed: 0 }),
                    ..Default::default()
                },
            },
            meta: None,
        };

        let json = serde_json::to_string_pretty(&receipt).unwrap();
        let parsed: QualityReceipt = serde_json::from_str(&json).unwrap();

        assert_eq!(receipt, parsed);
    }

    #[test]
    fn test_quality_receipt_builder() {
        let receipt = QualityReceipt::builder()
            .pr(123)
            .run_id("test-run")
            .quality(Quality {
                boundaries: Boundaries { modules_touched: 5, ..Default::default() },
                ..Default::default()
            })
            .build();

        assert_eq!(receipt.pr, Some(123));
        assert_eq!(receipt.quality.boundaries.modules_touched, 5);
    }

    #[test]
    fn test_boundary_rating_serde() {
        assert_eq!(serde_json::to_string(&BoundaryRating::Improved).unwrap(), r#""improved""#);
        assert_eq!(serde_json::to_string(&BoundaryRating::Neutral).unwrap(), r#""neutral""#);
        assert_eq!(serde_json::to_string(&BoundaryRating::Degraded).unwrap(), r#""degraded""#);
    }

    #[test]
    fn test_test_depth_rating_serde() {
        assert_eq!(serde_json::to_string(&TestDepthRating::Hardened).unwrap(), r#""hardened""#);
        assert_eq!(serde_json::to_string(&TestDepthRating::Mixed).unwrap(), r#""mixed""#);
        assert_eq!(serde_json::to_string(&TestDepthRating::Shallow).unwrap(), r#""shallow""#);
    }

    #[test]
    fn test_empty_hotspots_serialize() {
        // Regression test: empty hotspots must serialize (schema requires the field)
        let boundaries = Boundaries { modules_touched: 1, hotspots: vec![], ..Default::default() };
        let json = serde_json::to_string(&boundaries).unwrap();
        assert!(json.contains(r#""hotspots":[]"#), "empty hotspots must be serialized");
    }
}
