//! BDD acceptance test framework for governance validation.
//!
//! This crate provides Cucumber-based acceptance testing infrastructure for the
//! Rust-as-Spec platform, including step definitions, test world setup, and
//! AC coverage tracking that validates Acceptance Criteria against executable tests.

pub mod coverage_writer;
pub mod steps;
pub mod world;

pub use coverage_writer::AcCoverageWriter;
pub use world::World;
