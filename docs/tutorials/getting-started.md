# Tutorial: Getting Started with the Rust Template

**Time:** 30 minutes
**Goal:** Get the template running, understand core concepts, make your first change
**Prerequisites:** Git, Nix installed

> **⚠️ Note:** This tutorial references a "refunds" feature as a teaching example. The template itself only ships with template-core endpoints (`/health`, `/version`, `/api/echo`). References to refunds are fictional examples you would implement following the patterns shown. See `docs/PILOT-PROJECT-PLAN.md` for a complete real-world example using a Task Management API.

---

## What You'll Learn

By the end of this tutorial, you'll have:
- ✅ Cloned and validated the template
- ✅ Run all core commands (check, bdd, bundle)
- ✅ Started the HTTP service
- ✅ Made a simple change to an endpoint
- ✅ Understood the AC-first workflow

---

## Step 1: Clone and Enter Environment (5 minutes)

### Clone the template

```bash
git clone https://github.com/your-org/rust-template.git
cd rust-template
```

### Enter Nix development shell

This installs all tools (Rust, Python, OPA, etc.) in a reproducible way:

```bash
nix develop
```

**Expected output:**
```
warning: creating lock file...
(nix:rust-template-env)
```

You're now in a shell with all dependencies. To exit later, just type `exit`.

### Validate everything works

```bash
cargo run -p xtask -- quickstart
```

**Expected output:**
```
======================================
  Rust Template Quick Start
======================================

[1/5] Checking environment...
  ✓ cargo 1.91.0
  ✓ rustc 1.91.0

[2/5] Running xtask check...
  ✓ Format check passed
  ✓ Clippy passed
  ✓ Tests passed

[3/5] Running BDD acceptance tests...
  ✓ BDD scenarios passed
  ✓ JUnit output created

[4/5] Testing LLM context bundler...
  ✓ Bundle command executed
  ✓ Bundle created (2708 bytes)

[5/5] Testing helper commands...
  ✓ Core commands validated

======================================
✓ Template validation passed!
======================================
```

If you see this, everything is working! 🎉

---

## Step 2: Explore the Codebase (5 minutes)

### Open the project in your editor

```bash
code .  # Or vim, emacs, etc.
```

### Key files to open

Open these files side-by-side to see the architecture:

1. **specs/spec_ledger.yaml** - The source of truth for requirements
2. **specs/features/refunds.feature** - BDD scenarios
3. **crates/core/src/lib.rs** - Domain logic
4. **crates/app-http/src/main.rs** - HTTP endpoints

### Understand the flow

**Specification → Test → Code → Validation**

```
specs/spec_ledger.yaml           (AC-123: "Customer can create refund")
        ↓
specs/features/refunds.feature   (@AC-123 Gherkin scenario)
        ↓
crates/core/src/lib.rs           (refund_ok() function)
        ↓
crates/app-http/src/main.rs      (POST /refunds endpoint)
        ↓
target/junit/acceptance.xml      (Test results)
```

---

## Step 3: Run the HTTP Service (5 minutes)

### Start the service

```bash
cd crates/app-http
cargo run
```

**Expected output:**
```
INFO app_http: Starting HTTP service
INFO app_http: Listening on 0.0.0.0:8080
```

### Test the endpoints

In another terminal:

```bash
# Health check
curl http://localhost:8080/health

# Expected: {"status":"ok","service":"refunds-api"}
```

```bash
# Create a refund
curl -X POST http://localhost:8080/refunds \
  -H "Content-Type: application/json" \
  -d '{"order_id":"ORD-123","amount_cents":5000}'

# Expected: {"refund_id":"REF-...","order_id":"ORD-123","amount_cents":5000,"status":"pending"}
```

### See structured logs

Notice the server logs when you make requests:

```
INFO app_http::create_refund{order_id="ORD-123" amount=5000}: Creating refund
INFO app_http::create_refund{order_id="ORD-123" amount=5000}: Refund created refund_id="REF-abc123"
```

This is structured logging in action - `order_id` and `amount` are queryable fields.

### Stop the service

Press `Ctrl+C` in the server terminal.

---

## Step 4: Make Your First Change (10 minutes)

Let's add a new field to the refund response.

### 1. Update the response DTO

**File:** `crates/app-http/src/main.rs`

Find the `CreateRefundResponse` struct and add a `created_at` field:

```rust
#[derive(Debug, Serialize)]
struct CreateRefundResponse {
    refund_id: String,
    order_id: String,
    amount_cents: u64,
    status: String,
    created_at: String,  // ← Add this
}
```

### 2. Update the handler

In the `create_refund` function, add the timestamp:

```rust
Ok(Json(CreateRefundResponse {
    refund_id: refund.id,
    order_id: payload.order_id,
    amount_cents: payload.amount_cents,
    status: "pending".to_string(),
    created_at: chrono::Utc::now().to_rfc3339(),  // ← Add this
}))
```

### 3. Run checks

```bash
cargo run -p xtask -- check
```

**Expected:**
```
Running format check...
Running clippy...
Running tests...
✓ All checks passed
```

### 4. Test the change

Start the server again:
```bash
cargo run -p app-http
```

Make a request:
```bash
curl -X POST http://localhost:8080/refunds \
  -H "Content-Type: application/json" \
  -d '{"order_id":"ORD-456","amount_cents":3000}'
```

**Expected response now includes `created_at`:**
```json
{
  "refund_id": "REF-...",
  "order_id": "ORD-456",
  "amount_cents": 3000,
  "status": "pending",
  "created_at": "2025-11-13T10:30:00Z"
}
```

Success! You've made your first change. 🎉

---

## Step 5: Understand AC-First Workflow (5 minutes)

The change you just made was *not* AC-first. Let's see the proper workflow.

### The AC-First Loop

```
1. Spec: Add AC to specs/spec_ledger.yaml
2. Test: Write BDD scenario with @AC-#### tag
3. Code: Implement to make test pass
4. Validate: Run xtask bdd, check feature_status.md
```

### Example: Adding "refund reason" field (New AC)

> **Note**: We're creating a **new** AC (AC-124) as an example. It doesn't exist yet in the template's `spec_ledger.yaml`.

**1. Update spec (`specs/spec_ledger.yaml`):**

Add this new AC under the existing `AC-123`:

```yaml
acceptance_criteria:
  - id: AC-124  # ← NEW: Add this AC
    text: "Refund request includes optional reason"
    tests:
      - type: bdd
        tag: "@AC-124"
```

**2. Write scenario (`specs/features/refunds.feature`):**

Add this new scenario to the feature file:

```gherkin
@AC-124  # ← References the new AC you just added
Scenario: Create refund with reason
  Given an order "ORD-789" totalling 10000 cents
  When I POST /refunds with { "orderId": "ORD-789", "amountCents": 10000, "reason": "damaged goods" }
  Then I receive 201 with a "refundId"
  And the response includes "reason" with value "damaged goods"
```

**3. Implement (update DTOs and handler)**

**4. Validate:**
```bash
cargo run -p xtask -- bdd
cargo run -p xtask -- ac-status
cat docs/feature_status.md  # ← AC-124 should show as passing
```

### Why AC-First?

- **Traceability:** Every feature maps to business requirement
- **Documentation:** Feature status shows what's working
- **Governance:** Policies enforce that ACs have tests
- **Communication:** Non-developers can read Gherkin scenarios

---

## Step 6: Explore LLM Context Bundles (5 minutes)

The template includes LLM context bundler for AI-assisted coding.

### Generate a bundle

```bash
cargo run -p xtask -- bundle implement_ac
```

**Output:**
```
Generating LLM context bundle for task: implement_ac
Bundle written to: .llm/bundle/implement_ac.md
```

### Look at the bundle

```bash
cat .llm/bundle/implement_ac.md
```

You'll see it includes:
- specs/spec_ledger.yaml
- specs/features/*.feature
- crates/core/src/**
- crates/acceptance/src/**

### Use it with an LLM

Copy the bundle content and paste into ChatGPT/Claude with a prompt like:

> "Looking at this codebase context, implement AC-124: 'Refund request includes optional reason'. Show me the diffs for:
> 1. CreateRefundRequest DTO
> 2. Handler logic
> 3. BDD step definition"

**Important:** Always validate LLM output:
- ✅ Run `xtask check`
- ✅ Run `xtask bdd`
- ✅ Review diffs carefully

See `docs/how-to/use-llm-bundles.md` for best practices.

---

## What You've Learned

✅ **Environment:** Nix dev shell provides reproducible setup
✅ **Validation:** `xtask quickstart` verifies everything works
✅ **Architecture:** Specs → Tests → Core → HTTP (hexagonal)
✅ **Development:** Make changes, run `xtask check` before committing
✅ **AC-First:** Proper workflow is spec → test → code → validate
✅ **LLM Integration:** Context bundles provide focused AI assistance

---

## Next Steps

### For Learning:
- 📖 **Read:** `docs/explanation/architecture.md` - Understand design decisions
- 📖 **Tutorial:** `docs/tutorials/first-ac-change.md` - Complete AC workflow

### For Building:
- 🛠️ **How-to:** `docs/how-to/new-service-from-template.md` - Adapt for your service
- 🛠️ **How-to:** `docs/how-to/use-llm-bundles.md` - LLM best practices

### For Reference:
- 📚 **API:** `TEMPLATE_API.md` - All xtask commands and schemas
- 📚 **Profiles:** `docs/reference/branch-protection-profiles.md` - CI configuration

---

## Troubleshooting

**`nix develop` fails:**
- Ensure Nix is installed:
  ```bash
  curl -L https://nixos.org/nix/install -o nix_install.sh
  sh nix_install.sh --daemon
  rm nix_install.sh
  ```
- Check Nix flakes are enabled: `nix-env --version`

**`xtask quickstart` fails on format check:**
- Run `cargo fmt --all` first
- Then retry `cargo run -p xtask -- quickstart`

**HTTP service won't start:**
- Check port 8080 isn't in use: `lsof -i :8080`
- Try different port: Edit `crates/app-http/src/main.rs`, change `8080` to your chosen port

**BDD tests fail:**
- Check `specs/features/*.feature` syntax
- Run `cargo test -p acceptance` for detailed errors
- Verify step definitions in `crates/acceptance/src/steps/`

---

## Summary

You've now seen the template's core workflow:

1. **Validate:** `xtask quickstart` - everything works
2. **Develop:** Make changes in `crates/`
3. **Check:** `xtask check` - quality gates pass
4. **Test:** `xtask bdd` - scenarios pass
5. **Iterate:** Repeat until feature complete

The template enforces quality by default - doing things right is the path of least resistance.

Welcome to AC-first, policy-driven, LLM-native, governance-bounded Rust development! 🦀

