//! Shared HTTP API types for governance endpoints.
//!
//! This crate provides shared data types used across multiple gov-http-* crates,
//! enabling decoupling between sibling modules. Types here are pure data structures
//! with serialization support, no HTTP framework dependencies.
//!
//! # Types
//!
//! - [`Question`] - Question artifact for flow decision points

mod question;

use gov_model::YamlResource;
pub use question::*;

impl YamlResource for Question {
    fn id(&self) -> &str {
        &self.id
    }
}
