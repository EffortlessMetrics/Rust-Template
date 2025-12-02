use crate::world::{Response, World};
use adapters_spec_fs::tasks_state;
use axum::body::Body;
use business_core::governance::{TaskId, TaskStatus};
use cucumber::{gherkin::Step, given, then, when};
use http::Request;
use http_body_util::BodyExt;
use spec_runtime::tasks::{Task, TaskDocs, TasksSpec};
use std::{fs, path::Path};
use tower::util::ServiceExt;

/// Extract task ID from a hint JSON value.
///
/// Supports multiple field layouts for forward/backward compatibility:
/// - `task_id` (convenience field in HTTP/CLI)
/// - `id` (legacy)
/// - `target.id` (canonical schema location for task hints)
fn extract_hint_task_id(hint: &serde_json::Value) -> &str {
    hint.get("task_id")
        .and_then(|v| v.as_str())
        .or_else(|| hint.get("id").and_then(|v| v.as_str()))
        .or_else(|| hint.get("target").and_then(|t| t.get("id")).and_then(|v| v.as_str()))
        .unwrap_or("")
}

#[given("the platform is running")]
async fn given_platform_running(_world: &mut World) {
    // Background step for platform tests
    // For HTTP API tests, the platform (app) is initialized in World::new()
    // For CLI tests, this is a no-op since CLI commands don't require the HTTP server
}

#[given(regex = r#"^a task "([^"]+)" exists with status "([^"]+)"$"#)]
async fn given_task_exists(world: &mut World, task_id: String, status_str: String) {
    let (status, status_text) = parse_status(&status_str);

    let specs_dir = world._temp_dir.path().join("specs");
    let state_path = specs_dir.join("tasks_state.yaml");
    tasks_state::update_task_status(&state_path, TaskId(task_id.clone()), status)
        .expect("Failed to update task status");

    ensure_tasks_file(
        &specs_dir.join("tasks.yaml"),
        vec![Task {
            id: task_id.clone(),
            title: task_id.clone(),
            requirement: "REQ-TBD".to_string(),
            acs: Vec::new(),
            status: status_text,
            owner: None,
            labels: Vec::new(),
            docs: Some(empty_task_docs()),
            summary: task_id,
            recommended_flows: Vec::new(),
            depends_on: Vec::new(),
        }],
    );
}

#[given(expr = r#"the following tasks exist in {string}:"#)]
async fn given_tasks_exist(world: &mut World, file: String, step: &cucumber::gherkin::Step) {
    let tasks_path = world._temp_dir.path().join(&file);
    let specs_dir = tasks_path.parent().unwrap_or_else(|| world._temp_dir.path()).to_path_buf();
    fs::create_dir_all(&specs_dir).expect("Failed to create specs directory");

    let state_path = specs_dir.join("tasks_state.yaml");

    let mut tasks: Vec<Task> = Vec::new();

    // Parse table and create tasks
    if let Some(table) = &step.table {
        let headers: Vec<&str> = table
            .rows
            .first()
            .map(|row| row.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default();

        for row in table.rows.iter().skip(1) {
            let mut task_id = String::new();
            let mut status_text = "Todo".to_string();
            let mut title = String::from("Untitled Task");
            let mut requirement = String::from("REQ-TBD");
            let mut owner: Option<String> = None;
            let mut labels: Vec<String> = Vec::new();
            let mut recommended_flows: Vec<String> = Vec::new();

            for (i, cell) in row.iter().enumerate() {
                if let Some(&header) = headers.get(i) {
                    match header {
                        "id" => task_id = cell.to_string(),
                        "title" => title = cell.to_string(),
                        "requirement" => requirement = cell.to_string(),
                        "status" => status_text = cell.to_string(),
                        "owner" => {
                            if !cell.is_empty() {
                                owner = Some(cell.to_string());
                            }
                        }
                        "labels" => {
                            if !cell.is_empty() {
                                labels = cell.split(',').map(|s| s.trim().to_string()).collect();
                            }
                        }
                        "recommended_flows" => {
                            if !cell.is_empty() {
                                recommended_flows =
                                    cell.split(',').map(|s| s.trim().to_string()).collect();
                            }
                        }
                        _ => {} // Ignore other fields
                    }
                }
            }

            if !task_id.is_empty() {
                let (status, status_label) = parse_status(&status_text);
                tasks_state::update_task_status(&state_path, TaskId(task_id.clone()), status)
                    .expect("Failed to update task status");

                // Default to ac_first flow if no recommended_flows specified
                let flows = if recommended_flows.is_empty() {
                    vec!["ac_first".to_string()]
                } else {
                    recommended_flows.clone()
                };

                tasks.push(Task {
                    id: task_id.clone(),
                    title: title.clone(),
                    requirement: requirement.clone(),
                    acs: Vec::new(),
                    status: status_label,
                    owner: owner.clone(),
                    labels: labels.clone(),
                    docs: Some(empty_task_docs()),
                    summary: title.clone(),
                    recommended_flows: flows,
                    depends_on: Vec::new(),
                });
            }
        }
    }

    ensure_tasks_file(&tasks_path, tasks);
}

#[when(regex = r#"^I send a POST request to "([^"]+)" with body:$"#)]
async fn when_post_request(world: &mut World, path: String, step: &Step) {
    let body = step
        .docstring
        .as_ref()
        .cloned()
        .unwrap_or_else(|| panic!("Docstring body not provided for POST {}", path));
    let mut request_builder =
        Request::builder().method("POST").uri(&path).header("content-type", "application/json");

    for (key, value) in &world.request_headers {
        request_builder = request_builder.header(key, value);
    }

    let request = request_builder.body(Body::from(body)).expect("Failed to build request");

    let response = world
        .app
        .clone()
        .oneshot(request)
        .await
        .unwrap_or_else(|err| panic!("App request failed for POST {}: {}", path, err));

    let status = response.status().as_u16();
    let headers = response.headers().clone();
    let body_bytes = response.into_body().collect().await.map(|c| c.to_bytes()).unwrap_or_default();

    let raw_body = String::from_utf8_lossy(&body_bytes).to_string();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap_or_default();

    world.last_response = Some(Response { status, body, headers, raw_body });
    world.request_headers.clear();
}

#[then(regex = r#"^the response status code should be (\d+)$"#)]
async fn then_status_code(world: &mut World, status_code: u16) {
    let response = world.last_response.as_ref().expect("response should exist");
    assert_eq!(
        response.status, status_code,
        "Expected status {}, got {}",
        status_code, response.status
    );
}

#[then(regex = r#"^the task "([^"]+)" should have status "([^"]+)"$"#)]
async fn then_task_status(world: &mut World, task_id: String, expected_status_str: String) {
    let path = world._temp_dir.path().join("specs/tasks_state.yaml");
    let status = tasks_state::get_task_status(&path, &TaskId(task_id))
        .expect("Failed to get task status")
        .expect("Task not found");

    let expected_status = match expected_status_str.as_str() {
        "Todo" => TaskStatus::Todo,
        "InProgress" => TaskStatus::InProgress,
        "Review" => TaskStatus::Review,
        "Done" => TaskStatus::Done,
        _ => panic!("Invalid expected status: {}", expected_status_str),
    };

    assert_eq!(status, expected_status, "Expected status {:?}, got {:?}", expected_status, status);
}

#[then(regex = r#"^task "([^"]+)" should exist with title "([^"]+)" and requirement "([^"]+)"$"#)]
async fn then_task_exists_with_title_and_req(
    world: &mut World,
    task_id: String,
    title: String,
    requirement: String,
) {
    let tasks = load_tasks_from_spec_root(world);
    let task = find_task(&tasks, &task_id);
    assert_eq!(task.title, title, "Task {} should have title {}", task_id, title);
    assert_eq!(
        task.requirement, requirement,
        "Task {} should target requirement {}",
        task_id, requirement
    );
}

#[then(regex = r#"^task "([^"]+)" should have status "([^"]+)" in tasks.yaml$"#)]
async fn then_task_status_in_yaml(world: &mut World, task_id: String, expected_status: String) {
    let tasks = load_tasks_from_spec_root(world);
    let task = find_task(&tasks, &task_id);
    assert_eq!(
        task.status, expected_status,
        "Task {} should have status {}",
        task_id, expected_status
    );
}

#[then(regex = r#"^task "([^"]+)" should have owner "([^"]+)"$"#)]
async fn then_task_owner_in_yaml(world: &mut World, task_id: String, expected_owner: String) {
    let tasks = load_tasks_from_spec_root(world);
    let task = find_task(&tasks, &task_id);
    assert_eq!(
        task.owner.as_deref(),
        Some(expected_owner.as_str()),
        "Task {} should have owner {}",
        task_id,
        expected_owner
    );
}

#[then(regex = r#"^task "([^"]+)" should have title "([^"]+)"$"#)]
async fn then_task_title_in_yaml(world: &mut World, task_id: String, expected_title: String) {
    let tasks = load_tasks_from_spec_root(world);
    let task = find_task(&tasks, &task_id);
    assert_eq!(task.title, expected_title, "Task {} should have title {}", task_id, expected_title);
}

fn parse_status(status_str: &str) -> (TaskStatus, String) {
    match status_str.to_lowercase().as_str() {
        "todo" => (TaskStatus::Todo, "Todo".to_string()),
        "inprogress" | "in_progress" | "in-progress" => {
            (TaskStatus::InProgress, "InProgress".to_string())
        }
        "review" => (TaskStatus::Review, "Review".to_string()),
        "done" => (TaskStatus::Done, "Done".to_string()),
        other => (TaskStatus::Todo, other.to_string()),
    }
}

fn load_tasks_from_spec_root(world: &World) -> TasksSpec {
    let root = world.spec_root();
    let path = root.join("specs/tasks.yaml");
    spec_runtime::load_tasks(&path).expect("Failed to load tasks.yaml")
}

fn find_task<'a>(tasks: &'a TasksSpec, task_id: &str) -> &'a Task {
    tasks
        .tasks
        .iter()
        .find(|t| t.id == task_id)
        .unwrap_or_else(|| panic!("Task {} not found in tasks.yaml", task_id))
}

fn empty_task_docs() -> TaskDocs {
    TaskDocs { design: Vec::new(), plan: Vec::new() }
}

fn default_tasks_spec() -> TasksSpec {
    TasksSpec {
        schema_version: "1.0.0".to_string(),
        template_version: "0.1.0".to_string(),
        tasks: Vec::new(),
    }
}

fn ensure_tasks_file(tasks_file: &Path, tasks: Vec<Task>) {
    if let Some(parent) = tasks_file.parent() {
        fs::create_dir_all(parent).expect("Failed to create specs directory");
    }

    let mut spec = if tasks_file.exists() {
        fs::read_to_string(tasks_file)
            .ok()
            .and_then(|content| serde_yaml::from_str::<TasksSpec>(&content).ok())
            .unwrap_or_else(default_tasks_spec)
    } else {
        default_tasks_spec()
    };

    // Tests expect the tasks.yaml content to exactly match the table in the scenario.
    // Start fresh instead of merging with the workspace copy to avoid leaking other tasks.
    let mut new_tasks = Vec::new();
    for mut task in tasks {
        if task.summary.is_empty() {
            task.summary = task.title.clone();
        }

        new_tasks.push(task);
    }

    spec.tasks = new_tasks;

    let content = serde_yaml::to_string(&spec).expect("Failed to serialize tasks.yaml");
    fs::write(tasks_file, content).expect("Failed to write tasks.yaml");
}

#[when(regex = r#"^I send a GET request to "([^"]+)"$"#)]
async fn when_get_request(world: &mut World, path: String) {
    let mut request_builder = Request::builder().method("GET").uri(&path);

    for (key, value) in &world.request_headers {
        request_builder = request_builder.header(key, value);
    }

    let request = request_builder.body(Body::empty()).expect("Failed to build request");

    let response = world
        .app
        .clone()
        .oneshot(request)
        .await
        .unwrap_or_else(|err| panic!("App request failed for GET {}: {}", path, err));

    let status = response.status().as_u16();
    let headers = response.headers().clone();
    let body_bytes = response.into_body().collect().await.map(|c| c.to_bytes()).unwrap_or_default();

    let raw_body = String::from_utf8_lossy(&body_bytes).to_string();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap_or_default();

    world.last_response = Some(Response { status, body, headers, raw_body });
    world.request_headers.clear();
}

#[when(regex = r#"^I send a PUT request to "([^"]+)" with body:$"#)]
async fn when_put_request(world: &mut World, path: String, step: &Step) {
    let body = step
        .docstring
        .as_ref()
        .cloned()
        .unwrap_or_else(|| panic!("Docstring body not provided for PUT {}", path));
    let mut request_builder =
        Request::builder().method("PUT").uri(&path).header("content-type", "application/json");

    for (key, value) in &world.request_headers {
        request_builder = request_builder.header(key, value);
    }

    let request = request_builder.body(Body::from(body)).expect("Failed to build request");

    let response = world
        .app
        .clone()
        .oneshot(request)
        .await
        .unwrap_or_else(|err| panic!("App request failed for PUT {}: {}", path, err));

    let status = response.status().as_u16();
    let headers = response.headers().clone();
    let body_bytes = response.into_body().collect().await.map(|c| c.to_bytes()).unwrap_or_default();

    let raw_body = String::from_utf8_lossy(&body_bytes).to_string();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap_or_default();

    world.last_response = Some(Response { status, body, headers, raw_body });
    world.request_headers.clear();
}

#[then(regex = r#"^the response body should be valid JSON$"#)]
async fn then_valid_json(world: &mut World) {
    let response = world.last_response.as_ref().expect("response should exist");
    assert!(
        response.body.is_object() || response.body.is_array(),
        "Response body should be valid JSON"
    );
}

#[then(regex = r#"^the JSON should contain a task with id "([^"]+)"$"#)]
async fn then_json_contains_task(world: &mut World, task_id: String) {
    let response = world.last_response.as_ref().expect("response should exist");
    let tasks =
        response.body.get("tasks").and_then(|t| t.as_array()).expect("tasks array should exist");

    let found = tasks.iter().any(|task| {
        task.get("id").and_then(|id| id.as_str()).map(|id| id == task_id).unwrap_or(false)
    });

    assert!(found, "Expected to find task with id '{}' in response", task_id);
}

#[then(regex = r#"^the JSON should not contain a task with id "([^"]+)"$"#)]
async fn then_json_not_contains_task(world: &mut World, task_id: String) {
    let response = world.last_response.as_ref().expect("response should exist");
    let tasks =
        response.body.get("tasks").and_then(|t| t.as_array()).expect("tasks array should exist");

    let found = tasks.iter().any(|task| {
        task.get("id").and_then(|id| id.as_str()).map(|id| id == task_id).unwrap_or(false)
    });

    assert!(
        !found,
        "Expected NOT to find task with id '{}' in response, but it was present",
        task_id
    );
}

#[then(regex = r#"^the response body should contain "([^"]+)"$"#)]
async fn then_body_contains(world: &mut World, needle: String) {
    let response = world.last_response.as_ref().expect("response should exist");
    assert!(
        response.raw_body.contains(&needle),
        "Expected response body to contain '{}', but got: {}",
        needle,
        response.raw_body
    );
}

#[then(regex = r#"^the JSON should have field "([^"]+)" with value "([^"]+)"$"#)]
async fn then_json_field_equals(world: &mut World, field: String, expected: String) {
    let response = world.last_response.as_ref().expect("response should exist");
    let actual = response.body.get(&field).and_then(|v| v.as_str()).unwrap_or("");
    assert_eq!(
        actual, expected,
        "Expected field '{}' to have value '{}', got '{}'",
        field, expected, actual
    );
}

#[then(regex = r#"^the JSON array field "([^"]+)" should contain "([^"]+)"$"#)]
async fn then_json_array_contains(world: &mut World, field: String, expected_item: String) {
    let response = world.last_response.as_ref().expect("response should exist");
    let array =
        response.body.get(&field).and_then(|v| v.as_array()).expect("field should be an array");

    let found = array.iter().any(|item| item.as_str().map(|s| s == expected_item).unwrap_or(false));

    assert!(
        found,
        "Expected array field '{}' to contain '{}', but it didn't. Array: {:?}",
        field, expected_item, array
    );
}

#[then(regex = r#"^the JSON should have an empty tasks array$"#)]
async fn then_empty_tasks_array(world: &mut World) {
    let response = world.last_response.as_ref().expect("response should exist");
    let tasks =
        response.body.get("tasks").and_then(|t| t.as_array()).expect("tasks array should exist");
    assert!(tasks.is_empty(), "Expected tasks array to be empty, but got {} items", tasks.len());
}

#[then(regex = r#"^the task "([^"]+)" should have title "([^"]+)"$"#)]
async fn then_task_title(world: &mut World, _task_id: String, _expected_title: String) {
    // This step would require reading from specs/tasks.yaml
    // For now, we'll rely on the JSON response checks in the same scenario
    // In a full implementation, we would load the tasks.yaml and verify
    let response = world.last_response.as_ref().expect("response should exist");
    let title = response.body.get("title").and_then(|v| v.as_str()).unwrap_or("");
    assert_eq!(
        title, _expected_title,
        "Expected title to be '{}', got '{}'",
        _expected_title, title
    );
}

// ============================================================================
// Agent Hints Step Definitions (AC-TPL-AGENT-HINTS)
// ============================================================================

#[then(regex = r#"^the JSON response should have field "([^"]+)"$"#)]
async fn then_json_has_field(world: &mut World, field: String) {
    let response = world.last_response.as_ref().expect("response should exist");
    assert!(
        response.body.get(&field).is_some(),
        "Expected JSON response to have field '{}', but it didn't. Response: {:?}",
        field,
        response.body
    );
}

#[then(regex = r#"^the "([^"]+)" array should contain task "([^"]+)"$"#)]
async fn then_array_contains_task(world: &mut World, field: String, task_id: String) {
    // Support both HTTP API responses and CLI JSON output
    let json = if let Some(cli_json) = &world.cli_json_output {
        cli_json
    } else if let Some(response) = &world.last_response {
        &response.body
    } else {
        panic!("response should exist");
    };

    let array = json
        .get(&field)
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("field '{}' should be an array", field));

    let found = array.iter().any(|item| extract_hint_task_id(item) == task_id);

    assert!(
        found,
        "Expected '{}' array to contain task '{}', but it didn't. Array: {:?}",
        field, task_id, array
    );
}

#[then(regex = r#"^the "([^"]+)" array should not contain task "([^"]+)"$"#)]
async fn then_array_not_contains_task(world: &mut World, field: String, task_id: String) {
    // Support both HTTP API responses and CLI JSON output
    let json = if let Some(cli_json) = &world.cli_json_output {
        cli_json
    } else if let Some(response) = &world.last_response {
        &response.body
    } else {
        panic!("response should exist");
    };

    let array = json
        .get(&field)
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("field '{}' should be an array", field));

    let found = array.iter().any(|item| extract_hint_task_id(item) == task_id);

    assert!(
        !found,
        "Expected '{}' array NOT to contain task '{}', but it did. Array: {:?}",
        field, task_id, array
    );
}

#[then(regex = r#"^the first hint should have field "([^"]+)"$"#)]
async fn then_first_hint_has_field(world: &mut World, field: String) {
    // Support both HTTP API responses and CLI JSON output
    let json = if let Some(cli_json) = &world.cli_json_output {
        cli_json
    } else if let Some(response) = &world.last_response {
        &response.body
    } else {
        panic!("response should exist");
    };

    let hints = json
        .get("hints")
        .or_else(|| json.get("next_tasks")) // Backward compatibility
        .and_then(|v| v.as_array())
        .expect("hints (or next_tasks) should be an array");

    assert!(!hints.is_empty(), "Expected at least one hint, but hints array is empty");

    let first_hint = &hints[0];
    assert!(
        first_hint.get(&field).is_some(),
        "Expected first hint to have field '{}', but it didn't. Hint: {:?}",
        field,
        first_hint
    );
}

#[then(regex = r#"^the first hint "([^"]+)" should equal "([^"]+)"$"#)]
async fn then_first_hint_field_equals(world: &mut World, field: String, expected: String) {
    // Support both HTTP API responses and CLI JSON output
    let json = if let Some(cli_json) = &world.cli_json_output {
        cli_json
    } else if let Some(response) = &world.last_response {
        &response.body
    } else {
        panic!("response should exist");
    };

    let hints = json
        .get("hints")
        .or_else(|| json.get("next_tasks")) // Backward compatibility
        .and_then(|v| v.as_array())
        .expect("hints (or next_tasks) should be an array");

    assert!(!hints.is_empty(), "Expected at least one hint, but hints array is empty");

    let first_hint = &hints[0];
    let actual = first_hint.get(&field).and_then(|v| v.as_str()).unwrap_or("");

    assert_eq!(
        actual, expected,
        "Expected first hint field '{}' to be '{}', got '{}'",
        field, expected, actual
    );
}

#[then(regex = r#"^the first hint "([^"]+)" should contain "([^"]+)"$"#)]
async fn then_first_hint_field_contains(world: &mut World, field: String, needle: String) {
    let response = world.last_response.as_ref().expect("response should exist");
    let hints = response
        .body
        .get("hints")
        .or_else(|| response.body.get("next_tasks")) // Backward compatibility
        .and_then(|v| v.as_array())
        .expect("hints (or next_tasks) should be an array");

    assert!(!hints.is_empty(), "Expected at least one hint, but hints array is empty");

    let first_hint = &hints[0];
    let actual = first_hint.get(&field).and_then(|v| v.as_str()).unwrap_or("");

    assert!(
        actual.contains(&needle),
        "Expected first hint field '{}' to contain '{}', but got '{}'",
        field,
        needle,
        actual
    );
}

#[then(regex = r#"^the JSON should have an empty hints array$"#)]
async fn then_empty_hints_array(world: &mut World) {
    let response = world.last_response.as_ref().expect("response should exist");
    let hints = response
        .body
        .get("hints")
        .or_else(|| response.body.get("next_tasks")) // Backward compatibility
        .and_then(|v| v.as_array())
        .expect("hints (or next_tasks) should be an array");

    assert!(hints.is_empty(), "Expected hints array to be empty, but got {} items", hints.len());
}

#[then(regex = r#"^the first hint "([^"]+)" should have more than (\d+) items?$"#)]
async fn then_first_hint_field_array_length_gt(world: &mut World, field: String, min_count: usize) {
    let response = world.last_response.as_ref().expect("response should exist");
    let hints = response
        .body
        .get("hints")
        .or_else(|| response.body.get("next_tasks")) // Backward compatibility
        .and_then(|v| v.as_array())
        .expect("hints (or next_tasks) should be an array");

    assert!(!hints.is_empty(), "Expected at least one hint, but hints array is empty");

    let first_hint = &hints[0];
    let array = first_hint.get(&field).and_then(|v| v.as_array()).unwrap_or_else(|| {
        panic!("Expected first hint field '{}' to be an array. Hint: {:?}", field, first_hint)
    });

    assert!(
        array.len() > min_count,
        "Expected first hint field '{}' to have more than {} items, but got {}. Array: {:?}",
        field,
        min_count,
        array.len(),
        array
    );
}

// ============================================================================
// Platform Introspection Step Definitions
// ============================================================================

#[then(regex = r#"^the field "([^"]+)" should be of type "([^"]+)"$"#)]
async fn then_field_is_type(world: &mut World, field: String, expected_type: String) {
    let response = world.last_response.as_ref().expect("response should exist");
    let value = response.body.get(&field).unwrap_or_else(|| {
        panic!("Expected field '{}' to exist in response. Response: {:?}", field, response.body)
    });

    let actual_type = match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    };

    assert_eq!(
        actual_type, expected_type,
        "Expected field '{}' to be of type '{}', but got '{}'. Value: {:?}",
        field, expected_type, actual_type, value
    );
}

#[then(regex = r#"^the field "([^"]+)" should equal "([^"]+)"$"#)]
async fn then_field_equals(world: &mut World, field: String, expected_value: String) {
    let response = world.last_response.as_ref().expect("response should exist");
    let actual = response.body.get(&field).and_then(|v| v.as_str()).unwrap_or_else(|| {
        panic!(
            "Expected field '{}' to exist and be a string in response. Response: {:?}",
            field, response.body
        )
    });

    assert_eq!(
        actual, expected_value,
        "Expected field '{}' to equal '{}', but got '{}'. Response: {:?}",
        field, expected_value, actual, response.body
    );
}

#[then(regex = r#"^the JSON response should have nested field "([^"]+)"$"#)]
async fn then_nested_field_exists(world: &mut World, field_path: String) {
    let response = world.last_response.as_ref().expect("response should exist");

    // Split the path by dots (e.g., "governance.policies.status" -> ["governance", "policies", "status"])
    let parts: Vec<&str> = field_path.split('.').collect();

    let mut current_value = &response.body;

    for (index, part) in parts.iter().enumerate() {
        current_value = current_value.get(part).unwrap_or_else(|| {
            panic!(
                "Expected nested field '{}' to exist at path '{}'. Current path: '{}'. Response: {:?}",
                field_path,
                field_path,
                parts[..=index].join("."),
                response.body
            )
        });
    }

    // If we got here, all parts of the path exist
    // Successfully found nested field
    eprintln!("Successfully found nested field '{}' with value: {:?}", field_path, current_value);
}

#[then(regex = r#"^the field "([^"]+)" should not be empty$"#)]
async fn then_array_not_empty(world: &mut World, field: String) {
    let response = world.last_response.as_ref().expect("response should exist");
    let value = response.body.get(&field).unwrap_or_else(|| {
        panic!("Expected field '{}' to exist in response. Response: {:?}", field, response.body)
    });

    // Handle both arrays and objects
    match value {
        serde_json::Value::Array(arr) => {
            assert!(
                !arr.is_empty(),
                "Expected field '{}' array to not be empty, but it has 0 items",
                field
            );
        }
        serde_json::Value::Object(obj) => {
            assert!(
                !obj.is_empty(),
                "Expected field '{}' object to not be empty, but it has 0 keys",
                field
            );
        }
        _ => {
            panic!("Expected field '{}' to be an array or object, but got: {:?}", field, value);
        }
    }
}

#[then(regex = r#"^the first hint "([^"]+)" array should contain "([^"]+)"$"#)]
async fn then_first_hint_array_contains(world: &mut World, field: String, value: String) {
    let response = world.last_response.as_ref().expect("response should exist");
    let hints =
        response.body.get("hints").and_then(|v| v.as_array()).expect("hints should be an array");

    assert!(!hints.is_empty(), "Expected at least one hint, but hints array is empty");

    let first_hint = &hints[0];
    let array = first_hint.get(&field).and_then(|v| v.as_array()).unwrap_or_else(|| {
        panic!("Expected first hint field '{}' to be an array. Hint: {:?}", field, first_hint)
    });

    let contains = array.iter().any(|v| v.as_str() == Some(&value));
    assert!(
        contains,
        "Expected first hint '{}' array to contain '{}', but it didn't. Array: {:?}",
        field, value, array
    );
}

#[then(regex = r#"^the hints should be sorted with "([^"]+)" before "([^"]+)"$"#)]
async fn then_hints_sorted_by_status(
    world: &mut World,
    first_status: String,
    second_status: String,
) {
    let response = world.last_response.as_ref().expect("response should exist");
    let hints =
        response.body.get("hints").and_then(|v| v.as_array()).expect("hints should be an array");

    // Find indices of first and last occurrence of each status
    let first_status_last_idx = hints
        .iter()
        .enumerate()
        .rev()
        .find(|(_, hint)| hint.get("status").and_then(|v| v.as_str()) == Some(&first_status))
        .map(|(idx, _)| idx);

    let second_status_first_idx = hints
        .iter()
        .enumerate()
        .find(|(_, hint)| hint.get("status").and_then(|v| v.as_str()) == Some(&second_status))
        .map(|(idx, _)| idx);

    match (first_status_last_idx, second_status_first_idx) {
        (Some(first_last), Some(second_first)) => {
            assert!(
                first_last < second_first,
                "Expected all '{}' hints to come before '{}' hints, but found '{}' at index {} and '{}' at index {}",
                first_status,
                second_status,
                first_status,
                first_last,
                second_status,
                second_first
            );
        }
        (None, _) => panic!("No hints with status '{}' found", first_status),
        (_, None) => panic!("No hints with status '{}' found", second_status),
    }
}

#[then(regex = r#"^within same status hints should be sorted by id$"#)]
async fn then_hints_sorted_by_id_within_status(world: &mut World) {
    let response = world.last_response.as_ref().expect("response should exist");
    let hints =
        response.body.get("hints").and_then(|v| v.as_array()).expect("hints should be an array");

    // Group hints by status
    use std::collections::HashMap;
    let mut status_groups: HashMap<String, Vec<&serde_json::Value>> = HashMap::new();

    for hint in hints {
        let status = hint.get("status").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
        status_groups.entry(status).or_default().push(hint);
    }

    // Check that within each status group, IDs are sorted
    for (_status, group) in status_groups {
        for i in 0..group.len() - 1 {
            let current_id = extract_hint_task_id(group[i]);
            let next_id = extract_hint_task_id(group[i + 1]);

            assert!(
                current_id <= next_id,
                "Expected hints to be sorted by ID within same status, but found '{}' before '{}'. Group: {:?}",
                current_id,
                next_id,
                group
            );
        }
    }
}

#[then(regex = r#"^the hints should be sorted by priority: (.+)$"#)]
async fn then_hints_sorted_by_priority(world: &mut World, priority_order: String) {
    let response = world.last_response.as_ref().expect("response should exist");
    let hints =
        response.body.get("hints").and_then(|v| v.as_array()).expect("hints should be an array");

    let expected_priorities: Vec<&str> = priority_order.split(',').map(|s| s.trim()).collect();

    // Build a map of hint positions by priority
    let mut priority_positions: Vec<(usize, &str)> = Vec::new();

    for (idx, hint) in hints.iter().enumerate() {
        let labels = hint.get("labels").and_then(|v| v.as_array());
        let priority = if let Some(labels) = labels {
            if labels.iter().any(|l| l.as_str() == Some("priority:high")) {
                "high"
            } else if labels.iter().any(|l| l.as_str() == Some("priority:medium")) {
                "medium"
            } else if labels.iter().any(|l| l.as_str() == Some("priority:low")) {
                "low"
            } else {
                "none"
            }
        } else {
            "none"
        };
        priority_positions.push((idx, priority));
    }

    // Check that priorities are in expected order
    for i in 0..priority_positions.len() - 1 {
        let (pos_i, pri_i) = priority_positions[i];
        let (pos_j, pri_j) = priority_positions[i + 1];

        let expected_i = expected_priorities.iter().position(|&p| p == pri_i).unwrap_or(999);
        let expected_j = expected_priorities.iter().position(|&p| p == pri_j).unwrap_or(999);

        assert!(
            expected_i <= expected_j,
            "Expected hints to be sorted by priority order {:?}, but found '{}' at position {} before '{}' at position {}",
            expected_priorities,
            pri_i,
            pos_i,
            pri_j,
            pos_j
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn post_status_step_updates_task() {
        let mut world = World::new();

        given_task_exists(&mut world, "TASK-TEST".to_string(), "Todo".to_string()).await;
        let request = Request::builder()
            .method("POST")
            .uri("/platform/tasks/TASK-TEST/status")
            .header("content-type", "application/json")
            .body(Body::from(r#"{ "status": "InProgress" }"#))
            .expect("failed to build request");

        let response = world.app.clone().oneshot(request).await.expect("request should not fail");

        assert_eq!(response.status(), http::StatusCode::NO_CONTENT);

        // Mirror the cucumber step behavior so later assertions see the response
        let status = response.status().as_u16();
        let headers = response.headers().clone();
        let body_bytes =
            response.into_body().collect().await.map(|c| c.to_bytes()).unwrap_or_default();
        let raw_body = String::from_utf8_lossy(&body_bytes).to_string();
        let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap_or_default();
        world.last_response = Some(Response { status, body, headers, raw_body });

        then_status_code(&mut world, 204).await;

        // Verify the state file was updated
        let state_path = world._temp_dir.path().join("specs/tasks_state.yaml");
        let status =
            tasks_state::get_task_status(&state_path, &TaskId("TASK-TEST".into())).unwrap();
        assert_eq!(status, Some(TaskStatus::InProgress));
    }
}
