//! Environment detection helpers for CI and automation contexts.
//!
//! This module provides utilities for detecting the current execution environment:
//! - CI environments (GitHub Actions, GitLab CI, etc.)
//! - Non-interactive mode
//! - Low-resource mode
//! - BDD test skipping
//!
//! ## Environment Variables
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
/// This checks for standard CI environment variables:
/// - `CI` (generic)
/// - `GITHUB_ACTIONS`
/// - `GITLAB_CI`
/// - `CIRCLECI`
/// - `JENKINS_URL`
/// - `BUILDKITE`
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
/// - `XTASK_NONINTERACTIVE` is set (any value)
/// - Running in CI environment
pub fn is_noninteractive() -> bool {
    env::var("XTASK_NONINTERACTIVE").is_ok() || is_ci()
}

/// Returns true if running in low-resource mode.
///
/// This is true when `XTASK_LOW_RESOURCES` is set (any value).
pub fn is_low_resources() -> bool {
    env::var("XTASK_LOW_RESOURCES").is_ok()
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
    use testing::process::EnvVarGuard;

    /// Environment variable names recognized by this module
    const CI_ENV_VARS: &[&str] =
        &["CI", "GITHUB_ACTIONS", "GITLAB_CI", "CIRCLECI", "JENKINS_URL", "BUILDKITE"];

    const XTASK_ENV_VARS: &[&str] =
        &["XTASK_NONINTERACTIVE", "XTASK_LOW_RESOURCES", "XTASK_SKIP_BDD"];

    /// All env vars that these tests care about
    fn all_env_vars() -> Vec<&'static str> {
        CI_ENV_VARS.iter().chain(XTASK_ENV_VARS.iter()).copied().collect()
    }

    /// Helper to run a test with specific env vars set
    fn with_env<F, R>(vars: &[(&'static str, &str)], f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let guard = EnvVarGuard::new(&all_env_vars());

        // Clear all to get a clean state
        for &var in CI_ENV_VARS.iter().chain(XTASK_ENV_VARS.iter()) {
            guard.remove(var);
        }

        // Set the requested vars
        for (key, value) in vars {
            guard.set(key, value);
        }

        f()
    }

    // =========================================================================
    // is_ci() tests
    // =========================================================================

    #[test]
    fn test_is_ci_false_when_no_ci_vars() {
        with_env(&[], || {
            assert!(!is_ci(), "is_ci() should return false when no CI vars are set");
        });
    }

    #[test]
    fn test_is_ci_true_when_ci_var_set() {
        with_env(&[("CI", "1")], || {
            assert!(is_ci(), "is_ci() should return true when CI=1");
        });
    }

    #[test]
    fn test_is_ci_true_for_any_ci_value() {
        // CI var just needs to be present, value doesn't matter
        with_env(&[("CI", "true")], || {
            assert!(is_ci(), "is_ci() should return true when CI=true");
        });
        with_env(&[("CI", "")], || {
            assert!(is_ci(), "is_ci() should return true even when CI is empty");
        });
    }

    #[test]
    fn test_is_ci_detects_github_actions() {
        with_env(&[("GITHUB_ACTIONS", "true")], || {
            assert!(is_ci(), "is_ci() should detect GITHUB_ACTIONS");
        });
    }

    #[test]
    fn test_is_ci_detects_gitlab_ci() {
        with_env(&[("GITLAB_CI", "true")], || {
            assert!(is_ci(), "is_ci() should detect GITLAB_CI");
        });
    }

    #[test]
    fn test_is_ci_detects_circleci() {
        with_env(&[("CIRCLECI", "true")], || {
            assert!(is_ci(), "is_ci() should detect CIRCLECI");
        });
    }

    #[test]
    fn test_is_ci_detects_jenkins() {
        with_env(&[("JENKINS_URL", "http://jenkins.example.com")], || {
            assert!(is_ci(), "is_ci() should detect JENKINS_URL");
        });
    }

    #[test]
    fn test_is_ci_detects_buildkite() {
        with_env(&[("BUILDKITE", "true")], || {
            assert!(is_ci(), "is_ci() should detect BUILDKITE");
        });
    }

    // =========================================================================
    // is_noninteractive() tests
    // =========================================================================

    #[test]
    fn test_is_noninteractive_false_when_nothing_set() {
        with_env(&[], || {
            assert!(
                !is_noninteractive(),
                "is_noninteractive() should return false when no env vars set"
            );
        });
    }

    #[test]
    fn test_is_noninteractive_true_when_explicitly_set() {
        with_env(&[("XTASK_NONINTERACTIVE", "1")], || {
            assert!(
                is_noninteractive(),
                "is_noninteractive() should return true when XTASK_NONINTERACTIVE=1"
            );
        });
    }

    #[test]
    fn test_is_noninteractive_true_for_any_value() {
        // Note: uses .is_ok() which means any value triggers the mode
        with_env(&[("XTASK_NONINTERACTIVE", "0")], || {
            assert!(
                is_noninteractive(),
                "is_noninteractive() triggers on any value (using .is_ok())"
            );
        });
        with_env(&[("XTASK_NONINTERACTIVE", "true")], || {
            assert!(
                is_noninteractive(),
                "is_noninteractive() triggers on any value (using .is_ok())"
            );
        });
    }

    #[test]
    fn test_is_noninteractive_true_when_ci() {
        with_env(&[("CI", "1")], || {
            assert!(
                is_noninteractive(),
                "is_noninteractive() should return true in CI environment"
            );
        });
    }

    // =========================================================================
    // is_low_resources() tests
    // =========================================================================

    #[test]
    fn test_is_low_resources_false_when_not_set() {
        with_env(&[], || {
            assert!(!is_low_resources(), "is_low_resources() should return false by default");
        });
    }

    #[test]
    fn test_is_low_resources_true_when_set() {
        with_env(&[("XTASK_LOW_RESOURCES", "1")], || {
            assert!(
                is_low_resources(),
                "is_low_resources() should return true when XTASK_LOW_RESOURCES=1"
            );
        });
    }

    #[test]
    fn test_is_low_resources_true_for_any_value() {
        // Note: uses .is_ok() which means any value triggers the mode
        with_env(&[("XTASK_LOW_RESOURCES", "true")], || {
            assert!(
                is_low_resources(),
                "is_low_resources() triggers on any value (using .is_ok())"
            );
        });
    }

    // =========================================================================
    // should_skip_bdd() tests
    // =========================================================================

    #[test]
    fn test_should_skip_bdd_false_when_not_set() {
        with_env(&[], || {
            assert!(!should_skip_bdd(), "should_skip_bdd() should return false by default");
        });
    }

    #[test]
    fn test_should_skip_bdd_true_when_explicitly_set() {
        with_env(&[("XTASK_SKIP_BDD", "1")], || {
            assert!(
                should_skip_bdd(),
                "should_skip_bdd() should return true when XTASK_SKIP_BDD=1"
            );
        });
    }

    #[test]
    fn test_should_skip_bdd_true_when_low_resources() {
        with_env(&[("XTASK_LOW_RESOURCES", "1")], || {
            assert!(should_skip_bdd(), "should_skip_bdd() should return true in low-resource mode");
        });
    }

    // =========================================================================
    // describe_mode() tests
    // =========================================================================

    #[test]
    fn test_describe_mode_interactive_default() {
        with_env(&[], || {
            assert_eq!(describe_mode(), "interactive", "Default mode should be 'interactive'");
        });
    }

    #[test]
    fn test_describe_mode_ci() {
        with_env(&[("CI", "1")], || {
            assert_eq!(describe_mode(), "CI", "Mode should be 'CI' when CI=1");
        });
    }

    #[test]
    fn test_describe_mode_ci_low_resources() {
        with_env(&[("CI", "1"), ("XTASK_LOW_RESOURCES", "1")], || {
            assert_eq!(
                describe_mode(),
                "CI (low-resources)",
                "Mode should show both CI and low-resources"
            );
        });
    }

    #[test]
    fn test_describe_mode_noninteractive() {
        with_env(&[("XTASK_NONINTERACTIVE", "1")], || {
            assert_eq!(describe_mode(), "non-interactive", "Mode should be 'non-interactive'");
        });
    }

    #[test]
    fn test_describe_mode_low_resources() {
        with_env(&[("XTASK_LOW_RESOURCES", "1")], || {
            assert_eq!(describe_mode(), "low-resources", "Mode should be 'low-resources'");
        });
    }
}
