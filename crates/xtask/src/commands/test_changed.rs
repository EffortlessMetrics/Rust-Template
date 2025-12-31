//! Smart test runner that only runs tests affected by git changes.
//!
//! Algorithm:
//! 1. Get changed files via `git diff --name-only <base>...HEAD`
//! 2. Classify files by prefix and determine test scope
//! 3. Build and execute test plan
//! 4. Report results

use anyhow::{Context, Result};
use colored::Colorize;
use std::{collections::HashSet, env, fs, path::PathBuf, process::Command};

use super::ac_parsing::{AC_PATTERN_WITH_AT, parse_features_with_metadata};

/// Arguments for test-changed command
#[derive(Debug, Clone)]
pub struct TestChangedArgs {
    /// Git ref to compare against (default: origin/main)
    pub base: String,
    /// Whether to run in plan-only mode (do not execute tests)
    pub plan_only: bool,
}

impl Default for TestChangedArgs {
    fn default() -> Self {
        Self { base: "origin/main".to_string(), plan_only: false }
    }
}

/// BDD execution plan derived from changed files
#[derive(Debug, Clone)]
pub enum BddPlan {
    /// No BDD needed for these changes
    None { reason: String },
    /// Run a subset of scenarios filtered by AC tags
    Tags { ac_tags: Vec<String>, reason: String },
    /// Run the full BDD suite
    All { reason: String },
}

fn canonical_tag(tag: &str) -> String {
    format!("@{}", tag.trim_start_matches('@'))
}

/// Render a tag expression in canonical @AC-... form for Cucumber
pub fn format_tag_expression(tags: &[String]) -> String {
    tags.iter().map(|t| canonical_tag(t)).collect::<Vec<_>>().join(" or ")
}

/// Represents a test command to execute
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TestCommand {
    /// Run cargo test for a specific package
    CargoTest { package: String, description: String },
    /// Run BDD tests for specific AC tags
    BddWithTags { ac_tags: Vec<String>, description: String },
    /// Run all BDD tests
    BddAll { description: String },
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

    fn add_bdd_all(&mut self, description: &str) {
        self.commands.push(TestCommand::BddAll { description: description.to_string() });
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
        if self.commands.iter().any(|c| matches!(c, TestCommand::BddAll { .. })) {
            self.commands.retain(|c| !matches!(c, TestCommand::BddWithTags { .. }));
        }
    }
}

/// Get list of changed files from git
pub fn get_changed_files(base: &str) -> Result<(String, Vec<String>)> {
    let base_ref = resolve_base_ref(base);
    let mut files = HashSet::new();

    // 1) Changes between base and HEAD (committed diff)
    let history_diff = Command::new("git")
        .args(["diff", "--name-only", &format!("{}...HEAD", base_ref)])
        .output()?;
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
            files.insert(normalize_path(line));
        }
    }

    let mut files: Vec<_> = files.into_iter().collect();
    files.sort();

    Ok((base_ref, files))
}

fn resolve_base_ref(base: &str) -> String {
    let mut candidates = Vec::new();
    if !base.is_empty() {
        candidates.push(base.to_string());
    }

    for fallback in ["origin/main", "main", "master", "HEAD"] {
        if base != fallback {
            candidates.push(fallback.to_string());
        }
    }

    for candidate in candidates {
        if let Ok(output) = Command::new("git").args(["rev-parse", "--verify", &candidate]).output()
            && output.status.success()
        {
            return candidate;
        }
    }

    base.to_string()
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/").trim_start_matches("./").to_string()
}

fn plan_only_mode() -> bool {
    match std::env::var("XTASK_TEST_CHANGED_PLAN_ONLY") {
        Ok(val) => {
            let v = val.trim();
            !(v.is_empty()
                || v == "0"
                || v.eq_ignore_ascii_case("false")
                || v.eq_ignore_ascii_case("no"))
        }
        Err(_) => false,
    }
}

/// Extract AC tags from changed feature files
fn extract_ac_tags_from_features(changed_features: &[String]) -> Result<Vec<String>> {
    use crate::kernel::layout_for_repo;

    let mut ac_tags = HashSet::new();

    // Parse all feature files to get metadata
    let features_dir = layout_for_repo().features_dir;
    if !features_dir.exists() {
        return Ok(Vec::new());
    }

    let scenarios = parse_features_with_metadata(&features_dir)?;

    // For each changed feature file, extract AC tags from scenarios in that file
    for feature_file in changed_features {
        let feature_path = PathBuf::from(feature_file);
        let normalized = normalize_path(feature_file);
        let normalized_rel = feature_path
            .strip_prefix(features_dir.parent().unwrap_or(&features_dir))
            .map(|p| normalize_path(&p.to_string_lossy()))
            .unwrap_or_else(|_| normalized.clone());

        for scenario in scenarios.values() {
            let scenario_path = normalize_path(&scenario.file);
            if scenario_path == normalized
                || scenario_path == normalized_rel
                || normalized.ends_with(&scenario_path)
                || scenario_path.ends_with(&normalized)
            {
                ac_tags.insert(scenario.ac_id.clone());
            }
        }

        // Fallback: parse AC tags directly from the feature file so we don't miss newly-added tags
        if let Ok(content) = fs::read_to_string(&feature_path) {
            for caps in AC_PATTERN_WITH_AT.captures_iter(&content) {
                if let Some(ac) = caps.get(1) {
                    ac_tags.insert(ac.as_str().to_string());
                }
            }
        }
    }

    let mut acs: Vec<_> = ac_tags.into_iter().collect();
    acs.sort();
    Ok(acs)
}

/// Derive the BDD execution plan from changed files
pub fn bdd_plan_from_changes(changed_files: &[String]) -> Result<BddPlan> {
    if changed_files.is_empty() {
        return Ok(BddPlan::None { reason: "No changes detected".to_string() });
    }

    if changed_files.iter().any(|f| f == "specs/spec_ledger.yaml") {
        return Ok(BddPlan::All { reason: "spec_ledger.yaml changed".to_string() });
    }

    if changed_files.iter().any(|f| f.starts_with("crates/acceptance/")) {
        return Ok(BddPlan::All { reason: "acceptance harness changed".to_string() });
    }

    let feature_files: Vec<String> = changed_files
        .iter()
        .filter(|f| f.starts_with("specs/features/") && f.ends_with(".feature"))
        .cloned()
        .collect();

    if !feature_files.is_empty() {
        let ac_tags = extract_ac_tags_from_features(&feature_files)?;
        if !ac_tags.is_empty() {
            return Ok(BddPlan::Tags { ac_tags, reason: "feature files changed".to_string() });
        }
    }

    Ok(BddPlan::None { reason: "No BDD-relevant changes detected".to_string() })
}

/// Build test plan based on changed files
fn build_test_plan(changed_files: Vec<String>, bdd_plan: &BddPlan) -> Result<TestPlan> {
    let mut plan = TestPlan::new();

    if changed_files.is_empty() {
        return Ok(plan);
    }

    // Track file types
    let mut has_xtask = false;
    let mut has_app_http = false;
    let mut has_spec_runtime = false;
    let mut has_business_core = false;
    let mut has_acceptance = false;
    let mut only_docs = true;

    for file in &changed_files {
        if file == "specs/spec_ledger.yaml"
            || (file.starts_with("specs/features/") && file.ends_with(".feature"))
        {
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
        // Handle BDD coverage
        match bdd_plan {
            BddPlan::All { reason } => plan.add_bdd_all(reason),
            BddPlan::Tags { ac_tags, reason } => plan.add_bdd_with_tags(ac_tags.clone(), reason),
            BddPlan::None { .. } => {}
        }

        // Handle crate changes
        if has_xtask {
            plan.add_cargo_test("xtask", "xtask crate changes");
            // Also run xtask_devex BDD scenarios
            plan.add_bdd_with_tags(vec!["AC-PLT-018".to_string()], "xtask devex contract");
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
            plan.add_bdd_all("Acceptance harness changed");
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
    let plan_only = plan_only_mode();

    match cmd {
        TestCommand::CargoTest { package, description } => {
            println!("  {} cargo test -p {}", run_icon, package);
            if plan_only {
                println!("    {} {} (plan-only)", ok_icon, description);
                return Ok(true);
            }

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
            let tag_expr = format_tag_expression(ac_tags);
            println!(
                "  {} cargo test -p acceptance --test acceptance (tags: \"{}\")",
                run_icon, tag_expr
            );

            if plan_only {
                println!(
                    "    {} {} (plan-only, CUCUMBER_TAG_EXPRESSION=\"{}\")",
                    ok_icon, description, tag_expr
                );
                return Ok(true);
            }

            if crate::env::should_skip_bdd() {
                let reason = if crate::env::is_low_resources() {
                    "XTASK_LOW_RESOURCES=1"
                } else {
                    "XTASK_SKIP_BDD=1"
                };
                println!("    {} Skipping acceptance tests ({})", "[skip]".yellow(), reason);
                return Ok(true);
            }

            let mut cmd = crate::cargo_cmd("test", &["-p", "acceptance", "--test", "acceptance"]);
            cmd.env("CUCUMBER_TAG_EXPRESSION", &tag_expr);
            let output = cmd.output()?;

            // Use semantic BDD success detection (not just exit code)
            if crate::commands::bdd::is_bdd_success(&output) {
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
        TestCommand::BddAll { description } => {
            println!("  {} cargo test -p acceptance --test acceptance", run_icon);
            let mut cmd = crate::cargo_cmd("test", &["-p", "acceptance", "--test", "acceptance"]);
            cmd.env_remove("CUCUMBER_TAG_EXPRESSION");
            if plan_only {
                println!("    {} {} (plan-only)", ok_icon, description);
                return Ok(true);
            }
            if crate::env::should_skip_bdd() {
                let reason = if crate::env::is_low_resources() {
                    "XTASK_LOW_RESOURCES=1"
                } else {
                    "XTASK_SKIP_BDD=1"
                };
                println!("    {} Skipping acceptance tests ({})", "[skip]".yellow(), reason);
                return Ok(true);
            }
            let output = cmd.output()?;

            // Use semantic BDD success detection (not just exit code)
            if crate::commands::bdd::is_bdd_success(&output) {
                println!("    {} {}", ok_icon, description);
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
            if plan_only {
                println!("    {} Documentation checks (plan-only)", ok_icon);
                return Ok(true);
            }
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
            if plan_only {
                println!("    {} Graph invariants (plan-only)", ok_icon);
                return Ok(true);
            }
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

    let base =
        env::var("XTASK_CHANGED_BASE").ok().filter(|b| !b.trim().is_empty()).unwrap_or(args.base);

    // Get changed files
    let plan_only = args.plan_only || plan_only_mode();
    let (base_ref, changed_files) =
        get_changed_files(&base).context("Failed to get changed files")?;

    if changed_files.is_empty() {
        println!("{}", "No changes detected - no tests needed.".green());
        return Ok(());
    }

    println!("\nChanged files (vs {}):", base_ref.cyan());
    for file in &changed_files {
        println!("  - {}", file);
    }

    // Derive BDD plan from the changes
    let bdd_plan = bdd_plan_from_changes(&changed_files)?;

    // Build test plan
    let plan = build_test_plan(changed_files, &bdd_plan)?;

    // Capture resolved BDD tag expression (if any) for plan-only output
    let resolved_tag_expr = match &bdd_plan {
        BddPlan::Tags { ac_tags, .. } if !ac_tags.is_empty() => {
            Some(format_tag_expression(ac_tags))
        }
        _ => None,
    };

    if plan.is_empty() {
        println!("\n{}", "No tests needed for these changes.".green());
        if plan_only {
            println!(
                "\nPlan-only mode enabled (XTASK_TEST_CHANGED_PLAN_ONLY); commands will be reported \
                 but not executed."
            );
            if let Some(expr) = &resolved_tag_expr {
                println!("CUCUMBER_TAG_EXPRESSION=\"{}\"", expr);
            }
        }
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
                let tags = ac_tags.iter().map(|t| canonical_tag(t)).collect::<Vec<_>>().join(", ");
                println!("  {}. Run BDD: {} ({})", i + 1, tags, description);
            }
            TestCommand::BddAll { description } => {
                println!("  {}. Run all BDD scenarios ({})", i + 1, description);
            }
            TestCommand::DocsCheck => {
                println!("  {}. Run documentation checks", i + 1);
            }
            TestCommand::GraphInvariants => {
                println!("  {}. Validate graph invariants", i + 1);
            }
        }
    }

    if plan_only {
        println!(
            "\nPlan-only mode enabled (XTASK_TEST_CHANGED_PLAN_ONLY); commands will be reported \
             but not executed."
        );
        if let Some(expr) = &resolved_tag_expr {
            println!("Resolved BDD tag expression: {}", expr);
            println!("CUCUMBER_TAG_EXPRESSION=\"{}\"", expr);
        }
        return Ok(());
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
        let (_base, files) = get_changed_files("HEAD").expect("get_changed_files");

        assert!(files.contains(&"tracked.txt".to_string()));
        assert!(files.contains(&"staged.txt".to_string()));
        assert!(files.contains(&"untracked.txt".to_string()));
    }
}
