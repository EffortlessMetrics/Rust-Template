use axum::Router;
use cucumber::World as CucumberWorld;
use std::collections::HashMap;

/// Test world state - includes real HTTP router for integration testing
#[derive(Debug, CucumberWorld)]
pub struct World {
    /// Real HTTP router (in-process, no network)
    pub app: Router,
    /// Orders in the system (test data)
    pub orders: HashMap<String, Order>,
    /// Last HTTP response
    pub last_response: Option<Response>,
}

impl Default for World {
    fn default() -> Self {
        // Initialize telemetry for tests (idempotent)
        telemetry::init();

        Self {
            app: app_http::app(), // Real HTTP router from app-http crate
            orders: HashMap::new(),
            last_response: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: String,
    pub total_cents: u64,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub body: serde_json::Value,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }
}
