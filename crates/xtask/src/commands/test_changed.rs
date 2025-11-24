//! Smart test runner that only runs tests affected by git changes.
//!
//! Algorithm:
//! 1. Get changed files via `git diff --name-only <base>...HEAD`
//! 2. Classify files by prefix and determine test scope
//! 3. Build and execute test plan
//! 4. Report results

use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;

use super::ac_parsing::parse_features_with_metadata;

/// Arguments for test-changed command
#[derive(Debug, Clone)]
pub struct TestChangedArgs {
    /// Git ref to compare against (default: origin/main)
    pub base: String,
}

impl Default for TestChangedArgs {
    fn default() -> Self {
        Self { base: "origin/main".to_string() }
    }
}

/// Represents a test command to execute
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TestCommand {
    /// Run cargo test for a specific package
    CargoTest { package: String, description: String },
    /// Run BDD tests for specific AC tags
    BddWithTags { ac_tags: Vec<String>, description: String },
    /// Run all BDD tests
    BddAll,
    /// Run docs-check command
    DocsCheck,
    /// Run graph invariants tests
    GraphInvariants,
}

/// Test plan builder
struct TestPlan {
    commands: Vec<TestCommand>,
}

impl TestPlan {
    fn new() -> Self {
        Self { commands: Vec::new() }
    }

    fn add_cargo_test(&mut self, package: &str, description: &str) {
        self.commands.push(TestCommand::CargoTest {
            package: package.to_string(),
            description: description.to_string(),
        });
    }

    fn add_bdd_with_tags(&mut self, ac_tags: Vec<String>, description: &str) {
        if !ac_tags.is_empty() {
            self.commands
                .push(TestCommand::BddWithTags { ac_tags, description: description.to_string() });
        }
    }

    fn add_bdd_all(&mut self) {
        self.commands.push(TestCommand::BddAll);
    }

    fn add_docs_check(&mut self) {
        self.commands.push(TestCommand::DocsCheck);
    }

    fn add_graph_invariants(&mut self) {
        self.commands.push(TestCommand::GraphInvariants);
    }

    fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    fn deduplicate(&mut self) {
        // Use HashSet to remove duplicates while preserving order
        let mut seen = HashSet::new();
        self.commands.retain(|cmd| seen.insert(cmd.clone()));

        // If we have BddAll, remove all BddWithTags commands
        if self.commands.iter().any(|c| matches!(c, TestCommand::BddAll)) {
            self.commands.retain(|c| !matches!(c, TestCommand::BddWithTags { .. }));
        }
    }
}

/// Get list of changed files from git
fn get_changed_files(base: &str) -> Result<Vec<String>> {
    let mut files = HashSet::new();

    // 1) Changes between base and HEAD (committed diff)
    let history_diff =
        Command::new("git").args(["diff", "--name-only", &format!("{}...HEAD", base)]).output()?;
    if !history_diff.status.success() {
        let stderr = String::from_utf8_lossy(&history_diff.stderr);
        anyhow::bail!("git diff (history) failed: {}", stderr.trim());
    }

    // 2) Staged changes vs HEAD
    let staged_diff = Command::new("git").args(["diff", "--name-only", "--cached"]).output()?;
    if !staged_diff.status.success() {
        let stderr = String::from_utf8_lossy(&staged_diff.stderr);
        anyhow::bail!("git diff (staged) failed: {}", stderr.trim());
    }

    // 3) Unstaged working tree changes vs HEAD
    let worktree_diff = Command::new("git").args(["diff", "--name-only"]).output()?;
    if !worktree_diff.status.success() {
        let stderr = String::from_utf8_lossy(&worktree_diff.stderr);
        anyhow::bail!("git diff (worktree) failed: {}", stderr.trim());
    }

    // 4) Newly created, untracked files
    let untracked = Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard"])
        .output()
        .context("Failed to list untracked files")?;
    if !untracked.status.success() {
        let stderr = String::from_utf8_lossy(&untracked.stderr);
        anyhow::bail!("git ls-files failed: {}", stderr.trim());
    }

    for output in [&history_diff, &staged_diff, &worktree_diff, &untracked] {
        for line in
            String::from_utf8_lossy(&output.stdout).lines().map(str::trim).filter(|l| !l.is_empty())
        {
            files.insert(line.to_string());
        }
    }

    let mut files: Vec<_> = files.into_iter().collect();
    files.sort();

    Ok(files)
}

/// Extract AC tags from changed feature files
fn extract_ac_tags_from_features(changed_features: &[String]) -> Result<Vec<String>> {
    let mut ac_tags = HashSet::new();

    // Parse all feature files to get metadata
    let features_dir = PathBuf::from("specs/features");
    if !features_dir.exists() {
        return Ok(Vec::new());
    }

    let scenarios = parse_features_with_metadata(&features_dir)?;

    // For each changed feature file, extract AC tags from scenarios in that file
    for feature_file in changed_features {
        for scenario in scenarios.values() {
            // Check if this scenario is in the changed file
            if scenario.file.contains(feature_file) || feature_file.contains(&scenario.file) {
                ac_tags.insert(scenario.ac_id.clone());
            }
        }
    }

    Ok(ac_tags.into_iter().collect())
}

/// Build test plan based on changed files
fn build_test_plan(changed_files: Vec<String>) -> Result<TestPlan> {
    let mut plan = TestPlan::new();

    if changed_files.is_empty() {
        return Ok(plan);
    }

    // Track file types
    let mut has_spec_ledger = false;
    let mut has_feature_files = Vec::new();
    let mut has_xtask = false;
    let mut has_app_http = false;
    let mut has_spec_runtime = false;
    let mut has_business_core = false;
    let mut has_acceptance = false;
    let mut only_docs = true;

    for file in &changed_files {
        if file == "specs/spec_ledger.yaml" {
            has_spec_ledger = true;
            only_docs = false;
        } else if file.starts_with("specs/features/") && file.ends_with(".feature") {
            has_feature_files.push(file.clone());
            only_docs = false;
        } else if file.starts_with("crates/xtask/") {
            has_xtask = true;
            only_docs = false;
        } else if file.starts_with("crates/app-http/") {
            has_app_http = true;
            only_docs = false;
        } else if file.starts_with("crates/spec-runtime/") {
            has_spec_runtime = true;
            only_docs = false;
        } else if file.starts_with("crates/business-core/") {
            has_business_core = true;
            only_docs = false;
        } else if file.starts_with("crates/acceptance/") {
            has_acceptance = true;
            only_docs = false;
        } else if !file.starts_with("docs/") {
            // Something other than docs changed
            only_docs = false;
        }
    }

    // Build test plan based on changes
    if only_docs {
        // Only docs changed - run docs-check if it exists
        plan.add_docs_check();
    } else {
        // Handle spec_ledger changes
        if has_spec_ledger {
            // If spec ledger changed, we might need to run all BDD tests
            // For now, we'll run all BDD since we can't easily determine which ACs changed
            plan.add_bdd_all();
        }

        // Handle feature file changes
        if !has_feature_files.is_empty() {
            let ac_tags = extract_ac_tags_from_features(&has_feature_files)?;
            if !ac_tags.is_empty() {
                plan.add_bdd_with_tags(ac_tags, "Changed feature files");
            }
        }

        // Handle crate changes
        if has_xtask {
            plan.add_cargo_test("xtask", "xtask crate changes");
            // Also run xtask_devex BDD scenarios
            plan.add_bdd_with_tags(vec!["@AC-PLT-018".to_string()], "xtask devex contract");
        }

        if has_app_http {
            plan.add_cargo_test("app-http", "app-http crate changes");
            // Run platform UI/API BDD scenarios
            // These would be tagged with specific ACs - for now just note the pattern
        }

        if has_spec_runtime {
            plan.add_cargo_test("spec-runtime", "spec-runtime crate changes");
            plan.add_graph_invariants();
        }

        if has_business_core {
            plan.add_cargo_test("business-core", "business-core crate changes");
        }

        if has_acceptance {
            // If acceptance tests themselves changed, run all BDD
            plan.add_bdd_all();
        }
    }

    // Remove duplicates
    plan.deduplicate();

    Ok(plan)
}

/// Execute a test command
fn execute_test_command(cmd: &TestCommand) -> Result<bool> {
    let run_icon = "[run]".cyan();
    let ok_icon = "[ok]".green();
    let fail_icon = "[fail]".red();

    match cmd {
        TestCommand::CargoTest { package, description } => {
            println!("  {} cargo test -p {}", run_icon, package);
            let output = crate::cargo_cmd("test", &["-p", package]).output()?;

            if output.status.success() {
                println!("    {} {}", ok_icon, description);
                Ok(true)
            } else {
                println!("    {} {}", fail_icon, description);
                // Show failure details
                if !output.stdout.is_empty() {
                    eprintln!("{}", String::from_utf8_lossy(&output.stdout));
                }
                if !output.stderr.is_empty() {
                    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                }
                Ok(false)
            }
        }
        TestCommand::BddWithTags { ac_tags, description } => {
            let tags_arg = ac_tags.join(" or ");
            println!(
                "  {} cargo test -p acceptance --test acceptance -- --tags \"{}\"",
                run_icon, tags_arg
            );

            let output = crate::cargo_cmd(
                "test",
                &["-p", "acceptance", "--test", "acceptance", "--", "--tags", &tags_arg],
            )
            .output()?;

            if output.status.success() {
                println!("    {} {}", ok_icon, description);
                Ok(true)
            } else {
                println!("    {} {}", fail_icon, description);
                if !output.stdout.is_empty() {
                    eprintln!("{}", String::from_utf8_lossy(&output.stdout));
                }
                if !output.stderr.is_empty() {
                    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                }
                Ok(false)
            }
        }
        TestCommand::BddAll => {
            println!("  {} cargo test -p acceptance --test acceptance", run_icon);
            let output =
                crate::cargo_cmd("test", &["-p", "acceptance", "--test", "acceptance"]).output()?;

            if output.status.success() {
                println!("    {} All BDD scenarios", ok_icon);
                Ok(true)
            } else {
                println!("    {} BDD scenarios failed", fail_icon);
                if !output.stdout.is_empty() {
                    eprintln!("{}", String::from_utf8_lossy(&output.stdout));
                }
                if !output.stderr.is_empty() {
                    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                }
                Ok(false)
            }
        }
        TestCommand::DocsCheck => {
            println!("  {} cargo xtask docs-check", run_icon);
            let output =
                Command::new("cargo").args(["run", "-p", "xtask", "--", "docs-check"]).output()?;

            if output.status.success() {
                println!("    {} Documentation checks", ok_icon);
                Ok(true)
            } else {
                println!("    {} Documentation checks failed", fail_icon);
                if !output.stdout.is_empty() {
                    eprintln!("{}", String::from_utf8_lossy(&output.stdout));
                }
                if !output.stderr.is_empty() {
                    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                }
                Ok(false)
            }
        }
        TestCommand::GraphInvariants => {
            println!("  {} cargo xtask graph-export --check", run_icon);
            let output = Command::new("cargo")
                .args(["run", "-p", "xtask", "--", "graph-export", "--check"])
                .output()?;

            if output.status.success() {
                println!("    {} Graph invariants", ok_icon);
                Ok(true)
            } else {
                println!("    {} Graph invariants failed", fail_icon);
                if !output.stdout.is_empty() {
                    eprintln!("{}", String::from_utf8_lossy(&output.stdout));
                }
                if !output.stderr.is_empty() {
                    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                }
                Ok(false)
            }
        }
    }
}

/// Run test-changed command
pub fn run(args: TestChangedArgs) -> Result<()> {
    println!("{}", "Analyzing changed files...".bold());

    // Get changed files
    let changed_files = get_changed_files(&args.base).context("Failed to get changed files")?;

    if changed_files.is_empty() {
        println!("{}", "No changes detected - no tests needed.".green());
        return Ok(());
    }

    println!("\nChanged files (vs {}):", args.base.cyan());
    for file in &changed_files {
        println!("  - {}", file);
    }

    // Build test plan
    let plan = build_test_plan(changed_files)?;

    if plan.is_empty() {
        println!("\n{}", "No tests needed for these changes.".green());
        return Ok(());
    }

    // Display test plan
    println!("\n{}", "Test plan:".bold());
    for (i, cmd) in plan.commands.iter().enumerate() {
        match cmd {
            TestCommand::CargoTest { package, description } => {
                println!("  {}. Run unit tests: {} ({})", i + 1, package, description);
            }
            TestCommand::BddWithTags { ac_tags, description } => {
                println!("  {}. Run BDD: {} ({})", i + 1, ac_tags.join(", "), description);
            }
            TestCommand::BddAll => {
                println!("  {}. Run all BDD scenarios", i + 1);
            }
            TestCommand::DocsCheck => {
                println!("  {}. Run documentation checks", i + 1);
            }
            TestCommand::GraphInvariants => {
                println!("  {}. Validate graph invariants", i + 1);
            }
        }
    }

    // Execute test plan
    println!("\n{}", "Executing tests:".bold());
    let mut all_passed = true;

    for cmd in &plan.commands {
        match execute_test_command(cmd) {
            Ok(passed) => {
                if !passed {
                    all_passed = false;
                }
            }
            Err(e) => {
                eprintln!("    {} Failed to execute test: {}", "[fail]".red(), e);
                all_passed = false;
            }
        }
    }

    println!();
    if all_passed {
        println!("{}", "[ok] All tests passed".green().bold());
        Ok(())
    } else {
        anyhow::bail!("Some tests failed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs, path::Path};
    use tempfile::TempDir;

    struct CwdGuard(std::path::PathBuf);

    impl CwdGuard {
        fn new(path: &Path) -> Self {
            let prev = env::current_dir().expect("current_dir");
            env::set_current_dir(path).expect("set_current_dir");
            Self(prev)
        }
    }

    impl Drop for CwdGuard {
        fn drop(&mut self) {
            let _ = env::set_current_dir(&self.0);
        }
    }

    fn git(repo: &Path, args: &[&str]) {
        let status =
            Command::new("git").current_dir(repo).args(args).status().expect("run git command");
        assert!(status.success(), "git {:?} failed", args);
    }

    #[test]
    fn collects_staged_worktree_and_untracked_changes() {
        let repo = TempDir::new().expect("tempdir");
        git(repo.path(), &["init", "-q"]);
        git(repo.path(), &["config", "user.email", "ci@example.com"]);
        git(repo.path(), &["config", "user.name", "CI"]);

        fs::write(repo.path().join("tracked.txt"), "base").expect("write tracked");
        git(repo.path(), &["add", "tracked.txt"]);
        git(repo.path(), &["commit", "-m", "init", "-q"]);

        // Modify tracked file (unstaged)
        fs::write(repo.path().join("tracked.txt"), "changed").expect("write tracked updated");
        // Add staged file
        fs::write(repo.path().join("staged.txt"), "staged").expect("write staged");
        git(repo.path(), &["add", "staged.txt"]);
        // Add untracked file
        fs::write(repo.path().join("untracked.txt"), "untracked").expect("write untracked");

        let _guard = CwdGuard::new(repo.path());
        let files = get_changed_files("HEAD").expect("get_changed_files");

        assert!(files.contains(&"tracked.txt".to_string()));
        assert!(files.contains(&"staged.txt".to_string()));
        assert!(files.contains(&"untracked.txt".to_string()));
    }
}
