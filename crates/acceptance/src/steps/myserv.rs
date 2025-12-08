//! Step definitions for MYSERV domain features (AC-MYSERV-001, AC-MYSERV-002, AC-MYSERV-003, AC-MYSERV-004)
//!
//! This module implements BDD steps for the todo management feature.

use crate::world::World;
use cucumber::{given, then, when};

// ============================================================================
// Given Steps - Test Data Setup
// ============================================================================

#[given(regex = r"^the user has existing todos$")]
async fn given_user_has_todos(_world: &mut World) {
    // The todos handler initializes with sample data,
    // so we don't need to do anything here. This step
    // documents the precondition.
}

#[given(regex = r"^the user has no todos$")]
async fn given_user_has_no_todos(world: &mut World) {
    // For AC-MYSERV-002: Clear all todos to start with empty list
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    let request = Request::builder()
        .method("DELETE")
        .uri("/todos/clear")
        .body(Body::empty())
        .expect("Failed to build DELETE request");

    let response = world
        .app
        .clone()
        .oneshot(request)
        .await
        .expect("Failed to send DELETE request to clear todos");

    assert_eq!(response.status().as_u16(), 204, "Expected 204 No Content from /todos/clear");
}

#[given(regex = r"^the service is running$")]
async fn given_service_running(_world: &mut World) {
    // The World initializes the app router on creation,
    // so this is a no-op that documents the precondition.
}

// ============================================================================
// When Steps - HTTP Actions
// ============================================================================

// Note: Common HTTP steps like "I send a GET request to" and "I send a POST request to"
// are defined in governance_tasks.rs and are reusable across all features.

#[when(regex = r#"^I send a DELETE request to "([^"]+)"$"#)]
async fn when_delete_request(world: &mut World, path: String) {
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    let request = Request::builder()
        .method("DELETE")
        .uri(&path)
        .body(Body::empty())
        .expect("Failed to build DELETE request");

    let response = world.app.clone().oneshot(request).await.expect("Failed to send DELETE request");

    let status = response.status();
    let headers = response.headers().clone();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");

    let body_str = String::from_utf8_lossy(&body_bytes);
    let body_json = if body_str.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_str(&body_str).unwrap_or(serde_json::Value::String(body_str.to_string()))
    };

    world.last_response = Some(crate::world::Response {
        status: status.as_u16(),
        body: body_json,
        headers,
        raw_body: body_str.to_string(),
    });
}

#[when(regex = r#"^I send a POST request to "([^"]+)" with invalid JSON$"#)]
async fn when_post_invalid_json(world: &mut World, path: String) {
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    let invalid_json = "{this is not valid json}";

    let request = Request::builder()
        .method("POST")
        .uri(&path)
        .header("content-type", "application/json")
        .body(Body::from(invalid_json))
        .expect("Failed to build POST request");

    let response = world.app.clone().oneshot(request).await.expect("Failed to send POST request");

    let status = response.status();
    let headers = response.headers().clone();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");

    let body_str = String::from_utf8_lossy(&body_bytes);
    let body_json =
        serde_json::from_str(&body_str).unwrap_or(serde_json::Value::String(body_str.to_string()));

    world.last_response = Some(crate::world::Response {
        status: status.as_u16(),
        body: body_json,
        headers,
        raw_body: body_str.to_string(),
    });
}

// ============================================================================
// Then Steps - MYSERV-specific Response Assertions
// ============================================================================

#[then(regex = r"^the response should be a JSON array$")]
async fn then_response_is_json_array(world: &mut World) {
    let response = world.last_response.as_ref().expect("No response available");

    assert!(
        response.body.is_array(),
        "Expected response to be a JSON array, got: {}",
        response.body
    );
}

#[then(regex = r"^the response array should be empty$")]
async fn then_response_array_is_empty(world: &mut World) {
    let response = world.last_response.as_ref().expect("No response available");

    let array = response.body.as_array().expect("Response should be an array");
    assert!(array.is_empty(), "Expected empty array, got {} items", array.len());
}

#[then(regex = r"^the response should contain an error message$")]
async fn then_response_has_error(world: &mut World) {
    let response = world.last_response.as_ref().expect("No response available");

    // Check if response is an object with an error field
    // or if it's a string error message
    let has_error = response.body.is_object()
        && (response.body.get("error").is_some()
            || response.body.get("message").is_some()
            || response.body.get("detail").is_some());

    assert!(
        has_error || response.body.is_string(),
        "Expected response to contain an error message, got: {}",
        response.body
    );
}

#[then(regex = r#"^the todo with id "([^"]+)" should not be in the list$"#)]
async fn then_todo_not_in_list(world: &mut World, todo_id: String) {
    let response = world.last_response.as_ref().expect("No response available");

    let todos = response.body.as_array().expect("Response should be an array");

    for todo in todos {
        if let Some(id) = todo.get("id").and_then(|v| v.as_str()) {
            assert_ne!(id, todo_id, "Todo with id '{}' should not be in the list", todo_id);
        }
    }
}

#[then(regex = r#"^each todo should have an? "([^"]+)" and "([^"]+)" field$"#)]
async fn then_each_todo_has_fields(world: &mut World, field1: String, field2: String) {
    let response = world.last_response.as_ref().expect("No response available");

    let todos = response.body.as_array().expect("Response should be an array for this step");

    assert!(!todos.is_empty(), "Expected at least one todo in the array");

    for (index, todo) in todos.iter().enumerate() {
        assert!(
            todo.get(&field1).is_some(),
            "Todo at index {} should have '{}' field. Todo: {}",
            index,
            field1,
            todo
        );

        assert!(
            todo.get(&field2).is_some(),
            "Todo at index {} should have '{}' field. Todo: {}",
            index,
            field2,
            todo
        );
    }
}
