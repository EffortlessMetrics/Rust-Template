//! Core governance task library for xtask binaries.
//!
//! This crate provides reusable components for governance validation:
//! - Environment detection (CI, low resources, etc.)
//! - Selftest step definitions and pipeline
//! - Common validation utilities
//!
//! Service repos create thin `xtask` binaries that call into this library.

pub mod env;
pub mod selftest;
pub mod validation;

pub use env::{describe_mode, is_ci, is_low_resources, is_noninteractive, should_skip_bdd};
pub use selftest::{
    SELFTEST_STEP_COUNT, SELFTEST_STEPS, SelftestResult, SelftestStep, StepResult, make_step,
    run_selftest_pipeline, run_step,
};
