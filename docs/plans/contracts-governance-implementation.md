# Implementation Plan: Contracts Governance System

**Date:** 2025-12-05
**Status:** Proposed
**Addresses:** Documentation drift for governed facts (selftest steps, kernel AC count, etc.)

## Problem Statement

In a Rust-as-Spec repo, numbers like "11-step selftest gate" and "61 kernel ACs" are **governed facts** that currently drift across documentation. When selftest steps change from 10→11, we must manually hunt down every reference. This is error-prone and violates the repo's own principles.

Current drift examples found:
- "7-step", "8-step", "10-step", "11-step" all appear in docs
- "52 kernel ACs", "61 kernel ACs" in different places
- These are stale references that weren't updated when the source of truth changed

## Proposed Solution

Implement a **contracts governance system** mirroring the existing version manifest pattern:

1. **Single source of truth:** Compute governed facts from code/specs
2. **Declarative manifest:** Define where these facts appear in docs
3. **Automated enforcement:** `contracts-fmt` and `contracts-check` commands
4. **CI integration:** Wire into `docs-check` and `release-prepare`

---

## Implementation Steps

### Phase 1: Core Infrastructure

#### Step 1.1: Create `contracts.rs` module

**File:** `crates/xtask/src/contracts.rs`

```rust
//! Contracts module for computing and managing governed facts.
//!
//! This module provides utilities for:
//! - Computing governed facts from specs/code (selftest steps, kernel AC count, etc.)
//! - Loading contracts manifests that declare where facts appear in docs
//! - Planning and applying fact updates atomically

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

    /// List of /platform/* endpoints (derived from OpenAPI or router)
    pub platform_endpoints: Vec<String>,

    /// List of required CI checks (derived from workflows or manifest)
    pub required_checks: Vec<String>,
}

impl ContractsSnapshot {
    /// Compute the contracts snapshot from repository sources.
    pub fn compute(repo_root: &Path) -> Result<Self> {
        // 1. Selftest step count - from selftest.rs constant or enum
        let selftest_step_count = compute_selftest_step_count(repo_root)?;

        // 2. Kernel AC count - from spec_ledger.yaml
        let kernel_ac_count = compute_kernel_ac_count(repo_root)?;

        // 3. Platform endpoints - from OpenAPI spec
        let platform_endpoints = compute_platform_endpoints(repo_root)?;

        // 4. Required checks - from devex_flows.yaml or CI config
        let required_checks = compute_required_checks(repo_root)?;

        Ok(Self {
            selftest_step_count,
            kernel_ac_count,
            platform_endpoints,
            required_checks,
        })
    }
}

fn compute_selftest_step_count(repo_root: &Path) -> Result<usize> {
    // Read selftest.rs and count "[N/M]" pattern to determine total steps
    let selftest_path = repo_root.join("crates/xtask/src/commands/selftest.rs");
    let content = fs::read_to_string(&selftest_path)
        .context("Failed to read selftest.rs")?;

    // Find the highest step number in "[N/M]" patterns
    let re = Regex::new(r#"\[(\d+)/(\d+)\]"#)?;
    let mut max_total = 0usize;

    for cap in re.captures_iter(&content) {
        if let Ok(total) = cap[2].parse::<usize>() {
            max_total = max_total.max(total);
        }
    }

    if max_total == 0 {
        anyhow::bail!("Could not determine selftest step count from selftest.rs");
    }

    Ok(max_total)
}

fn compute_kernel_ac_count(repo_root: &Path) -> Result<usize> {
    let ledger_path = repo_root.join("specs/spec_ledger.yaml");
    let content = fs::read_to_string(&ledger_path)
        .context("Failed to read spec_ledger.yaml")?;

    // Parse ledger and count ACs where must_have_ac=true (or defaults to true)
    #[derive(Deserialize)]
    struct Ledger { stories: Vec<Story> }

    #[derive(Deserialize)]
    struct Story { requirements: Vec<Requirement> }

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

    fn default_true() -> bool { true }

    let ledger: Ledger = serde_yaml::from_str(&content)?;

    let count = ledger.stories.iter()
        .flat_map(|s| &s.requirements)
        .flat_map(|r| &r.acceptance_criteria)
        .filter(|ac| ac.must_have_ac)
        .count();

    Ok(count)
}

fn compute_platform_endpoints(repo_root: &Path) -> Result<Vec<String>> {
    let openapi_path = repo_root.join("specs/openapi/openapi.yaml");
    if !openapi_path.exists() {
        return Ok(vec![]);
    }

    let content = fs::read_to_string(&openapi_path)?;
    let mut endpoints = Vec::new();

    // Simple extraction: find lines that look like path definitions
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("/platform/") && trimmed.ends_with(':') {
            let path = trimmed.trim_end_matches(':');
            endpoints.push(path.to_string());
        }
    }

    endpoints.sort();
    Ok(endpoints)
}

fn compute_required_checks(repo_root: &Path) -> Result<Vec<String>> {
    // For now, derive from devex_flows.yaml or use a static list
    // Could be enhanced to parse .github/workflows/*.yml
    let devex_path = repo_root.join("specs/devex_flows.yaml");

    if devex_path.exists() {
        let content = fs::read_to_string(&devex_path)?;
        let spec: serde_yaml::Value = serde_yaml::from_str(&content)?;

        // Extract required commands
        let mut checks = Vec::new();
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
        return Ok(checks);
    }

    Ok(vec![])
}
```

#### Step 1.2: Create contracts manifest

**File:** `specs/contracts_manifest.yaml`

```yaml
# Contracts Manifest for Documentation Governance
# Schema Version: 1.0
# Purpose: Declares governed facts and where they appear in documentation.
#
# This manifest works with `cargo xtask contracts-fmt` and `contracts-check`
# to ensure documentation numbers stay synchronized with their source of truth.

schema_version: "1.0"
description: "Governed facts manifest - ensures doc numbers match code/specs"

contracts:
  # Selftest step count
  selftest_step_count:
    source: "crates/xtask/src/commands/selftest.rs"
    description: "Number of steps in the selftest governance gate"
    patterns:
      - file: "README.md"
        regex: "(?P<n>\\d+)-step selftest gate"
        template: "{n}-step selftest gate"

      - file: "CLAUDE.md"
        regex: "(?P<n>\\d+)-step governance gate"
        template: "{n}-step governance gate"

      - file: "docs/ROADMAP.md"
        regex: "(?P<n1>\\d+)/(?P<n2>\\d+) gates"
        template: "{n}/{n} gates"

      - file: "docs/QUICKSTART.md"
        regex: "(?P<n>\\d+)-step governance validation"
        template: "{n}-step governance validation"

      - file: "docs/explanation/template-architecture.md"
        regex: "the (?P<n>\\d+)-step governance validation"
        template: "the {n}-step governance validation"

      - file: "docs/how-to/run-agent-pilot.md"
        regex: "\\((?P<n>\\d+)-step governance gate\\)"
        template: "({n}-step governance gate)"

      - file: "docs/reference/xtask-commands.md"
        regex: "Comprehensive (?P<n>\\d+)-step validation"
        template: "Comprehensive {n}-step validation"

      # Skills files - mark as optional (may not all need updating)
      - file: ".claude/skills/governed-feature-dev/SKILL.md"
        regex: "passes \\((?P<n1>\\d+)/(?P<n2>\\d+) steps\\)"
        template: "passes ({n}/{n} steps)"
        required: false

      - file: ".claude/skills/governed-release/SKILL.md"
        regex: "passes \\((?P<n1>\\d+)/(?P<n2>\\d+) steps\\)"
        template: "passes ({n}/{n} steps)"
        required: false

      - file: ".claude/skills/governed-maintenance/SKILL.md"
        regex: "passes \\((?P<n1>\\d+)/(?P<n2>\\d+) steps\\)"
        template: "passes ({n}/{n} steps)"
        required: false

      - file: ".claude/skills/governed-governance-debug/SKILL.md"
        regex: "Selftest passes \\((?P<n1>\\d+)/(?P<n2>\\d+) steps\\)"
        template: "Selftest passes ({n}/{n} steps)"
        required: false

  # Kernel AC count
  kernel_ac_count:
    source: "specs/spec_ledger.yaml"
    description: "Count of kernel ACs (must_have_ac=true)"
    patterns:
      - file: "docs/feature_status_notes.md"
        regex: "\\*\\*Kernel ACs.*\\*\\*: (?P<n>\\d+)"
        template: "**Kernel ACs (must_have_ac: true):** {n}"

      - file: "docs/feature_status_notes.md"
        regex: "All (?P<n>\\d+) kernel ACs"
        template: "All {n} kernel ACs"

      - file: "docs/KERNEL_SNAPSHOT.md"
        regex: "(?P<n>\\d+) kernel ACs"
        template: "{n} kernel ACs"
        required: false

# Validation settings
validation:
  # Fail if a required pattern doesn't match
  strict_patterns: true

  # Warn about numbers that look like contract values but aren't in manifest
  detect_orphans: true

  # Orphan detection patterns (regex for numbers that might be contract values)
  orphan_patterns:
    - "\\d+-step"
    - "\\d+/\\d+ gate"
    - "\\d+ kernel AC"
```

### Phase 2: Commands Implementation

#### Step 2.1: Create `contracts.rs` command module

**File:** `crates/xtask/src/commands/contracts.rs`

```rust
//! Contracts governance commands: fmt and check.

use anyhow::{Context, Result};
use colored::Colorize;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::contracts::ContractsSnapshot;

#[derive(Debug, Deserialize)]
struct ContractsManifest {
    schema_version: String,
    #[allow(dead_code)]
    description: Option<String>,
    contracts: HashMap<String, ContractDef>,
    #[serde(default)]
    validation: ValidationSettings,
}

#[derive(Debug, Deserialize)]
struct ContractDef {
    #[allow(dead_code)]
    source: String,
    #[allow(dead_code)]
    description: Option<String>,
    patterns: Vec<PatternDef>,
}

#[derive(Debug, Deserialize)]
struct PatternDef {
    file: String,
    regex: String,
    template: String,
    #[serde(default = "default_true")]
    required: bool,
}

fn default_true() -> bool { true }

#[derive(Debug, Default, Deserialize)]
struct ValidationSettings {
    #[serde(default)]
    strict_patterns: bool,
    #[serde(default)]
    detect_orphans: bool,
}

/// A planned edit for contract synchronization.
#[derive(Debug)]
pub struct ContractEdit {
    pub file: String,
    pub line_number: usize,
    pub old_text: String,
    pub new_text: String,
    pub contract_name: String,
}

/// Run contracts-check (dry-run validation).
pub fn check(repo_root: &Path) -> Result<()> {
    fmt_impl(repo_root, true)
}

/// Run contracts-fmt (apply changes).
pub fn fmt(repo_root: &Path) -> Result<()> {
    fmt_impl(repo_root, false)
}

fn fmt_impl(repo_root: &Path, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("{}", "📋 Checking contract governance...".blue().bold());
    } else {
        println!("{}", "📋 Synchronizing contract facts...".blue().bold());
    }
    println!();

    // 1. Compute the snapshot from sources
    let snapshot = ContractsSnapshot::compute(repo_root)?;

    println!("Computed facts from source:");
    println!("  • Selftest steps: {}", snapshot.selftest_step_count);
    println!("  • Kernel ACs: {}", snapshot.kernel_ac_count);
    println!("  • Platform endpoints: {}", snapshot.platform_endpoints.len());
    println!("  • Required checks: {}", snapshot.required_checks.len());
    println!();

    // 2. Load the manifest
    let manifest_path = repo_root.join("specs/contracts_manifest.yaml");
    if !manifest_path.exists() {
        if dry_run {
            println!("{}", "No contracts manifest found - skipping".yellow());
            return Ok(());
        }
        anyhow::bail!("contracts_manifest.yaml not found at {}", manifest_path.display());
    }

    let manifest_content = fs::read_to_string(&manifest_path)?;
    let manifest: ContractsManifest = serde_yaml::from_str(&manifest_content)
        .context("Failed to parse contracts_manifest.yaml")?;

    // 3. Plan changes
    let edits = plan_contract_edits(repo_root, &snapshot, &manifest)?;

    if edits.is_empty() {
        println!("{}", "✓ All contract facts are synchronized".green());
        return Ok(());
    }

    // 4. Apply or report
    if dry_run {
        println!("{}", "Contract drift detected:".yellow().bold());
        println!();

        for edit in &edits {
            println!("{}:{}", edit.file.cyan(), edit.line_number);
            println!("  Contract: {}", edit.contract_name.dimmed());
            println!("  {} {}", "-".red(), edit.old_text.dimmed());
            println!("  {} {}", "+".green(), edit.new_text.green());
            println!();
        }

        let files: std::collections::HashSet<_> = edits.iter().map(|e| &e.file).collect();
        anyhow::bail!(
            "contracts-check found {} edit(s) across {} file(s). Run `cargo xtask contracts-fmt` to fix.",
            edits.len(),
            files.len()
        );
    }

    // Apply edits
    apply_contract_edits(&edits)?;

    println!("{}", format!("✓ Applied {} contract updates", edits.len()).green());
    Ok(())
}

fn plan_contract_edits(
    repo_root: &Path,
    snapshot: &ContractsSnapshot,
    manifest: &ContractsManifest,
) -> Result<Vec<ContractEdit>> {
    let mut edits = Vec::new();

    for (contract_name, contract_def) in &manifest.contracts {
        // Get the value for this contract
        let value = match contract_name.as_str() {
            "selftest_step_count" => snapshot.selftest_step_count,
            "kernel_ac_count" => snapshot.kernel_ac_count,
            _ => continue, // Skip unknown contracts
        };

        for pattern_def in &contract_def.patterns {
            let file_path = repo_root.join(&pattern_def.file);

            if !file_path.exists() {
                if pattern_def.required {
                    eprintln!("  {} File not found: {}", "⚠".yellow(), pattern_def.file);
                }
                continue;
            }

            let content = fs::read_to_string(&file_path)?;
            let re = Regex::new(&pattern_def.regex)
                .with_context(|| format!("Invalid regex for {}: {}", contract_name, pattern_def.regex))?;

            for (line_num, line) in content.lines().enumerate() {
                if let Some(caps) = re.captures(line) {
                    // Check if the captured value matches current
                    let current_match = caps.get(0).map(|m| m.as_str()).unwrap_or("");

                    // Build the expected text using the template
                    let expected = pattern_def.template
                        .replace("{n}", &value.to_string())
                        .replace("{n1}", &value.to_string())
                        .replace("{n2}", &value.to_string());

                    // If they differ, plan an edit
                    if !line.contains(&expected) {
                        let new_line = re.replace(line, &expected).to_string();

                        if new_line != line {
                            edits.push(ContractEdit {
                                file: pattern_def.file.clone(),
                                line_number: line_num + 1,
                                old_text: line.to_string(),
                                new_text: new_line,
                                contract_name: contract_name.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(edits)
}

fn apply_contract_edits(edits: &[ContractEdit]) -> Result<()> {
    use std::collections::HashMap;

    // Group edits by file
    let mut by_file: HashMap<&str, Vec<&ContractEdit>> = HashMap::new();
    for edit in edits {
        by_file.entry(&edit.file).or_default().push(edit);
    }

    for (file_path, file_edits) in by_file {
        let content = fs::read_to_string(file_path)?;
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        // Sort edits by line number descending to avoid index shifting
        let mut sorted_edits = file_edits;
        sorted_edits.sort_by(|a, b| b.line_number.cmp(&a.line_number));

        for edit in sorted_edits {
            let idx = edit.line_number - 1;
            if idx < lines.len() && lines[idx] == edit.old_text {
                lines[idx] = edit.new_text.clone();
            }
        }

        // Write atomically
        let temp_path = format!("{}.tmp", file_path);
        let new_content = lines.join("\n") + "\n";
        fs::write(&temp_path, &new_content)?;
        fs::rename(&temp_path, file_path)?;

        println!("  {} {}", "✓".green(), file_path);
    }

    Ok(())
}
```

#### Step 2.2: Register commands in main.rs

Add to the `Commands` enum:

```rust
/// Check that governed facts in docs match their sources
#[command(next_help_heading = "📚 Documentation")]
ContractsCheck,

/// Synchronize governed facts from code/specs to docs
#[command(next_help_heading = "📚 Documentation")]
ContractsFmt,
```

Add match arms:

```rust
Commands::ContractsCheck => {
    commands::contracts::check(&repo_root)
}
Commands::ContractsFmt => {
    commands::contracts::fmt(&repo_root)
}
```

Add to `all_command_names()`:

```rust
"contracts-check",
"contracts-fmt",
```

#### Step 2.3: Register in mod.rs

```rust
pub mod contracts;
```

### Phase 3: Integration

#### Step 3.1: Wire into docs-check

In `docs_check.rs`, add after existing checks:

```rust
// Check contract facts synchronization
print!("Contract facts... ");
match crate::commands::contracts::check(&repo_root) {
    Ok(_) => println!("{}", "✓ Synchronized".green()),
    Err(e) => {
        println!("{}", "✗ Drift detected".red());
        eprintln!("  {}", e);
        eprintln!();
        eprintln!("{}", "To fix contract drift:".bold());
        eprintln!("  1. Run {} to synchronize", "cargo xtask contracts-fmt".cyan());
        eprintln!("  2. Commit the updated documentation");
        issues += 1;
    }
}
```

#### Step 3.2: Wire into release-prepare (optional)

In `release_prepare.rs`, after version updates:

```rust
// Synchronize contract facts in docs
println!("\n{}", "Synchronizing contract facts...".blue());
if !dry_run {
    crate::commands::contracts::fmt(&repo_root)?;
} else {
    if let Err(e) = crate::commands::contracts::check(&repo_root) {
        println!("  {} Contract facts would be updated", "→".yellow());
    }
}
```

### Phase 4: Testing and Documentation

#### Step 4.1: Add unit tests

In `contracts.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selftest_step_count_extraction() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap().parent().unwrap().to_path_buf();

        let count = compute_selftest_step_count(&repo_root).unwrap();
        assert!(count >= 10, "Should have at least 10 selftest steps");
        assert!(count <= 20, "Sanity check: shouldn't exceed 20 steps");
    }

    #[test]
    fn test_kernel_ac_count_extraction() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap().parent().unwrap().to_path_buf();

        let count = compute_kernel_ac_count(&repo_root).unwrap();
        assert!(count >= 50, "Should have at least 50 kernel ACs");
    }

    #[test]
    fn test_snapshot_computation() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap().parent().unwrap().to_path_buf();

        let snapshot = ContractsSnapshot::compute(&repo_root).unwrap();

        assert!(snapshot.selftest_step_count > 0);
        assert!(snapshot.kernel_ac_count > 0);
    }
}
```

#### Step 4.2: Update documentation

Add to `docs/reference/xtask-commands.md`:

```markdown
### contracts-check

Check that governed facts in documentation match their sources.

```bash
cargo xtask contracts-check
```

### contracts-fmt

Synchronize governed facts from code/specs to documentation.

```bash
cargo xtask contracts-fmt
```

```

---

## Files to Create/Modify

| File | Action | Purpose |
|------|--------|---------|
| `crates/xtask/src/contracts.rs` | Create | Snapshot computation |
| `crates/xtask/src/commands/contracts.rs` | Create | fmt/check commands |
| `crates/xtask/src/commands/mod.rs` | Modify | Register module |
| `crates/xtask/src/main.rs` | Modify | Add Commands enum + match arms |
| `specs/contracts_manifest.yaml` | Create | Pattern definitions |
| `crates/xtask/src/commands/docs_check.rs` | Modify | Integration |
| `docs/reference/xtask-commands.md` | Modify | Documentation |

---

## Success Criteria

1. `cargo xtask contracts-check` detects when selftest step count in docs doesn't match code
2. `cargo xtask contracts-fmt` updates all documented step counts to match
3. `cargo xtask docs-check` includes contracts validation
4. All existing tests continue to pass
5. The system correctly identifies the current selftest step count (11) and kernel AC count

---

## Future Enhancements

1. **Platform endpoint validation:** Verify documented endpoints match OpenAPI spec
2. **CI check validation:** Verify documented CI checks match workflow files
3. **Orphan detection:** Warn about numbers that look like contract values but aren't governed
4. **Historical suppression:** Support `<!-- doclint:disable contracts -->` comments

---

## Estimated Scope

- **Core implementation:** ~400 lines of Rust
- **Manifest:** ~100 lines of YAML
- **Documentation updates:** ~50 lines of Markdown
- **Testing:** ~100 lines of Rust tests

This follows the existing patterns in versioning.rs and aligns with the repo's governance philosophy: if it can drift, automate it.
