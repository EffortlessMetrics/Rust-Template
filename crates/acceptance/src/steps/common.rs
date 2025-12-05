/// Common reusable step definitions for file operations and basic assertions.
///
/// This module provides a library of frequently-used BDD steps that can be
/// reused across multiple feature files. It follows the principle of keeping
/// step definitions DRY (Don't Repeat Yourself).
///
/// # Categories
///
/// - **File Operations**: Checking existence, content, permissions
/// - **Command Execution**: Running commands and checking exit codes
/// - **JSON Assertions**: Validating JSON structure and content
/// - **String Assertions**: Content matching and regex patterns
///
/// # Usage
///
/// These steps are automatically available to all Cucumber scenarios once
/// this module is imported in `steps/mod.rs`.
use crate::world::World;
use cucumber::{gherkin::Step, given, then, when};
use regex::Regex;
use std::fs;

// ============================================================================
// Helper Functions
// ============================================================================

/// Get the workspace root for the current test world.
fn workspace_root(world: &World) -> std::path::PathBuf {
    world.xtask_context().test_repo_path.clone().unwrap_or_else(|| world.spec_root().to_path_buf())
}

/// Resolve a path relative to the workspace root.
fn resolve_path(world: &World, path_str: &str) -> std::path::PathBuf {
    let root = workspace_root(world);
    if path_str.starts_with('/') || path_str.starts_with("./") || path_str.starts_with("../") {
        root.join(path_str.trim_start_matches('/'))
    } else {
        root.join(path_str)
    }
}

/// Read file contents as a string.
fn read_file_content(world: &World, path_str: &str) -> Result<String, std::io::Error> {
    let path = resolve_path(world, path_str);
    fs::read_to_string(path)
}

// ============================================================================
// File Existence Steps
// ============================================================================
// NOTE: "the file should exist" step is defined in xtask_devex.rs to avoid ambiguity

#[then(regex = r#"^the file "([^"]+)" should not exist$"#)]
async fn then_file_should_not_exist(world: &mut World, file_path: String) {
    let path = resolve_path(world, &file_path);
    assert!(
        !path.exists(),
        "File should not exist at: {}\nResolved path: {}",
        file_path,
        path.display()
    );
}

#[then(regex = r#"^the directory "([^"]+)" should exist$"#)]
async fn then_directory_should_exist(world: &mut World, dir_path: String) {
    let path = resolve_path(world, &dir_path);
    assert!(
        path.exists() && path.is_dir(),
        "Directory should exist at: {}\nResolved path: {}",
        dir_path,
        path.display()
    );
}

#[then(regex = r#"^the directory "([^"]+)" should not exist$"#)]
async fn then_directory_should_not_exist(world: &mut World, dir_path: String) {
    let path = resolve_path(world, &dir_path);
    assert!(
        !path.exists(),
        "Directory should not exist at: {}\nResolved path: {}",
        dir_path,
        path.display()
    );
}

// ============================================================================
// File Content Steps
// ============================================================================

#[then(regex = r#"^the file "([^"]+)" should contain "([^"]+)"$"#)]
async fn then_file_should_contain(world: &mut World, file_path: String, expected_text: String) {
    let content = read_file_content(world, &file_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read file '{}': {}\nResolved path: {}",
            file_path,
            e,
            resolve_path(world, &file_path).display()
        )
    });

    assert!(
        content.contains(&expected_text),
        "File '{}' should contain '{}'\nActual content:\n{}",
        file_path,
        expected_text,
        content
    );
}

#[then(regex = r#"^the file "([^"]+)" should not contain "([^"]+)"$"#)]
async fn then_file_should_not_contain(
    world: &mut World,
    file_path: String,
    unexpected_text: String,
) {
    let content = read_file_content(world, &file_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read file '{}': {}\nResolved path: {}",
            file_path,
            e,
            resolve_path(world, &file_path).display()
        )
    });

    assert!(
        !content.contains(&unexpected_text),
        "File '{}' should not contain '{}'\nActual content:\n{}",
        file_path,
        unexpected_text,
        content
    );
}

#[then(regex = r#"^the file "([^"]+)" should match pattern "([^"]+)"$"#)]
async fn then_file_should_match_pattern(world: &mut World, file_path: String, pattern_str: String) {
    let content = read_file_content(world, &file_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read file '{}': {}\nResolved path: {}",
            file_path,
            e,
            resolve_path(world, &file_path).display()
        )
    });

    let pattern = Regex::new(&pattern_str)
        .unwrap_or_else(|e| panic!("Invalid regex pattern '{}': {}", pattern_str, e));

    assert!(
        pattern.is_match(&content),
        "File '{}' should match pattern '{}'\nActual content:\n{}",
        file_path,
        pattern_str,
        content
    );
}

#[then(regex = r#"^the file "([^"]+)" should be empty$"#)]
async fn then_file_should_be_empty(world: &mut World, file_path: String) {
    let content = read_file_content(world, &file_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read file '{}': {}\nResolved path: {}",
            file_path,
            e,
            resolve_path(world, &file_path).display()
        )
    });

    assert!(
        content.trim().is_empty(),
        "File '{}' should be empty\nActual content:\n{}",
        file_path,
        content
    );
}

#[then(regex = r#"^the file "([^"]+)" should not be empty$"#)]
async fn then_file_should_not_be_empty(world: &mut World, file_path: String) {
    let content = read_file_content(world, &file_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read file '{}': {}\nResolved path: {}",
            file_path,
            e,
            resolve_path(world, &file_path).display()
        )
    });

    assert!(!content.trim().is_empty(), "File '{}' should not be empty", file_path);
}

// ============================================================================
// File Permission Steps (Unix only)
// ============================================================================

#[cfg(unix)]
#[then(regex = r#"^(?:the )?file "([^"]+)" should be executable$"#)]
async fn then_file_should_be_executable(world: &mut World, file_path: String) {
    use std::os::unix::fs::PermissionsExt;

    let path = resolve_path(world, &file_path);
    let metadata = fs::metadata(&path)
        .unwrap_or_else(|e| panic!("Failed to read metadata for '{}': {}", file_path, e));

    let perms = metadata.permissions();
    let is_executable = perms.mode() & 0o111 != 0;

    assert!(is_executable, "File '{}' should be executable (mode: {:o})", file_path, perms.mode());
}

#[cfg(unix)]
#[then(regex = r#"^the file "([^"]+)" should not be executable$"#)]
async fn then_file_should_not_be_executable(world: &mut World, file_path: String) {
    use std::os::unix::fs::PermissionsExt;

    let path = resolve_path(world, &file_path);
    let metadata = fs::metadata(&path)
        .unwrap_or_else(|e| panic!("Failed to read metadata for '{}': {}", file_path, e));

    let perms = metadata.permissions();
    let is_executable = perms.mode() & 0o111 != 0;

    assert!(
        !is_executable,
        "File '{}' should not be executable (mode: {:o})",
        file_path,
        perms.mode()
    );
}

// ============================================================================
// Command Exit Code Steps
// ============================================================================
// NOTE: Exit code steps are defined in agent_hints.rs and xtask_devex.rs
// to avoid ambiguity. Use those instead.

// ============================================================================
// JSON Assertion Steps
// ============================================================================
// NOTE: JSON output field presence steps are defined in agent_hints.rs
// to avoid ambiguity. Use those instead.

#[then(regex = r#"^the JSON output should not have field "([^"]+)"$"#)]
async fn then_json_output_should_not_have_field(world: &mut World, field_name: String) {
    let json = world
        .cli_json_output
        .as_ref()
        .expect("No JSON output available - did you parse the command output as JSON?");

    assert!(
        json.get(&field_name).is_none(),
        "JSON output should not have field '{}'\nActual JSON:\n{}",
        field_name,
        serde_json::to_string_pretty(json).unwrap_or_else(|_| format!("{:?}", json))
    );
}

#[then(regex = r#"^the JSON field "([^"]+)" should equal "([^"]+)"$"#)]
async fn then_json_field_should_equal(
    world: &mut World,
    field_name: String,
    expected_value: String,
) {
    let json = world
        .cli_json_output
        .as_ref()
        .expect("No JSON output available - did you parse the command output as JSON?");

    let actual_value = json.get(&field_name).and_then(|v| v.as_str()).unwrap_or_else(|| {
        panic!(
            "Field '{}' not found or not a string in JSON:\n{}",
            field_name,
            serde_json::to_string_pretty(json).unwrap_or_else(|_| format!("{:?}", json))
        )
    });

    assert_eq!(
        actual_value, expected_value,
        "JSON field '{}' should equal '{}', but got '{}'",
        field_name, expected_value, actual_value
    );
}

#[then(regex = r#"^the JSON field "([^"]+)" should contain "([^"]+)"$"#)]
async fn then_json_field_should_contain(
    world: &mut World,
    field_name: String,
    expected_substring: String,
) {
    let json = world
        .cli_json_output
        .as_ref()
        .expect("No JSON output available - did you parse the command output as JSON?");

    let actual_value = json.get(&field_name).and_then(|v| v.as_str()).unwrap_or_else(|| {
        panic!(
            "Field '{}' not found or not a string in JSON:\n{}",
            field_name,
            serde_json::to_string_pretty(json).unwrap_or_else(|_| format!("{:?}", json))
        )
    });

    assert!(
        actual_value.contains(&expected_substring),
        "JSON field '{}' should contain '{}', but got '{}'",
        field_name,
        expected_substring,
        actual_value
    );
}

// ============================================================================
// String Assertion Steps
// ============================================================================

// NOTE: This step is commented out to avoid ambiguity with the more flexible version
// in xtask_devex.rs which supports escaped quotes: ^the output (?:should )?contain(?:s)? "((?:\\.|[^"])*)"$
//
// #[then(regex = r#"^the output should contain "([^"]+)"$"#)]
// async fn then_output_should_contain_alt(world: &mut World, expected_text: String) {
//     let output = world
//         .xtask_context()
//         .last_command_output
//         .as_ref()
//         .expect("No command output available");
//
//     assert!(
//         output.contains(&expected_text),
//         "Output should contain '{}'\nActual output:\n{}",
//         expected_text,
//         output
//     );
// }

#[then(regex = r#"^the output should not contain "([^"]+)"$"#)]
async fn then_output_should_not_contain(world: &mut World, unexpected_text: String) {
    let output =
        world.xtask_context().last_command_output.as_ref().expect("No command output available");

    assert!(
        !output.contains(&unexpected_text),
        "Output should not contain '{}'\nActual output:\n{}",
        unexpected_text,
        output
    );
}

#[then(regex = r#"^the output should match pattern "([^"]+)"$"#)]
async fn then_output_should_match_pattern(world: &mut World, pattern_str: String) {
    let output =
        world.xtask_context().last_command_output.as_ref().expect("No command output available");

    let pattern = Regex::new(&pattern_str)
        .unwrap_or_else(|e| panic!("Invalid regex pattern '{}': {}", pattern_str, e));

    assert!(
        pattern.is_match(output),
        "Output should match pattern '{}'\nActual output:\n{}",
        pattern_str,
        output
    );
}

#[then(regex = r#"^the output should be empty$"#)]
async fn then_output_should_be_empty(world: &mut World) {
    let output =
        world.xtask_context().last_command_output.as_ref().expect("No command output available");

    assert!(output.trim().is_empty(), "Output should be empty\nActual output:\n{}", output);
}

#[then(regex = r#"^the output should not be empty$"#)]
async fn then_output_should_not_be_empty(world: &mut World) {
    let output =
        world.xtask_context().last_command_output.as_ref().expect("No command output available");

    assert!(!output.trim().is_empty(), "Output should not be empty");
}

// ============================================================================
// Given Steps - File Setup
// ============================================================================

#[given(regex = r#"^a file "([^"]+)" with content:$"#)]
async fn given_file_with_content(world: &mut World, file_path: String, step: &Step) {
    let path = resolve_path(world, &file_path);

    // Extract content from the docstring
    let content = step.docstring.as_deref().unwrap_or("");

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|e| {
            panic!("Failed to create parent directories for '{}': {}", file_path, e)
        });
    }

    fs::write(&path, content)
        .unwrap_or_else(|e| panic!("Failed to write file '{}': {}", file_path, e));
}

#[given(regex = r#"^a file "([^"]+)" exists$"#)]
async fn given_file_exists(world: &mut World, file_path: String) {
    let path = resolve_path(world, &file_path);

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|e| {
            panic!("Failed to create parent directories for '{}': {}", file_path, e)
        });
    }

    // Create an empty file if it doesn't exist
    if !path.exists() {
        fs::write(&path, "")
            .unwrap_or_else(|e| panic!("Failed to create file '{}': {}", file_path, e));
    }
}

#[given(regex = r#"^a directory "([^"]+)" exists$"#)]
async fn given_directory_exists(world: &mut World, dir_path: String) {
    let path = resolve_path(world, &dir_path);
    fs::create_dir_all(&path)
        .unwrap_or_else(|e| panic!("Failed to create directory '{}': {}", dir_path, e));
}

// ============================================================================
// When Steps - File Operations
// ============================================================================

#[when(regex = r#"^I delete the file "([^"]+)"$"#)]
async fn when_delete_file(world: &mut World, file_path: String) {
    let path = resolve_path(world, &file_path);
    if path.exists() {
        fs::remove_file(&path)
            .unwrap_or_else(|e| panic!("Failed to delete file '{}': {}", file_path, e));
    }
}

#[when(regex = r#"^I delete the directory "([^"]+)"$"#)]
async fn when_delete_directory(world: &mut World, dir_path: String) {
    let path = resolve_path(world, &dir_path);
    if path.exists() {
        fs::remove_dir_all(&path)
            .unwrap_or_else(|e| panic!("Failed to delete directory '{}': {}", dir_path, e));
    }
}

#[when(regex = r#"^I create a file "([^"]+)" with content "([^"]+)"$"#)]
async fn when_create_file_with_content(world: &mut World, file_path: String, content: String) {
    let path = resolve_path(world, &file_path);

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|e| {
            panic!("Failed to create parent directories for '{}': {}", file_path, e)
        });
    }

    fs::write(&path, content)
        .unwrap_or_else(|e| panic!("Failed to write file '{}': {}", file_path, e));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_path_absolute() {
        let world = World::default();
        let path = resolve_path(&world, "/test/file.txt");
        assert!(path.ends_with("test/file.txt"));
    }

    #[test]
    fn test_resolve_path_relative() {
        let world = World::default();
        let path = resolve_path(&world, "test/file.txt");
        assert!(path.ends_with("test/file.txt"));
    }
}
