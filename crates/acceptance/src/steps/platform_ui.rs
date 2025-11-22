use crate::world::{Response, World};
use axum::body::Body;
use cucumber::{given, then, when};
use http::Request;
use http_body_util::BodyExt;
use tower::util::ServiceExt;

// ============================================================================
// Platform UI Step Definitions
// ============================================================================

#[given("the platform is running with UI enabled")]
async fn given_platform_running(_world: &mut World) {
    // The platform is always running in tests via World::default()
    // UI is enabled by default in the app router
}

#[when(regex = r#"^I GET "([^"]+)"$"#)]
async fn when_get_url(world: &mut World, url: String) {
    // Extract path from URL (strip http://localhost:8080 if present)
    let path = url.strip_prefix("http://localhost:8080").unwrap_or(&url).to_string();

    let mut request_builder = Request::builder().method("GET").uri(&path);

    // Add request headers if any
    for (key, value) in &world.request_headers {
        request_builder = request_builder.header(key, value);
    }

    let request = request_builder
        .body(Body::empty())
        .map_err(|e| tracing::warn!("Invalid request: {}", e))
        .unwrap_or_else(|_| Request::builder().uri("/").body(Body::empty()).unwrap());

    // Call the router - this is the REAL HTTP stack!
    let response = world.app.clone().oneshot(request).await.unwrap_or_else(|e| {
        tracing::warn!("App request failed: {}", e);
        use http::Response as HttpResponse;
        HttpResponse::builder().status(500).body(Body::empty()).unwrap()
    });

    // Extract status, headers, and body
    let status = response.status().as_u16();
    let headers = response.headers().clone();
    let body_bytes = response.into_body().collect().await.map(|c| c.to_bytes()).unwrap_or_default();

    let raw_body = String::from_utf8_lossy(&body_bytes).to_string();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap_or_default();

    world.last_response = Some(Response { status, body, headers, raw_body });
    // Clear request headers after use
    world.request_headers.clear();
}

#[then(regex = r"^the response status should be (\d+)$")]
async fn then_response_status(world: &mut World, expected_status: String) {
    let expected = expected_status.parse::<u16>().expect("Status code should be a valid number");
    let response =
        world.last_response.as_ref().expect("No response available - did a request step run?");

    assert_eq!(response.status, expected, "Expected status {}, got {}", expected, response.status);
}

#[then(regex = r#"^the response content-type should be "([^"]+)"$"#)]
async fn then_content_type(world: &mut World, expected_content_type: String) {
    let response =
        world.last_response.as_ref().expect("No response available - did a request step run?");

    let content_type =
        response.headers.get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("");

    // Allow partial match for content-type (e.g., "text/html" matches "text/html; charset=utf-8")
    assert!(
        content_type.starts_with(&expected_content_type),
        "Expected content-type to start with '{}', got '{}'",
        expected_content_type,
        content_type
    );
}
