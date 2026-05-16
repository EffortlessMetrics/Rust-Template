//! In-memory todo storage used by HTTP handlers.

use model::Todo;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared state for todos - in-memory storage.
#[derive(Clone)]
pub struct TodosState {
    todos: Arc<RwLock<Vec<Todo>>>,
}

/// Todos state trait for handlers.
///
/// This trait defines minimal interface required for todo handlers.
pub trait TodosStateTrait: Clone + Send + Sync + 'static {
    /// Get the inner TodosState.
    fn todos_state(&self) -> TodosState;
}

impl TodosState {
    /// Create a new TodosState with sample data.
    pub fn new() -> Self {
        let todos = vec![
            Todo { id: "todo-1".to_string(), title: "Learn Rust-as-Spec patterns".to_string() },
            Todo { id: "todo-2".to_string(), title: "Implement AC-MYSERV-001".to_string() },
        ];

        Self { todos: Arc::new(RwLock::new(todos)) }
    }

    /// Create a new TodosState with empty todos.
    ///
    /// Used for testing empty array scenario.
    pub fn empty() -> Self {
        Self { todos: Arc::new(RwLock::new(vec![])) }
    }

    /// Get all todos.
    pub async fn get_all(&self) -> Vec<Todo> {
        self.todos.read().await.clone()
    }

    /// Add a new todo.
    pub async fn add(&self, todo: Todo) {
        self.todos.write().await.push(todo);
    }

    /// Try to add a new todo, returning error if ID exists.
    ///
    /// This performs both check and insertion under a single write lock
    /// to avoid race conditions and extra locking overhead.
    pub async fn try_add(&self, id: String, title: String) -> Result<Todo, String> {
        let mut guard = self.todos.write().await;
        if guard.iter().any(|t| t.id == id) {
            return Err(id);
        }
        let todo = Todo { id, title };
        guard.push(todo.clone());
        Ok(todo)
    }

    /// Check if a todo with the given ID exists.
    pub async fn exists(&self, id: &str) -> bool {
        self.todos.read().await.iter().any(|t| t.id == id)
    }

    /// Delete a todo by ID, returns true if found and deleted.
    pub async fn delete(&self, id: &str) -> bool {
        let mut guard = self.todos.write().await;
        let original_len = guard.len();
        guard.retain(|t| t.id != id);
        guard.len() < original_len
    }

    /// Clear all todos.
    pub async fn clear(&self) {
        self.todos.write().await.clear();
    }
}

impl Default for TodosState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_todos_state_initialization() {
        let state = TodosState::new();
        let todos = state.get_all().await;

        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].id, "todo-1");
        assert_eq!(todos[0].title, "Learn Rust-as-Spec patterns");
    }

    #[tokio::test]
    async fn test_todo_has_required_fields() {
        let state = TodosState::new();
        let todos = state.get_all().await;

        // Each todo must have id and title
        for todo in todos {
            assert!(!todo.id.is_empty(), "Todo must have id");
            assert!(!todo.title.is_empty(), "Todo must have title");
        }
    }

    #[tokio::test]
    async fn test_empty_todos_state() {
        // Empty list is valid
        let state = TodosState::empty();
        let todos = state.get_all().await;

        assert_eq!(todos.len(), 0);
    }

    #[tokio::test]
    async fn test_add_todo() {
        let state = TodosState::empty();
        let todo = Todo { id: "test-1".to_string(), title: "Test todo".to_string() };

        state.add(todo).await;

        let todos = state.get_all().await;
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].id, "test-1");
    }

    #[tokio::test]
    async fn test_delete_todo() {
        let state = TodosState::new();

        let deleted = state.delete("todo-1").await;
        assert!(deleted, "Todo should be found and deleted");

        let todos = state.get_all().await;
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].id, "todo-2");
    }

    #[tokio::test]
    async fn test_delete_nonexistent_todo() {
        let state = TodosState::new();

        let deleted = state.delete("non-existent").await;
        assert!(!deleted, "Non-existent todo should not be found");

        let todos = state.get_all().await;
        assert_eq!(todos.len(), 2); // Original count unchanged
    }
}
