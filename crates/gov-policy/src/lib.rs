//! Rego policy bundle and Conftest runner.
//!
//! This crate provides:
//! - Embedded Rego policies for governance validation
//! - A Conftest runner interface
//! - Policy test result types

pub mod policies;
pub mod runner;

pub use runner::{POLICY_AREAS, PolicyTestError, PolicyTestResult, run_policy_tests};
