<!-- doclint:disable orphan-version -->
<!-- External: This document references external tool versions that are not tied to template version. -->
# Tutorial: Getting Started with the Rust-as-Spec Platform Cell

**Time:** 30 minutes
**Goal:** Get the template running, understand core concepts, make your first change
**Prerequisites:** Git, Nix installed

---

## What You'll Learn

By the end of this tutorial, you'll have:
- Cloned and validated the template
- Run all core commands (check, bdd, bundle)
- Started the HTTP service and explored platform endpoints
- Made a simple change
- Understood the AC-first workflow

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
cargo xtask dev-up
```

**Expected output:**
```
======================================
  Rust-as-Spec Platform Cell Bootstrap
======================================

[1/5] Checking environment...
  ✓ cargo 1.91.0
  ✓ rustc 1.91.0

[2/5] Running xtask check...
  ✓ Format check passed
  ✓ Clippy passed
  ✓ Tests passed

[3/5] Running kernel smoke test...
  ✓ Kernel baseline validated

[4/5] Installing git hooks...
  ✓ Pre-commit hook installed

[5/5] Running AC status...
  ✓ All template ACs passing

======================================
✓ Environment ready!
======================================
```

If you see this, everything is working!

---

## Step 2: Explore the Codebase (5 minutes)

### Open the project in your editor

```bash
code .  # Or vim, emacs, etc.
```

### Key files to open

Open these files side-by-side to see the architecture:

1. **specs/spec_ledger.yaml** - The source of truth for requirements
2. **specs/features/*.feature** - BDD scenarios
3. **crates/business-core/src/lib.rs** - Domain logic
4. **crates/app-http/src/lib.rs** - HTTP endpoints

### Understand the flow

**Specification → Test → Code → Validation**

```
specs/spec_ledger.yaml           (AC-TPL-001: "GET /health returns 200")
        ↓
specs/features/health.feature    (@AC-TPL-001 Gherkin scenario)
        ↓
crates/app-http/src/lib.rs       (health endpoint handler)
        ↓
target/junit/acceptance.xml      (Test results)
```

---

## Step 3: Run the HTTP Service (5 minutes)

### Start the service

```bash
cargo run -p app-http
```

**Expected output:**
```
INFO app_http: Starting HTTP service
INFO app_http: Listening on 0.0.0.0:8080
```

### Test the core endpoints

In another terminal:

```bash
# Health check (AC-TPL-001)
curl http://localhost:8080/health

# Expected: {"status":"ok","service":"template-service"}
```

```bash
# Version info (AC-TPL-002)
curl http://localhost:8080/version

# Expected: {"version":"3.3.6","gitSha":"abc123..."}
```

### Explore platform introspection endpoints

These are the kernel's governance surfaces:

```bash
# Platform status - governance health, ledger counts, policy status
curl http://localhost:8080/platform/status | jq

# Documentation index with validation
curl http://localhost:8080/platform/docs/index | jq

# Debug info for development
curl http://localhost:8080/platform/debug/info | jq
```

The `/platform/*` endpoints expose the same governance data that CI enforces.

### Stop the service

Press `Ctrl+C` in the server terminal.

---

## Step 4: Make Your First Change (10 minutes)

Let's add a new debug field to `/platform/debug/info`.

### 1. Find the handler

**File:** `crates/app-http/src/routes/platform.rs`

Look for the `debug_info` handler function.

### 2. Add a new field

In the debug info response struct, add a `template_version` field.

### 3. Run checks

```bash
cargo xtask check
```

**Expected:**
```
Running format check...
Running clippy...
Running tests...
✓ All checks passed
```

### 4. Test the change

Start the server again and verify:
```bash
cargo run -p app-http &
curl http://localhost:8080/platform/debug/info | jq .template_version
```

Success! You've made your first change.

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

### Example: Adding a new platform endpoint

**1. Add AC to spec (`specs/spec_ledger.yaml`):**

```yaml
acceptance_criteria:
  - id: AC-PLT-NEW
    text: "GET /platform/metrics returns prometheus format"
    tests:
      - type: bdd
        tag: "@AC-PLT-NEW"
```

**2. Write scenario (`specs/features/platform.feature`):**

```gherkin
@AC-PLT-NEW
Scenario: Platform metrics endpoint returns prometheus format
  When I GET /platform/metrics
  Then I receive 200
  And the response content-type is "text/plain"
  And the response contains "http_requests_total"
```

**3. Implement (add handler and wire route)**

**4. Validate:**
```bash
cargo xtask bdd
cargo xtask ac-status
cat docs/feature_status.md  # AC-PLT-NEW should show as passing
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
cargo xtask bundle implement_ac
```

**Output:**
```
Generating LLM context bundle for task: implement_ac
Bundle written to: bundle/implement_ac/context.md
```

### Look at the bundle

```bash
cat bundle/implement_ac/context.md
```

You'll see it includes:
- specs/spec_ledger.yaml
- specs/features/*.feature
- Relevant source files

### Use it with an LLM

Copy the bundle content and paste into an LLM with a prompt like:

> "Looking at this codebase context, implement AC-PLT-METRICS: 'Platform metrics returns prometheus format'. Show me the diffs for:
> 1. Route registration
> 2. Handler implementation
> 3. BDD step definition"

**Important:** Always validate LLM output:
- Run `xtask check`
- Run `xtask bdd`
- Review diffs carefully

See `docs/how-to/use-llm-bundles.md` for best practices.

---

## What You've Learned

- **Environment:** Nix dev shell provides reproducible setup
- **Validation:** `xtask dev-up` verifies everything works
- **Architecture:** Specs → Tests → Core → HTTP (hexagonal)
- **Development:** Make changes, run `xtask check` before committing
- **AC-First:** Proper workflow is spec → test → code → validate
- **LLM Integration:** Context bundles provide focused AI assistance

---

## Next Steps

### For Learning:
- **Read:** `docs/explanation/architecture.md` - Understand design decisions
- **Tutorial:** `docs/tutorials/first-ac-change.md` - Complete AC workflow

### For Building:
- **How-to:** `docs/how-to/new-service-from-template.md` - Adapt for your service
- **How-to:** `docs/how-to/use-llm-bundles.md` - LLM best practices

### For Reference:
- **API:** `TEMPLATE_API.md` - All xtask commands and schemas
- **Profiles:** `docs/reference/branch-protection-profiles.md` - CI configuration

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

**`xtask dev-up` fails on format check:**
- Run `cargo fmt --all` first
- Then retry `cargo xtask dev-up`

**HTTP service won't start:**
- Check port 8080 isn't in use: `lsof -i :8080`
- Try different port: Set `HTTP_PORT=9090` environment variable

**BDD tests fail:**
- Check `specs/features/*.feature` syntax
- Run `cargo test -p acceptance` for detailed errors
- Verify step definitions in `crates/acceptance/src/steps/`

---

## Summary

You've now seen the template's core workflow:

1. **Validate:** `xtask dev-up` - everything works
2. **Develop:** Make changes in `crates/`
3. **Check:** `xtask check` - quality gates pass
4. **Test:** `xtask bdd` - scenarios pass
5. **Iterate:** Repeat until feature complete

The template enforces quality by default - doing things right is the path of least resistance.

Welcome to Rust-as-Spec development!

