# Tutorial: Day 7 - Your First Real Feature

**You've mastered the basics.** Now let's build something substantial that demonstrates the full power of AC-driven development with proper domain modeling, validation, and multi-layer architecture.

**Time:** 90 minutes
**Prerequisites:**
- Completed [Day 1: First Change](day-1-first-change.md)
- Comfortable with Rust basics
- Understanding of HTTP APIs and REST principles

---

## What You'll Build

A **Task Management** feature with proper CRUD operations:

- Create a task with title and description
- Retrieve a task by ID
- List all tasks
- Proper validation, error handling, and domain modeling
- Full vertical integration: model → core → app-http → BDD tests

This demonstrates:
- ✅ Multi-layer hexagonal architecture
- ✅ Domain-driven design patterns
- ✅ Multiple ACs working together
- ✅ Proper error handling with error codes
- ✅ Request/response DTOs vs domain models
- ✅ AC traceability across all layers

---

## Architecture Preview

```
┌─────────────────────────────────────────────┐
│  HTTP Layer (app-http)                      │
│  - Routes: POST /tasks, GET /tasks/:id      │
│  - DTOs: CreateTaskRequest, TaskResponse    │
│  - Error mapping                            │
└─────────────────┬───────────────────────────┘
                  │ calls
                  ▼
┌─────────────────────────────────────────────┐
│  Domain Layer (core)                        │
│  - Business logic: create_task, get_task    │
│  - Validation: title required, max length   │
│  - No HTTP knowledge                        │
└─────────────────┬───────────────────────────┘
                  │ uses
                  ▼
┌─────────────────────────────────────────────┐
│  Model Layer (model)                        │
│  - Entities: Task                           │
│  - Value objects: TaskId, TaskStatus        │
└─────────────────────────────────────────────┘
```

**Key principle:** Dependencies point inward. HTTP knows about core, core knows about model, but never the reverse.

---

## Step 1: Plan Your ACs (10 min)

Before writing code, define clear acceptance criteria. This is the **spec-first** philosophy.

### User Story

Create `specs/userstories/US-TASK-001.md`:

```markdown
# US-TASK-001: Task Management

## As a user
I want to manage my tasks
So that I can track what needs to be done

## Business Value

Enable basic task tracking for users. Foundation for future task management features.

## Requirements

- REQ-TASK-001: Create tasks
- REQ-TASK-002: Retrieve tasks
- REQ-TASK-003: List tasks

## Out of Scope (v1)

- Task completion/status updates
- Task deletion
- Task assignment
- Due dates
```

### Acceptance Criteria

Add to `specs/spec_ledger.yaml`:

```yaml
stories:
  # ... existing stories ...

  - id: US-TASK-001
    title: "Task Management"
    requirements:
      - id: REQ-TASK-001
        title: "Create Tasks"
        acceptance_criteria:
          - id: AC-TASK-001
            text: "User can create a task with title and description"
            tests: [{ type: bdd, tag: "@AC-TASK-001" }]
          - id: AC-TASK-002
            text: "Task creation fails if title is empty or exceeds 200 characters"
            tests: [{ type: bdd, tag: "@AC-TASK-002" }]

      - id: REQ-TASK-002
        title: "Retrieve Tasks"
        acceptance_criteria:
          - id: AC-TASK-003
            text: "User can retrieve a task by ID"
            tests: [{ type: bdd, tag: "@AC-TASK-003" }]
          - id: AC-TASK-004
            text: "Retrieving non-existent task returns 404 with appropriate error"
            tests: [{ type: bdd, tag: "@AC-TASK-004" }]

      - id: REQ-TASK-003
        title: "List Tasks"
        acceptance_criteria:
          - id: AC-TASK-005
            text: "User can list all tasks"
            tests: [{ type: bdd, tag: "@AC-TASK-005" }]
```

**Validate:**

```bash
cargo run -p xtask -- policy-test
```

Expected: `✓ All ACs have tests`

---

## Step 2: Write Gherkin Scenarios (15 min)

Create `specs/features/tasks.feature`:

```gherkin
Feature: Task Management
  As a user
  I want to manage my tasks
  So that I can track what needs to be done

  @AC-TASK-001
  Scenario: Create a task successfully
    When I POST /tasks with JSON:
      """
      {
        "title": "Write documentation",
        "description": "Complete the Day 7 tutorial"
      }
      """
    Then I receive a 201 response
    And the response body contains "taskId" field
    And the response body contains "title" field
    And the response body contains "description" field
    And the "title" field equals "Write documentation"

  @AC-TASK-002
  Scenario: Cannot create task with empty title
    When I POST /tasks with JSON:
      """
      {
        "title": "",
        "description": "This should fail"
      }
      """
    Then I receive a 400 response
    And the response body contains "error" field
    And the "error" field contains "MISSING_FIELD"

  @AC-TASK-002
  Scenario: Cannot create task with title exceeding 200 characters
    When I POST /tasks with JSON:
      """
      {
        "title": "Lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod tempor incididunt ut labore et dolore magna aliqua Ut enim ad minim veniam quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat Duis aute irure",
        "description": "This title is 201 characters"
      }
      """
    Then I receive a 400 response
    And the response body contains "error" field
    And the "error" field contains "INVALID_INPUT"

  @AC-TASK-003
  Scenario: Retrieve a task by ID
    Given a task exists with id "task-123"
    When I GET /tasks/task-123
    Then I receive a 200 response
    And the response body contains "taskId" field
    And the "taskId" field equals "task-123"

  @AC-TASK-004
  Scenario: Retrieve non-existent task returns 404
    When I GET /tasks/nonexistent-task
    Then I receive a 404 response
    And the response body contains "error" field
    And the "error" field contains "NOT_FOUND"

  @AC-TASK-005
  Scenario: List all tasks
    Given the following tasks exist:
      | taskId  | title           | description          |
      | task-1  | First task      | Description 1        |
      | task-2  | Second task     | Description 2        |
    When I GET /tasks
    Then I receive a 200 response
    And the response body contains "tasks" field
    And the "tasks" array contains 2 items
```

---

## Step 3: Implement the Model Layer (10 min)

Open `crates/model/src/lib.rs` and add:

```rust
use serde::{Deserialize, Serialize};

/// Task entity - represents a todo item
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
}

/// Task identifier - newtype pattern for type safety
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TaskId(pub String);

impl TaskId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn from_string(s: String) -> Self {
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

/// Task status - enum for type safety
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Completed,
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl Task {
    /// Create a new task with generated ID
    pub fn new(title: String, description: Option<String>) -> Self {
        Self {
            id: TaskId::new(),
            title,
            description,
            status: TaskStatus::Pending,
        }
    }

    /// Create a task with a specific ID (for testing)
    pub fn with_id(id: TaskId, title: String, description: Option<String>) -> Self {
        Self { id, title, description, status: TaskStatus::Pending }
    }
}
```

**Add dependency to `crates/model/Cargo.toml`:**

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1.0", features = ["v4"] }
```

---

## Step 4: Implement the Core Layer (15 min)

Open `crates/core/src/lib.rs` and replace with:

```rust
use model::{Task, TaskId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub mod errors;

pub use errors::CoreError;

/// In-memory task repository (trait would go here for real DB)
///
/// In production, this would be a trait implemented by app-db:
/// ```
/// pub trait TaskRepository {
///     async fn save(&self, task: Task) -> Result<(), CoreError>;
///     async fn find_by_id(&self, id: &TaskId) -> Result<Option<Task>, CoreError>;
///     async fn list_all(&self) -> Result<Vec<Task>, CoreError>;
/// }
/// ```
#[derive(Clone, Default)]
pub struct TaskStore {
    tasks: Arc<RwLock<HashMap<String, Task>>>,
}

impl TaskStore {
    pub fn new() -> Self {
        Self { tasks: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub fn save(&self, task: Task) {
        let mut tasks = self.tasks.write().unwrap();
        tasks.insert(task.id.as_str().to_string(), task);
    }

    pub fn find_by_id(&self, id: &TaskId) -> Option<Task> {
        let tasks = self.tasks.read().unwrap();
        tasks.get(id.as_str()).cloned()
    }

    pub fn list_all(&self) -> Vec<Task> {
        let tasks = self.tasks.read().unwrap();
        tasks.values().cloned().collect()
    }
}

/// Create a new task with validation
///
/// AC-TASK-001: Allow task creation with title and description
/// AC-TASK-002: Validate title is non-empty and <= 200 chars
pub fn create_task(
    store: &TaskStore,
    title: String,
    description: Option<String>,
) -> Result<Task, CoreError> {
    // Validation: title required
    if title.trim().is_empty() {
        return Err(CoreError::ValidationError {
            field: "title".to_string(),
            message: "Title cannot be empty".to_string(),
        });
    }

    // Validation: title max length
    if title.len() > 200 {
        return Err(CoreError::ValidationError {
            field: "title".to_string(),
            message: "Title cannot exceed 200 characters".to_string(),
        });
    }

    let task = Task::new(title, description);
    store.save(task.clone());

    Ok(task)
}

/// Retrieve a task by ID
///
/// AC-TASK-003: Allow retrieval by ID
/// AC-TASK-004: Return error for non-existent tasks
pub fn get_task(store: &TaskStore, id: &TaskId) -> Result<Task, CoreError> {
    store.find_by_id(id).ok_or_else(|| CoreError::NotFound {
        entity: "Task".to_string(),
        id: id.as_str().to_string(),
    })
}

/// List all tasks
///
/// AC-TASK-005: Return all tasks
pub fn list_tasks(store: &TaskStore) -> Vec<Task> {
    store.list_all()
}
```

**Create `crates/core/src/errors.rs`:**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Validation error in field '{field}': {message}")]
    ValidationError { field: String, message: String },

    #[error("{entity} not found: {id}")]
    NotFound { entity: String, id: String },
}
```

**Add dependencies to `crates/core/Cargo.toml`:**

```toml
[dependencies]
model = { path = "../model" }
thiserror = "2.0"
```

---

## Step 5: Implement the HTTP Layer (20 min)

Open `crates/app-http/src/lib.rs` and add:

### 5a. Add routes

```rust
use axum::extract::Path;
use std::sync::Arc;

pub fn app() -> Router {
    let task_store = Arc::new(core::TaskStore::new());

    Router::new()
        // Template core endpoints
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/api/echo", post(echo))
        // Task management endpoints
        .route("/tasks", post(create_task_handler))
        .route("/tasks", get(list_tasks_handler))
        .route("/tasks/:id", get(get_task_handler))
        .layer(Extension(task_store))
        .layer(axum::middleware::from_fn(middleware::request_id_middleware))
        .layer(TraceLayer::new_for_http())
}
```

### 5b. Add handlers

```rust
/// Create a new task
///
/// Demonstrates:
/// - AC-TASK-001: Task creation with title and description
/// - AC-TASK-002: Validation enforcement
/// - Calling core domain logic
/// - Error mapping from core → HTTP
#[instrument(skip(request_id, store, payload))]
async fn create_task_handler(
    Extension(request_id): Extension<RequestId>,
    Extension(store): Extension<Arc<core::TaskStore>>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<TaskResponse>), AppError> {
    info!("Creating task");

    let task = core::create_task(&store, payload.title, payload.description).map_err(|e| {
        match e {
            core::CoreError::ValidationError { field, message } => {
                AppError::validation_error(ErrorCode::MissingField, &message)
                    .with_context("field", &field)
                    .with_ac_id("AC-TASK-002")
                    .with_request_id(request_id.as_str())
            }
            _ => AppError::internal_error(&e.to_string()).with_request_id(request_id.as_str()),
        }
    })?;

    info!(task_id = %task.id.as_str(), "Task created");

    Ok((StatusCode::CREATED, Json(TaskResponse::from(task))))
}

/// Get a task by ID
///
/// AC-TASK-003: Retrieve task by ID
/// AC-TASK-004: 404 for non-existent tasks
#[instrument(skip(request_id, store))]
async fn get_task_handler(
    Extension(request_id): Extension<RequestId>,
    Extension(store): Extension<Arc<core::TaskStore>>,
    Path(id): Path<String>,
) -> Result<Json<TaskResponse>, AppError> {
    info!(task_id = %id, "Retrieving task");

    let task_id = model::TaskId::from_string(id);
    let task = core::get_task(&store, &task_id).map_err(|e| match e {
        core::CoreError::NotFound { entity, id } => AppError::not_found(&entity, &id)
            .with_ac_id("AC-TASK-004")
            .with_request_id(request_id.as_str()),
        _ => AppError::internal_error(&e.to_string()).with_request_id(request_id.as_str()),
    })?;

    Ok(Json(TaskResponse::from(task)))
}

/// List all tasks
///
/// AC-TASK-005: Return all tasks
#[instrument(skip(store))]
async fn list_tasks_handler(
    Extension(store): Extension<Arc<core::TaskStore>>,
) -> Json<TaskListResponse> {
    info!("Listing all tasks");

    let tasks = core::list_tasks(&store);
    Json(TaskListResponse { tasks: tasks.into_iter().map(TaskResponse::from).collect() })
}
```

### 5c. Add DTOs

```rust
// Task DTOs
#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    #[serde(rename = "taskId")]
    pub task_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: String,
}

impl From<model::Task> for TaskResponse {
    fn from(task: model::Task) -> Self {
        Self {
            task_id: task.id.as_str().to_string(),
            title: task.title,
            description: task.description,
            status: format!("{:?}", task.status).to_lowercase(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TaskListResponse {
    pub tasks: Vec<TaskResponse>,
}
```

### 5d. Update error codes

In `crates/app-http/src/errors.rs`, ensure `ErrorCode` includes:

```rust
pub enum ErrorCode {
    MissingField,
    InvalidInput,
    NotFound,
    InternalError,
}
```

And add `not_found` constructor to `AppError`:

```rust
impl AppError {
    // ... existing methods ...

    pub fn not_found(entity: &str, id: &str) -> Self {
        Self {
            kind: ErrorKind::NotFound,
            code: ErrorCode::NotFound,
            message: format!("{} not found: {}", entity, id),
            context: serde_json::Map::new(),
            ac_id: None,
            feature_id: None,
            request_id: None,
        }
    }
}
```

**Add dependencies to `crates/app-http/Cargo.toml`:**

```toml
[dependencies]
core = { path = "../core" }
model = { path = "../model" }
# ... existing dependencies
```

---

## Step 6: Implement BDD Step Definitions (15 min)

Create `crates/acceptance/src/steps/tasks.rs`:

```rust
use crate::world::{Response, World};
use axum::body::Body;
use cucumber::{gherkin::Table, given, then, when};
use http::Request;
use http_body_util::BodyExt;
use serde_json::json;
use tower::util::ServiceExt;

#[when(regex = r"^I POST /tasks with JSON:$")]
async fn when_post_task(world: &mut World, step: &cucumber::Step) {
    let json_body = step.docstring().expect("JSON body required");

    let request = Request::builder()
        .method("POST")
        .uri("/tasks")
        .header("content-type", "application/json")
        .body(Body::from(json_body.to_string()))
        .expect("valid request");

    let response = world.app.clone().oneshot(request).await.expect("request should succeed");

    let status = response.status().as_u16();
    let headers = response.headers().clone();
    let body_bytes =
        response.into_body().collect().await.expect("body should be readable").to_bytes();

    let body: serde_json::Value =
        serde_json::from_slice(&body_bytes).unwrap_or_else(|_| json!({}));

    world.last_response = Some(Response { status, body, headers });
}

#[when(regex = r"^I GET /tasks/(.+)$")]
async fn when_get_task_by_id(world: &mut World, task_id: String) {
    let request =
        Request::builder().method("GET").uri(format!("/tasks/{}", task_id)).body(Body::empty()).expect("valid request");

    let response = world.app.clone().oneshot(request).await.expect("request should succeed");

    let status = response.status().as_u16();
    let headers = response.headers().clone();
    let body_bytes =
        response.into_body().collect().await.expect("body should be readable").to_bytes();

    let body: serde_json::Value =
        serde_json::from_slice(&body_bytes).unwrap_or_else(|_| json!({}));

    world.last_response = Some(Response { status, body, headers });
}

#[when(regex = r"^I GET /tasks$")]
async fn when_get_all_tasks(world: &mut World) {
    let request = Request::builder().method("GET").uri("/tasks").body(Body::empty()).expect("valid request");

    let response = world.app.clone().oneshot(request).await.expect("request should succeed");

    let status = response.status().as_u16();
    let headers = response.headers().clone();
    let body_bytes =
        response.into_body().collect().await.expect("body should be readable").to_bytes();

    let body: serde_json::Value =
        serde_json::from_slice(&body_bytes).unwrap_or_else(|_| json!({}));

    world.last_response = Some(Response { status, body, headers });
}

#[given(regex = r#"^a task exists with id "([^"]+)"$"#)]
async fn given_task_exists(world: &mut World, task_id: String) {
    // Create task via HTTP to ensure full stack works
    let request_body = json!({
        "title": "Test task",
        "description": "Created for testing"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/tasks")
        .header("content-type", "application/json")
        .body(Body::from(request_body.to_string()))
        .expect("valid request");

    let response = world.app.clone().oneshot(request).await.expect("request should succeed");
    assert_eq!(response.status(), 201, "Failed to create test task");

    // Store the created task ID in world state
    world.test_data.insert(task_id.clone(), task_id);
}

#[given(regex = r"^the following tasks exist:$")]
async fn given_tasks_exist(world: &mut World, step: &cucumber::Step) {
    let table = step.table().expect("table required");

    for row in table.rows.iter().skip(1) {
        // skip header
        let task_id = &row[0];
        let title = &row[1];
        let description = &row[2];

        let request_body = json!({
            "title": title,
            "description": description
        });

        let request = Request::builder()
            .method("POST")
            .uri("/tasks")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .expect("valid request");

        let response = world.app.clone().oneshot(request).await.expect("request should succeed");
        assert_eq!(response.status(), 201, "Failed to create test task");

        world.test_data.insert(task_id.clone(), task_id.clone());
    }
}

#[then(regex = r#"^the "([^"]+)" field equals "([^"]+)"$"#)]
async fn then_field_equals(world: &mut World, field: String, expected: String) {
    let response = world.last_response.as_ref().expect("response should exist");
    let actual = response
        .body
        .get(&field)
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("Field '{}' not found in response: {:?}", field, response.body));

    assert_eq!(actual, expected, "Field '{}' mismatch", field);
}

#[then(regex = r#"^the "([^"]+)" field contains "([^"]+)"$"#)]
async fn then_field_contains(world: &mut World, field: String, substring: String) {
    let response = world.last_response.as_ref().expect("response should exist");
    let actual = response.body.get(&field).and_then(|v| v.as_str()).unwrap_or_else(|| {
        // Maybe it's an object - try to serialize
        response.body.get(&field).map(|v| v.to_string()).unwrap_or_default()
    });

    assert!(
        actual.contains(&substring),
        "Field '{}' should contain '{}', got '{}'",
        field,
        substring,
        actual
    );
}

#[then(regex = r#"^the "([^"]+)" array contains (\d+) items$"#)]
async fn then_array_size(world: &mut World, field: String, count: String) {
    let response = world.last_response.as_ref().expect("response should exist");
    let array = response
        .body
        .get(&field)
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("Field '{}' is not an array: {:?}", field, response.body));

    let expected_count = count.parse::<usize>().expect("valid number");
    assert_eq!(array.len(), expected_count, "Array '{}' size mismatch", field);
}
```

**Update `crates/acceptance/src/steps/mod.rs`:**

```rust
pub mod template_core;
pub mod tasks;
```

**Update `crates/acceptance/src/world.rs` to add test_data:**

```rust
use std::collections::HashMap;

#[derive(Debug, World)]
pub struct World {
    pub app: Router,
    pub last_response: Option<Response>,
    pub request_headers: http::HeaderMap,
    pub test_data: HashMap<String, String>, // ← ADD THIS
}

impl Default for World {
    fn default() -> Self {
        Self {
            app: app_http::app(),
            last_response: None,
            request_headers: http::HeaderMap::new(),
            test_data: HashMap::new(), // ← ADD THIS
        }
    }
}
```

---

## Step 7: Run BDD Tests (5 min)

```bash
cargo run -p xtask -- bdd
```

**Expected output:**
```
Feature: Task Management

  @AC-TASK-001
  Scenario: Create a task successfully                         ✓

  @AC-TASK-002
  Scenario: Cannot create task with empty title                ✓

  @AC-TASK-002
  Scenario: Cannot create task with title exceeding 200 chars  ✓

  @AC-TASK-003
  Scenario: Retrieve a task by ID                              ✓

  @AC-TASK-004
  Scenario: Retrieve non-existent task returns 404             ✓

  @AC-TASK-005
  Scenario: List all tasks                                     ✓

Feature: Template Core Endpoints
  ... (4 scenarios pass)

2 features
10 scenarios (10 passed)
30 steps (30 passed)

✅ Acceptance tests passed
```

**If tests fail**, debug with:
```bash
cargo run -p xtask -- bdd --verbose
```

---

## Step 8: Generate AC Status (2 min)

```bash
cargo run -p xtask -- ac-status
```

**Open `docs/feature_status.md`:**

```markdown
| AC ID | Story | Requirement | Status | Scenarios |
|-------|-------|-------------|--------|----------|
| AC-TPL-001 | US-TPL-001 | REQ-TPL-HEALTH | ✅ pass | 1 |
| AC-TPL-002 | US-TPL-001 | REQ-TPL-VERSION | ✅ pass | 1 |
| AC-TPL-003 | US-TPL-001 | REQ-TPL-ERROR-HANDLING | ✅ pass | 1 |
| AC-TPL-004 | US-TPL-001 | REQ-TPL-ERROR-HANDLING | ✅ pass | 3 |
| AC-TASK-001 | US-TASK-001 | REQ-TASK-001 | ✅ pass | 1 |
| AC-TASK-002 | US-TASK-001 | REQ-TASK-001 | ✅ pass | 2 |
| AC-TASK-003 | US-TASK-001 | REQ-TASK-002 | ✅ pass | 1 |
| AC-TASK-004 | US-TASK-001 | REQ-TASK-002 | ✅ pass | 1 |
| AC-TASK-005 | US-TASK-001 | REQ-TASK-003 | ✅ pass | 1 |
```

**Your feature is now fully tracked!** Every AC has passing tests and is documented.

---

## Step 9: Generate LLM Bundle (3 min)

See what context an LLM would get for this feature:

```bash
cargo run -p xtask -- bundle implement_feature
```

**Check the output:** `.llm/bundle/implement_feature.md`

This bundle includes:
- Your spec ledger with all ACs
- User story markdown
- Feature files
- All application code (model, core, app-http)

**You can now paste this into an LLM** to ask questions like:
- "How does AC-TASK-002 validation work?"
- "Show me the data flow for creating a task"
- "Add a new AC for task deletion"

See [Use LLM Bundles](../how-to/use-llm-bundles.md) for workflows.

---

## Step 10: (Optional) Deploy to Dev (5 min)

If you have Kubernetes configured:

```bash
cargo run -p xtask -- deploy --env dev
```

This would:
1. Build Docker image
2. Push to registry
3. Apply Kubernetes manifests
4. Wait for rollout

*Note: Actual deployment requires infrastructure setup. See [Deploy to Dev](../how-to/deploy-dev.md).*

---

## What You Built

You just implemented a complete feature with:

- ✅ **5 ACs** covering creation, validation, retrieval, and listing
- ✅ **6 BDD scenarios** testing happy paths and error cases
- ✅ **3-layer architecture** (model → core → app-http)
- ✅ **Proper domain modeling** with Task, TaskId, TaskStatus
- ✅ **Validation in core** (title required, max length)
- ✅ **Error handling** with typed errors and HTTP mapping
- ✅ **Full test coverage** at integration level (BDD hits real HTTP stack)
- ✅ **AC traceability** (every behavior links to an AC)
- ✅ **LLM-ready context** (bundle shows entire feature)

---

## Key Patterns You Learned

### 1. Hexagonal Architecture

```
HTTP (app-http) → Core (core) → Model (model)
      ↓                ↓              ↓
    Routes         Use cases      Entities
    DTOs           Validation     Value objects
    Error mapping  Business logic Type safety
```

**Rule:** Dependencies point inward. Core never imports app-http.

### 2. AC-First Development

```
1. Spec:     Add AC to ledger
2. Scenario: Write Gherkin with @AC-XXX tag
3. Red:      Run BDD, see it fail
4. Implement: Add code to satisfy AC
5. Green:    Run BDD, see it pass
6. Verify:   Generate AC status
```

**Rule:** No code without an AC. No AC without a test.

### 3. Error Handling Layers

```
Core → CoreError (domain errors)
  ↓
HTTP → AppError (HTTP errors with status codes)
  ↓
BDD → Assert on status + error code
```

**Rule:** Core defines business errors, HTTP maps to status codes.

### 4. DTO vs Domain Model

```
HTTP Layer:     CreateTaskRequest, TaskResponse
                (JSON serialization, camelCase)

Domain Layer:   Task, TaskId, TaskStatus
                (type safety, business rules)
```

**Rule:** DTOs are HTTP boundary, domain models are core.

---

## Common Issues

### "Compile error: cannot find type `TaskStore`"

**Fix:** Add `core = { path = "../core" }` to `crates/app-http/Cargo.toml`

### "BDD scenario fails with 500 instead of 400"

**Cause:** Validation error not mapped correctly

**Fix:** Check error mapping in handler - ensure `CoreError::ValidationError` maps to `AppError::validation_error`

### "AC status shows 0 scenarios for AC-TASK-XXX"

**Cause:** BDD tag doesn't match ledger

**Fix:** Ensure exact match: `@AC-TASK-001` in feature file and `tag: "@AC-TASK-001"` in ledger

### "Task not found after creation"

**Cause:** Different `TaskStore` instances (not shared via Extension)

**Fix:** Ensure `Router` uses `.layer(Extension(task_store))` and all handlers extract it

---

## Next Steps

### Add More Features

Now that you understand the pattern, add:
- Task completion (PUT /tasks/:id/complete)
- Task deletion (DELETE /tasks/:id)
- Task search/filtering (GET /tasks?status=pending)

Each follows the same pattern:
1. Add AC to ledger
2. Write Gherkin scenario
3. Implement model → core → app-http
4. Run BDD
5. Generate AC status

### Add Persistence

Replace `TaskStore` with a real database:
1. Create `crates/app-db` crate
2. Define `TaskRepository` trait in `core`
3. Implement trait in `app-db` (PostgreSQL, MongoDB, etc.)
4. Inject repository into handlers

See [Architecture Explanation](../explanation/architecture.md) for details on ports and adapters.

### Add OpenAPI Spec

Document your API:
1. Edit `specs/openapi/openapi.yaml`
2. Add `/tasks` endpoints with schemas
3. Run `cargo run -p xtask -- check-openapi` (if configured)
4. CI will catch breaking changes

### Use LLM for Next AC

Try this workflow:
```bash
# Generate bundle
cargo run -p xtask -- bundle implement_ac

# Paste into LLM with:
"Looking at this task feature, implement AC-TASK-006:
'User can complete a task by ID'.

Show me:
1. Core function in crates/core/src/lib.rs
2. Handler in crates/app-http/src/lib.rs
3. BDD scenario in specs/features/tasks.feature"
```

See [Use LLM Bundles](../how-to/use-llm-bundles.md) for examples.

---

## Summary

You've completed a real-world feature using the full template workflow:

- **Planned** with user stories and ACs
- **Specified** with Gherkin scenarios
- **Implemented** across three layers (model, core, app-http)
- **Validated** with BDD tests hitting real HTTP stack
- **Tracked** with auto-generated AC status
- **Bundled** for LLM-assisted development

This is how you build **every** feature in this template. The structure gives you:
- Traceability (AC → test → code)
- Testability (BDD validates production paths)
- Maintainability (clear separation of concerns)
- LLM-friendliness (consistent structure, focused bundles)

**Now you're ready to build production features.** Go forth and ship!

See also:
- [Adoption Patterns](../explanation/adoption-patterns.md) - How to use this template in your organization
- [Use LLM Bundles](../how-to/use-llm-bundles.md) - AI-assisted development workflows
- [Architecture](../explanation/architecture.md) - Deep dive into design decisions
