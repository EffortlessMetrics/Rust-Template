use crate::world::{Response, World};
use axum::body::Body;
use cucumber::{given, then, when};
use http::Request;
use http_body_util::BodyExt;
use tower::util::ServiceExt;

// ============================================================================
// Template Core Step Definitions - Keep these in your service
// ============================================================================

#[when(regex = r"^I GET (/health|/version)$")]
async fn when_get_endpoint(world: &mut World, path: String) {
    let mut request_builder = Request::builder().method("GET").uri(&path);

    // Add request headers if any
    for (key, value) in &world.request_headers {
        request_builder = request_builder.header(key, value);
    }

    let request = request_builder.body(Body::empty()).map_err(|e| tracing::warn!("Invalid request: {}", e)).unwrap_or_else(|_| Request::builder().uri("/").body(Body::empty()).unwrap());

    // Call the router - this is the REAL HTTP stack!
    let response = world.app.clone().oneshot(request).await.unwrap_or_else(|e| {
        tracing::warn!("App request failed: {}", e);
        use http::Response;
        Response::builder().status(500).body(Body::empty()).unwrap()
    });

    // Extract status, headers, and body
    let status = response.status().as_u16();
    let headers = response.headers().clone();
    let body_bytes = response.into_body().collect().await.map(|c| c.to_bytes()).unwrap_or_default();

    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap_or_default();

    world.last_response = Some(Response { status, body, headers });
    // Clear request headers after use
    world.request_headers.clear();
}

#[then(regex = r#"^I receive (\d+) with status "([^"]+)"$"#)]
async fn then_receive_with_status(world: &mut World, status_code: String, status_value: String) {
    let status_code = status_code.parse::<u16>().unwrap_or(0);
    let response = world.last_response.as_ref().expect("response should exist");

    assert_eq!(
        response.status, status_code,
        "Expected status {}, got {}",
        status_code, response.status
    );

    let actual_status =
        response.body.get("status").and_then(|v| v.as_str()).unwrap_or("");

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
    let status_code = status_code.parse::<u16>().unwrap_or(0);
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

// ============================================================================
// Error Envelope Step Definitions (AC-TPL-003, AC-TPL-004)
// ============================================================================

#[when(regex = r#"^I POST /api/echo with invalid data \{ "message": "([^"]*)" \}$"#)]
async fn when_post_echo_invalid(world: &mut World, message: String) {
    let request_body = serde_json::json!({
        "message": message
    });

    let mut request_builder = Request::builder()
        .method("POST")
        .uri("/api/echo")
        .header("content-type", "application/json");

    // Add any request headers from world
    for (key, value) in &world.request_headers {
        request_builder = request_builder.header(key, value);
    }

    let request = request_builder.body(Body::from(request_body.to_string())).map_err(|e| tracing::warn!("Invalid request: {}", e)).unwrap_or_else(|_| Request::builder().uri("/").body(Body::empty()).unwrap());

    let response = world.app.clone().oneshot(request).await.expect("request should succeed");

    let status = response.status().as_u16();
    let headers = response.headers().clone();
    let body_bytes = response.into_body().collect().await.map(|c| c.to_bytes()).unwrap_or_default();

    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap_or_default();

    world.last_response = Some(Response { status, body, headers });
    world.request_headers.clear();
}

#[then(regex = r"^I receive a (\d+)xx response$")]
async fn then_receive_status_range(world: &mut World, status_range: String) {
    let response = world.last_response.as_ref().expect("response should exist");
    let expected_range = status_range.parse::<u16>().unwrap_or(0) * 100;
    assert!(
        response.status >= expected_range && response.status < expected_range + 100,
        "Expected {}xx status, got {}",
        status_range,
        response.status
    );
}

#[then(regex = r"^I receive a (\d+) response$")]
async fn then_receive_exact_status(world: &mut World, status_code: String) {
    let expected = status_code.parse::<u16>().unwrap_or(0);
    let response = world.last_response.as_ref().expect("response should exist");
    assert_eq!(response.status, expected, "Expected status {}, got {}", expected, response.status);
}

#[then(regex = r#"^the response body contains "([^"]+)" field$"#)]
async fn then_response_has_field(world: &mut World, field: String) {
    let response = world.last_response.as_ref().expect("response should exist");
    assert!(
        response.body.get(&field).is_some(),
        "Response body should contain '{}' field. Body: {}",
        field,
        response.body
    );
}

#[then(regex = r#"^the response includes "([^"]+)" header$"#)]
async fn then_response_has_header(world: &mut World, header_name: String) {
    let response = world.last_response.as_ref().expect("response should exist");
    let header_name_lower = header_name.to_lowercase();
    assert!(
        response.headers.get(&header_name_lower).is_some(),
        "Response should include '{}' header. Headers: {:?}",
        header_name,
        response.headers
    );
}

#[then(regex = r#"^the "([^"]+)" field in response body matches the "([^"]+)" header$"#)]
async fn then_body_field_matches_header(
    world: &mut World,
    body_field: String,
    header_name: String,
) {
    let response = world.last_response.as_ref().expect("response should exist");

    let body_value = response
        .body
        .get(&body_field)
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let header_value = response
        .headers
        .get(header_name.to_lowercase())
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    assert_eq!(
        body_value, header_value,
        "Body field '{}' value '{}' should match header '{}' value '{}'",
        body_field, body_value, header_name, header_value
    );
}

#[given(regex = r#"^I set "([^"]+)" header to "([^"]+)"$"#)]
async fn given_set_header(world: &mut World, header_name: String, header_value: String) {
    use http::HeaderValue;
    use http::header::HeaderName;

    let name = HeaderName::from_bytes(header_name.as_bytes()).expect("valid header name");
    let value = HeaderValue::from_str(&header_value).expect("valid header value");
    world.request_headers.insert(name, value);
}

#[then(regex = r#"^the response includes "([^"]+)" header with value "([^"]+)"$"#)]
async fn then_response_header_equals(
    world: &mut World,
    header_name: String,
    expected_value: String,
) {
    let response = world.last_response.as_ref().unwrap_or(&Response { status: 0, body: serde_json::json!({}), headers: http::HeaderMap::new() });
    let header_value = response
        .headers
        .get(header_name.to_lowercase())
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    assert_eq!(
        header_value, expected_value,
        "Expected header '{}' to be '{}', got '{}'",
        header_name, expected_value, header_value
    );
}

#[then(regex = r#"^the "([^"]+)" field in response body equals "([^"]+)"$"#)]
async fn then_body_field_equals(world: &mut World, field_name: String, expected_value: String) {
    let response = world.last_response.as_ref().unwrap_or(&Response { status: 0, body: serde_json::json!({}), headers: http::HeaderMap::new() });
    let actual_value = response
        .body
        .get(&field_name)
        .and_then(|v| v.as_str())
        .unwrap_or("");

    assert_eq!(
        actual_value, expected_value,
        "Expected field '{}' to be '{}', got '{}'",
        field_name, expected_value, actual_value
    );
}

#[then(regex = r#"^the "([^"]+)" header is a valid UUID or request identifier$"#)]
async fn then_header_is_uuid(world: &mut World, header_name: String) {
    let response = world.last_response.as_ref().unwrap_or(&Response { status: 0, body: serde_json::json!({}), headers: http::HeaderMap::new() });
    let header_value = response
        .headers
        .get(header_name.to_lowercase())
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Check if it's a valid UUID or at least a non-empty identifier
    assert!(
        !header_value.is_empty() && header_value.len() >= 8,
        "Header '{}' should be a valid request identifier, got '{}'",
        header_name,
        header_value
    );

    // Optionally, try to parse as UUID if it looks like one
    if header_value.contains('-') && header_value.len() == 36 {
        use uuid::Uuid;
        assert!(
            Uuid::parse_str(header_value).is_ok(),
            "Header '{}' looks like a UUID but failed to parse: '{}'",
            header_name,
            header_value
        );
    }
}
