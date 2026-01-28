//! Receipt operations for generating structured evidence from gate execution.
//!
//! This module provides utilities for:
//! - Generating gate receipts (fmt, clippy, tests)
//! - Generating economics receipts (DevLT, compute spend)
//! - Generating quality receipts (code quality metrics)
//! - Generating telemetry receipts (probe execution results)
//! - Generating timeline receipts (commit history analysis)
//! - Validating receipts against schemas
//!
//! ## Design
//!
//! Receipts provide machine-readable evidence of gate execution for:
//! - CI pipelines
//! - IDP integrations
//! - Audit trails
//! - Agent workflows

pub mod economics;
pub mod forensic;
pub mod friction;
pub mod gate;
pub mod git;
pub mod historian;
pub mod quality;
pub mod run_id;
pub mod telemetry;
pub mod timeline;
pub mod validate;

pub use economics::{ReceiptsEconomicsArgs, run_economics};
pub use forensic::{ReceiptsForensicArgs, run_forensic};
pub use friction::{FrictionCategory, categorize_friction_zones};
pub use gate::{ReceiptsGateArgs, run_gate};
pub use git::{get_current_commit_full, get_current_commit_short, get_ref_sha};
pub use historian::{
    HistorianQualityAppendix, extract_historian_appendix_json as extract_historian_appendix,
    parse_historian_appendix,
};
pub use quality::{ReceiptsQualityArgs, run_quality};
pub use run_id::generate_run_id;
pub use telemetry::{ReceiptsTelemetryArgs, run_telemetry};
pub use timeline::{
    FRICTION_EXCLUDE_PATTERNS, ReceiptsTimelineArgs, normalize_path_separators, run_timeline,
    should_exclude_path,
};
pub use validate::{ReceiptsValidateArgs, run_validate};

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use gov_receipts::{Environment, GateReceipt, GateResult, GateStatus};

    #[test]
    fn gate_receipt_uses_gov_receipts_types() {
        // Verify we're using gov-receipts crate types
        let receipt = GateReceipt::builder()
            .run_id("test-run")
            .commit("abc123")
            .started_at(Utc::now())
            .finished_at(Utc::now())
            .gate(GateResult {
                name: "fmt".to_string(),
                command: "cargo fmt --all --check".to_string(),
                status: GateStatus::Pass,
                duration_ms: 1234,
                details: None,
            })
            .overall_status(GateStatus::Pass)
            .repo_version("3.3.14")
            .environment(Environment {
                os: "linux".to_string(),
                rust_version: "1.83.0".to_string(),
                nix_shell: true,
            })
            .build();

        assert!(receipt.all_passed());
        assert_eq!(receipt.run_id, "test-run");
    }

    #[test]
    fn gate_receipt_optional_pr() {
        let receipt = GateReceipt::builder()
            .run_id("test-run")
            .pr(123)
            .commit("abc123")
            .started_at(Utc::now())
            .finished_at(Utc::now())
            .overall_status(GateStatus::Pass)
            .repo_version("3.3.14")
            .environment(Environment {
                os: "linux".to_string(),
                rust_version: "1.83.0".to_string(),
                nix_shell: false,
            })
            .build();

        assert_eq!(receipt.pr, Some(123));

        // Verify JSON serialization includes pr
        let json = serde_json::to_string(&receipt).unwrap();
        assert!(json.contains(r#"\"pr\":123"#));
    }

    #[test]
    fn gate_status_serialization() {
        assert_eq!(serde_json::to_string(&GateStatus::Pass).unwrap(), r#"\"pass\""#);
        assert_eq!(serde_json::to_string(&GateStatus::Fail).unwrap(), r#"\"fail\""#);
        assert_eq!(serde_json::to_string(&GateStatus::Skipped).unwrap(), r#"\"skipped\""#);
    }
}
