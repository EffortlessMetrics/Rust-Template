//! Shared testing utilities for safe process-global state manipulation.
//!
//! This crate provides RAII guards for environment variable and working directory
//! manipulation in tests. These guards:
//!
//! - Serialize access via a global lock (preventing concurrent mutation)
//! - Snapshot state before modification
//! - Restore state on Drop (panic-safe)
//!
//! # Why this exists
//!
//! In Rust 2024 edition, `std::env::set_var` and `std::env::remove_var` are `unsafe`
//! because environment variables are process-global state. Concurrent reads during
//! mutation cause undefined behavior. This crate centralizes the unsafe operations
//! behind safe interfaces with proper synchronization.
//!
//! # Usage
//!
//! ```no_run
//! use testing::process::{EnvVarGuard, CwdGuard};
//!
//! #[test]
//! fn test_with_custom_env() {
//!     let guard = EnvVarGuard::new(&["MY_VAR"]);
//!     guard.set("MY_VAR", "test-value");
//!
//!     // Test code here...
//!
//! } // MY_VAR automatically restored to original value
//! ```

pub mod fakes;
pub mod process;
