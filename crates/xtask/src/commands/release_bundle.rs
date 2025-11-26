use anyhow::{Context, Result};
use colored::Colorize;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Normalized task status used for release evidence generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskStatus {
    Todo,
    InProgress,
    Review,
    Done,
}

impl TaskStatus {
    fn is_done(&self) -> bool {
        matches!(self, TaskStatus::Done)
    }
}

/// Parse task status strings from tasks.yaml or tasks_state.yaml into a normalized enum.
fn parse_task_status(status: &str) -> Option<TaskStatus> {
    match status.to_lowercase().as_str() {
        "todo" | "open" => Some(TaskStatus::Todo),
        "in_progress" | "inprogress" | "in-progress" => Some(TaskStatus::InProgress),
        "review" => Some(TaskStatus::Review),
        "done" | "closed" | "complete" | "completed" => Some(TaskStatus::Done),
        _ => None,
    }
}

#[derive(Debug, Default, Deserialize)]
struct TasksState {
    tasks: HashMap<String, String>,
}

fn load_task_status_overrides(root: &Path) -> Result<HashMap<String, TaskStatus>> {
    let path = root.join("specs/tasks_state.yaml");
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let content =
        fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
    let state: TasksState =
        serde_yaml::from_str(&content).context("Failed to parse specs/tasks_state.yaml")?;

    let mut overrides = HashMap::new();
    for (id, status) in state.tasks {
        if let Some(normalized) = parse_task_status(&status) {
            overrides.insert(id, normalized);
        }
    }

    Ok(overrides)
}

/// Generate a release evidence bundle for the specified version.
///
/// This command collects comprehensive evidence for a release, including:
/// - Completed tasks from specs/tasks.yaml
/// - Linked requirements and acceptance criteria
/// - AC deltas (added/modified/removed) since the last tagged release
/// - Architecture decision records (ADRs)
/// - Git changelog since the last tag
/// - Governance status (selftest and policy results)
/// - Resolved friction log entries
///
/// The evidence bundle is written to `release_evidence/vX.Y.Z.md` and can be
/// fed to an LLM or used manually to generate CHANGELOG.md entries.
pub fn run(version: &str) -> Result<()> {
    println!(
        "{}",
        format!("📦 Generating release evidence bundle for {}...", version).blue().bold()
    );
    println!();

    // Validate version format
    if !version.chars().next().unwrap_or('0').is_numeric() {
        anyhow::bail!("Version should be format X.Y.Z (e.g., 3.1.0)");
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().unwrap().parent().unwrap();

    // Create release_evidence directory if it doesn't exist
    let evidence_dir = root.join("release_evidence");
    fs::create_dir_all(&evidence_dir).context("Failed to create release_evidence directory")?;

    let output_path = evidence_dir.join(format!("v{}.md", version));

    println!("📋 Collecting evidence...");
    println!();

    // Collect all evidence sections
    let tasks_section = collect_tasks_section(root, version)?;
    let acs_section = collect_acs_section(root, &tasks_section.0)?;
    let ac_delta_section = collect_ac_delta_section(root)?;
    let adrs_section = collect_adrs_section(root)?;
    let git_section = collect_git_changelog(root)?;
    let governance_section = collect_governance_status(root)?;
    let friction_section = collect_friction_entries(root)?;

    // Generate markdown output
    let mut content = String::new();
    content.push_str(&format!("# Release Evidence: v{}\n\n", version));
    content.push_str(&format!(
        "**Generated:** {}\n\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));
    content.push_str("---\n\n");

    content.push_str("## Tasks Completed\n\n");
    content.push_str(&tasks_section.1);
    content.push_str("\n---\n\n");

    content.push_str("## Acceptance Criteria & Requirements\n\n");
    content.push_str(&acs_section);
    content.push_str("\n---\n\n");

    content.push_str("## AC Changes Since Last Release\n\n");
    content.push_str(&ac_delta_section);
    content.push_str("\n---\n\n");

    content.push_str("## Architecture Decisions\n\n");
    content.push_str(&adrs_section);
    content.push_str("\n---\n\n");

    content.push_str("## Git Changelog\n\n");
    content.push_str(&git_section);
    content.push_str("\n---\n\n");

    content.push_str("## Governance Status\n\n");
    content.push_str(&governance_section);
    content.push_str("\n---\n\n");

    content.push_str("## Resolved Friction\n\n");
    content.push_str(&friction_section);
    content.push('\n');

    // Write evidence file
    fs::write(&output_path, &content)
        .with_context(|| format!("Failed to write evidence file: {}", output_path.display()))?;

    println!();
    println!("{} Evidence bundle written to: {}", "✓".green(), output_path.display());
    println!();
    println!("{}", "Next steps:".bold());
    println!("  1. Review evidence: {}", format!("cat {}", output_path.display()).cyan());
    println!("  2. Feed to LLM for changelog generation");
    println!("  3. Update CHANGELOG.md with generated content");

    Ok(())
}

/// Collect completed tasks (returns list of requirement IDs and markdown content)
fn collect_tasks_section(root: &Path, _version: &str) -> Result<(Vec<String>, String)> {
    let tasks_spec = spec_runtime::load_tasks(&root.join("specs/tasks.yaml"))?;
    let status_overrides = load_task_status_overrides(root)?;

    let mut requirement_ids = Vec::new();
    let mut content = String::new();

    // Filter tasks with final status "done" (consider tasks_state overrides)
    let done_tasks: Vec<_> = tasks_spec
        .tasks
        .iter()
        .filter(|t| {
            status_overrides
                .get(&t.id)
                .copied()
                .or_else(|| parse_task_status(&t.status))
                .unwrap_or(TaskStatus::Todo)
                .is_done()
        })
        .collect();

    if done_tasks.is_empty() {
        content.push_str("*No tasks marked as done for this release.*\n");
    } else {
        content.push_str(&format!("**Total completed:** {} tasks\n\n", done_tasks.len()));

        for task in done_tasks {
            requirement_ids.push(task.requirement.clone());
            content.push_str(&format!("### {}\n\n", task.id));
            content.push_str(&format!("**Title:** {}\n\n", task.title));
            content.push_str(&format!("**Requirement:** {}\n\n", task.requirement));
            content.push_str(&format!("**ACs:** {}\n\n", task.acs.join(", ")));
            if !task.labels.is_empty() {
                content.push_str(&format!("**Labels:** {}\n\n", task.labels.join(", ")));
            }
            content.push_str(&format!("**Summary:** {}\n\n", task.summary));
        }
    }

    Ok((requirement_ids, content))
}

/// Collect ACs and Requirements linked to tasks
fn collect_acs_section(root: &Path, requirement_ids: &[String]) -> Result<String> {
    let ledger = spec_runtime::load_spec_ledger(&root.join("specs/spec_ledger.yaml"))?;

    let mut content = String::new();
    let req_set: HashSet<&str> = requirement_ids.iter().map(|s| s.as_str()).collect();

    for story in &ledger.stories {
        for req in &story.requirements {
            if req_set.contains(req.id.as_str()) {
                content.push_str(&format!("### {} - {}\n\n", req.id, req.title));
                content.push_str(&format!("**Story:** {} - {}\n\n", story.id, story.title));

                if !req.tags.is_empty() {
                    content.push_str(&format!("**Tags:** {}\n\n", req.tags.join(", ")));
                }

                if !req.acceptance_criteria.is_empty() {
                    content.push_str("**Acceptance Criteria:**\n\n");
                    for ac in &req.acceptance_criteria {
                        content.push_str(&format!("- **{}**: {}\n", ac.id, ac.text));
                    }
                    content.push('\n');
                }
            }
        }
    }

    if content.is_empty() {
        content.push_str("*No requirements linked to completed tasks.*\n");
    }

    Ok(content)
}

/// Collect ADR references
fn collect_adrs_section(root: &Path) -> Result<String> {
    let adr_dir = root.join("docs/adr");
    let mut content = String::new();

    if !adr_dir.exists() {
        content.push_str("*No ADR directory found.*\n");
        return Ok(content);
    }

    let mut adrs = Vec::new();
    for entry in fs::read_dir(&adr_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("md")
            && let Some(filename) = path.file_name().and_then(|s| s.to_str())
        {
            adrs.push(filename.to_string());
        }
    }

    adrs.sort();

    if adrs.is_empty() {
        content.push_str("*No ADRs found.*\n");
    } else {
        content.push_str(&format!("**Total ADRs:** {}\n\n", adrs.len()));
        for adr in adrs {
            content.push_str(&format!("- {}\n", adr));
        }
    }

    Ok(content)
}

/// Collect git changelog since last tag
fn collect_git_changelog(root: &Path) -> Result<String> {
    let mut content = String::new();

    // Get last tag
    let last_tag_output =
        Command::new("git").args(["describe", "--tags", "--abbrev=0"]).current_dir(root).output();

    let range = match last_tag_output {
        Ok(output) if output.status.success() => {
            let last_tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
            content.push_str(&format!("**Since tag:** {}\n\n", last_tag));
            format!("{}..HEAD", last_tag)
        }
        _ => {
            content.push_str("**Since:** (no previous tag)\n\n");
            "HEAD".to_string()
        }
    };

    // Get git log
    let log_output = Command::new("git")
        .args(["log", &range, "--pretty=format:- %h %s (%an)", "--no-merges"])
        .current_dir(root)
        .output()
        .context("Failed to run git log")?;

    if log_output.status.success() {
        let log = String::from_utf8_lossy(&log_output.stdout);
        if log.trim().is_empty() {
            content.push_str("*No commits since last tag.*\n");
        } else {
            content.push_str(&log);
            content.push('\n');
        }
    } else {
        content.push_str("*Failed to retrieve git log.*\n");
    }

    Ok(content)
}

/// Collect governance status
fn collect_governance_status(root: &Path) -> Result<String> {
    let mut content = String::new();

    content.push_str("### Selftest Status\n\n");

    // Run selftest in low-resource mode and capture output
    let selftest_output = Command::new("cargo")
        .args(["run", "-p", "xtask", "--", "selftest"])
        .env("XTASK_LOW_RESOURCES", "1")
        .current_dir(root)
        .output();

    match selftest_output {
        Ok(output) => {
            let status = if output.status.success() { "✅ PASSED" } else { "❌ FAILED" };
            content.push_str(&format!("**Status:** {}\n\n", status));

            // Include abbreviated output
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if !stdout.is_empty() {
                content.push_str("```\n");
                // Only include last 30 lines to keep it concise
                let lines: Vec<&str> = stdout.lines().collect();
                let start = lines.len().saturating_sub(30);
                for line in &lines[start..] {
                    content.push_str(line);
                    content.push('\n');
                }
                content.push_str("```\n\n");
            }

            if !stderr.is_empty() && !output.status.success() {
                content.push_str("**Errors:**\n```\n");
                content.push_str(&stderr);
                content.push_str("```\n\n");
            }
        }
        Err(e) => {
            content.push_str(&format!("**Status:** ⚠️ Unable to run selftest: {}\n\n", e));
        }
    }

    // Check policy status
    content.push_str("### Policy Status\n\n");
    let policy_status_path = root.join("target/policy_status.json");
    if policy_status_path.exists() {
        match fs::read_to_string(&policy_status_path) {
            Ok(policy_json) => {
                content.push_str("```json\n");
                content.push_str(&policy_json);
                content.push_str("\n```\n");
            }
            Err(_) => {
                content.push_str("*Policy status file exists but could not be read.*\n");
            }
        }
    } else {
        content.push_str("*No policy status file found (target/policy_status.json).*\n");
    }

    Ok(content)
}

/// Collect resolved friction entries
fn collect_friction_entries(root: &Path) -> Result<String> {
    let mut content = String::new();
    let friction_log_path = root.join("FRICTION_LOG.md");

    if !friction_log_path.exists() {
        content.push_str("*No friction log found.*\n");
        return Ok(content);
    }

    let friction_log =
        fs::read_to_string(&friction_log_path).context("Failed to read FRICTION_LOG.md")?;

    // Look for resolved entries (status: resolved)
    let mut resolved_count = 0;
    let mut in_entry = false;
    let mut current_entry = String::new();

    for line in friction_log.lines() {
        if line.starts_with("###") {
            if in_entry && !current_entry.is_empty() {
                content.push_str(&current_entry);
                content.push('\n');
            }
            in_entry = true;
            current_entry.clear();
            current_entry.push_str(line);
            current_entry.push('\n');
        } else if in_entry {
            current_entry.push_str(line);
            current_entry.push('\n');

            if line.contains("status: resolved") || line.contains("Status: resolved") {
                resolved_count += 1;
            }
        }
    }

    // Add last entry if it was resolved
    if in_entry && !current_entry.is_empty() && current_entry.contains("resolved") {
        content.push_str(&current_entry);
    }

    if resolved_count == 0 {
        content.clear();
        content.push_str("*No resolved friction entries found.*\n");
    } else {
        let mut final_content = format!("**Total resolved entries:** {}\n\n", resolved_count);
        final_content.push_str(&content);
        content = final_content;
    }

    Ok(content)
}

/// Represents the delta between two versions' ACs
#[derive(Debug)]
struct AcDelta {
    added: Vec<AcInfo>,
    modified: Vec<(AcInfo, AcInfo)>, // (old, new)
    removed: Vec<AcInfo>,
}

#[derive(Debug, Clone)]
struct AcInfo {
    id: String,
    text: String,
    requirement_id: String,
}

/// Load spec_ledger from a specific git tag
fn load_spec_ledger_from_tag(root: &Path, tag: &str) -> Result<Option<spec_runtime::SpecLedger>> {
    // Try to extract spec_ledger.yaml from the git tag
    let output = Command::new("git")
        .args(["show", &format!("{}:specs/spec_ledger.yaml", tag)])
        .current_dir(root)
        .output()
        .context("Failed to run git show")?;

    if !output.status.success() {
        // Tag doesn't have spec_ledger.yaml, return None
        return Ok(None);
    }

    // Parse the ledger directly from the output
    let content =
        String::from_utf8(output.stdout).context("Failed to decode spec_ledger.yaml from git")?;

    let ledger: spec_runtime::SpecLedger =
        serde_yaml::from_str(&content).context("Failed to parse spec_ledger.yaml from tag")?;

    Ok(Some(ledger))
}

/// Extract all ACs from a spec ledger
fn extract_acs_from_ledger(ledger: &spec_runtime::SpecLedger) -> HashMap<String, AcInfo> {
    let mut acs = HashMap::new();

    for story in &ledger.stories {
        for req in &story.requirements {
            for ac in &req.acceptance_criteria {
                acs.insert(
                    ac.id.clone(),
                    AcInfo {
                        id: ac.id.clone(),
                        text: ac.text.clone(),
                        requirement_id: req.id.clone(),
                    },
                );
            }
        }
    }

    acs
}

/// Calculate AC deltas between current version and previous tag
fn calculate_ac_delta(root: &Path) -> Result<Option<AcDelta>> {
    // Get last tag
    let last_tag_output =
        Command::new("git").args(["describe", "--tags", "--abbrev=0"]).current_dir(root).output();

    let last_tag = match last_tag_output {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => {
            // No previous tag found
            return Ok(None);
        }
    };

    // Load current spec_ledger
    let current_ledger = spec_runtime::load_spec_ledger(&root.join("specs/spec_ledger.yaml"))
        .context("Failed to load current spec_ledger.yaml")?;

    // Load previous spec_ledger from tag
    let previous_ledger = match load_spec_ledger_from_tag(root, &last_tag)? {
        Some(ledger) => ledger,
        None => {
            // Previous tag doesn't have spec_ledger.yaml
            return Ok(None);
        }
    };

    // Extract ACs from both versions
    let current_acs = extract_acs_from_ledger(&current_ledger);
    let previous_acs = extract_acs_from_ledger(&previous_ledger);

    // Calculate deltas
    let mut added = Vec::new();
    let mut modified = Vec::new();
    let mut removed = Vec::new();

    // Find added and modified ACs
    for (id, current_ac) in &current_acs {
        if let Some(previous_ac) = previous_acs.get(id) {
            // AC exists in both versions - check if modified
            if current_ac.text != previous_ac.text {
                modified.push((previous_ac.clone(), current_ac.clone()));
            }
        } else {
            // AC only exists in current version - it's new
            added.push(current_ac.clone());
        }
    }

    // Find removed ACs
    for (id, previous_ac) in &previous_acs {
        if !current_acs.contains_key(id) {
            removed.push(previous_ac.clone());
        }
    }

    // Sort by ID for consistent output
    added.sort_by(|a, b| a.id.cmp(&b.id));
    modified.sort_by(|a, b| a.0.id.cmp(&b.0.id));
    removed.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(Some(AcDelta { added, modified, removed }))
}

/// Collect AC delta information since last release
fn collect_ac_delta_section(root: &Path) -> Result<String> {
    let mut content = String::new();

    match calculate_ac_delta(root)? {
        None => {
            content.push_str("*No previous release found for comparison.*\n");
        }
        Some(delta) => {
            let total_changes = delta.added.len() + delta.modified.len() + delta.removed.len();

            if total_changes == 0 {
                content.push_str("*No AC changes since last release.*\n");
            } else {
                // Summary counts
                content.push_str(&format!(
                    "**Total changes:** {} (Added: {}, Modified: {}, Removed: {})\n\n",
                    total_changes,
                    delta.added.len(),
                    delta.modified.len(),
                    delta.removed.len()
                ));

                // Added ACs
                if !delta.added.is_empty() {
                    content.push_str("### Added ACs\n\n");
                    for ac in &delta.added {
                        content.push_str(&format!(
                            "- **{}** ({}): {}\n",
                            ac.id, ac.requirement_id, ac.text
                        ));
                    }
                    content.push('\n');
                }

                // Modified ACs
                if !delta.modified.is_empty() {
                    content.push_str("### Modified ACs\n\n");
                    for (old, new) in &delta.modified {
                        content.push_str(&format!("- **{}** ({}):\n", new.id, new.requirement_id));
                        content.push_str(&format!("  - **Before:** {}\n", old.text));
                        content.push_str(&format!("  - **After:** {}\n", new.text));
                    }
                    content.push('\n');
                }

                // Removed ACs
                if !delta.removed.is_empty() {
                    content.push_str("### Removed ACs\n\n");
                    for ac in &delta.removed {
                        content.push_str(&format!(
                            "- **{}** ({}): {}\n",
                            ac.id, ac.requirement_id, ac.text
                        ));
                    }
                    content.push('\n');
                }
            }
        }
    }

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn tasks_state_overrides_tasks_yaml_status() {
        let temp = tempdir().unwrap();
        let specs_dir = temp.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        let tasks_yaml = r#"
schema_version: "1.0"
template_version: "1.0"
tasks:
  - id: TASK-1
    title: "Example task"
    requirement: REQ-1
    acs: []
    status: todo
    owner:
    labels: []
    docs:
      design: []
      plan: []
    summary: "Example summary"
    recommended_flows: []
  - id: TASK-2
    title: "Still pending"
    requirement: REQ-2
    acs: []
    status: todo
    owner:
    labels: []
    docs:
      design: []
      plan: []
    summary: "Another summary"
    recommended_flows: []
"#;

        fs::write(specs_dir.join("tasks.yaml"), tasks_yaml.trim()).unwrap();
        fs::write(specs_dir.join("tasks_state.yaml"), "tasks:\n  TASK-1: Done\n").unwrap();

        let (requirements, content) = collect_tasks_section(temp.path(), "1.0.0").unwrap();

        assert!(requirements.contains(&"REQ-1".to_string()));
        assert!(content.contains("TASK-1"));
        assert!(!content.contains("TASK-2"));
        assert!(!content.contains("No tasks marked as done"));
    }
}
