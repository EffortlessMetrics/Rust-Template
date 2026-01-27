//! Receipt schema types and serialization rules
//!
//! This crate provides core receipt types for governance and CI/CD evidence.
//! It focuses on types and serialization - IO should be handled elsewhere.
//!
//! # Design Philosophy
//!
//! Receipts should be:
//! - **Structured**: Use typed fields for all data
//! - **Serializable**: Support JSON/YAML for evidence storage
//! - **Validatable**: Include validation rules where applicable
//! - **Minimal**: No IO, no HTTP, just types and validation

pub mod dossier;
pub mod economics;
pub mod gate;
pub mod meta;
pub mod quality;
pub mod telemetry;
pub mod timeline;

pub use dossier::{Dossier, Erratum, ExhibitScore, FactoryDelta, Finding, Intent, Scope};
pub use economics::{
    ComputeSpend, Confidence, DevLtMinutes, EconomicsReceipt, Iterations, ValueDelivered,
};
pub use gate::{
    Environment, GateDetails, GateReceipt, GateResult, GateStatus, SelftestDetails, TestDetails,
};
pub use meta::{MetaConfidence, ReceiptMeta};
pub use quality::{
    Boundaries, BoundaryRating, Contract, ContractChange, LlmBoundaryAssessment, LlmConfidence,
    LlmTestDepthAssessment, Quality, QualityReceipt, Risks, TestDepthRating, UnsafeDelta,
    Verification,
};
pub use telemetry::{
    ChangeSurface, Contracts, GeigerSummary, ProbeProfile, ProbeResult, ProbeStatus, Safety,
    SkippedProbe, Structure, TelemetryReceipt, TelemetryVerification,
};
pub use timeline::{
    Convergence, Event, EventType, FrictionZone, Oscillation, OscillationAction, OscillationType,
    Session, SessionClassification, TimelineConfidence, TimelineReceipt, Topology, WallClock,
};

/// Error type for receipt validation
#[derive(Debug, thiserror::Error)]
pub enum ReceiptError {
    #[error("Invalid schema version: {0}")]
    InvalidSchemaVersion(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid value for field '{field}': {reason}")]
    InvalidValue { field: String, reason: String },

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Result type for receipt operations
pub type ReceiptResult<T> = Result<T, ReceiptError>;

/// Schema validation helper
///
/// Validates that a receipt has the correct schema version.
pub fn validate_schema_version(expected: &str, actual: &str) -> ReceiptResult<()> {
    if expected == actual {
        Ok(())
    } else {
        Err(ReceiptError::InvalidSchemaVersion(format!("expected {}, got {}", expected, actual)))
    }
}

/// Common receipt trait for shared functionality
///
/// All receipt types should implement this trait for common operations.
pub trait Receipt: serde::Serialize + serde::de::DeserializeOwned {
    /// Get the schema version
    fn schema_version(&self) -> &str;

    /// Validate the receipt
    fn validate(&self) -> ReceiptResult<()>;

    /// Convert to JSON
    fn to_json(&self) -> ReceiptResult<String> {
        serde_json::to_string(self).map_err(ReceiptError::Json)
    }

    /// Convert to pretty JSON
    fn to_json_pretty(&self) -> ReceiptResult<String> {
        serde_json::to_string_pretty(self).map_err(ReceiptError::Json)
    }

    /// Parse from JSON
    fn from_json(json: &str) -> ReceiptResult<Self> {
        serde_json::from_str(json).map_err(ReceiptError::Json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_schema_version_success() {
        assert!(validate_schema_version("1.0", "1.0").is_ok());
    }

    #[test]
    fn test_validate_schema_version_failure() {
        assert!(validate_schema_version("1.0", "2.0").is_err());
    }

    #[test]
    fn test_receipt_error_display() {
        let err = ReceiptError::InvalidSchemaVersion("1.0".to_string());
        assert!(err.to_string().contains("Invalid schema version"));
    }

    #[test]
    fn test_receipt_error_invalid_value() {
        let err = ReceiptError::InvalidValue {
            field: "amount".to_string(),
            reason: "must be positive".to_string(),
        };
        assert!(err.to_string().contains("amount"));
        assert!(err.to_string().contains("must be positive"));
    }
}
