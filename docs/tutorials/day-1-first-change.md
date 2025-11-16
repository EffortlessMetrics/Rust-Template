# Tutorial: Day 1 - Your First Change

**Welcome to the template!** This guide walks you through making your first meaningful change in 30 minutes.

You'll add a trivial acceptance criterion, write a Gherkin scenario, implement it, and see it go green. This demonstrates the AC-first development loop that makes this template powerful.

**Time:** 30 minutes
**Prerequisites:**
- Git installed
- Nix installed (or see alternative setup in README)
- Basic familiarity with HTTP APIs and testing

---

## What You'll Build

A simple "echo" endpoint enhancement that demonstrates the full AC development cycle:

- **AC ID:** `AC-DEMO-001`
- **Behavior:** "GET /api/echo returns a greeting message"
- **Implementation:** Simple handler in app-http
- **Tests:** BDD scenario that validates the behavior

This is intentionally trivial so you can focus on the *process*, not the domain complexity.

---

## Step 1: Clone and Validate (5 min)

### Clone the template

```bash
git clone https://github.com/your-org/rust-template.git
cd rust-template
```

### Enter the development environment

```bash
nix develop
```

**Expected:** You'll see a new shell prompt with all tools available (Rust, conftest, etc.)

If you don't have Nix:
```bash
# Install Nix first
curl -L https://nixos.org/nix/install | sh
```

### Run quickstart validation

```bash
cargo run -p xtask -- quickstart
```

**Expected output:**
```
Running quickstart validation...
✓ Checking Rust toolchain...
✓ Building workspace...
✓ Running unit tests...
✓ Running acceptance tests...
✓ Testing policy enforcement...
✓ Generating AC status...
✓ Creating LLM bundle...

✅ Quickstart complete! Template is working.
```

If this fails, stop here and troubleshoot. The template must work before you modify it.

---

## Step 2: Add Your AC to the Ledger (3 min)

Open `specs/spec_ledger.yaml` and add a new story with your AC.

**Add this after the existing Template Core story:**

```yaml
stories:
  # ... existing US-TPL-001 story ...

  # Your first feature
  - id: US-DEMO-001
    title: "Demo Feature"
    requirements:
      - id: REQ-DEMO-ECHO
        title: "Echo Greeting"
        acceptance_criteria:
          - id: AC-DEMO-001
            text: "GET /api/echo returns a greeting message with status 200"
            tests: [{ type: bdd, tag: "@AC-DEMO-001" }]
```

**Key points:**
- `id: AC-DEMO-001` - This is your AC identifier
- `text:` - Clear, testable behavior statement
- `tests:` - Links to the BDD tag you'll use in the feature file

**Validate the ledger:**

```bash
cargo run -p xtask -- policy-test
```

**Expected:**
```
Testing policy: policy/ledger.rego
✓ All ACs have tests
✓ Ledger is valid
```

If this fails, you likely have a YAML syntax error. Check indentation and structure.

---

## Step 3: Write a Gherkin Scenario (5 min)

Create a new feature file: `specs/features/demo.feature`

```gherkin
Feature: Demo Echo Endpoint
  As a developer learning the template
  I want to see a working AC implementation
  So that I understand the development flow

  @AC-DEMO-001
  Scenario: Echo endpoint returns greeting
    When I GET /api/echo
    Then I receive 200 with JSON containing "message"
    And the "message" field equals "Hello from the template!"
```

**Key points:**
- `@AC-DEMO-001` - This tag links the scenario to your AC in the ledger
- Steps use existing step definitions from `crates/acceptance/src/steps/template_core.rs`
- Each scenario should test ONE acceptance criterion

**Note:** The step definitions `I GET /api/echo` and `I receive 200 with JSON containing` already exist in the template. We're reusing them!

---

## Step 4: Run BDD (Expect Red) (2 min)

```bash
cargo run -p xtask -- bdd
```

**Expected output:**
```
Feature: Demo Echo Endpoint

  Scenario: Echo endpoint returns greeting
    When I GET /api/echo
    Then I receive 200 with JSON containing "message"
      ✗ Step failed: request returned 404

1 scenario (1 failed)
```

**This is correct!** The test is failing because we haven't implemented the endpoint yet. This is TDD: test first, then implement.

---

## Step 5: Implement the Endpoint (8 min)

### 5a. Add the route

Open `crates/app-http/src/lib.rs` and add the route to the router:

```rust
pub fn app() -> Router {
    Router::new()
        // Template core endpoints - keep these
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/api/echo", post(echo)) // Existing echo (for error tests)
        .route("/api/echo", get(echo_greeting)) // ← ADD THIS
        // ... rest of the router
}
```

### 5b. Add the handler

In the same file, add the handler function below the existing handlers (around line 100):

```rust
/// Echo greeting endpoint - Demonstrates AC-DEMO-001
///
/// This is a minimal handler showing:
/// - Simple JSON response
/// - No complex logic needed
/// - Links to AC via comment
#[instrument]
async fn echo_greeting() -> impl IntoResponse {
    info!("Echo greeting requested");
    Json(EchoGreetingResponse { message: "Hello from the template!".to_string() })
}
```

### 5c. Add the DTO

Add the response struct with the other DTOs (around line 160):

```rust
#[derive(Debug, Serialize)]
struct EchoGreetingResponse {
    pub message: String,
}
```

**That's it!** No core logic needed for this trivial example. Real features would call into `crates/core`, but for Day 1 we're keeping it simple.

---

## Step 6: Run BDD (Expect Green) (2 min)

```bash
cargo run -p xtask -- bdd
```

**Expected output:**
```
Feature: Demo Echo Endpoint

  @AC-DEMO-001
  Scenario: Echo endpoint returns greeting
    When I GET /api/echo                                    ✓
    Then I receive 200 with JSON containing "message"       ✓
    And the "message" field equals "Hello from the template!" ✓

Feature: Template Core Endpoints
  ... (other scenarios pass)

2 features
5 scenarios (5 passed)
15 steps (15 passed)

✅ Acceptance tests passed
JUnit output: target/junit/acceptance.xml
```

**Success!** Your scenario is now green.

---

## Step 7: Verify AC Coverage (2 min)

Generate the AC status report:

```bash
cargo run -p xtask -- ac-status
```

**Expected output:**
```
✓ Generated docs/feature_status.md
✓ All ACs passed
```

Open `docs/feature_status.md`:

```markdown
# Feature Status

Auto-generated AC status from acceptance tests.

## AC Status Summary

| AC ID | Story | Requirement | Status | Scenarios |
|-------|-------|-------------|--------|----------|
| AC-TPL-001 | US-TPL-001 | REQ-TPL-HEALTH | ✅ pass | 1 |
| AC-TPL-002 | US-TPL-001 | REQ-TPL-VERSION | ✅ pass | 1 |
| AC-TPL-003 | US-TPL-001 | REQ-TPL-ERROR-HANDLING | ✅ pass | 1 |
| AC-TPL-004 | US-TPL-001 | REQ-TPL-ERROR-HANDLING | ✅ pass | 3 |
| AC-DEMO-001 | US-DEMO-001 | REQ-DEMO-ECHO | ✅ pass | 1 |
```

**See it?** Your `AC-DEMO-001` is now tracked and showing as passing. This report updates automatically whenever you run `xtask ac-status`.

---

## Step 8: Run Full Validation (3 min)

Before committing, always run the full test suite:

```bash
cargo run -p xtask -- selftest
```

**Expected output:**
```
Running comprehensive template validation...

[1/6] Code quality checks...
  ✓ rustfmt
  ✓ clippy
  ✓ unit tests

[2/6] BDD acceptance tests...
  ✓ 5 scenarios passed

[3/6] AC status mapping...
  ✓ All ACs have passing scenarios
  ✓ Generated docs/feature_status.md

[4/6] Policy validation...
  ✓ Ledger policies
  ✓ Feature policies
  ✓ Privacy policies

[5/6] LLM bundler...
  ✓ implement_ac bundle generated
  ✓ implement_feature bundle generated

[6/6] Documentation...
  ✓ All docs exist and are valid

✅ Full selftest passed!
```

If anything fails, fix it before moving on. `selftest` is your CI gate locally.

---

## What You Just Learned

You've completed the full AC-first development loop:

1. **Spec first** - Added `AC-DEMO-001` to `specs/spec_ledger.yaml`
2. **Test first** - Wrote Gherkin scenario with `@AC-DEMO-001` tag
3. **Red** - Ran BDD, saw it fail (404)
4. **Implement** - Added route + handler + DTO
5. **Green** - Ran BDD, saw it pass
6. **Verify** - Generated AC status report, saw your AC tracked
7. **Validate** - Ran full selftest to ensure nothing broke

This is the workflow for **every** feature you build in this template.

---

## The AC-First Development Loop

```
┌─────────────────────────────────────────────┐
│ 1. Add/update AC in ledger                  │
│    specs/spec_ledger.yaml                   │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│ 2. Write/update Gherkin scenario            │
│    specs/features/*.feature                 │
│    Tag with @AC-XXX                         │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│ 3. Run BDD (expect RED)                     │
│    cargo run -p xtask -- bdd                │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│ 4. Implement behavior                       │
│    - Add route (app-http)                   │
│    - Add handler (app-http)                 │
│    - Add core logic (core) if needed        │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│ 5. Run BDD (expect GREEN)                   │
│    cargo run -p xtask -- bdd                │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│ 6. Generate AC status                       │
│    cargo run -p xtask -- ac-status          │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│ 7. Run full validation                      │
│    cargo run -p xtask -- selftest           │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
               Commit!
```

---

## Next Steps

### Clean Up (Optional)

This was a demo AC. If you want to remove it before building real features:

1. Remove `AC-DEMO-001` from `specs/spec_ledger.yaml`
2. Delete `specs/features/demo.feature`
3. Remove the `echo_greeting` route and handler from `crates/app-http/src/lib.rs`
4. Run `cargo run -p xtask -- selftest` to verify cleanup

### Build Your First Real Feature

Ready for something more substantial? See:

**[Day 7: Your First Real Feature](day-7-first-real-feature.md)** - Implements a multi-layer feature with proper domain modeling, validation, and error handling.

### Understand the Architecture

To understand *why* we structure code this way:

- **[Architecture Explanation](../explanation/architecture.md)** - Hexagonal architecture, dependency rules
- **[Template Contracts](../explanation/TEMPLATE-CONTRACTS.md)** - What's stable, what you can change

### Use LLM Assistance

The template includes LLM context bundling for AI-assisted development:

- **[Use LLM Bundles](../how-to/use-llm-bundles.md)** - Generate focused context for AI coding

---

## Common Issues

### "BDD test fails with 404"

**Cause:** You forgot to add the route in `lib.rs`

**Fix:** Add `.route("/api/echo", get(echo_greeting))` to the `Router::new()` builder

### "Compile error: cannot find function `echo_greeting`"

**Cause:** Handler function not defined or misnamed

**Fix:** Ensure your handler is named exactly `echo_greeting` and is in scope

### "AC status shows 0 scenarios for AC-DEMO-001"

**Cause:** BDD tag doesn't match ledger tag

**Fix:** Ensure the feature file has `@AC-DEMO-001` (exact match, case-sensitive)

### "Policy test fails"

**Cause:** YAML syntax error in ledger

**Fix:** Check indentation, colons, and list syntax. YAML is whitespace-sensitive.

---

## Summary

In 30 minutes you:

- ✅ Cloned and validated the template
- ✅ Added an AC to the ledger
- ✅ Wrote a Gherkin scenario linked to your AC
- ✅ Implemented a simple endpoint
- ✅ Saw your AC go from red → green
- ✅ Generated AC coverage report showing your work
- ✅ Validated everything with `selftest`

You now understand the **AC-first development loop** that governs all work in this template.

**What makes this powerful:**
- Every behavior is traceable to an AC
- Every AC has automated tests
- AC status is auto-generated from test results
- LLMs can assist because the structure is consistent
- CI enforces the same checks you ran locally

Now go build something real! See **[Day 7: First Real Feature](day-7-first-real-feature.md)** for the next step.
