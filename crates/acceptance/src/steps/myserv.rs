//! Step definitions for MYSERV domain features (AC-MYSERV-001)
//!
//! This module implements BDD steps for the todo management feature.

use crate::world::World;
use cucumber::{given, then};

// ============================================================================
// Given Steps - Test Data Setup
// ============================================================================

#[given(regex = r"^the user has existing todos$")]
async fn given_user_has_todos(_world: &mut World) {
    // The todos handler initializes with sample data,
    // so we don't need to do anything here. This step
    // documents the precondition.
}

#[given(regex = r"^the service is running$")]
async fn given_service_running(_world: &mut World) {
    // The World initializes the app router on creation,
    // so this is a no-op that documents the precondition.
}

// Note: Common HTTP steps like "I send a GET request to" and
// "the response status should be" are defined in governance_tasks.rs
// and platform_ui.rs respectively, and are reusable across all features.

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
