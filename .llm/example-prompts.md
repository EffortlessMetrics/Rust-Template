# LLM Prompt Templates

This guide provides battle-tested prompt templates for common development tasks. Each template is designed to work with context bundles and enforce governance.

## Table of Contents

1. [Prompt Structure Guidelines](#prompt-structure-guidelines)
2. [Template 1: Implement New AC](#template-1-implement-new-ac)
3. [Template 2: Debug Failing Test](#template-2-debug-failing-test)
4. [Template 3: Add New Endpoint](#template-3-add-new-endpoint)
5. [Template 4: Refactor Code](#template-4-refactor-code)
6. [Template 5: Fix Policy Violation](#template-5-fix-policy-violation)
7. [Good vs Bad Prompts](#good-vs-bad-prompts)

---

## Prompt Structure Guidelines

**Effective prompts have:**

1. **Context** - Paste the relevant bundle
2. **Goal** - State what you want to achieve
3. **Constraints** - List what NOT to do
4. **Output Format** - Specify exact format (diffs, file paths, etc.)
5. **Verification** - How to validate the result

**Bundle Sizing:**
- `implement_ac` (250KB) - Focused AC implementation
- `implement_feature` (300KB) - Broader feature work
- `debug_tests` (200KB) - Test debugging

**What to Include from Bundles:**
```bash
# Always include full bundle, don't cherry-pick
cargo run -p xtask -- bundle implement_ac > /tmp/context.md

# Then in prompt:
"Here's the complete context bundle:
[PASTE ALL OF /tmp/context.md]"
```

---

## Template 1: Implement New AC

**Use Case:** You've added an AC to spec_ledger.yaml and need implementation.

**Prerequisites:**
1. AC exists in `specs/spec_ledger.yaml`
2. AC has `tests` array with BDD tag
3. Ran `cargo run -p xtask -- policy-test` successfully

**Generate Bundle:**
```bash
cargo run -p xtask -- bundle implement_ac > /tmp/ac-context.md
```

**Prompt Template:**
```markdown
I've added {AC-ID} to the ledger: "{AC text}"

Here's the complete context bundle showing our project structure and patterns:

[PASTE CONTENTS OF /tmp/ac-context.md]

Using the patterns in this bundle, implement {AC-ID}. Provide:

1. **Gherkin scenario** in specs/features/{feature_name}.feature:
   - Tag with @{AC-ID}
   - Follow existing scenario style from bundle
   - Test: {describe expected behavior}

2. **Step definitions** in crates/acceptance/src/steps/{module}.rs:
   - Reuse existing step patterns from bundle
   - Handle {specific test requirements}
   - Validate {expected outcomes}

3. **Core logic** in crates/core/src/{module}.rs:
   - {Describe business logic needed}
   - Use existing error types from bundle

4. **HTTP handler** in crates/app-http/src/lib.rs:
   - Route: {METHOD /path}
   - Use existing error handling pattern
   - Add tracing span with AC reference

**Constraints:**
- Do NOT invent new AC IDs (only use {AC-ID})
- Use existing error types (CoreError, AppError)
- Follow tracing pattern from existing handlers
- Match coding style in bundle
- Do NOT add dependencies without asking

**Output Format:**
Show me exact diffs for each file with:
- File path
- Line numbers (if editing existing file)
- Complete function/struct if new code

Show exact file paths and diffs.
```

**Example (filled in):**
```markdown
I've added AC-TPL-005 to the ledger: "GET /api/echo returns the input message"

Here's the complete context bundle showing our project structure and patterns:

[PASTE CONTENTS OF /tmp/ac-context.md]

Using the patterns in this bundle, implement AC-TPL-005. Provide:

1. **Gherkin scenario** in specs/features/template_core.feature:
   - Tag with @AC-TPL-005
   - Follow existing scenario style from bundle
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
- Do NOT add dependencies without asking

**Output Format:**
Show me exact diffs for each file with:
- File path
- Line numbers (if editing existing file)
- Complete function/struct if new code
```

**Verification After Applying:**
```bash
cargo run -p xtask -- selftest
cargo run -p xtask -- ac-status | grep {AC-ID}
```

---

## Template 2: Debug Failing Test

**Use Case:** BDD scenario fails, need root cause analysis.

**Prerequisites:**
1. Captured test failure output
2. Know which scenario failed (AC tag)

**Generate Bundle:**
```bash
cargo run -p xtask -- bundle debug_tests > /tmp/debug-context.md
```

**Prompt Template:**
```markdown
BDD scenario @{AC-ID} is failing with this error:

```
{PASTE EXACT ERROR OUTPUT}
```

Here's the debug bundle with all relevant code:

[PASTE /tmp/debug-context.md]

**Questions:**
1. What does the Gherkin scenario expect?
2. What is the actual behavior from the error?
3. Why is there a mismatch?
4. What file and line number needs to change?
5. Show me the exact fix needed

**Error Context:**
- Scenario: {scenario name}
- Step that failed: {step text}
- Expected: {expected result}
- Actual: {actual result}

**Constraints:**
- Do NOT change the Gherkin scenario unless it's wrong
- Preserve existing error messages (AC traceability)
- Fix only what's broken, don't refactor
```

**Example (filled in):**
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

**Error Context:**
- Scenario: Version endpoint reports build information
- Step that failed: Then I receive 200 with JSON containing "version" and "gitSha"
- Expected: Field "gitSha" (camelCase)
- Actual: Field "git_sha" (snake_case)

**Constraints:**
- Do NOT change the Gherkin scenario unless it's wrong
- Preserve existing error messages (AC traceability)
- Fix only what's broken, don't refactor
```

**Verification After Applying:**
```bash
cargo run -p xtask -- bdd
# Verify specific scenario passes
```

---

## Template 3: Add New Endpoint

**Use Case:** Add complete endpoint (route + handler + tests) for existing AC.

**Prerequisites:**
1. AC exists in ledger
2. Understand API contract (method, path, request/response)

**Generate Bundle:**
```bash
cargo run -p xtask -- bundle implement_feature > /tmp/feature-context.md
```

**Prompt Template:**
```markdown
Implement {AC-ID}: "{AC text}"

Bundle context:
[PASTE /tmp/feature-context.md]

**Implementation Requirements:**

Route: {METHOD /path}
Request Body (if POST/PUT):
```json
{example}
```

Response:
```json
{example}
```

**Tasks:**

1. **Model** (crates/model/src/{module}.rs or new file):
   ```rust
   // Show expected struct shape
   ```

2. **Core logic** (crates/core/src/{module}.rs):
   ```rust
   // Show expected function signature
   ```

3. **HTTP Handler** (crates/app-http/src/lib.rs):
   ```rust
   // Show expected handler signature and routing
   ```

4. **BDD Scenario** (specs/features/{feature}.feature):
   ```gherkin
   # Show expected scenario structure
   ```

5. **Step Definition** (crates/acceptance/src/steps/{module}.rs):
   Use existing step patterns for {GET/POST/etc} requests and validation.

**Constraints:**
- Follow existing code style from bundle
- Use RequestId middleware pattern
- Add tracing span with AC reference: `ac = "{AC-ID}"`
- Use existing error handling (AppError)
- Do NOT invent new AC IDs
- Do NOT add external dependencies

Show exact file paths and diffs.
```

**Example (filled in):**
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

1. **Model** (crates/model/src/metrics.rs or add to lib.rs):
   ```rust
   #[derive(Serialize)]
   pub struct MetricsResponse {
       pub request_count: u64,
       pub uptime_seconds: u64,
   }
   ```

2. **Core logic** (crates/core/src/metrics.rs):
   ```rust
   pub fn get_metrics() -> MetricsResponse {
       // Implementation here
   }
   ```

3. **HTTP Handler** (crates/app-http/src/lib.rs):
   ```rust
   #[tracing::instrument(name = "metrics", skip(request_id))]
   async fn metrics_handler(
       Extension(request_id): Extension<RequestId>
   ) -> Result<Json<MetricsResponse>, AppError>
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
- Add tracing span with AC reference: `ac = "AC-TPL-006"`
- Use existing error handling (AppError)
- Do NOT invent new AC IDs
- Do NOT add external dependencies

Show exact file paths and diffs.
```

**Verification After Applying:**
```bash
cargo run -p xtask -- selftest
```

---

## Template 4: Refactor Code

**Use Case:** Extract repeated code into reusable functions while keeping tests green.

**Prerequisites:**
1. Identified code duplication or complexity
2. All tests currently passing

**Generate Bundle:**
```bash
cargo run -p xtask -- bundle implement_feature > /tmp/refactor-context.md
```

**Prompt Template:**
```markdown
I see repeated {pattern description} in {file path}:

**Current Code (repeated {N} times):**
```rust
{PASTE REPEATED CODE EXAMPLE}
```

**Refactoring Goal:**
{Describe desired outcome}

**Requirements:**
- Create {helper function/module} in {target location}
- Replace all {N} occurrences with the helper
- Do NOT change error messages (tests depend on exact text)
- Do NOT change public API signatures
- Keep existing tracing/logging behavior

Bundle context:
[PASTE /tmp/refactor-context.md]

Show me:
1. The new helper function/module
2. Before/after for each call site
3. Any import changes needed
4. Which files need modification

**Constraints:**
- Preserve exact error messages (AC traceability)
- Keep all AC references in logs
- Do NOT change behavior, only structure
- Run tests to verify no regressions
```

**Example (filled in):**
```markdown
I see repeated validation error handling in crates/app-http/src/lib.rs:

**Current Code (repeated 5 times):**
```rust
let result = some_validation().map_err(|e| {
    error!(ac = "AC-TPL-003", error = ?e, "Validation failed");
    AppError::ValidationError(e.to_string())
})?;
```

**Refactoring Goal:**
Extract into reusable helper that logs and converts errors.

**Requirements:**
- Create helper in crates/app-http/src/helpers.rs
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

**Constraints:**
- Preserve exact error messages (AC traceability)
- Keep all AC references in logs
- Do NOT change behavior, only structure
- Run tests to verify no regressions
```

**Verification After Applying:**
```bash
cargo run -p xtask -- bdd  # Must pass - error messages unchanged
cargo run -p xtask -- policy-test  # AC references still valid
cargo run -p xtask -- selftest
```

---

## Template 5: Fix Policy Violation

**Use Case:** Policy test failed, need to understand and fix Rego error.

**Prerequisites:**
1. Ran `cargo run -p xtask -- policy-test` and captured output
2. Know which policy file failed

**Generate Bundle:**
```bash
cargo run -p xtask -- bundle implement_ac > /tmp/policy-context.md
```

**Prompt Template:**
```markdown
I'm getting this policy error:

```
{PASTE EXACT POLICY ERROR}
```

Here's the Rego policy that failed:

```rego
{PASTE CONTENTS OF policy/{file}.rego}
```

And here's the data that triggered the violation:

```yaml
{PASTE RELEVANT SECTION FROM specs/spec_ledger.yaml OR OTHER FILE}
```

**Questions:**
1. What does this Rego rule enforce and why?
2. What's wrong with my data?
3. Show me the correct structure to satisfy this policy
4. Are there examples in the bundle of correct entries?

Context from bundle:
[PASTE /tmp/policy-context.md showing examples of correct entries]

**Constraints:**
- Do NOT disable or modify the policy
- Fix the data to comply with the rule
- Explain WHY the rule exists (governance purpose)
```

**Example (filled in):**
```markdown
I'm getting this policy error:

```
AC 'AC-NOTIF-001' must have a non-empty 'tests' array.
```

Here's the Rego policy that failed:

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

And here's my spec_ledger.yaml entry:

```yaml
- id: AC-NOTIF-001
  text: "Customer receives email when refund is approved"
  # Missing tests array!
```

**Questions:**
1. What does this Rego rule enforce and why?
2. What's wrong with my AC-NOTIF-001 entry?
3. Show me the correct YAML structure to satisfy this policy
4. What would be the proper test entry for a BDD scenario?

Context from bundle:
[PASTE /tmp/policy-context.md showing examples like AC-TPL-001]

**Constraints:**
- Do NOT disable or modify the policy
- Fix the data to comply with the rule
- Explain WHY the rule exists (governance purpose)
```

**Verification After Applying:**
```bash
cargo run -p xtask -- policy-test
```

---

## Good vs Bad Prompts

### Example 1: Implementing an AC

**❌ Bad Prompt:**
```
Add an echo endpoint
```

**Problems:**
- No AC reference
- No context bundle
- No constraints
- Vague requirements
- LLM will invent AC ID

**✅ Good Prompt:**
```markdown
I've added AC-TPL-005 to the ledger: "GET /api/echo returns the input message"

Here's the complete context bundle:
[PASTE /tmp/ac-context.md]

Using patterns from this bundle, implement AC-TPL-005:
- Gherkin scenario in specs/features/template_core.feature tagged @AC-TPL-005
- Handler in crates/app-http/src/lib.rs following health endpoint pattern
- Step definitions reusing existing patterns

Constraints:
- Use AC-TPL-005 (already in ledger)
- Do NOT invent new AC IDs
- Follow tracing pattern from bundle

Show exact file paths and diffs.
```

**Why it's good:**
- ✅ References existing AC
- ✅ Includes bundle
- ✅ Specific files and patterns
- ✅ Clear constraints
- ✅ Specifies output format

---

### Example 2: Debugging

**❌ Bad Prompt:**
```
My test is failing, help
```

**Problems:**
- No error details
- No context
- No specific scenario
- Can't diagnose

**✅ Good Prompt:**
```markdown
BDD scenario @AC-TPL-002 fails with:

```
Expected field "gitSha" in response
Got: {"version": "0.1.0", "git_sha": "abc123"}
```

Debug bundle:
[PASTE /tmp/debug-context.md]

Questions:
1. What does Gherkin expect vs. what handler returns?
2. Why the mismatch (snake_case vs camelCase)?
3. Exact fix needed?

Don't change Gherkin unless it's wrong.
```

**Why it's good:**
- ✅ Exact error output
- ✅ Specific scenario (AC tag)
- ✅ Focused questions
- ✅ Clear constraint (don't change Gherkin)

---

### Example 3: Refactoring

**❌ Bad Prompt:**
```
Clean up the error handling
```

**Problems:**
- Vague scope
- No specific files
- No success criteria
- Could break everything

**✅ Good Prompt:**
```markdown
Extract repeated validation pattern in crates/app-http/src/lib.rs (5 occurrences):

```rust
let result = validate().map_err(|e| {
    error!(ac = "AC-TPL-003", error = ?e, "Validation failed");
    AppError::ValidationError(e.to_string())
})?;
```

Create helper in src/helpers.rs that:
- Logs with AC reference
- Converts to AppError
- Preserves exact error messages

Bundle: [PASTE /tmp/refactor-context.md]

Show before/after for all 5 call sites.
Do NOT change error messages (tests depend on them).
```

**Why it's good:**
- ✅ Specific pattern to extract
- ✅ Exact location and count
- ✅ Clear requirements
- ✅ Preserves test compatibility

---

## How to Structure Context in Prompts

**Include from Bundle:**

1. **Always paste full bundle** - Don't cherry-pick sections
2. **Label it clearly** - `Here's the context bundle:`
3. **Put it early** - Before asking questions

**Add Your Specific Context:**

1. **Error messages** - Exact output from failed tests/builds
2. **File locations** - Where you want changes
3. **Existing code snippets** - What needs to change
4. **Expected behavior** - What should happen

**Order:**
```markdown
1. State the goal (one sentence)
2. Paste full bundle
3. Add specific context (errors, files)
4. Ask focused questions
5. List constraints
6. Specify output format
```

---

## Bundle Sizing Guidelines

**When to use each bundle:**

| Bundle | Size | Use Case | Include Patterns |
|--------|------|----------|------------------|
| `implement_ac` | 250KB | Single AC implementation | Ledger, features, core, acceptance |
| `implement_feature` | 300KB | Multi-AC feature work | All specs, all crates |
| `debug_tests` | 200KB | Test debugging | Features, tests, app code |

**If bundle is too large:**
```bash
# Option 1: Use more specific bundle
cargo run -p xtask -- bundle implement_ac  # Instead of implement_feature

# Option 2: Create custom task in .llm/contextpack.yaml
tasks:
  debug_handler:
    max_bytes: 100000
    include:
      - crates/app-http/src/**/*.rs
      - specs/features/**/*.feature
    description: "Debug HTTP handler issues"
```

**If bundle is too small (missing context):**
```bash
# Edit .llm/contextpack.yaml to include more patterns
tasks:
  implement_ac:
    max_bytes: 250000
    include:
      - specs/spec_ledger.yaml
      - specs/features/**/*.feature
      - crates/core/src/**/*.rs
      - crates/model/src/**/*.rs  # ← Add model layer
```

---

## Summary: Prompt Checklist

Before sending your prompt, verify:

- [ ] Ran `cargo run -p xtask -- bundle {task}` and saved to file
- [ ] Pasted **full bundle** into prompt (not excerpts)
- [ ] Referenced specific AC ID (if applicable)
- [ ] Listed clear constraints ("Do NOT invent AC IDs")
- [ ] Specified exact output format (file paths, diffs)
- [ ] Included error messages if debugging
- [ ] Identified verification steps (`cargo run -p xtask -- selftest`)

**Remember:** Policies protect you from LLM mistakes. Always run `xtask selftest` after applying changes!
