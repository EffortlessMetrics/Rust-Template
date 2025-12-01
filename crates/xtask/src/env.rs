//! Environment detection helpers for CI and automation contexts.
//!
//! This module provides centralized functions for detecting CI environments,
//! non-interactive mode, and resource constraints. Use these instead of
//! duplicating env var checks across commands.
//!
//! # Environment Variables
//!
//! | Variable | Effect |
//! |----------|--------|
//! | `CI=1` | Standard CI detection (GitHub Actions, GitLab CI, etc.) |
//! | `XTASK_NONINTERACTIVE=1` | Force non-interactive mode |
//! | `XTASK_LOW_RESOURCES=1` | Enable resource-constrained mode |
//! | `XTASK_SKIP_BDD=1` | Skip BDD test suite |

use std::env;

/// Returns true if running in a CI environment.
///
/// Detects common CI providers: GitHub Actions, GitLab CI, CircleCI, Jenkins, etc.
///
/// # Example
/// ```
/// use xtask::env::is_ci;
/// if is_ci() {
///     println!("Running in CI mode");
/// }
/// ```
pub fn is_ci() -> bool {
    env::var("CI").is_ok()
        || env::var("GITHUB_ACTIONS").is_ok()
        || env::var("GITLAB_CI").is_ok()
        || env::var("CIRCLECI").is_ok()
        || env::var("JENKINS_URL").is_ok()
        || env::var("BUILDKITE").is_ok()
}

/// Returns true if running in non-interactive mode.
///
/// This is true when either:
/// - `XTASK_NONINTERACTIVE=1` is explicitly set, OR
/// - Running in a CI environment (see [`is_ci`])
///
/// Commands should use this to suppress prompts and ensure proper exit codes.
///
/// # Contract (AC-TPL-XTASK-NONINTERACTIVE)
/// When this returns true, commands MUST:
/// - Not prompt for user input
/// - Return exit code 0 on success, non-zero on failure
pub fn is_noninteractive() -> bool {
    env::var("XTASK_NONINTERACTIVE").ok().as_deref() == Some("1") || is_ci()
}

/// Returns true if low-resource mode is enabled.
///
/// When enabled (`XTASK_LOW_RESOURCES=1`):
/// - `CARGO_BUILD_JOBS` is set to 1 (sequential builds)
/// - sccache is disabled
/// - Format checks may be skipped
/// - BDD tests may be skipped
pub fn is_low_resources() -> bool {
    env::var("XTASK_LOW_RESOURCES").ok().as_deref() == Some("1")
}

/// Returns true if BDD tests should be skipped.
///
/// This is true when either:
/// - `XTASK_SKIP_BDD=1` is explicitly set, OR
/// - Low-resource mode is enabled
pub fn should_skip_bdd() -> bool {
    env::var("XTASK_SKIP_BDD").ok().as_deref() == Some("1") || is_low_resources()
}

/// Returns a description of the current environment mode.
///
/// Useful for logging which mode is active.
pub fn describe_mode() -> &'static str {
    if is_ci() && is_low_resources() {
        "CI (low-resources)"
    } else if is_ci() {
        "CI"
    } else if is_noninteractive() {
        "non-interactive"
    } else if is_low_resources() {
        "low-resources"
    } else {
        "interactive"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_describe_mode_default() {
        // In test environment, we can't easily manipulate env vars,
        // but we can verify the function returns a valid string
        let mode = describe_mode();
        assert!(!mode.is_empty());
    }
}
