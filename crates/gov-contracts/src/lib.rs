//! Governance contracts and schema definitions.
//!
//! This crate provides:
//! - Embedded schema assets (platform, config, UI contract)
//! - Schema loading and validation utilities
//! - Contract types for IDP integrations

pub mod platform;
pub mod schemas;
pub mod ui_contract;

pub use platform::{CoverageSummary, GovernanceStatus, PlatformStatus};
pub use ui_contract::{Region, Screen, UiContract};
