use crate::world::{Response, World};
use axum::body::Body;
use cucumber::{then, when};
use http::Request;
use http_body_util::BodyExt;
use tower::util::ServiceExt;

// ============================================================================
// Template Core Step Definitions - Keep these in your service
// ============================================================================

#[when(regex = r"^I GET (/health|/version)$")]
async fn when_get_endpoint(world: &mut World, path: String) {
    let request =
        Request::builder().method("GET").uri(&path).body(Body::empty()).expect("valid request");

    // Call the router - this is the REAL HTTP stack!
    let response = world.app.clone().oneshot(request).await.expect("request should succeed");

    // Extract status and body
    let status = response.status().as_u16();
    let body_bytes =
        response.into_body().collect().await.expect("body should be readable").to_bytes();

    let body: serde_json::Value =
        serde_json::from_slice(&body_bytes).expect("body should be valid JSON");

    world.last_response = Some(Response { status, body });
}

#[then(regex = r#"^I receive (\d+) with status "([^"]+)"$"#)]
async fn then_receive_with_status(world: &mut World, status_code: String, status_value: String) {
    let status_code = status_code.parse::<u16>().expect("valid status code");
    let response = world.last_response.as_ref().expect("response should exist");

    assert_eq!(
        response.status, status_code,
        "Expected status {}, got {}",
        status_code, response.status
    );

    let actual_status =
        response.body.get("status").and_then(|v| v.as_str()).expect("status field should exist");

    assert_eq!(
        actual_status, status_value,
        "Expected status '{}', got '{}'",
        status_value, actual_status
    );
}

#[then(regex = r#"^I receive (\d+) with JSON containing "([^"]+)" and "([^"]+)"$"#)]
async fn then_receive_with_fields(
    world: &mut World,
    status_code: String,
    field1: String,
    field2: String,
) {
    let status_code = status_code.parse::<u16>().expect("valid status code");
    let response = world.last_response.as_ref().expect("response should exist");

    assert_eq!(
        response.status, status_code,
        "Expected status {}, got {}",
        status_code, response.status
    );

    assert!(
        response.body.get(&field1).is_some(),
        "Expected field '{}' in response, got: {:?}",
        field1,
        response.body
    );

    assert!(
        response.body.get(&field2).is_some(),
        "Expected field '{}' in response, got: {:?}",
        field2,
        response.body
    );
}
