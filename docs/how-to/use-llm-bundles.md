# How-to: Use LLM Context Bundles

This guide shows you how to use the template's LLM context bundler to get AI assistance with focused, relevant context.

**Time:** 5 minutes
**Prerequisites:** Template cloned, xtask working

---

## What Are Context Bundles?

Context bundles are curated collections of files packaged as markdown, designed to give LLMs exactly the context they need for specific tasks without overwhelming them with irrelevant code.

**Benefits:**
- **Focused**: Only includes files relevant to the task
- **Bounded**: Respects token limits via `max_bytes`
- **Versioned**: Generated from specific git commits
- **Repeatable**: Same task → same bundle structure

---

## Available Tasks

Check `.llm/contextpack.yaml` for configured tasks:

```bash
cargo run -p xtask -- bundle implement_ac       # AC implementation context
cargo run -p xtask -- bundle implement_feature  # Feature development context
cargo run -p xtask -- bundle debug_tests        # Test debugging context
```

Each task defines:
- **include**: Glob patterns for files to bundle
- **max_bytes**: Size limit (default: 250KB)
- **description**: What the bundle is for

---

## When to Use LLM Bundles

### ✅ **Good Use Cases**

**1. Implementing or changing an AC**
```bash
# Generate context
cargo run -p xtask -- bundle implement_ac

# Paste bundle into LLM with prompt:
"Here is our ledger, specs, features, and core code.
Implement behavior to satisfy AC-123: 'Customer can view refund status'.
Show me the diffs you'd make to:
1. specs/spec_ledger.yaml
2. specs/features/*.feature
3. crates/core/src/*.rs
4. crates/acceptance/src/steps/*.rs"
```

**2. Investigating test failures**
```bash
# Generate debugging context
cargo run -p xtask -- bundle debug_tests

# Paste bundle with:
"The BDD scenario for AC-123 is failing with error: [paste error].
What's the likely cause and how should I fix it?"
```

**3. Understanding feature-AC relationships**
```bash
cargo run -p xtask -- bundle implement_feature

# Ask:
"Show me all ACs related to the refunds feature and their current implementation status."
```

### ❌ **Bad Use Cases**

**Don't use bundles for:**
- **Architecture changes** - too broad, needs human design
- **Policy changes** - requires careful human review and approval
- **Inventing new IDs** - AC/FT/flag IDs must come from the ledger
- **Cross-cutting refactors** - bundler can't capture all dependencies

---

## Best Practices

### 1. **Always verify LLM output**

LLMs can hallucinate. Before applying changes:
- ✅ Check AC IDs exist in `specs/spec_ledger.yaml`
- ✅ Verify BDD tags match ledger references
- ✅ Run `xtask check` after changes
- ✅ Run `xtask bdd` to validate scenarios

### 2. **Use specific prompts**

**Bad prompt:**
> "Fix this code"

**Good prompt:**
> "AC-123 requires that refund status is returned in the response.
> Current code at crates/core/src/refunds.rs:42 doesn't include status.
> Show me the minimal diff to add status field to RefundResponse."

### 3. **Don't let LLMs invent governance artifacts**

**Never accept:**
- New AC IDs not in the ledger
- New flag keys without owner/rollout plan
- New PII fields without retention policy

**Always:**
- Add new ACs to ledger first, then ask LLM to implement
- Define flags in `flags/registry.yaml` before referencing
- Add PII fields to `specs/privacy.yaml` with owner + retention

### 4. **Use bundles iteratively**

If the LLM's first attempt isn't quite right:
- Regenerate the bundle (it's fast)
- Provide the LLM's previous attempt as context
- Add specific constraints or examples

---

## Common Workflows

### Workflow 1: Add a new AC

**Scenario:** You need to add email notification when a refund is approved.

```bash
# 1. Human adds AC to ledger FIRST (governance requirement)
vim specs/spec_ledger.yaml
```

Add to ledger:
```yaml
- id: AC-125
  text: "Customer receives email when refund is approved"
  tests: [{ type: bdd, tag: "@AC-125" }]
```

```bash
# 2. Validate AC exists in ledger
cargo run -p xtask -- policy-test

# 3. Generate bundle with ledger context
cargo run -p xtask -- bundle implement_ac
```

**Prompt to LLM:**
```
I've added AC-125 to the ledger: "Customer receives email when refund is approved"

Using the provided bundle context, implement this AC. Show me:

1. New Gherkin scenario in specs/features/refunds.feature with @AC-125 tag
2. Step definitions in crates/acceptance/src/steps/refunds.rs
3. Core email notification logic in crates/core/src/notifications.rs
4. Handler updates in crates/app-http/src/lib.rs to trigger notification

Requirements:
- Use existing error handling patterns
- Add tracing spans for observability
- Include AC-125 in error messages for traceability
- Do NOT invent new AC IDs or feature flags
```

**Apply and validate:**
```bash
# 4. Review LLM output carefully
# 5. Apply changes to files
# 6. Run tests
cargo run -p xtask -- bdd

# 7. Verify AC shows up in status
cargo run -p xtask -- ac-status

# 8. Check AC-125 is passing
grep "AC-125" docs/feature_status.md
# Should show: | AC-125 | ... | ✅ pass | 1 |
```

---

### Workflow 2: Debug a failing scenario

**Scenario:** BDD test is failing with unexpected status code.

```bash
# 1. Run tests, capture failure
cargo run -p xtask -- bdd 2>&1 | tee test-output.txt

# Example failure:
# Scenario: Create a refund
#   ✗ Expected 201, got 400
#   Body: {"error":"INVALID_INPUT","message":"Amount must be positive"}
```

```bash
# 2. Generate debug bundle
cargo run -p xtask -- bundle debug_tests
```

**Prompt to LLM:**
```
The BDD scenario for AC-123 "Create a refund" is failing.

Expected: 201 Created
Actual: 400 Bad Request
Body: {"error":"INVALID_INPUT","message":"Amount must be positive"}

Looking at the bundle:
1. What validation is failing in the core layer?
2. Is the step definition passing the correct data?
3. What's the expected validation rule for amount?
4. Show me the exact fix needed

Context: The scenario is:
```gherkin
Given an order "ORD-123" totalling 5000 cents
When I POST /refunds with { "orderId": "ORD-123", "amountCents": 5000 }
Then I receive 201 with a "refundId"
```
```

**LLM might respond:**

> Looking at `crates/core/src/refunds.rs:42`, the validation requires:
> ```rust
> if amount_cents <= 0 {
>     return Err(CoreError::ValidationError { ... });
> }
> ```
>
> But the step definition in `crates/acceptance/src/steps/refunds.rs:15` is passing:
> ```rust
> amount_cents: -5000  // ← Bug: negative!
> ```
>
> **Fix:** Change step definition to:
> ```rust
> amount_cents: 5000  // Positive value
> ```

```bash
# 3. Apply fix
vim crates/acceptance/src/steps/refunds.rs

# 4. Re-run tests
cargo run -p xtask -- bdd
# ✓ All scenarios pass
```

---

### Workflow 3: Understand what an AC actually does

**Scenario:** New team member needs to understand existing ACs.

```bash
# Generate context
cargo run -p xtask -- bundle implement_ac
```

**Prompt to LLM:**
```
Explain what AC-TPL-001 actually requires and how it's implemented.

Show me:
1. The exact AC text from the ledger
2. The Gherkin scenario that tests it
3. The handler code that implements it
4. What would break if I removed this AC?
```

**LLM response:**

> **AC-TPL-001:** "GET /health returns 200 with status 'ok' when service is healthy"
>
> **Ledger entry:**
> ```yaml
> - id: AC-TPL-001
>   text: "GET /health returns 200 with status 'ok' when service is healthy"
>   tests: [{ type: bdd, tag: "@AC-TPL-001" }]
> ```
>
> **Gherkin scenario:** (specs/features/template_core.feature:7)
> ```gherkin
> @AC-TPL-001 @smoke
> Scenario: Health endpoint reports service is healthy
>   When I GET /health
>   Then I receive 200 with status "ok"
> ```
>
> **Implementation:** (crates/app-http/src/lib.rs:56)
> ```rust
> async fn health(Extension(_request_id): Extension<RequestId>) -> impl IntoResponse {
>     info!("Health check requested");
>     Json(HealthResponse {
>         status: "ok".to_string(),
>         service: "service-api".to_string()
>     })
> }
> ```
>
> **What would break:**
> - Kubernetes liveness/readiness probes would fail
> - Load balancers couldn't determine service health
> - Monitoring would have no heartbeat signal

---

### Workflow 4: Implement a new feature end-to-end

**Scenario:** Add task completion feature (builds on Day 7 tutorial).

```bash
# 1. Plan ACs (human decision)
vim specs/spec_ledger.yaml
```

Add:
```yaml
- id: AC-TASK-006
  text: "User can mark a task as completed"
  tests: [{ type: bdd, tag: "@AC-TASK-006" }]
- id: AC-TASK-007
  text: "Completed tasks have status 'completed' and timestamp"
  tests: [{ type: bdd, tag: "@AC-TASK-007" }]
```

```bash
# 2. Generate bundle
cargo run -p xtask -- bundle implement_feature
```

**Prompt to LLM:**
```
Implement task completion feature with ACs:
- AC-TASK-006: "User can mark a task as completed"
- AC-TASK-007: "Completed tasks have status 'completed' and timestamp"

Show me complete implementation:

1. **Model changes** (crates/model/src/lib.rs):
   - Add `completed_at: Option<DateTime>` to Task
   - Method: `task.complete() -> Result<Task, CoreError>`

2. **Core logic** (crates/core/src/lib.rs):
   - Function: `complete_task(store, task_id) -> Result<Task, CoreError>`
   - Validation: Task exists, not already completed

3. **HTTP handler** (crates/app-http/src/lib.rs):
   - Route: PUT /tasks/:id/complete
   - Handler: `complete_task_handler`
   - Response DTO: TaskResponse (updated)

4. **BDD scenarios** (specs/features/tasks.feature):
   - Two scenarios tagged @AC-TASK-006 and @AC-TASK-007

5. **Step definitions** (crates/acceptance/src/steps/tasks.rs):
   - New steps for PUT requests and timestamp validation

Use existing patterns from the bundle. Do not invent new error codes.
```

```bash
# 3. Apply LLM suggestions
# (Review each file carefully)

# 4. Validate
cargo run -p xtask -- selftest

# If errors, iterate:
cargo run -p xtask -- bundle debug_tests
# Paste error, ask LLM for fix
```

---

### Workflow 5: Refactor with LLM assistance

**Scenario:** Extract repeated validation logic into reusable function.

```bash
cargo run -p xtask -- bundle implement_feature
```

**Prompt to LLM:**
```
I see repeated validation pattern in crates/core/src/lib.rs:

```rust
// In create_task:
if title.trim().is_empty() {
    return Err(CoreError::ValidationError { ... });
}

// In update_task:
if title.trim().is_empty() {
    return Err(CoreError::ValidationError { ... });
}
```

Refactor to a shared validation function:

1. Create `validate_task_title(title: &str) -> Result<(), CoreError>`
2. Use it in both `create_task` and `update_task`
3. Ensure error messages stay the same (for AC traceability)
4. Update any affected tests

Show me the diffs.
```

---

### Workflow 6: Policy-protected LLM workflow

**Scenario:** LLM tries to invent a new AC ID.

```bash
cargo run -p xtask -- bundle implement_ac
```

**Prompt to LLM:**
```
Add a feature that deletes tasks.
```

**LLM might respond:**

> I'll add AC-TASK-999 for task deletion:
>
> ```yaml
> - id: AC-TASK-999
>   text: "User can delete a task by ID"
> ```

**DON'T apply this!** The LLM invented an AC ID.

**Correct workflow:**

```bash
# 1. Human adds AC to ledger FIRST
vim specs/spec_ledger.yaml
```

Add:
```yaml
- id: AC-TASK-008  # ← Sequential ID, not LLM's AC-999
  text: "User can delete a task by ID"
  tests: [{ type: bdd, tag: "@AC-TASK-008" }]
```

```bash
# 2. Validate policy
cargo run -p xtask -- policy-test
# ✓ AC-TASK-008 has tests

# 3. NOW ask LLM to implement
cargo run -p xtask -- bundle implement_ac
```

**Corrected prompt:**
```
I've added AC-TASK-008 to the ledger: "User can delete a task by ID"

Implement this AC using the existing patterns in the bundle.
Use AC-TASK-008 (not AC-TASK-999) in all references.
```

**Policy enforcement protects you:**

```bash
# If you accidentally committed AC-TASK-999 without tests:
git add specs/spec_ledger.yaml
git commit -m "Add task deletion"

# CI would fail:
cargo run -p xtask -- policy-test
# ✗ AC-TASK-999 not found in ledger
# ✗ AC without tests array

# You're forced to fix it before merge
```

**This is the power of policy-as-code:** LLMs can assist, but policies enforce governance.

---

## Customizing Tasks

Edit `.llm/contextpack.yaml` to add new tasks:

```yaml
tasks:
  my_custom_task:
    max_bytes: 150000
    include:
      - specs/spec_ledger.yaml
      - flags/*.yaml
      - crates/core/src/flags.rs
    description: "Context for working with feature flags"
```

Then use it:
```bash
cargo run -p xtask -- bundle my_custom_task
```

---

## Excluding Files with .llmignore

The `.llm/.llmignore` file lets you exclude files from bundles using **standard gitignore syntax**.

### Location

Create or edit: `.llm/.llmignore`

### Syntax

`.llmignore` uses **full gitignore semantics** - the same patterns you use in `.gitignore` work here.

For complete syntax reference, see [gitignore documentation](https://git-scm.com/docs/gitignore).

### Common Patterns

**1. Wildcard patterns:**
```
# Ignore all log files
*.log

# Ignore test files
test_*.rs
*_test.go
```

**2. Directory patterns:**
```
# Ignore build directories
target/
dist/
node_modules/
```

**3. Path anchoring:**
```
# Only at root
/ROOT_FILE.txt

# Anywhere in tree
Cargo.lock
```

**4. Recursive wildcards:**
```
# All .draft files in docs subdirectories
docs/**/*.draft

# All temporary files anywhere
**/*.tmp
```

**5. Negation (whitelist):**
```
# Ignore all logs except error.log
*.log
!error.log
```

**6. Character classes:**
```
# Ignore test0.rs through test9.rs
test[0-9].rs

# Files starting with a, b, or c
[abc]*.txt
```

### Example .llmignore

```
# Build artifacts
target/
*.lock
*.tmp

# IDE and OS files
.idea/
.vscode/
.DS_Store
*.swp

# Test and development files
*_test.go
test_*.rs

# Logs and databases
*.log
*.db

# Environment files
.env*

# Documentation drafts
docs/**/*.draft
*.bak
```

### How It Works

1. Files matched by `include` patterns in contextpack.yaml
2. `.llmignore` patterns are applied to exclude files (using gitignore semantics)
3. Remaining files are added to bundle (up to `max_bytes`)

### Tips

**Balance include patterns with .llmignore:**

```yaml
# Good: Use include for broad categories, .llmignore for exclusions
include:
  - crates/**/*.rs

# .llmignore:
# test_*.rs
# *_test.rs
# target/
```

**Be specific when possible:**

```yaml
# Even better: Be specific in contextpack.yaml
include:
  - crates/core/src/**/*.rs
  - crates/app-http/src/**/*.rs
  # More specific = less filtering needed
```

---

## Troubleshooting

**Bundle is empty or missing files:**
- Check that `include` globs match files tracked by git
- Use `git ls-files <pattern>` to test your globs
- Check `.llm/.llmignore` isn't excluding files you want

**Bundle exceeds max_bytes:**
- Reduce `max_bytes` limit
- Make `include` patterns more specific
- Split into multiple smaller tasks

**LLM gives bad suggestions:**
- Make your prompt more specific
- Include examples of desired output format
- Add explicit constraints ("Do not invent new AC IDs")

---

## Summary

**Do:**
- ✅ Use bundles for focused, well-defined tasks
- ✅ Verify all LLM output before applying
- ✅ Keep prompts specific and constrained
- ✅ Add governance artifacts (ACs, flags, PII) to specs *first*

**Don't:**
- ❌ Let LLMs invent AC/FT/flag IDs
- ❌ Accept policy changes without review
- ❌ Use bundles for architecture-level changes
- ❌ Skip running `xtask check` and `xtask bdd` after changes

LLM bundles are a **tool to amplify your productivity**, not a replacement for understanding your system. Use them to accelerate implementation of decisions you've already made.
