use cucumber::World as CucumberWorld;
use std::collections::HashMap;

/// Test world state
#[derive(Debug, Default, CucumberWorld)]
pub struct World {
    /// Orders in the system
    pub orders: HashMap<String, Order>,
    /// Last HTTP response
    pub last_response: Option<Response>,
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
