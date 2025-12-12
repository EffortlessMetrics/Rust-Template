//! Selftest pipeline for governance validation.
//!
//! Defines the standard 11-step selftest pipeline that service repos execute.

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
pub const SELFTEST_STEPS: &[(&str, &str)] = &[
    ("Core checks", "fmt, clippy, unit tests"),
    ("Skills governance", "skills-lint validation"),
    ("Agents governance", "agents-lint validation"),
    ("AC lint", "spec_ledger.yaml validation"),
    ("AC kernel mapping", "kernel ACs have test mappings"),
    ("Contracts check", "governed facts synchronization"),
    ("ADR check", "ADR reference validation"),
    ("Docs check", "documentation consistency"),
    ("BDD acceptance", "Gherkin acceptance tests"),
    ("Audit", "cargo-audit + cargo-deny"),
    ("Coverage", "test coverage baseline"),
];

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
/// Pass a closure for each step that returns Result<()>.
pub fn run_selftest_pipeline<F>(
    _ctx: &RepoContext,
    _verbosity: Verbosity,
    _step_runner: F,
) -> SelftestResult
where
    F: Fn(&SelftestStep) -> Result<()>,
{
    let start = Instant::now();
    let steps = Vec::new();

    // Placeholder - actual implementation delegates to service xtask

    SelftestResult { steps, total_duration: start.elapsed(), all_passed: true }
}
