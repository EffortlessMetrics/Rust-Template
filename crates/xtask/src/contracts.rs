//! Contracts module for computing and managing governed facts.
//!
//! This module provides utilities for:
//! - Computing governed facts from specs/code (selftest steps, kernel AC count, etc.)
//! - These facts are the "single source of truth" that documentation must reflect
//!
//! ## Design
//!
//! In a Rust-as-Spec repo, certain numbers like "11-step selftest gate" and "61 kernel ACs"
//! are **governed facts** that appear in documentation. When the source changes (e.g., adding
//! a new selftest step), all documentation references must be updated.
//!
//! This module computes those facts from their actual sources, enabling automated validation
//! and synchronization via `cargo xtask contracts-check` and `cargo xtask contracts-fmt`.

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Snapshot of all governed contract facts.
/// These are computed from code/specs, not hardcoded.
#[derive(Debug, Clone, Serialize)]
pub struct ContractsSnapshot {
    /// Number of selftest steps (derived from selftest.rs step count)
    pub selftest_step_count: usize,

    /// Count of kernel ACs (must_have_ac=true in spec_ledger.yaml)
    pub kernel_ac_count: usize,

    /// List of /platform/* endpoints (derived from OpenAPI spec)
    pub platform_endpoints: Vec<String>,

    /// List of required CI checks (derived from devex_flows.yaml)
    pub required_checks: Vec<String>,
}

impl ContractsSnapshot {
    /// Compute the contracts snapshot from repository sources.
    pub fn compute(repo_root: &Path) -> Result<Self> {
        // 1. Selftest step count - from selftest.rs "[N/M]" patterns
        let selftest_step_count = compute_selftest_step_count(repo_root)?;

        // 2. Kernel AC count - from spec_ledger.yaml (must_have_ac=true)
        let kernel_ac_count = compute_kernel_ac_count(repo_root)?;

        // 3. Platform endpoints - from OpenAPI spec
        let platform_endpoints = compute_platform_endpoints(repo_root)?;

        // 4. Required checks - from devex_flows.yaml
        let required_checks = compute_required_checks(repo_root)?;

        Ok(Self { selftest_step_count, kernel_ac_count, platform_endpoints, required_checks })
    }
}

/// Extract selftest step count from selftest.rs by parsing "[N/M]" patterns.
fn compute_selftest_step_count(repo_root: &Path) -> Result<usize> {
    let selftest_path = repo_root.join("crates/xtask/src/commands/selftest.rs");
    let content = fs::read_to_string(&selftest_path).context("Failed to read selftest.rs")?;

    // Find the highest total in "[N/M]" patterns (e.g., "[1/11]", "[2/11]", etc.)
    let re = Regex::new(r#"\[(\d+)/(\d+)\]"#).context("Failed to compile regex")?;
    let mut max_total = 0usize;

    for cap in re.captures_iter(&content) {
        if let Ok(total) = cap[2].parse::<usize>() {
            max_total = max_total.max(total);
        }
    }

    if max_total == 0 {
        anyhow::bail!(
            "Could not determine selftest step count from selftest.rs. \
             Expected to find patterns like [1/11], [2/11], etc."
        );
    }

    Ok(max_total)
}

/// Count kernel ACs (must_have_ac=true) from spec_ledger.yaml.
fn compute_kernel_ac_count(repo_root: &Path) -> Result<usize> {
    let ledger_path = repo_root.join("specs/spec_ledger.yaml");
    let content = fs::read_to_string(&ledger_path).context("Failed to read spec_ledger.yaml")?;

    // Parse the ledger to count ACs where must_have_ac=true
    // Note: We need to handle the case where must_have_ac defaults to true
    #[derive(Deserialize)]
    struct Ledger {
        stories: Vec<Story>,
    }

    #[derive(Deserialize)]
    struct Story {
        requirements: Vec<Requirement>,
    }

    #[derive(Deserialize)]
    struct Requirement {
        #[serde(default = "default_true")]
        must_have_ac: bool,
        acceptance_criteria: Vec<AcceptanceCriteria>,
    }

    #[derive(Deserialize)]
    struct AcceptanceCriteria {
        #[serde(default = "default_true")]
        must_have_ac: bool,
    }

    fn default_true() -> bool {
        true
    }

    let ledger: Ledger =
        serde_yaml::from_str(&content).context("Failed to parse spec_ledger.yaml")?;

    // Count ACs where both the requirement AND the AC have must_have_ac=true
    let count = ledger
        .stories
        .iter()
        .flat_map(|s| &s.requirements)
        .filter(|r| r.must_have_ac)
        .flat_map(|r| &r.acceptance_criteria)
        .filter(|ac| ac.must_have_ac)
        .count();

    Ok(count)
}

/// Extract platform endpoints from OpenAPI spec.
fn compute_platform_endpoints(repo_root: &Path) -> Result<Vec<String>> {
    let openapi_path = repo_root.join("specs/openapi/openapi.yaml");
    if !openapi_path.exists() {
        return Ok(vec![]);
    }

    let content = fs::read_to_string(&openapi_path)?;
    let mut endpoints = Vec::new();

    // Parse YAML and extract paths that start with /platform/
    // Using simple line parsing to avoid heavy dependencies
    for line in content.lines() {
        let trimmed = line.trim();
        // OpenAPI path definitions look like: "  /platform/status:" (indented, ends with colon)
        if trimmed.starts_with("/platform/") && trimmed.ends_with(':') {
            let path = trimmed.trim_end_matches(':');
            endpoints.push(path.to_string());
        }
    }

    endpoints.sort();
    endpoints.dedup();
    Ok(endpoints)
}

/// Extract required commands from devex_flows.yaml.
fn compute_required_checks(repo_root: &Path) -> Result<Vec<String>> {
    let devex_path = repo_root.join("specs/devex_flows.yaml");
    if !devex_path.exists() {
        return Ok(vec![]);
    }

    let content = fs::read_to_string(&devex_path)?;
    let spec: serde_yaml::Value =
        serde_yaml::from_str(&content).context("Failed to parse devex_flows.yaml")?;

    let mut checks = Vec::new();

    // Extract commands where required: true
    if let Some(commands) = spec.get("commands").and_then(|c| c.as_mapping()) {
        for (name, cmd) in commands {
            if let Some(required) = cmd.get("required").and_then(|r| r.as_bool()) {
                if required {
                    if let Some(name_str) = name.as_str() {
                        checks.push(name_str.to_string());
                    }
                }
            }
        }
    }

    checks.sort();
    Ok(checks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("xtask parent")
            .parent()
            .expect("crates parent")
            .to_path_buf()
    }

    #[test]
    fn test_selftest_step_count_extraction() {
        let root = repo_root();
        let count = compute_selftest_step_count(&root).expect("Should extract step count");

        // Selftest should have between 10 and 20 steps (reasonable bounds)
        assert!(count >= 10, "Should have at least 10 selftest steps, got {}", count);
        assert!(count <= 20, "Sanity check: shouldn't exceed 20 steps, got {}", count);
    }

    #[test]
    fn test_kernel_ac_count_extraction() {
        let root = repo_root();
        let count = compute_kernel_ac_count(&root).expect("Should extract kernel AC count");

        // Should have a reasonable number of kernel ACs
        assert!(count >= 40, "Should have at least 40 kernel ACs, got {}", count);
        assert!(count <= 200, "Sanity check: shouldn't exceed 200 kernel ACs, got {}", count);
    }

    #[test]
    fn test_platform_endpoints_extraction() {
        let root = repo_root();
        let endpoints =
            compute_platform_endpoints(&root).expect("Should extract platform endpoints");

        // Should have some platform endpoints if OpenAPI exists
        if root.join("specs/openapi/openapi.yaml").exists() {
            assert!(!endpoints.is_empty(), "Should have platform endpoints if OpenAPI exists");
            // All endpoints should start with /platform/
            for ep in &endpoints {
                assert!(
                    ep.starts_with("/platform/"),
                    "Endpoint should start with /platform/: {}",
                    ep
                );
            }
        }
    }

    #[test]
    fn test_required_checks_extraction() {
        let root = repo_root();
        let checks = compute_required_checks(&root).expect("Should extract required checks");

        // Should have some required checks if devex_flows.yaml exists
        if root.join("specs/devex_flows.yaml").exists() {
            assert!(!checks.is_empty(), "Should have required checks if devex_flows.yaml exists");
        }
    }

    #[test]
    fn test_full_snapshot_computation() {
        let root = repo_root();
        let snapshot = ContractsSnapshot::compute(&root).expect("Should compute snapshot");

        assert!(snapshot.selftest_step_count > 0, "Should have selftest steps");
        assert!(snapshot.kernel_ac_count > 0, "Should have kernel ACs");

        // Print for visibility during test runs
        eprintln!("Contracts snapshot:");
        eprintln!("  selftest_step_count: {}", snapshot.selftest_step_count);
        eprintln!("  kernel_ac_count: {}", snapshot.kernel_ac_count);
        eprintln!("  platform_endpoints: {} total", snapshot.platform_endpoints.len());
        eprintln!("  required_checks: {} total", snapshot.required_checks.len());
    }
}
