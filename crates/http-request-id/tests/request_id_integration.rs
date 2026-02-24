use axum::{Router, body::Body, http::Request, routing::get};
use tower::util::ServiceExt;
use uuid::Uuid;

use http_request_id::{REQUEST_ID_HEADER, request_id_middleware};

#[tokio::test]
async fn existing_request_id_is_echoed_in_response() {
    let app = Router::new()
        .route("/", get(|| async { "ok" }))
        .layer(axum::middleware::from_fn(request_id_middleware));

    let request = Request::builder()
        .uri("/")
        .header(REQUEST_ID_HEADER, "integration-trace-id")
        .body(Body::empty())
        .expect("valid request");

    let response = app.oneshot(request).await.expect("response");
    let value =
        response.headers().get(REQUEST_ID_HEADER).expect("request id header").to_str().unwrap();
    assert_eq!(value, "integration-trace-id");
}

#[tokio::test]
async fn missing_request_id_is_generated() {
    let app = Router::new()
        .route("/", get(|| async { "ok" }))
        .layer(axum::middleware::from_fn(request_id_middleware));

    let request = Request::builder().uri("/").body(Body::empty()).expect("valid request");
    let response = app.oneshot(request).await.expect("response");
    let value =
        response.headers().get(REQUEST_ID_HEADER).expect("request id header").to_str().unwrap();
    assert!(Uuid::parse_str(value).is_ok());
}
