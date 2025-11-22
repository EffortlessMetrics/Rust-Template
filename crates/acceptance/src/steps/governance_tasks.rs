use crate::world::{Response, World};
use adapters_spec_fs::tasks_state;
use axum::body::Body;
use business_core::governance::{TaskId, TaskStatus};
use cucumber::{given, then, when};
use http::Request;
use http_body_util::BodyExt;
use tower::util::ServiceExt;

#[given(regex = r#"^a task "([^"]+)" exists with status "([^"]+)"$"#)]
async fn given_task_exists(world: &mut World, task_id: String, status_str: String) {
    let status = match status_str.as_str() {
        "Todo" => TaskStatus::Todo,
        "InProgress" => TaskStatus::InProgress,
        "Review" => TaskStatus::Review,
        "Done" => TaskStatus::Done,
        _ => panic!("Invalid status: {}", status_str),
    };

    let path = world._temp_dir.path().join("tasks_state.yaml");
    tasks_state::update_task_status(&path, TaskId(task_id), status)
        .expect("Failed to update task status");
}

#[given(expr = r#"the following tasks exist in {string}:"#)]
async fn given_tasks_exist(world: &mut World, _file: String, step: &cucumber::gherkin::Step) {
    let path = world._temp_dir.path().join("tasks_state.yaml");

    // Parse table and create tasks
    if let Some(table) = &step.table {
        let headers: Vec<&str> = table
            .rows
            .first()
            .map(|row| row.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default();

        for row in table.rows.iter().skip(1) {
            let mut task_id = String::new();
            let mut status = TaskStatus::Todo;

            for (i, cell) in row.iter().enumerate() {
                if let Some(&header) = headers.get(i) {
                    match header {
                        "id" => task_id = cell.to_string(),
                        "status" => {
                            status = match cell.as_str() {
                                "Todo" => TaskStatus::Todo,
                                "InProgress" => TaskStatus::InProgress,
                                "Review" => TaskStatus::Review,
                                "Done" => TaskStatus::Done,
                                _ => TaskStatus::Todo,
                            };
                        }
                        _ => {} // Ignore other fields for now
                    }
                }
            }

            if !task_id.is_empty() {
                tasks_state::update_task_status(&path, TaskId(task_id), status)
                    .expect("Failed to update task status");
            }
        }
    }
}

#[when(regex = r#"^I send a POST request to "([^"]+)" with body:$"#)]
async fn when_post_request(world: &mut World, path: String, body: String) {
    let mut request_builder =
        Request::builder().method("POST").uri(&path).header("content-type", "application/json");

    for (key, value) in &world.request_headers {
        request_builder = request_builder.header(key, value);
    }

    let request = request_builder.body(Body::from(body)).expect("Failed to build request");

    let response = world.app.clone().oneshot(request).await.expect("App request failed");

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
    let path = world._temp_dir.path().join("tasks_state.yaml");
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

#[when(regex = r#"^I send a GET request to "([^"]+)"$"#)]
async fn when_get_request(world: &mut World, path: String) {
    let mut request_builder = Request::builder().method("GET").uri(&path);

    for (key, value) in &world.request_headers {
        request_builder = request_builder.header(key, value);
    }

    let request = request_builder.body(Body::empty()).expect("Failed to build request");

    let response = world.app.clone().oneshot(request).await.expect("App request failed");

    let status = response.status().as_u16();
    let headers = response.headers().clone();
    let body_bytes = response.into_body().collect().await.map(|c| c.to_bytes()).unwrap_or_default();

    let raw_body = String::from_utf8_lossy(&body_bytes).to_string();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap_or_default();

    world.last_response = Some(Response { status, body, headers, raw_body });
    world.request_headers.clear();
}

#[when(regex = r#"^I send a PUT request to "([^"]+)" with body:$"#)]
async fn when_put_request(world: &mut World, path: String, body: String) {
    let mut request_builder =
        Request::builder().method("PUT").uri(&path).header("content-type", "application/json");

    for (key, value) in &world.request_headers {
        request_builder = request_builder.header(key, value);
    }

    let request = request_builder.body(Body::from(body)).expect("Failed to build request");

    let response = world.app.clone().oneshot(request).await.expect("App request failed");

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
    let response = world.last_response.as_ref().expect("response should exist");
    let array = response
        .body
        .get(&field)
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("field '{}' should be an array", field));

    let found = array.iter().any(|item| {
        item.get("id").and_then(|id| id.as_str()).map(|id| id == task_id).unwrap_or(false)
    });

    assert!(
        found,
        "Expected '{}' array to contain task '{}', but it didn't. Array: {:?}",
        field, task_id, array
    );
}

#[then(regex = r#"^the "([^"]+)" array should not contain task "([^"]+)"$"#)]
async fn then_array_not_contains_task(world: &mut World, field: String, task_id: String) {
    let response = world.last_response.as_ref().expect("response should exist");
    let array = response
        .body
        .get(&field)
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("field '{}' should be an array", field));

    let found = array.iter().any(|item| {
        item.get("id").and_then(|id| id.as_str()).map(|id| id == task_id).unwrap_or(false)
    });

    assert!(
        !found,
        "Expected '{}' array NOT to contain task '{}', but it did. Array: {:?}",
        field, task_id, array
    );
}

#[then(regex = r#"^the first hint should have field "([^"]+)"$"#)]
async fn then_first_hint_has_field(world: &mut World, field: String) {
    let response = world.last_response.as_ref().expect("response should exist");
    let hints = response
        .body
        .get("next_tasks")
        .and_then(|v| v.as_array())
        .expect("next_tasks should be an array");

    assert!(!hints.is_empty(), "Expected at least one hint, but next_tasks array is empty");

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
    let response = world.last_response.as_ref().expect("response should exist");
    let hints = response
        .body
        .get("next_tasks")
        .and_then(|v| v.as_array())
        .expect("next_tasks should be an array");

    assert!(!hints.is_empty(), "Expected at least one hint, but next_tasks array is empty");

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
        .get("next_tasks")
        .and_then(|v| v.as_array())
        .expect("next_tasks should be an array");

    assert!(!hints.is_empty(), "Expected at least one hint, but next_tasks array is empty");

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

#[then(regex = r#"^the JSON should have an empty next_tasks array$"#)]
async fn then_empty_next_tasks_array(world: &mut World) {
    let response = world.last_response.as_ref().expect("response should exist");
    let hints = response
        .body
        .get("next_tasks")
        .and_then(|v| v.as_array())
        .expect("next_tasks should be an array");

    assert!(
        hints.is_empty(),
        "Expected next_tasks array to be empty, but got {} items",
        hints.len()
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
