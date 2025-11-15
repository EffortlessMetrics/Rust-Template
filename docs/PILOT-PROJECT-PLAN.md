# Pilot Project Plan: Task Management API

**Purpose:** Validate the v1.0.0 template through a realistic implementation using the full AC-first workflow.

**Timeline:** 1-2 sprints

**Goal:** Build a simple task management API that exercises all template features and captures friction points for v1.1.0 improvements.

---

## Why This Pilot?

The Task Management API is ideal because it:

- **Simple domain**: Tasks with CRUD operations + state transitions
- **Exercises HTTP layer**: Multiple endpoints, request validation, error handling
- **Tests domain logic**: Business rules, state machines, validation
- **Uses observability**: Request IDs, structured errors, tracing
- **Requires policies**: API design rules, error format standards
- **Small scope**: Can implement in 1-2 sprints
- **Familiar**: Everyone understands tasks, easy to reason about

---

## Project Setup

### 1. Create New Service from Template

```bash
# On GitHub
# 1. Navigate to rust-template repository
# 2. Click "Use this template" → "Create a new repository"
# 3. Name: "task-service-pilot"
# 4. Clone locally

git clone git@github.com:yourorg/task-service-pilot.git
cd task-service-pilot

# 5. Initialize Nix environment
nix develop

# 6. Verify template works
cargo run -p xtask -- selftest
```

### 2. Apply Minimal Protection Profile

```bash
# On GitHub repository settings
# 1. Go to Settings → Branches → Add branch protection rule
# 2. Branch name pattern: "main"
# 3. Enable:
#    - Require status checks before merging
#    - Require branches to be up to date
#    - Status checks: "Template Self-Test"
# 4. Save
```

### 3. Configure Repository

Update `Cargo.toml` project name and metadata:

```toml
[package]
name = "task-service"
version = "0.1.0"
edition = "2024"
description = "Task Management API - Template Pilot Project"
```

---

## Feature Roadmap

### Sprint 1: Core Task Management (2 weeks)

**Focus:** Basic CRUD operations, state transitions, full AC-first workflow

#### Feature 1: Create Task

**User Story:**

```yaml
# Add to specs/spec_ledger.yaml
user_stories:
  - id: US-001
    title: Create Task
    description: As a user, I want to create a task so I can track work items
    acceptance_criteria:
      - AC-001
      - AC-002
```

**Requirements:**

```yaml
requirements:
  - id: REQ-001
    title: Task Creation Endpoint
    description: Provide HTTP endpoint to create tasks with validation
    user_story: US-001
    priority: high
```

**Acceptance Criteria:**

```yaml
acceptance_criteria:
  - id: AC-001
    title: Valid Task Creation
    description: System accepts valid task creation requests
    requirement: REQ-001
    priority: must-have

  - id: AC-002
    title: Invalid Task Rejection
    description: System rejects tasks with invalid data
    requirement: REQ-001
    priority: must-have
```

**Gherkin Scenarios:**

```gherkin
# specs/features/task_creation.feature
Feature: Task Creation
  As a user
  I want to create tasks
  So I can track my work

  @AC-001
  Scenario: Create valid task
    Given the API is running
    When I POST to "/api/v1/tasks" with:
      """
      {
        "title": "Write pilot project report",
        "description": "Document friction points and learnings"
      }
      """
    Then the response status should be 201
    And the response should have header "x-request-id"
    And the response body should contain:
      """
      {
        "id": "<uuid>",
        "title": "Write pilot project report",
        "description": "Document friction points and learnings",
        "status": "open",
        "created_at": "<timestamp>"
      }
      """

  @AC-002
  Scenario: Reject task with missing title
    Given the API is running
    When I POST to "/api/v1/tasks" with:
      """
      {
        "description": "Task without title"
      }
      """
    Then the response status should be 400
    And the response should have header "x-request-id"
    And the response body should match error format:
      """
      {
        "error": "validation_error",
        "message": "Title is required",
        "request_id": "<uuid>"
      }
      """
```

**Implementation Checklist:**

- [ ] Add `Task` entity to `crates/core/src/domain/task.rs`
- [ ] Implement `CreateTaskRequest` validation
- [ ] Add `TaskService` with `create_task` method
- [ ] Implement `/api/v1/tasks` POST handler in `app-http`
- [ ] Add request ID middleware (already in template)
- [ ] Write BDD step definitions in `specs/steps/task_steps.rs`
- [ ] Run `cargo run -p xtask -- bdd` until green
- [ ] Run `cargo run -p xtask -- ac-status` to verify mapping
- [ ] Create policy for error format validation
- [ ] Run `cargo run -p xtask -- selftest`

#### Feature 2: List and Get Tasks

**ACs:**

- `AC-003`: List all tasks
- `AC-004`: Get task by ID
- `AC-005`: Return 404 for non-existent task

**Endpoints:**

- `GET /api/v1/tasks` - List all tasks
- `GET /api/v1/tasks/{id}` - Get specific task

#### Feature 3: Complete Task

**ACs:**

- `AC-006`: Mark task as complete
- `AC-007`: Reject completion of already-completed task
- `AC-008`: Track completion timestamp

**Endpoints:**

- `PUT /api/v1/tasks/{id}/complete` - Mark task complete

---

### Sprint 2: Advanced Features (2 weeks)

**Focus:** Persistence, filtering, observability validation

#### Feature 4: Task Persistence

**ACs:**

- `AC-009`: Tasks persist across restarts
- `AC-010`: Concurrent task creation is safe

**Implementation:**

- Add SQLite persistence layer
- Implement repository pattern
- Add database migrations

#### Feature 5: Task Filtering

**ACs:**

- `AC-011`: Filter tasks by status
- `AC-012`: Filter tasks by creation date range

**Endpoints:**

- `GET /api/v1/tasks?status=open`
- `GET /api/v1/tasks?created_after=2025-11-01`

#### Feature 6: Observability Validation

**ACs:**

- `AC-013`: All errors include request IDs
- `AC-014`: Request IDs appear in structured logs
- `AC-015`: Errors follow standard format

**Validation:**

- Review logs for request ID correlation
- Test error format compliance via policy tests
- Verify tracing spans

---

## Validation Checkpoints

### After Each Feature

Run the full validation suite:

```bash
# 1. Verify code quality
cargo run -p xtask -- check

# 2. Run BDD scenarios
cargo run -p xtask -- bdd

# 3. Verify AC mapping
cargo run -p xtask -- ac-status

# 4. Validate policies
cargo run -p xtask -- policy-test

# 5. Full selftest
cargo run -p xtask -- selftest

# 6. Generate LLM context
cargo run -p xtask -- bundle
```

### Document Friction Points

After each feature, document:

1. **What worked well?**
   - Which template features were helpful?
   - What felt natural?

2. **What was confusing?**
   - Where did you get stuck?
   - What docs were missing or unclear?

3. **What would improve the workflow?**
   - Missing xtask commands?
   - Better error messages?
   - Additional documentation?

Create a friction log:

```markdown
# Friction Log - Task Service Pilot

## Feature 1: Create Task

### ✅ What Worked
- AC-first workflow was clear
- Selftest caught missing error format
- Request ID middleware "just worked"

### ⚠️ Friction Points
- Had to look up Cucumber step definition syntax
- Unclear where to put domain validation vs HTTP validation
- Policy test error messages were cryptic

### 💡 Improvements for v1.1.0
- Add "Quick Start: Your First Feature" guide
- Document validation layer boundaries
- Improve conftest error formatting
```

---

## Success Criteria

The pilot is successful if:

1. ✅ All 15 ACs pass
2. ✅ `xtask selftest` passes on main branch
3. ✅ At least 3 friction points documented
4. ✅ Team completes pilot in < 3 sprints
5. ✅ Team would use template for next service

---

## Deliverables

1. **Working task-service repository**
   - All features implemented
   - All tests passing
   - Deployed (if applicable)

2. **Friction Log** (`docs/PILOT-FRICTION-LOG.md`)
   - What worked
   - What didn't
   - Specific improvement suggestions

3. **Retrospective Document** (`docs/PILOT-RETROSPECTIVE.md`)
   - Would you use this template again?
   - What would you change?
   - What should stay the same?

4. **v1.1.0 Backlog Issues**
   - Create GitHub issues for improvements
   - Prioritize by impact
   - Link to friction log

---

## Timeline

| Week | Focus | Deliverables |
|------|-------|--------------|
| 1 | Setup + Feature 1 (Create Task) | AC-001, AC-002 passing, First friction points documented |
| 2 | Features 2-3 (List/Get/Complete) | AC-003 through AC-008 passing |
| 3 | Feature 4 (Persistence) | AC-009, AC-010 passing, Database working |
| 4 | Features 5-6 + Polish | All ACs passing, Friction log complete |
| 5 | Retrospective + v1.1.0 Planning | Retro doc, GitHub issues created |

---

## How to Use This Plan

1. **Before Starting:**
   - Read the full plan
   - Set up the repository
   - Run `xtask selftest` to baseline

2. **During Implementation:**
   - Implement one feature at a time
   - Follow AC-first workflow strictly
   - Document friction immediately (don't wait)
   - Run validation after each feature

3. **After Completion:**
   - Complete retrospective
   - Create v1.1.0 issues
   - Share learnings with template maintainers

4. **Continuous:**
   - Update friction log as you go
   - Don't "work around" template limitations - document them
   - Ask questions and capture answers in docs

---

## Example: Complete Workflow for Feature 1

This shows the end-to-end flow for implementing "Create Task":

### Step 1: Define in Ledger

```bash
# Edit specs/spec_ledger.yaml
# Add US-001, REQ-001, AC-001, AC-002
```

### Step 2: Write Gherkin

```bash
# Create specs/features/task_creation.feature
# Add @AC-001 and @AC-002 scenarios
```

### Step 3: Run BDD (expect failures)

```bash
cargo run -p xtask -- bdd
# Expected: Scenarios fail (no implementation yet)
```

### Step 4: Implement Domain Logic

```rust
// crates/core/src/domain/task.rs
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
}

pub enum TaskStatus {
    Open,
    Completed { completed_at: DateTime<Utc> },
}

// Validation
impl Task {
    pub fn new(title: String, description: Option<String>) -> Result<Self, ValidationError> {
        if title.is_empty() {
            return Err(ValidationError::EmptyTitle);
        }
        Ok(Task {
            id: Uuid::new_v4(),
            title,
            description,
            status: TaskStatus::Open,
            created_at: Utc::now(),
        })
    }
}
```

### Step 5: Implement HTTP Handler

```rust
// crates/app-http/src/handlers/tasks.rs
pub async fn create_task(
    Json(req): Json<CreateTaskRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    let task = Task::new(req.title, req.description)
        .map_err(|e| AppError::validation("AC-002", &e.to_string()))?;

    Ok(Json(TaskResponse::from(task)))
}
```

### Step 6: Wire Up Routes

```rust
// crates/app-http/src/routes.rs
pub fn router() -> Router {
    Router::new()
        .route("/api/v1/tasks", post(handlers::tasks::create_task))
        .route("/health", get(handlers::health::health))
        .layer(middleware::from_fn(request_id_middleware))
}
```

### Step 7: Implement BDD Steps

```rust
// specs/steps/task_steps.rs
#[given("the API is running")]
async fn api_running(world: &mut TaskWorld) {
    world.start_server().await;
}

#[when(regex = r"^I POST to \"([^\"]*)\" with:$")]
async fn post_with_body(world: &mut TaskWorld, path: String, body: String) {
    world.response = world.client
        .post(&format!("http://localhost:3000{}", path))
        .json(&serde_json::from_str::<serde_json::Value>(&body).unwrap())
        .send()
        .await
        .unwrap();
}

#[then(regex = r"^the response status should be (\d+)$")]
async fn check_status(world: &mut TaskWorld, status: u16) {
    assert_eq!(world.response.status().as_u16(), status);
}
```

### Step 8: Run BDD Until Green

```bash
cargo run -p xtask -- bdd
# Iterate until: 2/2 scenarios passed
```

### Step 9: Verify AC Mapping

```bash
cargo run -p xtask -- ac-status
# Expected:
# ✅ AC-001: PASS
# ✅ AC-002: PASS
```

### Step 10: Add Policy Test

```rego
# policies/error_format.rego
package policies.error_format

deny[msg] {
    response := input.responses[_]
    response.status >= 400
    not response.body.error
    msg := sprintf("Error response missing 'error' field: %v", [response])
}
```

### Step 11: Run Policy Tests

```bash
cargo run -p xtask -- policy-test
# Expected: PASS
```

### Step 12: Full Validation

```bash
cargo run -p xtask -- selftest
# Expected: All checks pass
```

### Step 13: Commit

```bash
git add .
git commit -m "feat(tasks): implement task creation (AC-001, AC-002)"
git push
# CI runs and validates
```

### Step 14: Document Friction

Add to friction log:

```markdown
## Feature 1: Create Task (AC-001, AC-002)

### ✅ What Worked
- Clear separation: domain in core, HTTP in app-http
- Request ID middleware automatically added correlation
- BDD loop was fast and clear

### ⚠️ Friction
- Took 20 minutes to figure out Cucumber step regex syntax
- Unclear if validation belongs in Task::new or HTTP handler
- Had to manually wire up routes (would be nice to auto-discover)

### 💡 Ideas
- Add "Cucumber Cheat Sheet" to docs
- Add "Validation Boundaries" guide
- Consider axum-auto-routes crate
```

---

## Ready to Start?

Once v1.0.0 is tagged and protected, you can begin this pilot immediately. The plan is designed to:

- Exercise every template feature
- Follow the intended workflow exactly
- Surface real friction (not hypothetical)
- Generate actionable improvements for v1.1.0

**Next Steps:**

1. Tag v1.0.0
2. Create `task-service-pilot` repository from template
3. Start with Feature 1 (Create Task)
4. Document everything

Good luck! 🚀
