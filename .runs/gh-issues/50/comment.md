## Investigation Report: Issue #50 - Unit Tests for model/business-core

### Status
**Status:** fix-ready - Clear test gaps identified
**Local gates:** Code audit completed

### Evidence

**model crate** (`crates/model/src/lib.rs` - 55 LOC):
- `ExampleTask` struct (Debug, Clone, PartialEq, Eq)
- `ExampleTaskStatus` enum (3 variants: Pending, InProgress, Completed)
- `Todo` struct (with serde derives)
- **Current tests:** 0

**business-core crate** (`crates/business-core/src/lib.rs` - 128 LOC):
- `ExampleTaskRepository` trait (4 async methods)
- Use cases: `create_example_task`, `get_example_task`, `list_example_tasks`, `update_example_task_status`
- **Current tests:** 0

### Plan

**For model crate:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn todo_serialization_roundtrip() {
        let todo = Todo { id: "1".into(), title: "Test".into(), completed: false };
        let json = serde_json::to_string(&todo).unwrap();
        let parsed: Todo = serde_json::from_str(&json).unwrap();
        assert_eq!(todo, parsed);
    }
    
    #[test]
    fn example_task_status_variants() {
        assert_eq!(format!("{:?}", ExampleTaskStatus::Pending), "Pending");
    }
}
```

**For business-core crate:**
- Create mock `InMemoryTaskRepository` for testing
- Test `create_example_task` generates UUID with Pending status
- Test use case error propagation

**Test plan:**
```bash
cargo test -p model --lib
cargo test -p business-core --lib
```

### Decision / Next Action

**Recommend:** Keep open as **fix-ready**. Good first issue - ~2 hours of work. Follow gov-model pattern for inline `#[cfg(test)]` modules.
