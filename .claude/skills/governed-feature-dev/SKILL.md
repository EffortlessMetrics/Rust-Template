---
name: governed-feature-dev
description: |
  AC-first feature development workflow for the Rust-as-Spec platform cell. Use when implementing new features, adding ACs, or working on tasks with status=Todo. Follows the ac_first flow from devex_flows.yaml and uses xtask + /platform APIs for governance.
allowed-tools:
- Read
- Grep
- Glob
- Edit
- Write
- Bash
---

# Governed Feature Development

## When to Use

Use this Skill when:
- Implementing a new feature
- Adding or updating Acceptance Criteria (ACs)
- Working on tasks with status=Todo or status=InProgress
- User says "implement feature X", "add AC", or "build functionality Y"

## Prerequisites

- Repository is governed (has `specs/spec_ledger.yaml`)
- Platform is running (check via `GET /platform/status`)
- Task exists or can be created
- You have access to `cargo xtask` commands

## Workflow

This Skill follows the **ac_first** flow from `specs/devex_flows.yaml`.

### 1. Discover work

Get prioritized tasks from the platform:

```bash
# Get agent hints (recommended tasks)
curl http://localhost:3000/platform/agent/hints | jq

# Filter for Todo tasks
curl http://localhost:3000/platform/agent/hints | jq '.tasks[] | select(.status == "Todo")'

# Or list all tasks
curl http://localhost:3000/platform/tasks | jq
```

**Output:** List of tasks with IDs, titles, ACs, and recommended flows.

**Decision:**
- **If AC exists:** Note the AC ID and proceed to step 2
- **If AC missing:** Ask user for details, then create AC (see substep below)

#### Create AC if missing:

```bash
cargo xtask ac-new AC-ID "AC description text" \
  --story US-ID \
  --requirement REQ-ID

# Example:
cargo xtask ac-new AC-MYSERV-USERS-LIST \
  "GET /users returns list of users" \
  --story US-MYSERV-001 \
  --requirement REQ-MYSERV-USERS
```

### 2. Claim task

Update task status to InProgress:

```bash
TASK_ID="TASK-TPL-XXX-001"  # Replace with actual task ID

curl -X POST "http://localhost:3000/platform/tasks/${TASK_ID}/status" \
  -H "Content-Type: application/json" \
  -d '{"status": "InProgress"}'
```

**Verify:**
```bash
curl http://localhost:3000/platform/tasks/${TASK_ID} | jq '.status'
# Should return: "InProgress"
```

### 3. Generate bounded context

Get focused context for the AC:

```bash
cargo xtask bundle implement_ac
```

**Output:** `.llm/bundle/implement_ac.md` (max 250KB)

**Read this file** to understand:
- Relevant requirements and ACs
- Existing code structure
- Related design docs (ADRs)
- Architectural patterns
- Test examples

### 4. Write BDD scenario (Test-First)

Create or update a feature file in `specs/features/*.feature`:

```gherkin
@AC-YOUR-AC-ID
Scenario: Description matching AC text
  Given preconditions
  When action
  Then expected outcome
```

**Example:**
```gherkin
@AC-MYSERV-USERS-LIST
Scenario: GET /users returns list of users
  Given I am an authenticated user
  When I send GET to "/users"
  Then the response status should be 200
  And the response should be valid JSON
  And the response should contain a list of users
```

**Verify scenario is discovered:**
```bash
cargo xtask bdd --dry-run | grep "@AC-YOUR-AC-ID"
```

**Run BDD (should fail - not implemented yet):**
```bash
cargo xtask bdd
# Expected: Scenario fails (pending step definitions)
```

### 5. Implement the feature

Write code in the appropriate crate:

**Architecture guidelines:**
- **Business logic:** `crates/business-core`
- **HTTP handlers:** `crates/app-http`
- **Data access:** `crates/adapters`
- Follow hexagonal architecture (adapters → core)
- Use patterns from bundle context

**Example structure:**
```rust
// crates/business-core/src/users.rs
pub struct UserService {
    // core logic
}

impl UserService {
    pub fn list_users(&self) -> Result<Vec<User>> {
        // implementation
    }
}

// crates/app-http/src/routes/users.rs
pub async fn list_users_handler(
    State(service): State<UserService>
) -> Result<Json<Vec<User>>> {
    let users = service.list_users()?;
    Ok(Json(users))
}
```

**Register route:**
```rust
// crates/app-http/src/app.rs
.route("/users", get(users::list_users_handler))
```

### 6. Run BDD tests

```bash
cargo xtask bdd
```

**Expected:** Scenarios pass ✅

**If scenarios fail:**
- Check step definitions in `crates/acceptance/src/steps/`
- Verify scenario syntax (Gherkin)
- Debug feature implementation

### 7. Update AC mapping in spec_ledger.yaml

Ensure the AC has a test reference:

```yaml
acceptance_criteria:
  - id: AC-YOUR-AC-ID
    text: "Description"
    tests:
      - { type: bdd, tag: "@AC-YOUR-AC-ID" }  # Must match feature file tag
```

### 8. Run full validation

```bash
# Full governance check
cargo xtask selftest

# Or with low resources flag if needed
XTASK_LOW_RESOURCES=1 cargo xtask selftest
```

**Selftest steps (7 total):**
1. Core checks (fmt + clippy + tests)
2. BDD acceptance tests
3. AC mapping and coverage
4. LLM bundler validation
5. Policy tests
6. DevEx flows validation
7. Graph invariants

**If selftest fails:** Use the `governed-governance-debug` Skill to diagnose and fix.

### 9. Mark task as Done

```bash
curl -X POST "http://localhost:3000/platform/tasks/${TASK_ID}/status" \
  -H "Content-Type: application/json" \
  -d '{"status": "Done"}'
```

**Verify:**
```bash
curl http://localhost:3000/platform/tasks/${TASK_ID} | jq '.status'
# Should return: "Done"
```

## Exit Criteria

Feature is complete when:
- ✅ AC exists in `specs/spec_ledger.yaml`
- ✅ BDD scenario exists with matching `@AC-ID` tag
- ✅ Code implements the behavior
- ✅ `cargo xtask bdd` passes
- ✅ `cargo xtask selftest` passes (11/11 steps)
- ✅ Task status = Done
- ✅ No new policy violations

**Then:** Feature is ready for commit/PR.

## Error Handling

### If selftest fails:

**Use the `governed-governance-debug` Skill** to systematically diagnose which of the 7 steps failed and how to fix it.

### If BDD fails:

```bash
# Run with verbose output
cargo xtask bdd -- --format pretty

# Common issues:
# 1. Missing step definitions
#    → Check crates/acceptance/src/steps/
#    → Add missing step implementations

# 2. Feature syntax errors
#    → Validate Gherkin syntax
#    → Ensure proper indentation

# 3. Scenario tag mismatch
#    → Verify @AC-ID matches spec_ledger.yaml
```

### If platform APIs not reachable:

```bash
# Check if platform is running
curl http://localhost:3000/platform/status

# If not running:
cargo run -p app-http &
sleep 5

# Retry API calls
```

### If AC creation fails:

```bash
# Ensure parent requirement exists
grep "REQ-ID" specs/spec_ledger.yaml

# If missing, create requirement first (ask user for details)
```

## Examples

### Example 1: Implement existing AC

```bash
# 1. Discover work
curl http://localhost:3000/platform/agent/hints | jq '.tasks[0]'
# { "id": "TASK-TPL-USERS-001", "acs": ["AC-MYSERV-USERS-LIST"], ... }

# 2. Claim task
curl -X POST "http://localhost:3000/platform/tasks/TASK-TPL-USERS-001/status" \
  -H "Content-Type: application/json" -d '{"status": "InProgress"}'

# 3. Get context
cargo xtask bundle implement_ac
cat .llm/bundle/implement_ac.md

# 4. Write BDD
cat >> specs/features/users.feature <<'EOF'
@AC-MYSERV-USERS-LIST
Scenario: List users
  Given I am authenticated
  When I GET /users
  Then the response should be 200
EOF

# 5. Implement
# ... write code in crates/app-http/src/routes/users.rs ...

# 6. Test
cargo xtask bdd
cargo xtask selftest

# 7. Complete
curl -X POST "http://localhost:3000/platform/tasks/TASK-TPL-USERS-001/status" \
  -H "Content-Type: application/json" -d '{"status": "Done"}'
```

### Example 2: Create new AC and implement

```bash
# User asks: "Add endpoint to delete a user"

# 1. Create AC
cargo xtask ac-new AC-MYSERV-USERS-DELETE \
  "DELETE /users/:id removes the user" \
  --requirement REQ-MYSERV-USERS

# 2. Follow steps 3-9 from main workflow
# (bundle, BDD, implement, test, validate, mark done)
```

## Boundaries

**What this Skill does:**
✅ Guide AC-first development workflow
✅ Ensure governance contracts are followed
✅ Validate work via selftest
✅ Integrate with platform APIs for task management

**What this Skill does NOT do:**
❌ Generate complete code implementations (you write the code)
❌ Bypass governance (selftest is mandatory)
❌ Make architecture decisions (those need ADRs via `adr-new`)
❌ Deploy to production (use `governed-release` Skill)

## Success Criteria

Feature implementation successful when:
- ✅ All Exit Criteria met (see above)
- ✅ Code follows hexagonal architecture
- ✅ BDD scenarios provide clear acceptance evidence
- ✅ AC is traceable from requirement to test
- ✅ No governance drift introduced

## References

- **Flow definition:** `specs/devex_flows.yaml` (ac_first flow)
- **Bundler docs:** `docs/explanation/llm-bundler.md`
- **Architecture guide:** `docs/explanation/hexagonal-architecture.md`
- **Platform APIs:** http://localhost:3000/platform/status
- **Task board UI:** http://localhost:3000/ui/tasks
- **xtask reference:** `docs/reference/xtask-commands.md`

## Notes

- **AC-first is the contract:** Feature must start with an AC
- **BDD provides evidence:** Scenarios prove the AC is met
- **Selftest is the gate:** Passing selftest means work is valid
- **Platform APIs are authoritative:** Use APIs, not direct YAML parsing
- **Bundle provides context:** Always read it before implementing
