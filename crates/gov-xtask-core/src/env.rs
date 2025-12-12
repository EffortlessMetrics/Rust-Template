//! Environment detection utilities.
//!
//! Detects CI environments, resource constraints, and execution modes.

/// Check if running in a CI environment.
///
/// Detects: GitHub Actions, GitLab CI, CircleCI, Jenkins, BuildKite, Azure Pipelines.
pub fn is_ci() -> bool {
    std::env::var("CI").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || std::env::var("GITLAB_CI").is_ok()
        || std::env::var("CIRCLECI").is_ok()
        || std::env::var("JENKINS_URL").is_ok()
        || std::env::var("BUILDKITE").is_ok()
        || std::env::var("TF_BUILD").is_ok()
}

/// Check if running in non-interactive mode.
pub fn is_noninteractive() -> bool {
    std::env::var("XTASK_NONINTERACTIVE").is_ok() || is_ci()
}

/// Check if running in low-resource mode (sequential builds).
pub fn is_low_resources() -> bool {
    std::env::var("XTASK_LOW_RESOURCES").is_ok()
}

/// Check if BDD tests should be skipped.
pub fn should_skip_bdd() -> bool {
    std::env::var("XTASK_SKIP_BDD").is_ok()
}

/// Describe the current execution mode.
pub fn describe_mode() -> &'static str {
    if is_ci() {
        "ci"
    } else if is_low_resources() {
        "low-resources"
    } else {
        "normal"
    }
}

/// Verbosity level for command output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Verbosity {
    /// Minimal output.
    Quiet,
    /// Normal output.
    #[default]
    Normal,
    /// Verbose output with timing.
    Verbose,
}
