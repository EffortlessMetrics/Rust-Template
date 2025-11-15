use crate::world::{Order, Response, World};
use axum::body::Body;
use cucumber::{given, then, when};
use http::Request;
use http_body_util::BodyExt;
use tower::util::ServiceExt; // for oneshot

#[given(regex = r#"^an order "([^"]+)" totalling (\d+) cents$"#)]
async fn given_an_order(world: &mut World, order_id: String, total_cents: String) {
    let total_cents = total_cents.parse::<u64>().expect("valid amount");
    world.orders.insert(order_id.clone(), Order { id: order_id, total_cents });
}

#[when(regex = r#"^I POST /refunds with \{ "orderId": "([^"]+)", "amountCents": (\d+) \}$"#)]
async fn when_post_refunds(world: &mut World, order_id: String, amount_cents: String) {
    let amount_cents = amount_cents.parse::<u64>().expect("valid amount");

    // Verify order exists (test precondition), unless testing error scenarios
    // For error testing, we skip this check
    if let Some(_order) = world.orders.get(&order_id) {
        // Order exists, proceed normally
    }
    // If order doesn't exist, we'll still send the request (for error testing)

    // Build request JSON
    let request_body = serde_json::json!({
        "orderId": order_id,
        "amountCents": amount_cents
    });

    // Make real HTTP call through the router (in-process, no network)
    let mut request_builder = Request::builder()
        .method("POST")
        .uri("/refunds")
        .header("content-type", "application/json");

    // Add any request headers from world
    for (key, value) in &world.request_headers {
        request_builder = request_builder.header(key, value);
    }

    let request = request_builder
        .body(Body::from(request_body.to_string()))
        .expect("valid request");

    // Call the router - this is the REAL HTTP stack!
    let response = world.app.clone().oneshot(request).await.expect("request should succeed");

    // Extract status, headers, and body
    let status = response.status().as_u16();
    let headers = response.headers().clone();
    let body_bytes =
        response.into_body().collect().await.expect("body should be readable").to_bytes();

    let body: serde_json::Value =
        serde_json::from_slice(&body_bytes).expect("body should be valid JSON");

    world.last_response = Some(Response { status, body, headers });
    world.request_headers.clear();
}

#[then(regex = r#"^I receive (\d+) with a "([^"]+)"$"#)]
async fn then_receive_status_with_field(world: &mut World, status: String, field: String) {
    let expected_status = status.parse::<u16>().expect("valid status");

    let response = world.last_response.as_ref().expect("should have a response");

    assert_eq!(
        response.status, expected_status,
        "Expected status {}, got {}",
        expected_status, response.status
    );

    assert!(
        response.body.get(&field).is_some(),
        "Response should contain field '{}'. Body: {}",
        field,
        response.body
    );
}
