//! Request DTOs and validation rules for todo-related HTTP operations.

use serde::{Deserialize, Serialize};

/// Maximum allowed title length for a todo.
pub const MAX_TITLE_LENGTH: usize = 256;

/// Request body for creating a new todo.
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTodoRequest {
    /// Unique todo identifier
    pub id: String,
    /// Todo description
    pub title: String,
}

/// Validation errors for todo create requests.
#[derive(Debug, PartialEq, Eq)]
pub enum CreateTodoValidationError {
    /// The `id` field is missing or empty.
    MissingId,
    /// The `title` field is missing or empty.
    MissingTitle,
    /// The title exceeds the allowed maximum length.
    TitleTooLong { actual: usize, max: usize },
}

/// Validate a create-todo request according to API constraints.
pub fn validate_create_todo_request(
    request: &CreateTodoRequest,
) -> Result<(), CreateTodoValidationError> {
    if request.id.is_empty() {
        return Err(CreateTodoValidationError::MissingId);
    }

    if request.title.is_empty() {
        return Err(CreateTodoValidationError::MissingTitle);
    }

    if request.title.len() > MAX_TITLE_LENGTH {
        return Err(CreateTodoValidationError::TitleTooLong {
            actual: request.title.len(),
            max: MAX_TITLE_LENGTH,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_request() {
        let request =
            CreateTodoRequest { id: "todo-1".to_string(), title: "Write tests".to_string() };

        assert_eq!(validate_create_todo_request(&request), Ok(()));
    }

    #[test]
    fn rejects_missing_id() {
        let request = CreateTodoRequest { id: String::new(), title: "Write tests".to_string() };

        assert_eq!(
            validate_create_todo_request(&request),
            Err(CreateTodoValidationError::MissingId)
        );
    }

    #[test]
    fn rejects_missing_title() {
        let request = CreateTodoRequest { id: "todo-1".to_string(), title: String::new() };

        assert_eq!(
            validate_create_todo_request(&request),
            Err(CreateTodoValidationError::MissingTitle)
        );
    }

    #[test]
    fn rejects_title_over_limit() {
        let request =
            CreateTodoRequest { id: "todo-1".to_string(), title: "x".repeat(MAX_TITLE_LENGTH + 1) };

        assert_eq!(
            validate_create_todo_request(&request),
            Err(CreateTodoValidationError::TitleTooLong {
                actual: MAX_TITLE_LENGTH + 1,
                max: MAX_TITLE_LENGTH,
            })
        );
    }
}
