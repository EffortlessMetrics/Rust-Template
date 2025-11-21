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
