# MYSERV: The Canonical Teaching Example

**MYSERV** is the reference implementation demonstrating the **AC → BDD → Steps → Handler** pattern that powers the Rust-as-Spec template. It's a fully functional `/todos` CRUD API designed specifically as a learning resource.

## Why MYSERV Exists

MYSERV serves three critical purposes:

1. **Learning by Example**: Shows the complete chain from acceptance criteria to working code
2. **Pattern Reference**: Demonstrates best practices for AC-driven development
3. **Template Baseline**: Provides a working example that validates the template's governance mechanisms

When you're building a new feature, **start by studying MYSERV**. It demonstrates exactly how specs, BDD scenarios, step implementations, and handlers connect to create governed, testable software.

---

## Table of Contents

- [The Six Acceptance Criteria](#the-six-acceptance-criteria)
- [Deep Dive: AC-MYSERV-004 (Create Todo)](#deep-dive-ac-myserv-004-create-todo)
- [Running the Tests](#running-the-tests)
- [File Structure Overview](#file-structure-overview)
- [Using MYSERV as a Pattern](#using-myserv-as-a-pattern)
- [Related Documentation](#related-documentation)

---

## The Six Acceptance Criteria

MYSERV implements six acceptance criteria that demonstrate different aspects of HTTP API development:

### AC-MYSERV-001: List Todos (Happy Path)
**Requirement:** REQ-MYSERV-LIST-TODOS
**What it tests:** `GET /todos` returns a JSON array of todos
**Why it matters:** Basic happy-path read operation

### AC-MYSERV-002: Empty List Handling
**Requirement:** REQ-MYSERV-LIST-TODOS
**What it tests:** `GET /todos` returns empty array `[]` when no todos exist
**Why it matters:** Edge case handling - empty state is valid

### AC-MYSERV-003: Invalid Input Validation
**Requirement:** REQ-MYSERV-ERROR-HANDLING
**What it tests:** Invalid JSON or missing required fields return 400 with error message
**Why it matters:** Demonstrates structured error responses using `AppError`

### AC-MYSERV-004: Delete Operation
**Requirement:** REQ-MYSERV-DELETE-TODOS
**What it tests:** `DELETE /todos/:id` removes todo from list, 404 if not found
**Why it matters:** Demonstrates path parameters and idempotent operations

### AC-MYSERV-005: Duplicate Detection
**Requirement:** REQ-MYSERV-ERROR-HANDLING
**What it tests:** `POST /todos` with duplicate ID returns 409 Conflict
**Why it matters:** Business rule validation (uniqueness constraint)

### AC-MYSERV-006: Input Validation Rules
**Requirement:** REQ-MYSERV-ERROR-HANDLING
**What it tests:** Title must be non-empty and ≤256 characters
**Why it matters:** Field-level validation with meaningful error messages

---

## Deep Dive: AC-MYSERV-004 (Create Todo)

Let's trace **AC-MYSERV-004** through the entire system to understand the AC → BDD → Steps → Handler pattern.

### Step 1: Spec Ledger Definition

Location: `specs/spec_ledger.yaml` (lines 1829-1841)

```yaml
stories:
  - id: US-MYSERV-001
    title: "Todo Management"
    description: >
      Users can manage their personal todo list through the API.
    tags: [domain, myserv]
    requirements:
      - id: REQ-MYSERV-DELETE-TODOS
        title: "Delete Todos"
        tags: [domain, myserv]
        must_have_ac: false  # Example-only, not enforced by governance
        acceptance_criteria:
          - id: AC-MYSERV-004
            text: "DELETE /todos/:id removes the todo from the list"
            tags: [domain, myserv]
            must_have_ac: false
            tests:
              - type: bdd
                tag: "@AC-MYSERV-004"
                file: "specs/features/myserv_todos.feature"
```

**Key Points:**
- ACs live inside Requirements, which live inside Stories
- `must_have_ac: false` marks this as example-only (not enforced by `cargo xtask selftest`)
- `tests` array links the AC to its BDD scenarios

### Step 2: BDD Scenario

Location: `specs/features/myserv_todos.feature` (lines 42-54)

```gherkin
@AC-MYSERV-004
Scenario: DELETE /todos/:id removes the todo from the list
  Given the user has existing todos
  When I send a DELETE request to "/todos/todo-1"
  Then the response status should be 204
  When I send a GET request to "/todos"
  Then the response status should be 200
  And the response should be a JSON array
  And the todo with id "todo-1" should not be in the list

@AC-MYSERV-004
Scenario: DELETE /todos/:id with non-existent id returns 404
  When I send a DELETE request to "/todos/non-existent-id"
  Then the response status should be 404
```

**Key Points:**
- Tagged with `@AC-MYSERV-004` to link back to spec_ledger.yaml
- Uses Given/When/Then format (Gherkin syntax)
- Two scenarios: happy path (204 success) and error case (404 not found)
- Steps are reusable across multiple scenarios

### Step 3: Step Definitions

Location: `crates/acceptance/src/steps/myserv.rs` (lines 55-88)

```rust
#[when(regex = r#"^I send a DELETE request to "([^"]+)"$"#)]
async fn when_delete_request(world: &mut World, path: String) {
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    let request = Request::builder()
        .method("DELETE")
        .uri(&path)
        .body(Body::empty())
        .expect("Failed to build DELETE request");

    let response = world.app.clone()
        .oneshot(request)
        .await
        .expect("Failed to send DELETE request");

    let status = response.status();
    let headers = response.headers().clone();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");

    let body_str = String::from_utf8_lossy(&body_bytes);
    let body_json = if body_str.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_str(&body_str)
            .unwrap_or(serde_json::Value::String(body_str.to_string()))
    };

    world.last_response = Some(crate::world::Response {
        status: status.as_u16(),
        body: body_json,
        headers,
        raw_body: body_str.to_string(),
    });
}
```

**Key Points:**
- Uses `cucumber` crate's `#[when]` macro with regex pattern matching
- Extracts path parameter from Gherkin step text (`"/todos/todo-1"`)
- Calls the Axum app via `world.app.oneshot()` (in-memory HTTP test)
- Stores response in `world.last_response` for assertions in `Then` steps
- No database or network needed - pure in-process testing

### Step 4: Handler Implementation

Location: `crates/app-http/src/todos.rs` (lines 238-254)

```rust
/// DELETE /todos/:id - Delete a todo by ID
///
/// Implements AC-MYSERV-004: "DELETE /todos/:id removes the todo from the list"
///
/// # BDD Reference
/// Tagged with @AC-MYSERV-004 in specs/features/myserv_todos.feature
#[instrument(skip(state))]
async fn delete_todo(
    State(state): State<(AppState, TodosState)>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    info!(id = %id, "Deleting todo");

    let deleted = state.1.delete(&id)?;

    if deleted {
        info!(id = %id, "Todo deleted successfully");
        Ok(StatusCode::NO_CONTENT)  // 204 success
    } else {
        warn!(id = %id, "Todo not found");
        Err(AppError::not_found(format!("Todo with id '{}' not found", id)))  // 404 error
    }
}
```

**Key Points:**
- Function signature uses Axum extractors: `State`, `Path`
- Returns `Result<impl IntoResponse, AppError>` (standard template pattern)
- `#[instrument]` provides automatic tracing with structured logging
- Docstring explicitly references AC-MYSERV-004 for traceability
- Returns 204 No Content on success, 404 Not Found on missing ID

### Step 5: Model Definition

Location: `crates/model/src/lib.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Todo {
    pub id: String,
    pub title: String,
}
```

**Key Points:**
- Shared between `app-http` (handler) and `acceptance` (tests)
- Uses `serde` for JSON serialization
- Simple structure focused on teaching - no timestamps, user IDs, etc.

---

## Running the Tests

### Run All MYSERV Tests

```bash
cargo xtask bdd --tags @myserv
```

### Run a Specific AC

```bash
# Test only AC-MYSERV-004 scenarios
cargo xtask bdd --tags @AC-MYSERV-004

# Test AC-MYSERV-001 (list todos)
cargo xtask bdd --tags @AC-MYSERV-001

# Test error handling (AC-MYSERV-003, 005, 006)
cargo xtask bdd --tags @AC-MYSERV-003
cargo xtask bdd --tags @AC-MYSERV-005
cargo xtask bdd --tags @AC-MYSERV-006
```

### Run Unit Tests

```bash
# Run unit tests in todos.rs handler
cargo test -p app-http todos

# Run all unit tests
cargo xtask check
```

### Validate Governance

```bash
# Check AC → test mapping
cargo xtask ac-status

# See which tests cover AC-MYSERV-004
cargo xtask ac-tests AC-MYSERV-004

# Full governance validation (includes BDD + unit tests)
cargo xtask selftest
```

---

## File Structure Overview

Here's where everything lives:

```
Rust-Template/
├── specs/
│   ├── spec_ledger.yaml              # ACs defined here (lines 1779-1841)
│   └── features/
│       └── myserv_todos.feature      # BDD scenarios (Given/When/Then)
│
├── crates/
│   ├── model/
│   │   └── src/lib.rs                # Todo struct definition
│   │
│   ├── app-http/
│   │   └── src/
│   │       ├── todos.rs              # Handler implementation (IMPORTANT!)
│   │       └── lib.rs                # Router registration
│   │
│   └── acceptance/
│       └── src/
│           ├── steps/
│           │   ├── myserv.rs         # Step definitions (@given/@when/@then)
│           │   └── governance_tasks.rs  # Shared steps (GET/POST requests)
│           └── world.rs              # Test world (app instance, response storage)
│
└── forks/
    └── example-myservice/
        └── README.md                 # This file
```

---

## Using MYSERV as a Pattern

When implementing a new feature, follow this workflow:

### 1. Study MYSERV First

Before writing any code:
- Read this README completely
- Run the BDD tests: `cargo xtask bdd --tags @myserv`
- Trace one AC end-to-end (e.g., AC-MYSERV-004 above)
- Understand how specs, scenarios, steps, and handlers connect

### 2. Define Your AC in spec_ledger.yaml

```yaml
stories:
  - id: US-YOURFEATURE-001
    title: "Your Feature Name"
    requirements:
      - id: REQ-YOURFEATURE-ENDPOINT
        title: "Your Endpoint"
        acceptance_criteria:
          - id: AC-YOURFEATURE-001
            text: "GET /your-endpoint returns expected data"
            tests:
              - type: bdd
                tag: "@AC-YOURFEATURE-001"
                file: "specs/features/your_feature.feature"
```

### 3. Write BDD Scenario

Create `specs/features/your_feature.feature`:

```gherkin
@AC-YOURFEATURE-001
Scenario: Your scenario description
  Given some precondition
  When I send a GET request to "/your-endpoint"
  Then the response status should be 200
  And the response should match expectations
```

### 4. Implement Step Definitions

In `crates/acceptance/src/steps/your_feature.rs`:

```rust
#[given(regex = r"^some precondition$")]
async fn given_precondition(world: &mut World) {
    // Setup test state
}
```

**Tip:** Reuse existing steps from `governance_tasks.rs` when possible:
- `when_get_request()` - Generic GET handler
- `when_post_request()` - Generic POST handler
- `then_status_code()` - Status code assertion

### 5. Write the Handler

In `crates/app-http/src/your_feature.rs`:

```rust
/// GET /your-endpoint - Your description
///
/// Implements AC-YOURFEATURE-001: "..."
#[instrument(skip(state))]
async fn your_handler(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    // Implementation
    Ok(Json(response))
}
```

**Follow MYSERV patterns:**
- Use `#[instrument]` for tracing
- Return `Result<impl IntoResponse, AppError>`
- Reference AC IDs in docstrings
- Use `info!()` and `warn!()` for structured logging

### 6. Register Router

In `crates/app-http/src/lib.rs`:

```rust
use your_feature::router as your_feature_router;

pub fn create_router(state: AppState) -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .merge(todos::router(state.clone()))
        .merge(your_feature_router(state.clone()))  // Add your router
        // ...
}
```

### 7. Validate with the Ladder

```bash
# Quick checks (fmt, clippy, unit tests)
cargo xtask check

# Run your BDD scenarios
cargo xtask bdd --tags @AC-YOURFEATURE-001

# Verify AC mapping
cargo xtask ac-tests AC-YOURFEATURE-001

# Full governance gate (before committing)
cargo xtask selftest
```

---

## Key Patterns Demonstrated

### Error Handling (AC-MYSERV-003, 005, 006)

MYSERV shows how to use `AppError` for structured errors:

```rust
// 400 Bad Request - validation error
Err(AppError::bad_request("Title cannot be empty")
    .with_ac_id("AC-MYSERV-006"))

// 404 Not Found
Err(AppError::not_found(format!("Todo '{}' not found", id)))

// 409 Conflict - business rule violation
Err(AppError::new(
    StatusCode::CONFLICT,
    ErrorCode::Conflict,
    format!("Todo with id '{}' already exists", id)
).with_ac_id("AC-MYSERV-005"))
```

### State Management (In-Memory)

MYSERV uses `Arc<RwLock<Vec<Todo>>>` for simplicity:

```rust
#[derive(Clone)]
pub struct TodosState {
    todos: Arc<RwLock<Vec<Todo>>>,
}
```

**For production features:**
- Replace with a real database adapter
- Keep the same public interface (`get_all()`, `add()`, `delete()`)
- Step definitions and handlers remain unchanged (hexagonal architecture)

### Traceability

Every handler references its AC:

```rust
/// Implements AC-MYSERV-004: "DELETE /todos/:id removes the todo from the list"
```

This enables:
- `cargo xtask ac-tests AC-MYSERV-004` - find all tests for this AC
- `cargo xtask ac-status` - see AC health (pass/fail)
- `/platform/graph` API - visualize AC → test relationships

---

## Common Questions

### Q: Why are some ACs marked `must_have_ac: false`?

**A:** MYSERV is a teaching example, not a governance requirement. Setting `must_have_ac: false` means:
- `cargo xtask selftest` won't fail if MYSERV tests break
- You can delete MYSERV when you're ready to ship
- Template-level ACs (health checks, governance tasks) have `must_have_ac: true`

### Q: Can I delete MYSERV from my project?

**A:** Yes! Once you understand the pattern:
1. Remove `/todos` routes from `app-http/src/lib.rs`
2. Delete `crates/app-http/src/todos.rs`
3. Delete `crates/acceptance/src/steps/myserv.rs`
4. Delete `specs/features/myserv_todos.feature`
5. Remove MYSERV entries from `spec_ledger.yaml` (lines 1779-1841)

### Q: How do I add authentication to my handlers?

**A:** Study `specs/features/auth.feature` and `crates/acceptance/src/steps/auth.rs` for authentication patterns. MYSERV intentionally omits auth to keep the teaching example simple.

### Q: Why use BDD instead of just unit tests?

**A:** BDD scenarios test the entire request/response cycle (serialization, routing, error handling) through the public API. Unit tests validate individual functions. You need both - see `todos.rs` lines 274-345 for MYSERV unit tests.

### Q: How do I test MYSERV manually with curl?

```bash
# Start the service
cargo run -p app-http

# In another terminal:
curl http://localhost:8080/todos
curl -X POST http://localhost:8080/todos \
  -H "Content-Type: application/json" \
  -d '{"id":"test-1","title":"Learn MYSERV"}'
curl -X DELETE http://localhost:8080/todos/test-1
```

---

## Related Documentation

### Getting Started
- `docs/how-to/first-hour.md` - Template onboarding guide
- `docs/tutorials/day-1-first-change.md` - Your first code change
- `docs/tutorials/day-7-first-real-feature.md` - Building a full feature

### AC-Driven Development
- `docs/tutorials/first-ac-change.md` - How to modify an AC
- `CLAUDE.md` - Full governance workflow guide
- `docs/AGENT_GUIDE.md` - Using platform APIs for development

### Architecture
- `docs/explanation/TEMPLATE-CONTRACTS.md` - Design principles
- `specs/spec_ledger.yaml` - All ACs and requirements
- `specs/devex_flows.yaml` - Developer workflows

### Testing
- BDD scenarios: `specs/features/`
- Step definitions: `crates/acceptance/src/steps/`
- Run: `cargo xtask bdd --help` for all BDD options

---

## Summary

MYSERV demonstrates the complete **AC → BDD → Steps → Handler** pattern in 6 acceptance criteria:

1. **AC-MYSERV-001/002**: Basic CRUD + edge cases (list/empty list)
2. **AC-MYSERV-003**: Validation errors (400 Bad Request)
3. **AC-MYSERV-004**: Delete operations + path parameters (204/404)
4. **AC-MYSERV-005**: Business rules (409 Conflict on duplicate)
5. **AC-MYSERV-006**: Field validation (length + non-empty checks)

**Key Takeaway:** Every feature follows this chain:
1. Define AC in `spec_ledger.yaml`
2. Write scenario in `features/*.feature`
3. Implement steps in `acceptance/src/steps/*.rs`
4. Write handler in `app-http/src/*.rs`
5. Validate with `cargo xtask bdd` and `cargo xtask selftest`

Study MYSERV. Copy MYSERV. Modify MYSERV. Then delete MYSERV and build your real features using the same pattern.

**The pattern works because it's testable, traceable, and governed.**
