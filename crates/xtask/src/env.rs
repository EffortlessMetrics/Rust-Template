//! Environment detection helpers for CI and automation contexts.
//!
//! This module re-exports environment detection from gov-xtask-core and adds
//! xtask-specific enhancements. Use these instead of duplicating env var checks
//! across commands.
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

// Re-export core environment detection functions from gov-xtask-core
pub use gov_xtask_core::{is_ci, is_low_resources, is_noninteractive};

/// Returns true if BDD tests should be skipped.
///
/// This is true when either:
/// - `XTASK_SKIP_BDD=1` is explicitly set, OR
/// - Low-resource mode is enabled
///
/// Note: This extends gov-xtask-core's should_skip_bdd with xtask-specific logic.
pub fn should_skip_bdd() -> bool {
    env::var("XTASK_SKIP_BDD").ok().as_deref() == Some("1") || is_low_resources()
}

/// Returns a description of the current environment mode.
///
/// Useful for logging which mode is active.
///
/// Note: This extends gov-xtask-core's describe_mode with more detailed formatting.
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
    use std::sync::Mutex;

    /// Global lock to ensure env var tests don't run concurrently
    /// This prevents flaky tests from concurrent modifications
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// Environment variable names recognized by this module
    pub const CI_ENV_VARS: &[&str] =
        &["CI", "GITHUB_ACTIONS", "GITLAB_CI", "CIRCLECI", "JENKINS_URL", "BUILDKITE"];

    pub const XTASK_ENV_VARS: &[&str] =
        &["XTASK_NONINTERACTIVE", "XTASK_LOW_RESOURCES", "XTASK_SKIP_BDD"];

    /// Helper to run a test with specific env vars set, restoring afterward
    ///
    /// # Safety
    /// This uses unsafe env var manipulation which is safe in tests because:
    /// 1. Tests run with a global lock preventing concurrent access
    /// 2. All env vars are restored after the test completes
    fn with_env<F, R>(vars: &[(&str, &str)], f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _lock = ENV_LOCK.lock().unwrap();

        // Save current values and clear all relevant vars
        let saved: Vec<(String, Option<String>)> = CI_ENV_VARS
            .iter()
            .chain(XTASK_ENV_VARS.iter())
            .map(|&k| (k.to_string(), env::var(k).ok()))
            .collect();

        // SAFETY: We hold the ENV_LOCK and restore all values after the test
        unsafe {
            // Clear all to get a clean state
            for &var in CI_ENV_VARS.iter().chain(XTASK_ENV_VARS.iter()) {
                env::remove_var(var);
            }

            // Set the requested vars
            for (key, value) in vars {
                env::set_var(key, value);
            }
        }

        let result = f();

        // SAFETY: Restore original values - we hold the ENV_LOCK
        unsafe {
            for (key, original) in saved {
                match original {
                    Some(val) => env::set_var(&key, val),
                    None => env::remove_var(&key),
                }
            }
        }

        result
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
        // Note: gov-xtask-core uses .is_ok() which means any value triggers the mode
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
        // Note: gov-xtask-core uses .is_ok() which means any value triggers the mode
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

    // =========================================================================
    // Contract documentation tests
    // =========================================================================

    /// @AC-TPL-XTASK-NONINTERACTIVE: Verify CI env vars constant matches implementation
    #[test]
    fn test_ci_env_vars_match_is_ci_implementation() {
        // Verify each var in CI_ENV_VARS triggers is_ci()
        for &var in CI_ENV_VARS {
            with_env(&[(var, "1")], || {
                assert!(is_ci(), "CI var '{}' should trigger is_ci()", var);
            });
        }
    }

    /// @AC-TPL-XTASK-NONINTERACTIVE: Verify XTASK env vars constant matches implementation
    #[test]
    fn test_xtask_env_vars_are_recognized() {
        // XTASK_NONINTERACTIVE
        with_env(&[("XTASK_NONINTERACTIVE", "1")], || {
            assert!(is_noninteractive());
        });

        // XTASK_LOW_RESOURCES
        with_env(&[("XTASK_LOW_RESOURCES", "1")], || {
            assert!(is_low_resources());
        });

        // XTASK_SKIP_BDD
        with_env(&[("XTASK_SKIP_BDD", "1")], || {
            assert!(should_skip_bdd());
        });
    }
}
