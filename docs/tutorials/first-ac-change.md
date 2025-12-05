# Tutorial: Your First AC Change

This tutorial walks you through the complete AC-first development loop: from spec to code to test to verification.

**Time:** 15 minutes
**Prerequisites:** Template cloned, Nix devShell working

---

## The AC-First Loop

When developing with this template, follow this order:

1. **Spec**: Add or update AC in the ledger
2. **Scenario**: Create/update Gherkin scenario with `@AC-####`
3. **Code**: Implement the behavior
4. **Test**: Run acceptance tests
5. **Verify**: Check `feature_status.md`

Let's walk through each step with a platform-native example.

---

## Step 1: Add AC to the Ledger

Open `specs/spec_ledger.yaml` and add a new AC.

We'll add a new platform endpoint for task listing:

```yaml
stories:
  - id: US-PLT-TASKS
    title: "Platform Task Visibility"
    requirements:
      - id: REQ-PLT-TASKS-LIST
        text: "Platform exposes task list for agent consumption"
        acceptance_criteria:
          - id: AC-PLT-TASKS-LIST
            text: "GET /platform/tasks returns task list with status"
            tests: [{ type: bdd, tag: "@AC-PLT-TASKS-LIST" }]
```

**Key points:**
- Use descriptive AC IDs with prefixes (PLT for platform, TPL for template)
- Write clear, testable behavior statement
- Reference the BDD tag you'll use in the scenario

---

## Step 2: Write the BDD Scenario

Create or update `specs/features/platform_tasks.feature`:

```gherkin
Feature: Platform Task Visibility
  As a developer or agent
  I want to query platform tasks
  So that I can understand current work items

@AC-PLT-TASKS-LIST
Scenario: List all tasks
  When I GET /platform/tasks
  Then I receive 200
  And the response is JSON
  And the response contains "tasks" array
```

---

## Step 3: Implement the Handler

Add the route and handler in `crates/app-http/src/routes/platform.rs`:

```rust
pub async fn list_tasks() -> impl IntoResponse {
    let tasks = vec![]; // Load from tasks.yaml
    Json(serde_json::json!({ "tasks": tasks }))
}
```

Wire it in the router:

```rust
.route("/platform/tasks", get(list_tasks))
```

---

## Step 4: Run Tests

```bash
cargo xtask bdd
cargo xtask ac-status
```

---

## Step 5: Verify

Check the feature status report:

```bash
cat docs/feature_status.md
```

Your new AC should show as passing.

---

## Complete Loop Summary

This tutorial demonstrates the AC-first approach:

1. **Spec first** - Define what success looks like
2. **Test second** - Write the scenario before code
3. **Code third** - Implement to make tests pass
4. **Validate always** - Use xtask commands to verify

For full examples and troubleshooting, see `docs/explanation/architecture.md`.
