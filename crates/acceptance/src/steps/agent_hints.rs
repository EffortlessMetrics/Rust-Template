use crate::world::World;
use anyhow::{Context, Result, bail, ensure};
use cucumber::{then, when};
use shell_words;
use std::path::Path;
use std::process::Command;

// ============================================================================
// CLI Command Step Definitions for suggest-next
// ============================================================================

fn actual_workspace_root() -> std::path::PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap().parent().unwrap().to_path_buf()
}

#[when(regex = r#"^I run the command "([^"]+)"$"#)]
async fn when_run_command(world: &mut World, command: String) -> Result<()> {
    // Parse the command string (e.g., "cargo xtask suggest-next --format json")
    let parts: Vec<String> = shell_words::split(&command)
        .unwrap_or_else(|_| command.split_whitespace().map(|s| s.to_string()).collect());

    ensure!(!parts.is_empty(), "Command string is empty");

    let subcommand = if parts.len() >= 3 && parts[0] == "cargo" && parts[1] == "xtask" {
        Some(parts[2].as_str())
    } else {
        None
    };

    let root_path = world._temp_dir.path();
    let workspace_root = actual_workspace_root();

    // Build the command
    let mut cmd = if parts.len() >= 3 && parts[0] == "cargo" && parts[1] == "xtask" {
        // cargo xtask command - use the workspace manifest to run xtask
        let mut c = Command::new("cargo");
        let manifest = workspace_root.join("Cargo.toml");
        c.arg("run").arg("--manifest-path").arg(manifest).arg("-p").arg("xtask").arg("--");
        // Add subcommand and args
        for part in &parts[2..] {
            c.arg(part);
        }
        c
    } else {
        // Other command - run as-is
        let mut c = Command::new(&parts[0]);
        for part in &parts[1..] {
            c.arg(part);
        }
        c
    };

    // Set working directory to the temp directory (for task discovery)
    cmd.current_dir(root_path);

    // Set SPEC_ROOT to the temp directory so the command finds the test specs
    cmd.env("SPEC_ROOT", root_path);

    // Bypass xtask's automatic Nix wrapper since the temp directory has no flake.nix
    cmd.env("IN_NIX_SHELL", "1");

    // Avoid nested BDD runs during acceptance tests to prevent coverage clobbering.
    if subcommand != Some("bdd") {
        cmd.env("XTASK_SKIP_BDD", "1");
    }
    // Avoid leaking BDD selection env vars into child commands.
    cmd.env_remove("CUCUMBER_TAG_EXPRESSION");
    cmd.env_remove("CUCUMBER_FILTER_TAGS");

    // Execute the command
    let output = cmd.output().context("Failed to execute command")?;

    // Store the output in world - both in new CLI fields and xtask_context for compatibility
    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Store in new CLI fields
    world.cli_exit_code = Some(exit_code);
    world.cli_stdout = stdout.clone();
    world.cli_stderr = stderr.clone();

    // Also store in xtask_context for compatibility with existing steps
    world.xtask_context_mut().last_command_output = Some((stdout + &stderr).clone());
    world.xtask_context_mut().last_command_status = Some(exit_code);

    // Try to parse JSON output if it looks like JSON
    let stdout_trimmed = world.cli_stdout.trim();
    if (stdout_trimmed.starts_with('{') || stdout_trimmed.starts_with('['))
        && let Ok(json) = serde_json::from_str::<serde_json::Value>(stdout_trimmed)
    {
        world.cli_json_output = Some(json);
    }

    Ok(())
}

#[then(regex = r#"^the exit code should be (\d+)$"#)]
async fn then_exit_code(world: &mut World, expected_code: i32) -> Result<()> {
    let actual_code = world.cli_exit_code.context("No CLI command has been run")?;
    ensure!(
        actual_code == expected_code,
        "Expected exit code {}, but got {}. stderr: {}",
        expected_code,
        actual_code,
        world.cli_stderr
    );
    Ok(())
}

#[then(regex = r#"^the JSON output should have field "([^"]+)"$"#)]
async fn then_json_output_has_field(world: &mut World, field: String) -> Result<()> {
    let json =
        world.cli_json_output.as_ref().context("No JSON output available from CLI command")?;
    ensure!(
        json.get(&field).is_some(),
        "Expected JSON output to have field '{}', but it didn't. Response: {:?}",
        field,
        json
    );
    Ok(())
}

#[then(regex = r#"^the "([^"]+)" array should have (\d+) items?$"#)]
async fn then_json_array_has_count(
    world: &mut World,
    field: String,
    expected_count: usize,
) -> Result<()> {
    let json = if let Some(json) = &world.cli_json_output {
        json
    } else if let Some(response) = &world.last_response {
        &response.body
    } else {
        bail!("No JSON output available from either CLI command or HTTP response")
    };

    let array = json.get(&field).and_then(|v| v.as_array());
    ensure!(array.is_some(), "field '{}' should be an array", field);
    let array = array.unwrap();

    ensure!(
        array.len() == expected_count,
        "Expected '{}' array to have {} items, but got {}. Array: {:?}",
        field,
        expected_count,
        array.len(),
        array
    );
    Ok(())
}

#[then(regex = r#"^the first hint in JSON should have field "([^"]+)"$"#)]
async fn then_first_hint_in_json_has_field(world: &mut World, field: String) -> Result<()> {
    let json =
        world.cli_json_output.as_ref().context("No JSON output available from CLI command")?;
    let hints = json.get("hints").and_then(|v| v.as_array());
    ensure!(hints.is_some(), "JSON should have 'hints' array");
    let hints = hints.unwrap();

    ensure!(!hints.is_empty(), "Expected at least one hint, but hints array is empty");

    let first_hint = &hints[0];
    ensure!(
        first_hint.get(&field).is_some(),
        "Expected first hint to have field '{}', but it didn't. Hint: {:?}",
        field,
        first_hint
    );
    Ok(())
}

#[then(regex = r#"^the first hint "([^"]+)" should not be empty$"#)]
async fn then_first_hint_field_not_empty(world: &mut World, field: String) -> Result<()> {
    // Try CLI JSON first, then fall back to HTTP API response
    let hints = if let Some(json) = &world.cli_json_output {
        json.get("hints").and_then(|v| v.as_array())
    } else if let Some(response) = &world.last_response {
        response.body.get("hints").and_then(|v| v.as_array())
    } else {
        bail!("No JSON output available from either CLI command or HTTP response")
    };

    ensure!(hints.is_some(), "JSON should have 'hints' array");
    let hints = hints.unwrap();
    ensure!(!hints.is_empty(), "Expected at least one hint, but hints array is empty");

    let first_hint = &hints[0];
    let value = first_hint.get(&field);
    ensure!(value.is_some(), "Expected first hint to have field '{}', but it didn't", field);
    let value = value.unwrap();

    // Check that the value is not empty (works for strings and arrays)
    match value {
        serde_json::Value::String(s) => {
            ensure!(
                !s.is_empty(),
                "Expected first hint field '{}' to not be empty, but it was",
                field
            );
        }
        serde_json::Value::Array(arr) => {
            ensure!(
                !arr.is_empty(),
                "Expected first hint field '{}' array to not be empty, but it was",
                field
            );
        }
        serde_json::Value::Null => {
            bail!("Expected first hint field '{}' to not be null", field);
        }
        _ => {} // Other types are considered non-empty
    }
    Ok(())
}
