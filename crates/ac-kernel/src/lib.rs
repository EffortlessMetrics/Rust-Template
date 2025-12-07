//! AC Kernel: Core governance logic for the Rust-as-Spec platform.
//!
//! This crate contains the shared data model and logic for:
//! - AC (Acceptance Criteria) status tracking
//! - Coverage parsing and aggregation
//! - Ledger reading and AC metadata extraction
//! - History/trend analysis
//!
//! # Architecture
//!
//! The `ac-kernel` crate is designed to be the single source of truth for AC-related
//! types and logic. It is used by:
//!
//! - `xtask`: For `ac-status`, `ac-coverage`, `ac-history` commands
//! - `acceptance`: For writing coverage records during BDD test execution
//!
//! # Modules
//!
//! - [`model`]: Core data types (AcStatus, AcSource, Scenario, etc.)
//! - [`coverage`]: AC coverage JSONL parsing
//! - [`ledger`]: spec_ledger.yaml parsing
//! - [`history`]: Time-series snapshot analysis
//! - [`json`]: JSON output schemas for ac-status

pub mod coverage;
pub mod history;
pub mod json;
pub mod ledger;
pub mod model;

// Re-export commonly used types at the crate root
pub use coverage::{AcCoverageRecord, parse_ac_coverage};
pub use history::{
    AC_HISTORY_SCHEMA_VERSION, AcHistoryReport, LoadResult, SkippedFile, SnapshotDelta,
    SnapshotMetric, build_report, load_snapshots,
};
pub use json::{
    AC_STATUS_SCHEMA_VERSION, AcCategoryStats, AcJson, AcStatusJson, build_status_json,
};
pub use ledger::{AcDetails, AcMetadata, get_ac_details, parse_ledger_with_metadata};
pub use model::{AcSource, AcStatus, Scenario, TestMapping};
