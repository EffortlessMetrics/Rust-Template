use axum::Router;
use cucumber::World as CucumberWorld;
use http::HeaderMap;

/// Test world state - includes real HTTP router for integration testing
#[derive(Debug, CucumberWorld)]
pub struct World {
    /// Real HTTP router (in-process, no network)
    pub app: Router,
    /// Last HTTP response
    pub last_response: Option<Response>,
    /// Request headers to be sent with next request
    pub request_headers: HeaderMap,
}

impl Default for World {
    fn default() -> Self {
        // Initialize telemetry for tests (idempotent)
        telemetry::init();

        Self {
            app: app_http::app(), // Real HTTP router from app-http crate
            last_response: None,
            request_headers: HeaderMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub body: serde_json::Value,
    pub headers: HeaderMap,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }
}
