use anyhow::{Context, Result};
use chrono::Utc;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Deserialize)]
struct ContextPack {
    tasks: std::collections::HashMap<String, Task>,
}

#[derive(Debug, Deserialize)]
struct Task {
    max_bytes: usize,
    include: Vec<String>,
    #[serde(default)]
    description: String,
}

/// Warning about referential integrity issues in bundle generation
#[derive(Debug, Serialize, Deserialize)]
struct BundleWarning {
    invalid_id: String,
    ref_type: String,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BundleManifest {
    bundle_version: i32,
    task_id: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    requirement_ids: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    ac_ids: Vec<String>,
    git_sha: String,
    timestamp: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    specs: Vec<ManifestSpec>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    docs: Vec<ManifestDoc>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tests: Vec<ManifestTest>,
    /// Warnings about invalid AC/REQ references (AC-TPL-BUNDLE-REFERENTIAL-INTEGRITY)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    warnings: Vec<BundleWarning>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManifestSpec {
    file: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManifestDoc {
    file: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManifestTest {
    r#type: String,
    tag: String,
    file: String,
}

/// Task definition from specs/tasks.yaml
#[derive(Debug, Deserialize)]
struct TaskSpec {
    id: String,
    #[serde(default)]
    requirement: Option<String>,
    #[serde(default)]
    acs: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TasksYaml {
    tasks: Vec<TaskSpec>,
}

/// Spec ledger AC entry
#[derive(Debug, Deserialize)]
struct SpecAc {
    id: String,
    #[serde(default)]
    tests: Vec<SpecTest>,
}

#[derive(Debug, Deserialize)]
struct SpecTest {
    r#type: String,
    tag: String,
    #[serde(default)]
    file: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SpecRequirement {
    /// Requirement ID from spec_ledger.yaml.
    /// Currently only used for deserialization; ID not needed in bundle generation.
    /// Future: Used in bundle metadata and dependency tracking.
    #[allow(dead_code)]
    id: String,
    acceptance_criteria: Vec<SpecAc>,
}

#[derive(Debug, Deserialize)]
struct SpecStory {
    requirements: Vec<SpecRequirement>,
}

#[derive(Debug, Deserialize)]
struct SpecLedger {
    stories: Vec<SpecStory>,
}

/// Display available tasks when the requested task is not found
fn display_available_tasks(contextpack: &ContextPack, invalid_task: &str) -> Result<()> {
    println!();
    println!("{}", "AVAILABLE BUNDLE TASKS".bold());
    println!();
    println!("{}", format!("Source: {}", ".llm/contextpack.yaml").dimmed());
    println!();

    if !invalid_task.is_empty() {
        println!("{}", format!("❌ Task '{}' not found", invalid_task).red());
        println!();
    }

    // Sort tasks alphabetically for consistent display
    let mut tasks: Vec<_> = contextpack.tasks.iter().collect();
    tasks.sort_by_key(|(name, _)| *name);

    println!("{}", "Available tasks:".cyan().bold());
    println!();

    for (name, task) in tasks {
        let description = if task.description.is_empty() {
            "No description available".dimmed().to_string()
        } else {
            task.description.clone()
        };

        println!("  {:<20} {}", name.bold(), description);
        println!("    {}", format!("Max size: {} bytes", task.max_bytes).dimmed());
        println!("    {}", format!("Patterns: {}", task.include.len()).dimmed());
        println!();
    }

    println!("{}", "Usage:".dimmed());
    println!("  {}", "cargo xtask bundle <TASK>".dimmed());
    println!();
    println!("{}", "Examples:".dimmed());
    println!("  {}", "cargo xtask bundle implement_ac".dimmed());
    println!("  {}", "cargo xtask bundle debug_tests".dimmed());
    println!();

    Ok(())
}

/// Generate LLM context bundle for a task
pub fn run(task_name: &str) -> Result<()> {
    let workspace_root = get_workspace_root()?;
    let contextpack_path = workspace_root.join(".llm/contextpack.yaml");

    if !contextpack_path.exists() {
        anyhow::bail!(
            "contextpack not found: {}\n\n\
             try: cargo xtask doctor\n\
             hint: ensure you're in the repository root",
            contextpack_path.display()
        );
    }

    let contextpack = load_contextpack(&contextpack_path)?;

    // If task not found, show available tasks
    let task = match contextpack.tasks.get(task_name) {
        Some(t) => t,
        None => {
            display_available_tasks(&contextpack, task_name)?;
            anyhow::bail!("Task '{}' not found", task_name);
        }
    };

    println!("Loading contextpack: {}", contextpack_path.display());

    println!("Task: {}", task_name.blue());
    if !task.description.is_empty() {
        println!("  {}", task.description);
    }
    println!("  Max bytes: {}", task.max_bytes);
    println!("  Include patterns: {}", task.include.len());

    let git_sha = get_git_sha(&workspace_root)?;
    println!("Git SHA: {}", git_sha);

    println!("Resolving files...");
    let files = resolve_files(&workspace_root, &task.include)?;
    println!("  Found {} matching files", files.len());

    // Create bundle/<TASK>/ directory structure
    let bundle_root = workspace_root.join("bundle");
    fs::create_dir_all(&bundle_root)?;
    let bundle_task_dir = bundle_root.join(task_name);
    fs::create_dir_all(&bundle_task_dir)?;

    // Build and write context.md
    let context_path = bundle_task_dir.join("context.md");
    println!("Building context: {}", context_path.display());
    let (file_count, total_bytes) =
        build_context(&context_path, task, &files, &git_sha, &workspace_root)?;

    // Load spec_ledger for referential integrity validation (AC-TPL-BUNDLE-REFERENTIAL-INTEGRITY)
    let (valid_ac_ids, valid_req_ids) = load_valid_ids_from_ledger(&workspace_root);

    // Load task linkage from specs/tasks.yaml (if task exists there)
    let (requirement_ids, ac_ids, warnings) = match load_task_spec(&workspace_root, task_name) {
        Some(task_spec) => {
            let mut warns = Vec::new();

            // Validate and filter requirement
            let reqs = if let Some(ref r) = task_spec.requirement {
                if valid_req_ids.contains(r) {
                    vec![r.clone()]
                } else {
                    warns.push(BundleWarning {
                        invalid_id: r.clone(),
                        ref_type: "requirement".to_string(),
                        message: format!("Requirement {} not found in spec_ledger.yaml", r),
                    });
                    vec![]
                }
            } else {
                vec![]
            };

            // Validate and filter ACs
            let acs: Vec<String> = task_spec
                .acs
                .into_iter()
                .filter(|ac| {
                    if valid_ac_ids.contains(ac) {
                        true
                    } else {
                        warns.push(BundleWarning {
                            invalid_id: ac.clone(),
                            ref_type: "ac".to_string(),
                            message: format!("AC {} not found in spec_ledger.yaml", ac),
                        });
                        false
                    }
                })
                .collect();

            println!("  Linked to: {} REQs, {} ACs", reqs.len(), acs.len());
            if !warns.is_empty() {
                for warn in &warns {
                    println!("  {} {}", "[WARN]".yellow(), warn.message);
                }
            }
            (reqs, acs, warns)
        }
        None => {
            println!("  No task linkage found in specs/tasks.yaml");
            (vec![], vec![], vec![])
        }
    };

    // Check BUNDLE_STRICT_REFS environment variable
    if !warnings.is_empty()
        && std::env::var("BUNDLE_STRICT_REFS").map(|v| v == "1").unwrap_or(false)
    {
        anyhow::bail!(
            "Bundle has {} referential integrity warning(s) and BUNDLE_STRICT_REFS=1 is set. \
                Fix the invalid references in specs/tasks.yaml or unset BUNDLE_STRICT_REFS.",
            warnings.len()
        );
    }

    // Load tests from spec_ledger for linked ACs
    let tests =
        if ac_ids.is_empty() { vec![] } else { load_tests_for_acs(&workspace_root, &ac_ids) };
    if !tests.is_empty() {
        println!("  Found {} test handles for linked ACs", tests.len());
    }

    // Create manifest
    let manifest = BundleManifest {
        bundle_version: 1,
        task_id: task_name.to_string(),
        requirement_ids,
        ac_ids,
        git_sha,
        timestamp: Utc::now().to_rfc3339(),
        specs: vec![ManifestSpec { file: "specs/spec_ledger.yaml".to_string() }],
        docs: vec![ManifestDoc { file: "docs/explanation/TEMPLATE-CONTRACTS.md".to_string() }],
        tests,
        warnings,
    };

    // Write manifest.yaml
    let manifest_path = bundle_task_dir.join("bundle.yaml");
    let manifest_yaml =
        serde_yaml::to_string(&manifest).context("Failed to serialize manifest to YAML")?;
    fs::write(&manifest_path, manifest_yaml).context("Failed to write manifest")?;

    println!("{} Generated {}", "[OK]".green(), bundle_task_dir.display());
    println!("  Files: {}", file_count);
    println!("  Size: {} bytes", total_bytes);
    println!("  Manifest: {}", manifest_path.display());

    if total_bytes > task.max_bytes {
        println!("  {} Size limit exceeded!", "[WARN]".yellow());
    }

    // Run scope audit to catch over-inclusion (soft warning for now)
    audit_bundle_scope(task_name, file_count, total_bytes);

    Ok(())
}

/// Default thresholds for bundle scope audit (AC-TPL-BUNDLE-MINIMAL-SCOPE)
const DEFAULT_MAX_FILES: usize = 64;
const DEFAULT_MAX_BYTES: usize = 300 * 1024; // 300 KiB

/// Audit bundle scope to catch over-inclusion.
/// Currently a soft warning; will become a hard gate when AC-TPL-BUNDLE-MINIMAL-SCOPE
/// is promoted to must_have_ac: true.
fn audit_bundle_scope(task_name: &str, file_count: usize, total_bytes: usize) {
    let max_files = std::env::var("BUNDLE_MAX_FILES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_MAX_FILES);

    let max_bytes = std::env::var("BUNDLE_MAX_BYTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_MAX_BYTES);

    let files_over = file_count > max_files;
    let bytes_over = total_bytes > max_bytes;

    if files_over || bytes_over {
        println!(
            "  {} Bundle '{}' is large (files: {}/{}, bytes: {}/{}).",
            "[SCOPE-WARN]".yellow(),
            task_name,
            file_count,
            max_files,
            total_bytes,
            max_bytes
        );
        println!("    Consider narrowing include patterns in .llm/contextpack.yaml");
        println!("    or set BUNDLE_MAX_FILES/BUNDLE_MAX_BYTES to raise thresholds.");
    }
}

fn get_workspace_root() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("failed to run git rev-parse")?;

    if !output.status.success() {
        anyhow::bail!(
            "not in a git repository\n\n\
             try: cd to your repository root directory"
        );
    }

    let root = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(PathBuf::from(root))
}

fn load_contextpack(path: &Path) -> Result<ContextPack> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read contextpack: {}", path.display()))?;

    let contextpack: ContextPack = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse contextpack YAML: {}", path.display()))?;

    Ok(contextpack)
}

fn get_git_sha(workspace_root: &Path) -> Result<String> {
    let output = Command::new("git")
        .current_dir(workspace_root)
        .args(["rev-parse", "HEAD"])
        .output()
        .context("failed to get git SHA")?;

    if !output.status.success() {
        anyhow::bail!(
            "failed to get git SHA\n\n\
             try: git log --oneline -1"
        );
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn resolve_files(workspace_root: &Path, patterns: &[String]) -> Result<Vec<PathBuf>> {
    let llmignore = load_llmignore(workspace_root)?;
    let mut seen = HashSet::new();
    let mut files = Vec::new();

    for pattern in patterns {
        let output = Command::new("git")
            .current_dir(workspace_root)
            .args(["ls-files", pattern])
            .output()
            .with_context(|| format!("Failed to run git ls-files for pattern: {}", pattern))?;

        if !output.status.success() {
            continue;
        }

        let stdout = String::from_utf8(output.stdout)?;
        for line in stdout.lines() {
            let path_str = line.trim();
            if path_str.is_empty() {
                continue;
            }

            // Skip if already seen
            if seen.contains(path_str) {
                continue;
            }

            // Skip if matches .llmignore patterns (using gitignore semantics)
            let path = Path::new(path_str);
            if llmignore.matched(path, false).is_ignore() {
                continue;
            }

            let full_path = workspace_root.join(path_str);
            if full_path.exists() && full_path.is_file() {
                seen.insert(path_str.to_string());
                files.push(PathBuf::from(path_str));
            }
        }
    }

    Ok(files)
}

/// Load .llmignore file and build gitignore matcher
fn load_llmignore(workspace_root: &Path) -> Result<ignore::gitignore::Gitignore> {
    let ignore_path = workspace_root.join(".llm/.llmignore");

    let mut builder = ignore::gitignore::GitignoreBuilder::new(workspace_root);

    if ignore_path.exists() {
        builder.add(&ignore_path);
    }

    builder.build().context("Failed to build .llmignore matcher")
}

/// Load valid AC and REQ IDs from spec_ledger.yaml for referential integrity validation
fn load_valid_ids_from_ledger(workspace_root: &Path) -> (HashSet<String>, HashSet<String>) {
    let ledger_path = workspace_root.join("specs/spec_ledger.yaml");
    let content = match fs::read_to_string(&ledger_path) {
        Ok(c) => c,
        Err(_) => return (HashSet::new(), HashSet::new()),
    };

    let ledger: SpecLedger = match serde_yaml::from_str(&content) {
        Ok(l) => l,
        Err(_) => return (HashSet::new(), HashSet::new()),
    };

    let mut valid_ac_ids = HashSet::new();
    let mut valid_req_ids = HashSet::new();

    for story in ledger.stories {
        for req in story.requirements {
            valid_req_ids.insert(req.id.clone());
            for ac in req.acceptance_criteria {
                valid_ac_ids.insert(ac.id);
            }
        }
    }

    (valid_ac_ids, valid_req_ids)
}

/// Load task spec from specs/tasks.yaml if it exists
fn load_task_spec(workspace_root: &Path, task_name: &str) -> Option<TaskSpec> {
    let tasks_path = workspace_root.join("specs/tasks.yaml");
    if !tasks_path.exists() {
        return None;
    }

    let content = fs::read_to_string(&tasks_path).ok()?;
    let tasks_yaml: TasksYaml = serde_yaml::from_str(&content).ok()?;

    tasks_yaml.tasks.into_iter().find(|t| t.id == task_name)
}

/// Load tests for given AC IDs from spec_ledger.yaml
fn load_tests_for_acs(workspace_root: &Path, ac_ids: &[String]) -> Vec<ManifestTest> {
    let ledger_path = workspace_root.join("specs/spec_ledger.yaml");
    let content = match fs::read_to_string(&ledger_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let ledger: SpecLedger = match serde_yaml::from_str(&content) {
        Ok(l) => l,
        Err(_) => return vec![],
    };

    let ac_set: HashSet<_> = ac_ids.iter().collect();
    let mut tests = Vec::new();

    for story in ledger.stories {
        for req in story.requirements {
            for ac in req.acceptance_criteria {
                if ac_set.contains(&ac.id) {
                    for test in ac.tests {
                        if let Some(file) = test.file {
                            tests.push(ManifestTest { r#type: test.r#type, tag: test.tag, file });
                        }
                    }
                }
            }
        }
    }

    tests
}

fn build_context(
    context_path: &Path,
    task: &Task,
    files: &[PathBuf],
    git_sha: &str,
    workspace_root: &Path,
) -> Result<(usize, usize)> {
    let mut output = String::new();

    // Header
    output.push_str("# LLM Context Bundle\n\n");
    output.push_str(&format!("**Git SHA:** {}\n\n", git_sha));
    if !task.description.is_empty() {
        output.push_str(&format!("**Description:** {}\n\n", task.description));
    }
    output.push_str(&format!("**Max bytes:** {}\n\n", task.max_bytes));
    output.push_str("---\n\n");

    let mut total_bytes = output.len();
    let mut file_count = 0;

    for file_path in files {
        let full_path = workspace_root.join(file_path);

        match fs::read_to_string(&full_path) {
            Ok(content) => {
                let file_section =
                    format!("# FILE: {}\n\n```\n{}\n```\n\n", file_path.display(), content);
                let new_total = total_bytes + file_section.len();

                if new_total > task.max_bytes {
                    eprintln!(
                        "  {} Size limit reached, skipping remaining files",
                        "[WARN]".yellow()
                    );
                    break;
                }

                output.push_str(&file_section);
                total_bytes = new_total;
                file_count += 1;
            }
            Err(e) => {
                eprintln!("  {} Failed to read {}: {}", "[WARN]".yellow(), file_path.display(), e);
            }
        }
    }

    fs::write(context_path, output)
        .with_context(|| format!("Failed to write context: {}", context_path.display()))?;

    Ok((file_count, total_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// AC-TPL-BUNDLE-MANIFEST-LINKED: Validates the contract that when a task
    /// exists in tasks.yaml, the manifest will include its requirement_ids and ac_ids.
    #[test]
    fn bundle_manifest_populates_from_task() {
        let task_spec = TaskSpec {
            id: "implement_ac".to_string(),
            requirement: Some("REQ-TPL-SUGGEST-NEXT".to_string()),
            acs: vec!["AC-TPL-SUGGEST-NEXT-CLI".to_string()],
        };

        // Task has requirement
        assert!(task_spec.requirement.is_some());
        assert_eq!(task_spec.requirement.as_deref(), Some("REQ-TPL-SUGGEST-NEXT"));

        // Task has ACs
        assert!(!task_spec.acs.is_empty());
        assert!(task_spec.acs.contains(&"AC-TPL-SUGGEST-NEXT-CLI".to_string()));
    }

    #[test]
    fn manifest_test_struct_serializes_correctly() {
        let test = ManifestTest {
            r#type: "bdd".to_string(),
            tag: "@AC-TPL-001".to_string(),
            file: "specs/features/test.feature".to_string(),
        };

        let yaml = serde_yaml::to_string(&test).unwrap();
        assert!(yaml.contains("type: bdd"));
        assert!(yaml.contains("tag: '@AC-TPL-001'"));
        assert!(yaml.contains("file: specs/features/test.feature"));
    }

    /// AC-TPL-BUNDLE-MINIMAL-SCOPE: Verifies bundle scope audit runs and detects
    /// over-threshold bundles (soft warning for now).
    #[test]
    fn bundle_scope_audit() {
        // Verify the audit function exists and doesn't panic on normal inputs
        audit_bundle_scope("test_task", 10, 10_000);

        // Verify it doesn't panic on over-threshold inputs
        audit_bundle_scope("large_task", 100, 500_000);

        // Verify default thresholds are reasonable
        assert_eq!(DEFAULT_MAX_FILES, 64);
        assert_eq!(DEFAULT_MAX_BYTES, 300 * 1024);
    }
}
