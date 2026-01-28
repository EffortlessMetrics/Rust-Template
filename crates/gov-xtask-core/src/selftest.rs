//! Selftest pipeline for governance validation.
//!
//! Defines the standard 12-step selftest pipeline that service repos execute.

use crate::env::Verbosity;
use anyhow::Result;
use gov_model::context::RepoContext;
use std::time::{Duration, Instant};

/// A single selftest step.
#[derive(Debug, Clone)]
pub struct SelftestStep {
    /// Step number (1-indexed).
    pub number: usize,
    /// Total number of steps.
    pub total: usize,
    /// Step name.
    pub name: String,
    /// Step description.
    pub description: String,
}

impl SelftestStep {
    /// Format step header for output.
    pub fn header(&self) -> String {
        format!("[{}/{}] {}", self.number, self.total, self.name)
    }
}

/// Result of a selftest step.
#[derive(Debug, Clone)]
pub struct StepResult {
    /// Step that was executed.
    pub step: SelftestStep,
    /// Whether the step passed.
    pub passed: bool,
    /// Duration of the step.
    pub duration: Duration,
    /// Error message if failed.
    pub error: Option<String>,
}

/// Result of the full selftest pipeline.
#[derive(Debug)]
pub struct SelftestResult {
    /// Results for each step.
    pub steps: Vec<StepResult>,
    /// Total duration.
    pub total_duration: Duration,
    /// Whether all steps passed.
    pub all_passed: bool,
}

impl SelftestResult {
    /// Get summary of passed/failed steps.
    pub fn summary(&self) -> String {
        let passed = self.steps.iter().filter(|s| s.passed).count();
        let total = self.steps.len();
        format!("{}/{} steps passed in {:?}", passed, total, self.total_duration)
    }
}

/// Standard selftest step definitions.
///
/// These 12 steps define the governance validation pipeline.
/// Order and naming must match the actual xtask selftest implementation.
pub const SELFTEST_STEPS: &[(&str, &str)] = &[
    ("Core checks", "fmt, clippy, unit tests"),
    ("Skills governance", "skills-lint validation"),
    ("Agents governance", "agents-lint validation"),
    ("BDD acceptance", "Gherkin acceptance tests"),
    ("AC/ADR mapping", "AC status and ADR reference validation"),
    ("LLM bundler", "context bundle generation"),
    ("Policy tests", "conftest policy validation"),
    ("DevEx contract", "required commands exist"),
    ("Governance graph & UI", "graph invariants and UI contract"),
    ("AC coverage", "kernel AC test coverage"),
    ("Test coverage", "code coverage baseline (advisory)"),
];

/// Total number of selftest steps.
pub const SELFTEST_STEP_COUNT: usize = 11;

/// Run a function as a selftest step.
pub fn run_step<F>(step: SelftestStep, f: F) -> StepResult
where
    F: FnOnce() -> Result<()>,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();

    StepResult {
        step,
        passed: result.is_ok(),
        duration,
        error: result.err().map(|e| e.to_string()),
    }
}

/// Run the full selftest pipeline with the given context.
///
/// This is the main entry point for service repos.
/// The `step_runner` closure receives a step index (0-based) and step definition,
/// and should return `Ok(())` on success or an error on failure.
///
/// # Arguments
///
/// * `_ctx` - Repository context (currently unused, reserved for future use)
/// * `_verbosity` - Verbosity level for output (currently unused, reserved for future use)
/// * `step_runner` - Closure that executes each step
///
/// # Example
///
/// ```ignore
/// use gov_xtask_core::selftest::{run_selftest_pipeline, SELFTEST_STEPS};
/// use gov_model::RepoContext;
///
/// let ctx = RepoContext::new("/workspace");
/// let result = run_selftest_pipeline(&ctx, Verbosity::Normal, |step| {
///     println!("{}", step.header());
///     // Run step implementation...
///     Ok(())
/// });
/// ```
pub fn run_selftest_pipeline<F>(
    _ctx: &RepoContext,
    _verbosity: Verbosity,
    step_runner: F,
) -> SelftestResult
where
    F: Fn(&SelftestStep) -> Result<()>,
{
    let pipeline_start = Instant::now();
    let total = SELFTEST_STEPS.len();
    let mut results = Vec::with_capacity(total);

    for (i, (name, description)) in SELFTEST_STEPS.iter().enumerate() {
        let step = SelftestStep {
            number: i + 1,
            total,
            name: (*name).to_string(),
            description: (*description).to_string(),
        };

        let step_start = Instant::now();
        let result = step_runner(&step);
        let duration = step_start.elapsed();

        results.push(StepResult {
            step,
            passed: result.is_ok(),
            duration,
            error: result.err().map(|e| e.to_string()),
        });
    }

    let all_passed = results.iter().all(|r| r.passed);

    SelftestResult { steps: results, total_duration: pipeline_start.elapsed(), all_passed }
}

/// Create a step definition for a given index.
///
/// Useful when you need to construct a step outside the pipeline.
pub fn make_step(index: usize) -> Option<SelftestStep> {
    SELFTEST_STEPS.get(index).map(|(name, description)| SelftestStep {
        number: index + 1,
        total: SELFTEST_STEPS.len(),
        name: (*name).to_string(),
        description: (*description).to_string(),
    })
}
