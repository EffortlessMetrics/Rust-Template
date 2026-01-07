//! Receipt types for audit evidence in Rust-as-Spec platform.
//!
//! This crate provides structured types for governance receipts, which are
//! machine-generated proof that gates ran and what they found. Receipts are
//! the source of truth for claims in PR cover sheets.
//!
//! # Receipt Types
//!
//! - [`GateReceipt`] - Core receipt for gate execution results (fmt, clippy, tests, selftest)
//! - [`EconomicsReceipt`] - DevLT and compute spend tracking with confidence levels
//! - [`QualityReceipt`] - Code quality metrics, contract changes, boundary integrity
//! - [`TelemetryReceipt`] - Probe execution results and change surface analysis
//! - [`TimelineReceipt`] - PR evolution, friction zones, and convergence patterns
//! - [`Dossier`] - Structured PR analysis for casebook generation
//!
//! # Usage
//!
//! Receipts live in `.runs/` (ephemeral, gitignored) during development.
//! Version-controlled exhibits in `docs/audit/EXHIBITS/` preserve claims for posterity.
//!
//! # Example
//!
//! ```
//! use gov_receipts::{GateReceipt, GateStatus, GateResult, Environment};
//!
//! let receipt = GateReceipt::builder()
//!     .run_id("2026-01-07T14:32Z-pr209")
//!     .pr(209)
//!     .commit("abc123def")
//!     .started_at("2026-01-07T14:32:00Z".parse().unwrap())
//!     .finished_at("2026-01-07T14:35:42Z".parse().unwrap())
//!     .gate(GateResult {
//!         name: "tests".to_string(),
//!         command: "cargo test --all".to_string(),
//!         status: GateStatus::Pass,
//!         duration_ms: 23456,
//!         details: None,
//!     })
//!     .overall_status(GateStatus::Pass)
//!     .repo_version("v3.3.14")
//!     .environment(Environment {
//!         os: "linux".to_string(),
//!         rust_version: "1.83.0".to_string(),
//!         nix_shell: true,
//!     })
//!     .build();
//!
//! assert!(receipt.all_passed());
//! ```
//!
//! # Schema Versioning
//!
//! All receipts include a `schema_version` field for forward compatibility:
//! - Additive changes (new optional fields) keep the same version
//! - Breaking changes (required field changes) bump the version
//! - Tooling must handle missing optional fields gracefully

pub mod dossier;
pub mod economics;
pub mod gate;
pub mod quality;
pub mod telemetry;
pub mod timeline;

// Re-export main types for convenience
pub use dossier::{
    Dossier, DossierBuilder, Erratum, ExhibitScore, FactoryDelta, Finding, Intent, Scope,
};
pub use economics::{
    ComputeSpend, Confidence, DevLtMinutes, EconomicsReceipt, EconomicsReceiptBuilder, Iterations,
    ValueDelivered,
};
pub use gate::{
    Environment, GateDetails, GateReceipt, GateReceiptBuilder, GateResult, GateStatus,
    SelftestDetails, TestDetails,
};
pub use quality::{
    Boundaries, BoundaryRating, Contract, ContractChange, LlmBoundaryAssessment, LlmConfidence,
    LlmTestDepthAssessment, Quality, QualityReceipt, QualityReceiptBuilder, Risks, TestDepthRating,
    UnsafeDelta, Verification,
};
pub use telemetry::{
    ChangeSurface, Contracts, GeigerSummary, ProbeProfile, ProbeResult, ProbeStatus, Safety,
    SkippedProbe, Structure, TelemetryReceipt, TelemetryReceiptBuilder, TelemetryVerification,
};
pub use timeline::{
    Convergence, Event, EventType, FrictionZone, Oscillation, OscillationAction, OscillationType,
    Session, SessionClassification, TimelineConfidence, TimelineReceipt, TimelineReceiptBuilder,
    Topology, WallClock,
};
