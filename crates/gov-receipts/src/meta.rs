//! Meta provenance types for receipt re-analysis and method versioning.
//!
//! All receipts can optionally include a `ReceiptMeta` header that provides
//! provenance information for auditing, reproducibility, and method versioning.

use serde::{Deserialize, Serialize};

/// Meta provenance header for receipts.
///
/// This structure captures essential provenance information that enables:
/// - Re-analysis of the same inputs with updated methods
/// - Version tracking for analysis method evolution
/// - Audit trails with evidence pointers
/// - Confidence calibration based on inputs and assumptions
///
/// # Example
///
/// ```
/// use gov_receipts::{ReceiptMeta, MetaConfidence};
///
/// let meta = ReceiptMeta::builder()
///     .method_id("telemetry-v1")
///     .method_version(1)
///     .analysis_run_id("2026-01-07T14-32-00Z-pr209")
///     .input("git_diff")
///     .input("git_log")
///     .assumption("base branch is origin/main")
///     .confidence(MetaConfidence::Medium)
///     .evidence("crates/gov-receipts/src/lib.rs")
///     .evidence("63da971")
///     .build();
///
/// assert_eq!(meta.method_id, "telemetry-v1");
/// assert_eq!(meta.inputs_used.len(), 2);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiptMeta {
    /// Identifier for the analysis method used (e.g., "telemetry-v1", "timeline-v1").
    pub method_id: String,

    /// Version of the analysis method. Bump when logic changes materially.
    pub method_version: u32,

    /// Unique identifier for this analysis run (typically matches run_id).
    pub analysis_run_id: String,

    /// List of inputs relied on for this analysis (e.g., "git_diff", "gate.json", "pr_comments").
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub inputs_used: Vec<String>,

    /// Assumptions that materially affect conclusions.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub assumptions: Vec<String>,

    /// Confidence level of the analysis results.
    pub confidence: MetaConfidence,

    /// Evidence pointers (5-15 anchors): file paths, function names, receipt refs, commit SHAs.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub evidence_pointers: Vec<String>,
}

/// Confidence level for meta provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetaConfidence {
    /// Strong evidence supports the analysis (e.g., exhibit profile, full tooling).
    High,
    /// Reasonable inference from available data (e.g., full profile).
    Medium,
    /// Limited evidence or fast profile (e.g., fast profile, incomplete data).
    Low,
}

impl ReceiptMeta {
    /// Create a new meta provenance builder.
    pub fn builder() -> ReceiptMetaBuilder {
        ReceiptMetaBuilder::default()
    }
}

/// Builder for constructing `ReceiptMeta` instances.
#[derive(Debug, Default)]
pub struct ReceiptMetaBuilder {
    method_id: Option<String>,
    method_version: Option<u32>,
    analysis_run_id: Option<String>,
    inputs_used: Vec<String>,
    assumptions: Vec<String>,
    confidence: Option<MetaConfidence>,
    evidence_pointers: Vec<String>,
}

impl ReceiptMetaBuilder {
    /// Set the method ID.
    pub fn method_id(mut self, id: impl Into<String>) -> Self {
        self.method_id = Some(id.into());
        self
    }

    /// Set the method version.
    pub fn method_version(mut self, version: u32) -> Self {
        self.method_version = Some(version);
        self
    }

    /// Set the analysis run ID.
    pub fn analysis_run_id(mut self, id: impl Into<String>) -> Self {
        self.analysis_run_id = Some(id.into());
        self
    }

    /// Add an input that was used.
    pub fn input(mut self, input: impl Into<String>) -> Self {
        self.inputs_used.push(input.into());
        self
    }

    /// Set all inputs used.
    pub fn inputs_used(mut self, inputs: Vec<String>) -> Self {
        self.inputs_used = inputs;
        self
    }

    /// Add an assumption.
    pub fn assumption(mut self, assumption: impl Into<String>) -> Self {
        self.assumptions.push(assumption.into());
        self
    }

    /// Set all assumptions.
    pub fn assumptions(mut self, assumptions: Vec<String>) -> Self {
        self.assumptions = assumptions;
        self
    }

    /// Set the confidence level.
    pub fn confidence(mut self, confidence: MetaConfidence) -> Self {
        self.confidence = Some(confidence);
        self
    }

    /// Add an evidence pointer.
    pub fn evidence(mut self, pointer: impl Into<String>) -> Self {
        self.evidence_pointers.push(pointer.into());
        self
    }

    /// Set all evidence pointers.
    pub fn evidence_pointers(mut self, pointers: Vec<String>) -> Self {
        self.evidence_pointers = pointers;
        self
    }

    /// Build the receipt meta.
    ///
    /// # Panics
    ///
    /// Panics if method_id, method_version, analysis_run_id, or confidence is not set.
    pub fn build(self) -> ReceiptMeta {
        ReceiptMeta {
            method_id: self.method_id.expect("method_id is required"),
            method_version: self.method_version.expect("method_version is required"),
            analysis_run_id: self.analysis_run_id.expect("analysis_run_id is required"),
            inputs_used: self.inputs_used,
            assumptions: self.assumptions,
            confidence: self.confidence.expect("confidence is required"),
            evidence_pointers: self.evidence_pointers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receipt_meta_roundtrip() {
        let meta = ReceiptMeta {
            method_id: "telemetry-v1".to_string(),
            method_version: 1,
            analysis_run_id: "2026-01-07T14-32-00Z-pr209".to_string(),
            inputs_used: vec!["git_diff".to_string(), "git_log".to_string()],
            assumptions: vec!["base branch is origin/main".to_string()],
            confidence: MetaConfidence::Medium,
            evidence_pointers: vec!["crates/gov-receipts/src/lib.rs".to_string()],
        };

        let json = serde_json::to_string_pretty(&meta).unwrap();
        let parsed: ReceiptMeta = serde_json::from_str(&json).unwrap();

        assert_eq!(meta, parsed);
    }

    #[test]
    fn test_receipt_meta_builder() {
        let meta = ReceiptMeta::builder()
            .method_id("timeline-v1")
            .method_version(1)
            .analysis_run_id("test-run")
            .input("git_log")
            .input("commit_history")
            .assumption("30 minute session gap")
            .confidence(MetaConfidence::High)
            .evidence("abc123")
            .evidence("def456")
            .build();

        assert_eq!(meta.method_id, "timeline-v1");
        assert_eq!(meta.method_version, 1);
        assert_eq!(meta.inputs_used.len(), 2);
        assert_eq!(meta.assumptions.len(), 1);
        assert_eq!(meta.confidence, MetaConfidence::High);
        assert_eq!(meta.evidence_pointers.len(), 2);
    }

    #[test]
    fn test_meta_confidence_serde() {
        assert_eq!(serde_json::to_string(&MetaConfidence::High).unwrap(), r#""high""#);
        assert_eq!(serde_json::to_string(&MetaConfidence::Medium).unwrap(), r#""medium""#);
        assert_eq!(serde_json::to_string(&MetaConfidence::Low).unwrap(), r#""low""#);
    }

    #[test]
    fn test_empty_optional_fields_not_serialized() {
        let meta = ReceiptMeta::builder()
            .method_id("quality-v1")
            .method_version(1)
            .analysis_run_id("test")
            .confidence(MetaConfidence::Low)
            .build();

        let json = serde_json::to_string(&meta).unwrap();

        // Empty vectors should not appear in JSON
        assert!(!json.contains("inputs_used"));
        assert!(!json.contains("assumptions"));
        assert!(!json.contains("evidence_pointers"));
    }
}
