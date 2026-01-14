<!-- doclint:disable orphan-version -->
# Pilot Feature Ideas

**Purpose**: Suggested features for pilot projects that test the template across different complexity levels.

**Template Version**: v2.4.0
**Last Updated**: 2025-11-17

---

## Using This Guide

When starting a pilot project, pick 3-5 features that match your complexity needs:

- **Simple features** (30-60 min) - Test basic AC-to-code flow
- **Medium features** (2-4 hours) - Test multi-layer architecture and validation
- **Complex features** (1-2 days) - Test full observability, error handling, and integration

**Goal**: Expose friction in the template by building real features, not toy examples.

---

## Task Management API (Starter Pilot)

A simple CRUD API for managing tasks. Good for first-time pilot projects.

### Simple Features

#### 1. Create Task

**AC**: `AC-TSK-001`
**Story**: As a user, I can create a new task with a title and description.

**BDD Scenario**:

```gherkin
Feature: Task Management
  Scenario: User creates a task
    When I POST to "/tasks" with:
      """
      {
        "title": "Buy groceries",
        "description": "Get milk, eggs, bread"
      }
      """
    Then the response status is 201
    And the response body contains "id"
    And the task is stored with status "pending"
```

**What it tests**:
- Basic HTTP endpoint (POST)
- Model creation (Task entity)
- Core domain logic (simple validation)
- Database persistence (if using adapter)
- BDD test integration

**Expected friction**:
- How easy is it to add a new model?
- Do DTO ↔ model conversions feel natural?
- Is error handling clear?

---

#### 2. List All Tasks

**AC**: `AC-TSK-002`
**Story**: As a user, I can list all tasks.

**BDD Scenario**:

```gherkin
  Scenario: User lists all tasks
    Given the following tasks exist:
      | title        | status  |
      | Buy groceries| pending |
      | Write report | pending |
    When I GET "/tasks"
    Then the response status is 200
    And the response contains 2 tasks
```

**What it tests**:
- HTTP GET endpoint
- Repository pattern (list operation)
- Response serialization

**Expected friction**:
- How does pagination get added later?
- Is the repository pattern clear from examples?

---

### Medium Features

#### 3. Update Task Status

**AC**: `AC-TSK-003`
**Story**: As a user, I can update a task's status to "in_progress" or "completed".

**BDD Scenario**:

```gherkin
  Scenario: User completes a task
    Given a task "Buy groceries" exists with status "pending"
    When I PATCH "/tasks/{id}" with:
      """
      { "status": "completed" }
      """
    Then the response status is 200
    And the task status is "completed"
```

**What it tests**:
- PATCH endpoint
- State transition validation (pending → in_progress → completed)
- Domain rules in core layer
- Error handling (invalid status transitions)

**Expected friction**:
- Where do status transition rules live?
- How clear is validation error messaging?
- Does the LLM understand hexagonal layering?

---

#### 4. Search Tasks by Status

**AC**: `AC-TSK-004`
**Story**: As a user, I can filter tasks by status.

**BDD Scenario**:

```gherkin
  Scenario: User filters tasks by status
    Given the following tasks exist:
      | title        | status      |
      | Buy groceries| completed   |
      | Write report | in_progress |
      | Clean house  | pending     |
    When I GET "/tasks?status=pending"
    Then the response status is 200
    And the response contains 1 task
    And the task title is "Clean house"
```

**What it tests**:
- Query parameters
- Repository filtering logic
- Core domain queries

**Expected friction**:
- How do query params map to filters?
- Is repository pattern extensible?

---

### Complex Features

#### 5. Task Metrics with Tracing

**AC**: `AC-TSK-005`
**Story**: As an operator, I can see task completion metrics with distributed tracing.

**Implementation**:
- Add prometheus metrics counter for task completions
- Add tracing spans for task operations
- Export via OTLP (if `telemetry/otlp` feature enabled)

**BDD Scenario**:

```gherkin
  Scenario: Task completion emits metrics
    When I complete a task
    Then the "tasks_completed_total" counter increments
    And a trace span is emitted with operation "complete_task"
```

**What it tests**:
- Metrics integration (from v2.1.0)
- OTLP tracing (from v2.3.0)
- Feature flag usage (`telemetry/otlp`)
- Observability in hexagonal layers

**Expected friction**:
- Are telemetry examples clear?
- Does the LLM understand where to add metrics?
- Is OTLP setup documented well enough?

---

#### 6. Task Dependencies

**AC**: `AC-TSK-006`
**Story**: As a user, I can mark that Task A must complete before Task B can start.

**BDD Scenario**:

```gherkin
  Scenario: User creates dependent task
    Given task "Setup environment" exists
    When I POST to "/tasks" with:
      """
      {
        "title": "Run tests",
        "depends_on": ["setup-environment-id"]
      }
      """
    Then the response status is 201
    And the task is created with status "blocked"

  Scenario: Dependent task unblocks on completion
    Given task "Setup environment" is blocking task "Run tests"
    When task "Setup environment" is completed
    Then task "Run tests" status changes to "pending"
```

**What it tests**:
- Complex domain logic (dependency graph)
- State transitions with external dependencies
- Domain events (task completion triggers status change)
- Error handling (circular dependencies, missing dependencies)

**Expected friction**:
- Where do complex domain rules live?
- How does event propagation work?
- Is error handling for domain invariants clear?

---

## E-commerce Order API (Intermediate Pilot)

For pilots that need more complex business logic and state machines.

### Simple Features

#### 1. Create Order

**AC**: `AC-ORD-001`
**Story**: As a customer, I can create an order with line items.

**What it tests**:
- Nested entities (Order has many LineItems)
- Value objects (Money, Quantity)
- Basic validation (non-empty order)

---

#### 2. Calculate Order Total

**AC**: `AC-ORD-002`
**Story**: As a customer, I can see the total price of my order.

**What it tests**:
- Domain calculations in core layer
- Money arithmetic
- DTO transformation

---

### Medium Features

#### 3. Apply Discount Code

**AC**: `AC-ORD-003`
**Story**: As a customer, I can apply a discount code to my order.

**What it tests**:
- External service integration (discount service)
- Error handling (invalid code, expired code)
- Domain calculation with business rules

**Expected friction**:
- Where does external service integration go?
- How are adapter boundaries defined?

---

#### 4. Transition Order Status

**AC**: `AC-ORD-004`
**Story**: As a system, orders transition through states: pending → paid → shipped → delivered.

**What it tests**:
- State machine in domain layer
- Invalid transition handling
- Status-based business rules

---

### Complex Features

#### 5. Order Fulfillment with Tracing

**AC**: `AC-ORD-005`
**Story**: As an operator, I can trace order fulfillment across services.

**What it tests**:
- Distributed tracing across adapters
- OTLP integration with external services
- Metrics for fulfillment SLOs

---

#### 6. Idempotent Payment Processing

**AC**: `AC-ORD-006`
**Story**: As a system, duplicate payment requests for the same order are idempotent.

**What it tests**:
- Idempotency keys
- Database transaction handling
- Race condition handling
- Adapter integration (payment gateway)

**Expected friction**:
- How does database adapter integrate?
- Is transaction handling documented?
- How do adapters handle retries?

---

## User Authentication API (Advanced Pilot)

For pilots testing security, secrets management, and policy enforcement.

### Simple Features

#### 1. Register User

**AC**: `AC-AUTH-001`
**Story**: As a new user, I can register with email and password.

**What it tests**:
- Password hashing
- Input validation (email format)
- Database persistence

---

#### 2. Login User

**AC**: `AC-AUTH-002`
**Story**: As a registered user, I can log in and receive a session token.

**What it tests**:
- Password verification
- JWT token generation
- Error handling (wrong password)

---

### Medium Features

#### 3. Password Reset Flow

**AC**: `AC-AUTH-003`
**Story**: As a user, I can reset my password via email.

**What it tests**:
- Async workflows (send email)
- Time-limited tokens
- State management (reset tokens)

---

#### 4. Privacy Policy Check

**AC**: `AC-AUTH-004`
**Story**: As a policy enforcer, I ensure user emails are never logged or traced.

**What it tests**:
- OPA/Rego privacy policies
- Policy enforcement in telemetry
- Conftest integration

**Expected friction**:
- How do privacy policies work?
- Are policy examples clear?
- Does `policy-test` catch violations?

---

### Complex Features

#### 5. OAuth Integration

**AC**: `AC-AUTH-005`
**Story**: As a user, I can log in with GitHub OAuth.

**What it tests**:
- External OAuth flow
- Adapter integration
- State management (OAuth state parameter)
- Token exchange

**Expected friction**:
- Where do OAuth adapters live?
- How are secrets managed?
- Is adapter testing documented?

---

#### 6. Audit Trail with OTLP

**AC**: `AC-AUTH-006`
**Story**: As a security operator, I can trace all login attempts with OTLP.

**What it tests**:
- Security event logging
- OTLP tracing for audit
- Metrics for failed login attempts (security monitoring)

---

## Recommendations

### For Your First Pilot

**Start with Task Management API**:
1. AC-TSK-001 (Create Task) - 30 min
2. AC-TSK-002 (List Tasks) - 20 min
3. AC-TSK-003 (Update Status) - 60 min
4. AC-TSK-005 (Metrics + Tracing) - 90 min

**Total**: Half-day pilot that touches:
- Basic HTTP endpoints
- Domain validation
- Database persistence
- Observability stack

**Record friction after each feature** in `FRICTION_LOG.md`.

---

### For a Week-Long Pilot

**Full Task Management + Metrics**:
- All 6 Task Management features
- Add gRPC adapter integration (if testing adapters)
- Add BDD scenarios for all features
- Test LLM bundler on 3-4 features

**Expected outcomes**:
- Identify 5-10 friction points
- Validate LLM workflow end-to-end
- Understand template strengths/gaps
- Inform v2.3.1 or v2.4.0 planning

---

## Anti-Patterns

**Don't do this**:
- Implement features without BDD scenarios (skips governance testing)
- Skip friction logging (defeats pilot purpose)
- Fix friction in template during pilot (breaks boundary)
- Build toy examples that don't need observability/governance

**Do this**:
- Treat pilot as real service development
- Use LLM bundler for every feature
- Document every rough edge
- Test failure paths (invalid input, missing data)
- Enable OTLP and validate traces

---

## Next Steps

After completing your pilot:

1. **Review `FRICTION_LOG.md`**
   - Classify entries: 🔴 Blockers / 🟡 Annoyances / 🟢 Nice-to-have

2. **Decide on template action**
   - Blockers → Plan v2.3.1 patch
   - Annoyances → Plan v2.4.0 features
   - Good enough → Use template as-is

3. **Extract patterns**
   - Did certain features feel natural? → Document as examples
   - Did certain flows feel broken? → Fix in template
   - Did LLM workflow succeed? → Validate approach

4. **Use Release Playbook**
   - Follow [docs/RELEASE_PLAYBOOK.md](../RELEASE_PLAYBOOK.md)
   - Create release plan from friction log analysis
   - Apply governance process to template evolution

---

**Remember**: The pilot isn't about perfecting features—it's about finding template friction through real usage.
