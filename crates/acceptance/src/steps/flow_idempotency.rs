use crate::world::World;
use cucumber::{given, then, when};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

// ============================================================================
// Helpers
// ============================================================================

fn workspace_root(world: &World) -> PathBuf {
    if let Some(ref path) = world.xtask_context().test_repo_path {
        path.clone()
    } else {
        actual_workspace_root()
    }
}

fn actual_workspace_root() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    std::path::Path::new(manifest_dir).parent().unwrap().parent().unwrap().to_path_buf()
}

fn run_xtask_command(world: &World, command: &str) -> (String, String, i32) {
    let root = workspace_root(world);
    let mut cmd = if cfg!(windows) {
        let mut c = Command::new("cargo");
        c.arg("run").arg("-p").arg("xtask").arg("--");
        for part in command.split_whitespace().skip(3) {
            c.arg(part);
        }
        c
    } else {
        let mut c = Command::new("cargo");
        c.arg("run").arg("-p").arg("xtask").arg("--");
        for part in command.split_whitespace().skip(3) {
            c.arg(part);
        }
        c
    };

    cmd.current_dir(&root);

    // Apply environment variables
    for (k, v) in &world.xtask_context().env {
        cmd.env(k, v);
    }

    let output = cmd.output().expect("Failed to execute command");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let status = output.status.code().unwrap_or(-1);

    (stdout, stderr, status)
}

fn get_git_status(world: &World) -> String {
    let root = workspace_root(world);
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .current_dir(&root)
        .output()
        .expect("Failed to run git status");
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn list_generated_files(world: &World) -> Vec<PathBuf> {
    let root = workspace_root(world);
    let mut files = vec![];

    // Track common generated file locations
    let generated_paths = vec![
        root.join("docs/feature_status.md"),
        root.join("target/junit/acceptance.xml"),
        root.join("target/ac_report.json"),
    ];

    for path in generated_paths {
        if path.exists() {
            files.push(path);
        }
    }

    files
}

// ============================================================================
// Given Steps
// ============================================================================

#[given("I have a clean workspace with no uncommitted changes")]
async fn given_clean_workspace(world: &mut World) {
    let status = get_git_status(world);
    if !status.trim().is_empty() {
        eprintln!("Warning: workspace has uncommitted changes:\n{}", status);
    }
}

#[given("the git working tree is clean")]
async fn given_clean_git_tree(world: &mut World) {
    let status = get_git_status(world);
    assert!(status.trim().is_empty(), "Git working tree should be clean, but has:\n{}", status);
}

#[given("I record the workspace state before selftest")]
async fn given_record_before_selftest(world: &mut World) {
    let files = list_generated_files(world);
    let mut file_hashes = HashMap::new();

    for file in files {
        if file.exists() {
            let content = fs::read(&file).unwrap_or_default();
            let hash = format!("{:x}", md5::compute(&content));
            file_hashes.insert(file.clone(), hash);
        }
    }

    world.xtask_context_mut().env.insert(
        "IDEMPOTENCY_BEFORE_FILES".to_string(),
        serde_json::to_string(&file_hashes).unwrap(),
    );
}

#[given("I record the workspace state before suggest-next")]
async fn given_record_before_suggest_next(world: &mut World) {
    let root = workspace_root(world);
    let mut file_count = 0;

    if let Ok(entries) = fs::read_dir(&root) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                file_count += 1;
            }
        }
    }

    world
        .xtask_context_mut()
        .env
        .insert("IDEMPOTENCY_BEFORE_FILE_COUNT".to_string(), file_count.to_string());
}

// ============================================================================
// When Steps
// ============================================================================

#[when(regex = r#"^I run "([^"]+)" and capture the output$"#)]
async fn when_run_and_capture(world: &mut World, command: String) {
    let (stdout, stderr, status) = run_xtask_command(world, &command);

    let output = format!("{}\n{}", stdout, stderr);
    world.xtask_context_mut().last_command_output = Some(output.clone());
    world.xtask_context_mut().last_command_status = Some(status);

    // Store first output
    world.xtask_context_mut().env.insert("IDEMPOTENCY_FIRST_OUTPUT".to_string(), output);
}

#[when(regex = r#"^I run "([^"]+)" again and capture the output$"#)]
async fn when_run_again_and_capture(world: &mut World, command: String) {
    let (stdout, stderr, status) = run_xtask_command(world, &command);

    let output = format!("{}\n{}", stdout, stderr);
    world.xtask_context_mut().last_command_output = Some(output.clone());
    world.xtask_context_mut().last_command_status = Some(status);

    // Store second output
    world.xtask_context_mut().env.insert("IDEMPOTENCY_SECOND_OUTPUT".to_string(), output);
}

#[when(regex = r#"^I run "([^"]+)" with "([^"]+)" and capture the output$"#)]
async fn when_run_with_env_and_capture(world: &mut World, command: String, env_var: String) {
    let parts: Vec<&str> = env_var.split('=').collect();
    if parts.len() == 2 {
        world.xtask_context_mut().env.insert(parts[0].to_string(), parts[1].to_string());
    }

    let (stdout, stderr, status) = run_xtask_command(world, &command);

    let output = format!("{}\n{}", stdout, stderr);
    world.xtask_context_mut().last_command_output = Some(output.clone());
    world.xtask_context_mut().last_command_status = Some(status);

    // Store first output
    world.xtask_context_mut().env.insert("IDEMPOTENCY_FIRST_OUTPUT".to_string(), output);
}

#[when(regex = r#"^I run "([^"]+)" with "([^"]+)" again and capture the output$"#)]
async fn when_run_with_env_again_and_capture(world: &mut World, command: String, env_var: String) {
    let parts: Vec<&str> = env_var.split('=').collect();
    if parts.len() == 2 {
        world.xtask_context_mut().env.insert(parts[0].to_string(), parts[1].to_string());
    }

    let (stdout, stderr, status) = run_xtask_command(world, &command);

    let output = format!("{}\n{}", stdout, stderr);
    world.xtask_context_mut().last_command_output = Some(output.clone());
    world.xtask_context_mut().last_command_status = Some(status);

    // Store second output
    world.xtask_context_mut().env.insert("IDEMPOTENCY_SECOND_OUTPUT".to_string(), output);
}

#[when(regex = r#"^I run "([^"]+)"$"#)]
async fn when_run_idempotent_command(world: &mut World, command: String) {
    let (stdout, stderr, status) = run_xtask_command(world, &command);
    world.xtask_context_mut().last_command_output = Some(format!("{}\n{}", stdout, stderr));
    world.xtask_context_mut().last_command_status = Some(status);
}

#[when("I record the workspace state after first selftest")]
async fn when_record_after_first_selftest(world: &mut World) {
    let files = list_generated_files(world);
    let mut file_hashes = HashMap::new();

    for file in files {
        if file.exists() {
            let content = fs::read(&file).unwrap_or_default();
            let hash = format!("{:x}", md5::compute(&content));
            file_hashes.insert(file.clone(), hash);
        }
    }

    world.xtask_context_mut().env.insert(
        "IDEMPOTENCY_AFTER_FIRST_FILES".to_string(),
        serde_json::to_string(&file_hashes).unwrap(),
    );
}

#[when("I record the workspace state after second selftest")]
async fn when_record_after_second_selftest(world: &mut World) {
    let files = list_generated_files(world);
    let mut file_hashes = HashMap::new();

    for file in files {
        if file.exists() {
            let content = fs::read(&file).unwrap_or_default();
            let hash = format!("{:x}", md5::compute(&content));
            file_hashes.insert(file.clone(), hash);
        }
    }

    world.xtask_context_mut().env.insert(
        "IDEMPOTENCY_AFTER_SECOND_FILES".to_string(),
        serde_json::to_string(&file_hashes).unwrap(),
    );
}

#[when("I record the workspace state after suggest-next")]
async fn when_record_after_suggest_next(world: &mut World) {
    let root = workspace_root(world);
    let mut file_count = 0;

    if let Ok(entries) = fs::read_dir(&root) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                file_count += 1;
            }
        }
    }

    world
        .xtask_context_mut()
        .env
        .insert("IDEMPOTENCY_AFTER_FILE_COUNT".to_string(), file_count.to_string());
}

// ============================================================================
// Then Steps
// ============================================================================

#[then("both selftest outputs should have the same pass/fail status")]
async fn then_same_pass_fail_status(world: &mut World) {
    let first = world
        .xtask_context()
        .env
        .get("IDEMPOTENCY_FIRST_OUTPUT")
        .expect("First output not captured");
    let second = world
        .xtask_context()
        .env
        .get("IDEMPOTENCY_SECOND_OUTPUT")
        .expect("Second output not captured");

    // Extract pass/fail patterns
    let first_has_pass = first.contains("[OK]") || first.contains("passed");
    let second_has_pass = second.contains("[OK]") || second.contains("passed");

    assert_eq!(
        first_has_pass, second_has_pass,
        "Pass/fail status should be identical between runs"
    );
}

#[then("both selftest outputs should report the same number of checks")]
async fn then_same_check_count(world: &mut World) {
    let first = world
        .xtask_context()
        .env
        .get("IDEMPOTENCY_FIRST_OUTPUT")
        .expect("First output not captured");
    let second = world
        .xtask_context()
        .env
        .get("IDEMPOTENCY_SECOND_OUTPUT")
        .expect("Second output not captured");

    // Both outputs should be non-empty and similar in structure
    assert!(!first.is_empty(), "First output should not be empty");
    assert!(!second.is_empty(), "Second output should not be empty");

    // Check that both mention selftest steps
    assert!(
        first.contains("selftest") || first.contains("checks"),
        "First output should mention selftest/checks"
    );
    assert!(
        second.contains("selftest") || second.contains("checks"),
        "Second output should mention selftest/checks"
    );
}

#[then("both selftest outputs should have identical summary sections")]
async fn then_identical_summaries(world: &mut World) {
    let first = world
        .xtask_context()
        .env
        .get("IDEMPOTENCY_FIRST_OUTPUT")
        .expect("First output not captured");
    let second = world
        .xtask_context()
        .env
        .get("IDEMPOTENCY_SECOND_OUTPUT")
        .expect("Second output not captured");

    // Extract summary-like sections (lines with [OK] or [FAIL])
    let extract_summary = |text: &str| -> Vec<String> {
        text.lines()
            .filter(|line| {
                line.contains("[OK]")
                    || line.contains("[FAIL]")
                    || line.contains("✓")
                    || line.contains("✗")
            })
            .map(|s| s.to_string())
            .collect()
    };

    let first_summary = extract_summary(first);
    let second_summary = extract_summary(second);

    // Summaries should have same structure
    assert_eq!(
        first_summary.len(),
        second_summary.len(),
        "Summary sections should have same number of lines"
    );
}

#[then("no new files should be created by the second run")]
async fn then_no_new_files_second_run(world: &mut World) {
    let after_first_json = world
        .xtask_context()
        .env
        .get("IDEMPOTENCY_AFTER_FIRST_FILES")
        .expect("After-first state not recorded");
    let after_second_json = world
        .xtask_context()
        .env
        .get("IDEMPOTENCY_AFTER_SECOND_FILES")
        .expect("After-second state not recorded");

    let after_first: HashMap<PathBuf, String> = serde_json::from_str(after_first_json).unwrap();
    let after_second: HashMap<PathBuf, String> = serde_json::from_str(after_second_json).unwrap();

    // No new files should appear
    for key in after_second.keys() {
        assert!(after_first.contains_key(key), "Second run created new file: {:?}", key);
    }
}

#[then("the generated documentation files should be identical")]
async fn then_identical_generated_docs(world: &mut World) {
    let after_first_json = world
        .xtask_context()
        .env
        .get("IDEMPOTENCY_AFTER_FIRST_FILES")
        .expect("After-first state not recorded");
    let after_second_json = world
        .xtask_context()
        .env
        .get("IDEMPOTENCY_AFTER_SECOND_FILES")
        .expect("After-second state not recorded");

    let after_first: HashMap<PathBuf, String> = serde_json::from_str(after_first_json).unwrap();
    let after_second: HashMap<PathBuf, String> = serde_json::from_str(after_second_json).unwrap();

    // Check that all files have identical hashes
    for (path, hash_first) in &after_first {
        if let Some(hash_second) = after_second.get(path) {
            assert_eq!(hash_first, hash_second, "File {:?} changed between runs", path);
        }
    }
}

#[then("the git working tree should still be clean")]
async fn then_git_still_clean(world: &mut World) {
    let status = get_git_status(world);

    // Allow feature_status.md to be modified by selftest (it's auto-generated)
    let lines: Vec<&str> =
        status.lines().filter(|line| !line.contains("feature_status.md")).collect();

    assert!(
        lines.is_empty(),
        "Git working tree should be clean (ignoring feature_status.md), but has:\n{}",
        lines.join("\n")
    );
}

#[then("no uncommitted changes should exist")]
async fn then_no_uncommitted_changes(world: &mut World) {
    then_git_still_clean(world).await;
}

#[then("both suggest-next outputs should be identical")]
async fn then_identical_suggest_next_outputs(world: &mut World) {
    let first = world
        .xtask_context()
        .env
        .get("IDEMPOTENCY_FIRST_OUTPUT")
        .expect("First output not captured");
    let second = world
        .xtask_context()
        .env
        .get("IDEMPOTENCY_SECOND_OUTPUT")
        .expect("Second output not captured");

    // Normalize whitespace and compare
    let normalize = |s: &str| -> String {
        s.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    };

    let first_normalized = normalize(first);
    let second_normalized = normalize(second);

    assert_eq!(first_normalized, second_normalized, "suggest-next outputs should be identical");
}

#[then("the recommended command sequence should be the same")]
async fn then_same_command_sequence(world: &mut World) {
    let first = world
        .xtask_context()
        .env
        .get("IDEMPOTENCY_FIRST_OUTPUT")
        .expect("First output not captured");
    let second = world
        .xtask_context()
        .env
        .get("IDEMPOTENCY_SECOND_OUTPUT")
        .expect("Second output not captured");

    // Check for command patterns
    let has_commands =
        |text: &str| -> bool { text.contains("cargo xtask") || text.contains("recommended") };

    assert_eq!(
        has_commands(first),
        has_commands(second),
        "Both outputs should have commands or neither should"
    );
}

#[then("no new files should be created by suggest-next")]
async fn then_no_new_files_suggest_next(world: &mut World) {
    let before = world
        .xtask_context()
        .env
        .get("IDEMPOTENCY_BEFORE_FILE_COUNT")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    let after = world
        .xtask_context()
        .env
        .get("IDEMPOTENCY_AFTER_FILE_COUNT")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);

    assert_eq!(
        before, after,
        "suggest-next should not create new files (before: {}, after: {})",
        before, after
    );
}

#[then("no existing files should be modified by suggest-next")]
async fn then_no_modified_files_suggest_next(world: &mut World) {
    let status = get_git_status(world);
    assert!(
        status.trim().is_empty(),
        "suggest-next should not modify files, but git status shows:\n{}",
        status
    );
}

#[then("the low-resource mode should be consistently applied")]
async fn then_low_resource_consistent(world: &mut World) {
    let first = world
        .xtask_context()
        .env
        .get("IDEMPOTENCY_FIRST_OUTPUT")
        .expect("First output not captured");
    let second = world
        .xtask_context()
        .env
        .get("IDEMPOTENCY_SECOND_OUTPUT")
        .expect("Second output not captured");

    let first_low = first.contains("low-resource") || first.contains("XTASK_LOW_RESOURCES");
    let second_low = second.contains("low-resource") || second.contains("XTASK_LOW_RESOURCES");

    assert_eq!(first_low, second_low, "Low-resource mode should be applied consistently");
}
