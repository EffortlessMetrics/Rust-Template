//! Request DTOs and validation for todo endpoints.

use http_errors::{ErrorCode, HttpError};
use serde::{Deserialize, Serialize};

const MAX_TITLE_LENGTH: usize = 256;

/// Request body for creating a new todo.
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTodoRequest {
    /// Unique todo identifier
    pub id: String,
    /// Todo description
    pub title: String,
}

impl CreateTodoRequest {
    /// Validate that the request contains all required todo fields.
    pub fn validate(&self) -> Result<(), HttpError> {
        if self.id.is_empty() {
            return Err(HttpError::bad_request("Missing required field: id"));
        }

        if self.title.is_empty() {
            return Err(HttpError::bad_request("Missing required field: title"));
        }

        if self.title.len() > MAX_TITLE_LENGTH {
            return Err(HttpError::new(
                400,
                ErrorCode::InvalidFormat,
                format!("Title must not exceed {} characters", MAX_TITLE_LENGTH),
            ));
        }

        Ok(())
    }
}
