//! Minimal HTTP server for brownfield demo
//!
//! This is a simple existing codebase that we'll enhance with Rust IaC tooling.

use axum::{
    extract::Path,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Item {
    id: u64,
    name: String,
    description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateItemRequest {
    name: String,
    description: Option<String>,
}

type SharedState = Arc<Mutex<Vec<Item>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Shared state for items
    let state: SharedState = Arc::new(Mutex::new(vec![
        Item {
            id: 1,
            name: "Example Item".to_string(),
            description: Some("An example item in the demo server".to_string()),
        },
    ]));

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/items", get(list_items))
        .route("/items", post(create_item))
        .route("/items/{id}", get(get_item))
        .with_state(state);

    // Run server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("Server running on http://127.0.0.1:3000");
    println!("Endpoints:");
    println!("  GET  /health");
    println!("  GET  /items");
    println!("  POST /items");
    println!("  GET  /items/{{id}}");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn list_items(
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> Json<Vec<Item>> {
    let items = state.lock().unwrap();
    Json(items.clone())
}

async fn create_item(
    axum::extract::State(state): axum::extract::State<SharedState>,
    Json(req): Json<CreateItemRequest>,
) -> Json<Item> {
    let mut items = state.lock().unwrap();
    let new_id = items.iter().map(|i| i.id).max().unwrap_or(0) + 1;

    let new_item = Item {
        id: new_id,
        name: req.name,
        description: req.description,
    };

    items.push(new_item.clone());
    Json(new_item)
}

async fn get_item(
    axum::extract::State(state): axum::extract::State<SharedState>,
    Path(id): Path<u64>,
) -> Result<Json<Item>, axum::http::StatusCode> {
    let items = state.lock().unwrap();

    items
        .iter()
        .find(|item| item.id == id)
        .cloned()
        .map(Json)
        .ok_or(axum::http::StatusCode::NOT_FOUND)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_creation() {
        let item = Item {
            id: 1,
            name: "Test".to_string(),
            description: None,
        };
        assert_eq!(item.id, 1);
        assert_eq!(item.name, "Test");
    }

    #[test]
    fn test_create_item_request() {
        let req = CreateItemRequest {
            name: "New Item".to_string(),
            description: Some("Description".to_string()),
        };
        assert_eq!(req.name, "New Item");
    }
}
