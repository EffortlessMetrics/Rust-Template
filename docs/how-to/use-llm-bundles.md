<!-- doclint:disable orphan-version -->
# How-to: Use LLM Context Bundles

This guide shows you how to use the template's LLM context bundler to get AI assistance with focused, relevant context.

**Time:** 5 minutes
**Prerequisites:** Template cloned, xtask working

---

## Quick Reference: Implementing an AC with LLM + Bundles

**1. Generate the bundle:**

```bash
cargo run -p xtask -- bundle implement_ac
```

**2. Open `.llm/bundle/implement_ac.md` in your LLM tool** (Claude, ChatGPT, etc.)

**3. Use the suggested prompt from `.llm/contextpack.yaml`:**

- Understand the AC in `specs/spec_ledger.yaml`
- Inspect the BDD scenario in `specs/features/...`
- Identify the core and app-http changes needed
- Propose minimal code changes and tests

**4. Apply changes locally and verify:**

```bash
cargo run -p xtask -- selftest
```

**5. If everything passes:**

- Commit your changes
- Update AC status if needed

See the detailed workflows below for examples of each step.

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
Implement behavior to satisfy AC-PLT-TASKS-001: 'Platform lists tasks with status'.
Show me the diffs you'd make to:
1. specs/spec_ledger.yaml
2. specs/features/*.feature
3. crates/business-core/src/*.rs
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
"Show me all ACs related to the platform tasks feature and their current implementation status."
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
> "AC-PLT-TASKS-001 requires that task status is returned in the response.
> Current code at crates/business-core/src/tasks.rs:42 doesn't include status.
> Show me the minimal diff to add status field to TaskResponse."

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

The following workflows show concrete examples of using LLM bundles with policies that keep AI assistance safe and governed.

### Workflow 1: Implement a New AC

**Scenario:** Product owner added AC-TPL-005 to spec_ledger.yaml: "GET /api/echo returns the input message"

**Goal:** Implement from AC to working code using LLM assistance, verified by selftest.

**Step 1: Verify AC is in ledger (human governance)**

```bash
# Check that AC exists
grep "AC-TPL-005" specs/spec_ledger.yaml
```

Output:

```yaml
- id: AC-TPL-005
  text: "GET /api/echo returns the input message"
  tests: [{ type: bdd, tag: "@AC-TPL-005" }]
```

**Step 2: Generate context bundle**

```bash
cargo run -p xtask -- bundle implement_ac > /tmp/ac-context.md
```

This bundles:
- `specs/spec_ledger.yaml` - All ACs (including AC-TPL-005)
- `specs/features/**/*.feature` - Existing scenarios as examples
- `crates/core/src/**/*.rs` - Core business logic patterns
- `crates/acceptance/src/**/*.rs` - Step definition patterns

**Step 3: Craft precise prompt with bundle**

**Prompt to LLM:**

```markdown
I've added AC-TPL-005 to the ledger: "GET /api/echo returns the input message"

Here's the complete context bundle showing our project structure:

[PASTE CONTENTS OF /tmp/ac-context.md]

Using the patterns in this bundle, implement AC-TPL-005. Provide:

1. **Gherkin scenario** in specs/features/template_core.feature:
   - Tag with @AC-TPL-005
   - Follow existing scenario style (see @AC-TPL-001 example)
   - Test: GET /api/echo?message=hello returns {"echo": "hello"}

2. **Step definitions** in crates/acceptance/src/steps/template_core.rs:
   - Reuse existing step patterns from bundle
   - Handle query parameters
   - Validate JSON response structure

3. **Core logic** (if needed) in crates/core/src/lib.rs:
   - Simple echo function
   - No external dependencies

4. **HTTP handler** in crates/app-http/src/lib.rs:
   - Route: GET /api/echo
   - Use existing error handling pattern (see health endpoint)
   - Add tracing span with AC reference

**Constraints:**
- Do NOT invent new AC IDs (only use AC-TPL-005)
- Use existing error types (CoreError, AppError)
- Follow tracing pattern from health handler
- Match coding style in bundle

Show me the exact diffs for each file.
```

**Step 4: Review LLM output**

**Expected LLM response structure:**

```rust
// File: specs/features/template_core.feature
// Add after existing scenarios:

@AC-TPL-005
Scenario: Echo endpoint returns input message
  When I GET /api/echo?message=hello
  Then I receive 200 with JSON containing field "echo" with value "hello"

// File: crates/acceptance/src/steps/template_core.rs
// Add step definition:

#[when(expr = "I GET /api/echo?message={word}")]
async fn get_echo(world: &mut TemplateWorld, message: String) {
    let response = world.client
        .get(&format!("{}/api/echo?message={}", world.base_url, message))
        .send()
        .await
        .expect("Failed to send request");
    world.last_response = Some(response);
}

// File: crates/app-http/src/lib.rs
// Add handler:

#[tracing::instrument(name = "echo_handler", skip(request_id))]
async fn echo_handler(
    Extension(request_id): Extension<RequestId>,
    Query(params): Query<EchoQuery>
) -> Result<Json<EchoResponse>, AppError> {
    info!(ac = "AC-TPL-005", "Echo request received");
    Ok(Json(EchoResponse {
        echo: params.message
    }))
}

#[derive(Deserialize)]
struct EchoQuery {
    message: String,
}

#[derive(Serialize)]
struct EchoResponse {
    echo: String,
}
```

**What policies protect you:**
- ✅ `policy/ledger.rego` ensures AC-TPL-005 has tests array
- ✅ `policy/features.rego` ensures @AC-TPL-005 tag references real AC
- ✅ LLM can't invent AC-TPL-999 - it will fail policy check

**Step 5: Apply changes and verify**

```bash
# Apply the diffs manually (review each change!)
vim specs/features/template_core.feature
vim crates/acceptance/src/steps/template_core.rs
vim crates/app-http/src/lib.rs

# Run policy checks
cargo run -p xtask -- policy-test
# ✅ AC-TPL-005 has tests
# ✅ @AC-TPL-005 references existing AC

# Run BDD tests
cargo run -p xtask -- bdd
# ✅ Scenario: Echo endpoint returns input message ... passed

# Full selftest
cargo run -p xtask -- selftest
# ✅ All checks pass
# ✅ AC-TPL-005 implementation verified
```

**Step 6: Verify traceability**

```bash
# Check AC status report
cargo run -p xtask -- ac-status

# Should show:
# | AC-TPL-005 | GET /api/echo returns the input message | ✅ pass | 1 |
```

**Key learnings:**
- **Human adds AC first** → LLM implements → Policies verify
- Bundle provides **concrete examples** LLM can pattern-match
- Policies **prevent invented IDs** before code review
- Selftest **validates entire chain** (AC → test → code)

---

### Workflow 2: Fix a Failing Policy

**Scenario:** You committed a feature but `cargo run -p xtask -- policy-test` fails in CI.

**Error:**

```
Policy violation in policy/ledger.rego:
AC 'AC-NOTIF-001' must have a non-empty 'tests' array.
```

**Goal:** Use LLM to understand Rego error, identify root cause, fix without breaking other rules.

**Step 1: Understand the policy failure**

```bash
# See which policy failed
cargo run -p xtask -- policy-test
```

Output:

```
❌ FAIL: policy/ledger.rego
  AC 'AC-NOTIF-001' must have a non-empty 'tests' array.

Tested: specs/spec_ledger.yaml
```

**Step 2: Read the policy to understand the rule**

```bash
cat policy/ledger.rego
```

```rego
package main

deny[msg] {
    ac := input.stories[_].requirements[_].acceptance_criteria[_]
    not has_tests(ac)
    msg := sprintf("AC '%s' must have a non-empty 'tests' array.", [ac.id])
}

has_tests(ac) {
    is_array(ac.tests)
    count(ac.tests) > 0
}
```

**Translation:** Every AC must have a `tests` array with at least one test entry.

**Step 3: Generate bundle with policy context**

```bash
# Create custom bundle for policy debugging
cargo run -p xtask -- bundle implement_ac > /tmp/policy-context.md
```

**Step 4: Ask LLM to explain Rego error**

**Prompt to LLM:**

```markdown
I'm getting this policy error:

```

AC 'AC-NOTIF-001' must have a non-empty 'tests' array.

```

Here's the Rego policy:

[PASTE policy/ledger.rego contents]

And here's my spec_ledger.yaml entry:

```yaml
- id: AC-NOTIF-001
  text: "User receives notification when task is assigned"
  # Missing tests array!
```

Questions:
1. What does this Rego rule enforce and why?
2. What's wrong with my AC-NOTIF-001 entry?
3. Show me the correct YAML structure to satisfy this policy
4. What would be the proper test entry for a BDD scenario?

Context from bundle:
[PASTE /tmp/policy-context.md showing examples of correct AC entries]

```

**Expected LLM response:**

> **Policy Purpose:**
> The `ledger.rego` policy enforces that every acceptance criterion has associated tests.
> This ensures traceability: ACs without tests can't be verified.
>
> **What's Wrong:**
> Your AC-NOTIF-001 is missing the required `tests` array field.
>
> **Correct Structure:**
> ```yaml
> - id: AC-NOTIF-001
>   text: "User receives notification when task is assigned"
>   tests:
>     - type: bdd
>       tag: "@AC-NOTIF-001"
> ```
>
> **Explanation:**
> - `tests` must be an array (even for single test)
> - Each test needs `type` (e.g., "bdd", "unit")
> - The `tag` field references the Gherkin tag (@AC-NOTIF-001)
>
> **Example from bundle (AC-TPL-001):**
> ```yaml
> - id: AC-TPL-001
>   text: "GET /health returns 200 with status 'ok' when service is healthy"
>   tests: [ { type: bdd, tag: "@AC-TPL-001" } ]
> ```

**Step 5: Apply fix**

```bash
vim specs/spec_ledger.yaml
```

Update to:

```yaml
- id: AC-NOTIF-001
  text: "User receives notification when task is assigned"
  tests:
    - type: bdd
      tag: "@AC-NOTIF-001"
```

**Step 6: Verify policy now passes**

```bash
cargo run -p xtask -- policy-test
# ✅ All policies pass
# ✅ AC-NOTIF-001 has required tests array
```

**What policies protect you:**
- ✅ `policy/ledger.rego` - Prevents ACs without test coverage
- ✅ `policy/features.rego` - Ensures @AC-NOTIF-001 tag references real AC
- ✅ Policies run in CI - Can't merge without compliance

**Key learnings:**
- Rego errors are **governance violations**, not code bugs
- LLM can **translate Rego to human intent**
- Bundle shows **correct examples** from existing codebase
- Fix policy violations **before implementing code**

---

### Workflow 3: Add a New Endpoint Using Bundles

**Scenario:** Add GET /api/metrics endpoint for basic service metrics (request count, uptime).

**Goal:** Use `implement_feature` bundle with precise prompt template to generate full implementation.

**Step 1: Add AC to ledger (human governance)**

```bash
vim specs/spec_ledger.yaml
```

Add new requirement:

```yaml
- id: REQ-TPL-METRICS
  title: "Basic Service Metrics"
  acceptance_criteria:
    - id: AC-TPL-006
      text: "GET /api/metrics returns JSON with requestCount and uptimeSeconds"
      tests: [{ type: bdd, tag: "@AC-TPL-006" }]
```

**Step 2: Generate feature development bundle**

```bash
cargo run -p xtask -- bundle implement_feature > /tmp/feature-context.md
```

Includes:
- All crates source code (patterns for handlers, models, errors)
- OpenAPI specs (API design patterns)
- Feature files (scenario examples)

**Step 3: Use prompt template**

**Prompt to LLM:**

```markdown
Implement AC-TPL-006: "GET /api/metrics returns JSON with requestCount and uptimeSeconds"

Bundle context:
[PASTE /tmp/feature-context.md]

**Implementation Requirements:**

Route: GET /api/metrics
Response:
```json
{
  "requestCount": 42,
  "uptimeSeconds": 3600
}
```

**Tasks:**

1. **Model** (crates/model/src/lib.rs or new metrics.rs):

   ```rust
   #[derive(Serialize)]
   pub struct MetricsResponse {
       pub request_count: u64,
       pub uptime_seconds: u64,
   }
   ```

2. **Core logic** (crates/core/src/metrics.rs - create if needed):

   ```rust
   pub fn get_metrics() -> MetricsResponse {
       // Use lazy_static for counters or read from state
   }
   ```

3. **HTTP Handler** (crates/app-http/src/lib.rs):

   ```rust
   #[tracing::instrument(name = "metrics", skip(request_id))]
   async fn metrics_handler(
       Extension(request_id): Extension<RequestId>
   ) -> Result<Json<MetricsResponse>, AppError> {
       info!(ac = "AC-TPL-006", "Metrics requested");
       let metrics = core::get_metrics();
       Ok(Json(metrics))
   }
   ```

   Add route: `.route("/api/metrics", get(metrics_handler))`

4. **BDD Scenario** (specs/features/template_core.feature):

   ```gherkin
   @AC-TPL-006
   Scenario: Metrics endpoint returns service statistics
     When I GET /api/metrics
     Then I receive 200 with JSON containing "requestCount" and "uptimeSeconds"
   ```

5. **Step Definition** (crates/acceptance/src/steps/template_core.rs):
   Use existing step patterns for GET requests and JSON validation.

**Constraints:**
- Follow existing code style from bundle
- Use RequestId middleware pattern
- Add tracing span with AC reference
- Use existing error handling (AppError)
- Do NOT invent new AC IDs

Show exact file paths and diffs.

```

**Expected LLM Output:**

LLM provides complete implementation with:
- Exact file locations
- Import statements needed
- Where to add route in router
- Step definitions reusing existing patterns

**Step 4: Validate with selftest**

```bash
# Apply changes
# (Review each diff carefully!)

# Run full validation
cargo run -p xtask -- selftest

# Expected output:
# ✅ Policy tests pass
# ✅ BDD scenarios pass (@AC-TPL-006)
# ✅ AC-TPL-006 traceable in status report
```

**What the prompt template provides:**
- ✅ **Exact structure** - LLM knows what files to modify
- ✅ **Code snippets** - Shows expected implementation shape
- ✅ **Constraints** - Prevents common mistakes
- ✅ **Verification** - Clear success criteria

**Key learnings:**
- **Structured prompts** → more precise LLM output
- Bundle provides **working examples** to pattern-match
- Template forces **explicit file paths** (no guessing)
- Verification step **catches LLM hallucinations**

---

### Workflow 4: Refactor Under Policy Protection

**Scenario:** Refactor error handling code while keeping all ACs passing (trust-but-verify).

**Goal:** Use LLM to refactor, rely on policies and tests to catch regressions.

**Step 1: Identify refactoring opportunity**

```bash
# Found repeated error pattern in crates/app-http/src/lib.rs:
```

```rust
// Handler 1:
let result = validate_input(&request).map_err(|e| {
    error!(ac = "AC-TPL-003", error = ?e, "Validation failed");
    AppError::ValidationError(e.to_string())
})?;

// Handler 2:
let result = validate_email(&email).map_err(|e| {
    error!(ac = "AC-TPL-003", error = ?e, "Validation failed");
    AppError::ValidationError(e.to_string())
})?;

// Handler 3: (same pattern repeated 5+ times!)
```

**Step 2: Generate refactoring bundle**

```bash
cargo run -p xtask -- bundle implement_feature > /tmp/refactor-context.md
```

**Step 3: Ask LLM for refactoring**

**Prompt to LLM:**

```markdown
I see repeated validation error handling in crates/app-http/src/lib.rs.

Here's the pattern (repeated 5+ times):

```rust
let result = some_validation().map_err(|e| {
    error!(ac = "AC-TPL-003", error = ?e, "Validation failed");
    AppError::ValidationError(e.to_string())
})?;
```

**Refactoring Goal:**
Extract this into a reusable helper function that:
1. Logs the error with AC reference
2. Converts to AppError::ValidationError
3. Preserves existing error messages (for AC traceability)

**Requirements:**
- Create helper in crates/app-http/src/errors.rs (or new helpers.rs)
- Replace all 5 occurrences with the helper
- Do NOT change error messages (tests depend on exact text)
- Do NOT change AC references
- Keep existing type signatures

Bundle context:
[PASTE /tmp/refactor-context.md]

Show me:
1. The new helper function
2. Before/after for each call site
3. Any import changes needed

```

**Expected LLM Response:**

> **New Helper Function** (crates/app-http/src/helpers.rs):
> ```rust
> use tracing::error;
> use crate::AppError;
>
> pub fn validation_error<E: std::fmt::Display>(
>     ac: &str,
>     error: E
> ) -> AppError {
>     error!(ac = ac, error = %error, "Validation failed");
>     AppError::ValidationError(error.to_string())
> }
> ```
>
> **Before:**
> ```rust
> let result = validate_input(&request).map_err(|e| {
>     error!(ac = "AC-TPL-003", error = ?e, "Validation failed");
>     AppError::ValidationError(e.to_string())
> })?;
> ```
>
> **After:**
> ```rust
> let result = validate_input(&request)
>     .map_err(|e| validation_error("AC-TPL-003", e))?;
> ```

**Step 4: Apply refactoring**

```bash
# Create new file
vim crates/app-http/src/helpers.rs

# Update each handler
vim crates/app-http/src/lib.rs

# Add mod declaration
# mod helpers;
# use helpers::validation_error;
```

**Step 5: Verify no regressions (policies protect you!)**

```bash
# Run tests - error messages must be identical
cargo run -p xtask -- bdd
# ✅ All scenarios pass
# ✅ Error messages unchanged
# ✅ AC-TPL-003 still passing

# Run policy tests
cargo run -p xtask -- policy-test
# ✅ All policies pass
# ✅ AC references still valid

# Full selftest
cargo run -p xtask -- selftest
# ✅ No regressions detected
```

**What protected you during refactoring:**
- ✅ **BDD tests** - Validated error messages didn't change
- ✅ **Policy tests** - Ensured AC references still valid
- ✅ **Selftest** - Caught any broken imports or type errors
- ✅ **AC traceability** - Error logs still reference correct ACs

**If tests HAD failed:**

```bash
# Scenario: AC-TPL-003 error format changed
# ❌ Expected: "Validation failed: email invalid"
# ❌ Got: "ValidationError: email invalid"

# LLM changed the error format!
# Policies caught it before merge.
```

**Key learnings:**
- **Refactor freely** when you have comprehensive tests
- Policies provide **safety net** for governance compliance
- LLM does the tedious work, **tests verify correctness**
- "Trust but verify" - let automation catch mistakes

---

### Workflow 5: Debug a Failing BDD Scenario

**Scenario:** BDD test failing after refactoring - use LLM to understand feature context and fix.

**Goal:** Use `debug_tests` bundle to diagnose and fix without introducing regressions.

**Step 1: Run BDD and capture failure**

```bash
cargo run -p xtask -- bdd 2>&1 | tee /tmp/bdd-failure.txt
```

**Output:**

```
Feature: Template Core Endpoints

  @AC-TPL-001 @smoke
  Scenario: Health endpoint reports service is healthy
    When I GET /health
    Then I receive 200 with status "ok"
    ✅ PASSED

  @AC-TPL-002
  Scenario: Version endpoint reports build information
    When I GET /version
    Then I receive 200 with JSON containing "version" and "gitSha"
    ❌ FAILED

    Error: Expected field "gitSha" in response
    Got: {"version": "0.1.0", "git_sha": "abc123"}
                                 ^^^^^^^^ snake_case instead of camelCase!
```

**Step 2: Generate debug bundle**

```bash
cargo run -p xtask -- bundle debug_tests > /tmp/debug-context.md
```

Includes:
- Feature files (expected behavior)
- Step definitions (what the test checks)
- Application code (actual implementation)

**Step 3: Ask LLM with specific error context**

**Prompt to LLM:**

```markdown
BDD scenario @AC-TPL-002 is failing with this error:

```

Expected field "gitSha" in response
Got: {"version": "0.1.0", "git_sha": "abc123"}

```

Here's the debug bundle with all relevant code:

[PASTE /tmp/debug-context.md]

**Questions:**
1. What does the Gherkin scenario expect (field name)?
2. What is the handler currently returning?
3. Why is there a mismatch (snake_case vs camelCase)?
4. Show me the exact fix needed

**Error output:**
[PASTE relevant lines from /tmp/bdd-failure.txt]
```

**Expected LLM Response:**

> **Root Cause Analysis:**
>
> **1. Gherkin Expectation** (specs/features/template_core.feature:14):
>
> ```gherkin
> Then I receive 200 with JSON containing "version" and "gitSha"
> ```
>
> Expects: `gitSha` (camelCase)
>
> **2. Current Handler** (crates/app-http/src/lib.rs:78):
>
> ```rust
> #[derive(Serialize)]
> struct VersionResponse {
>     version: String,
>     git_sha: String,  // ← Serializes as "git_sha" (snake_case)
> }
> ```
>
> Returns: `git_sha` (snake_case)
>
> **3. Why Mismatch:**
> Serde defaults to snake_case for struct field names unless told otherwise.
> The test expects camelCase per API convention.
>
> **4. Fix:**
>
> **Option A: Rename field to match Rust conventions (use serde rename)**
>
> ```rust
> #[derive(Serialize)]
> struct VersionResponse {
>     version: String,
>     #[serde(rename = "gitSha")]
>     git_sha: String,
> }
> ```
>
> **Option B: Change Gherkin to match snake_case**
>
> ```gherkin
> Then I receive 200 with JSON containing "version" and "git_sha"
> ```
>
> **Recommendation:** Use Option A to maintain camelCase API convention.

**Step 4: Apply the fix**

```bash
vim crates/app-http/src/lib.rs
```

Update:

```rust
#[derive(Serialize)]
struct VersionResponse {
    version: String,
    #[serde(rename = "gitSha")]
    git_sha: String,
}
```

**Step 5: Re-run BDD tests**

```bash
cargo run -p xtask -- bdd

# Output:
# @AC-TPL-002
# Scenario: Version endpoint reports build information
#   ✅ PASSED
```

**Step 6: Verify no other regressions**

```bash
cargo run -p xtask -- selftest
# ✅ All tests pass
# ✅ AC-TPL-002 now passing
```

**What the debug bundle provided:**
- ✅ **Feature file** - Showed expected API contract
- ✅ **Step definitions** - Revealed exact assertion
- ✅ **Handler code** - Showed actual implementation
- ✅ **Full context** - LLM connected the dots

**Key learnings:**
- Debug bundles provide **all layers** (test → code → contract)
- LLM can **trace through** feature → step → handler
- Specific error output helps LLM **pinpoint exact mismatch**
- Fix verified by **re-running same test** that failed

---

## Policy-Protected LLM Workflow Summary

**The Safety Model:**

```
Human Governance → LLM Assistance → Policy Enforcement → Test Verification
       ↓                  ↓                  ↓                    ↓
   Add AC to ledger   Generate code    Verify compliance    Run selftest
   (spec_ledger.yaml)  (using bundles)  (policy/**.rego)    (BDD + checks)
```

**Example: LLM Tries to Invent an AC ID**

**Bad Prompt (no governance):**

```
Add a feature that deletes tasks.
```

**LLM Might Respond:**
> I'll add AC-TASK-999 for task deletion:
>
> ```yaml
> - id: AC-TASK-999
>   text: "User can delete a task by ID"
> ```

**DON'T APPLY THIS!** The LLM invented an AC ID outside governance.

**Correct Workflow (governance first):**

```bash
# 1. Human adds AC to ledger FIRST (governance decision)
vim specs/spec_ledger.yaml
```

Add:

```yaml
- id: AC-TASK-008  # ← Sequential ID following convention
  text: "User can delete a task by ID"
  tests: [{ type: bdd, tag: "@AC-TASK-008" }]
```

```bash
# 2. Validate policy compliance
cargo run -p xtask -- policy-test
# ✅ AC-TASK-008 has required tests array
```

```bash
# 3. NOW generate bundle and ask LLM to implement
cargo run -p xtask -- bundle implement_ac > /tmp/context.md
```

**Corrected Prompt:**

```markdown
I've added AC-TASK-008 to the ledger: "User can delete a task by ID"

Here's the context bundle:
[PASTE /tmp/context.md]

Implement AC-TASK-008 using existing patterns.

**Constraints:**
- Use AC-TASK-008 (it's already in the ledger)
- Do NOT invent new AC IDs
- Follow existing delete patterns from bundle
- Add @AC-TASK-008 tag to Gherkin scenario
```

**What Policies Protect You:**

| Policy | Protection | Example |
|--------|-----------|---------|
| `policy/ledger.rego` | Every AC must have tests | ❌ Rejects: AC without `tests` array |
| `policy/features.rego` | Tags must reference real ACs | ❌ Rejects: `@AC-FAKE-999` in Gherkin |
| `policy/llm.rego` | Contextpack must be valid | ❌ Rejects: Invalid bundle configuration |

**If You Bypass Governance:**

```bash
# Scenario: You accidentally apply LLM's invented AC-TASK-999
vim specs/spec_ledger.yaml  # Add AC-TASK-999
git add .
git commit -m "Add task deletion"

# Policy test runs (in pre-commit hook or CI):
cargo run -p xtask -- policy-test

# ❌ FAIL: Features reference AC-TASK-999 but it's non-sequential
# ❌ FAIL: AC-TASK-999 doesn't follow naming convention
# (Policies would need to be extended for these specific checks)

# Current policies catch:
# ✅ AC without tests array
# ✅ Feature tag referencing non-existent AC
# ✅ Invalid contextpack configuration
```

**Key Principle: Policies Enforce Governance at Commit Time**

```
                    Governance Layer
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
   spec_ledger.yaml   policy/*.rego    xtask policy-test
   (source of truth)  (rules)          (enforcement)
        │                 │                 │
        └─────────────────┼─────────────────┘
                          │
                    Blocks bad commits
                    (pre-commit hook)
```

**This is the power of policy-as-code:**
- ✅ LLMs can **assist implementation**
- ✅ Policies **enforce governance rules**
- ✅ Humans make **strategic decisions** (AC IDs, architecture)
- ✅ Automation **catches compliance violations** before merge

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
