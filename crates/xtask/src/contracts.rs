//! Contracts module re-exports from xtask-contracts library.
//!
//! This module provides utilities for:
//! - Computing governed facts from specs/code (selftest steps, kernel AC count, etc.)
//! - These facts are the "single source of truth" that documentation must reflect
//!
//! ## Design
//!
//! In a Rust-as-Spec repo, certain numbers like "12-step selftest gate" and "61 kernel ACs"
//! are **governed facts** that appear in documentation. When the source changes (e.g., adding
//! a new selftest step), all documentation references must be updated.
//!
//! This module re-exports the contracts library, enabling automated validation
//! and synchronization via `cargo xtask contracts-check` and `cargo xtask contracts-fmt`.

// Re-export all public items from xtask-contracts for cross-module use
#[expect(unused_imports, reason = "existing reviewed debt; tracked by lint policy ratchet")]
pub use xtask_contracts::{AcCounts, ContractsSnapshot};
