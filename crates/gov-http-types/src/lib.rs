//! Shared HTTP API types for governance endpoints.
//!
//! This crate provides shared data types used across multiple gov-http-* crates,
//! enabling decoupling between sibling modules. Types here are pure data structures
//! with serialization support, no HTTP framework dependencies.
//!
//! # Types
//!
//! - [`FrictionEntry`] - Friction log entry for process/tooling issues
//! - [`Question`] - Question artifact for flow decision points

mod friction;
mod question;

pub use friction::*;
pub use question::*;
