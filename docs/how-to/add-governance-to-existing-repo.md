# How To: Add Governance to an Existing Rust Repository

This guide shows you how to add the Rust Template's governance infrastructure to an **existing Rust project** without starting from scratch.

**Use this if you have:**
- An existing Rust workspace or single crate
- Production code you want to keep
- A desire to add AC-first development, policy enforcement, and LLM-safe workflows

**Estimated time:** 45-60 minutes for initial setup + 30-60 minutes for customization

---

## Prerequisites

### Required
- **Existing Rust project** - Either a workspace or single crate
- **Git repository** - Project is version controlled
- **Basic Rust knowledge** - Familiar with Cargo and workspaces
- **Command-line comfort** - Can run shell commands

### Recommended (but optional)
- **Existing test suite** - Makes AC mapping easier
- **CI/CD setup** - To integrate new checks
- **Team buy-in** - Governance works best with team adoption

### Tools
All tools are available in the Nix shell (optional but recommended):
```bash
# Option 1: Use Nix (recommended - all tools included)
nix develop

# Option 2: Install tools manually
# - Rust 1.89+ (rustup install stable)
# - conftest (for policy tests) - https://www.conftest.dev/install/
# - git (for tracking files)
```

---

## Overview: What You'll Add

This process adds **four governance layers** to your existing codebase:

1. **xtask orchestrator** - Single CLI for all dev operations
2. **Specification ledger** - YAML-based AC tracking
3. **Policy enforcement** - Rego rules for governance
4. **LLM context bundles** - Safe AI-assisted development

**You keep:**
- ✅ All existing code
- ✅ Your current architecture
- ✅ Existing tests and workflows
- ✅ Project structure

**You add:**
- ✅ AC-first workflow tooling
- ✅ Policy-as-code validation
- ✅ LLM context bundler
- ✅ Acceptance test framework (BDD)

**Security defaults**
- Starts open for quick spins.
- Set PLATFORM_AUTH_MODE=basic and PLATFORM_AUTH_TOKEN=<secret> in .env or config to require a token on write endpoints (e.g., task status updates).
- The app will warn at startup if basic is selected without a token.

---

## Step 1: Install the xtask Orchestrator (10 min)

The `xtask` crate provides a Rust-native CLI for all governance operations.

### 1.1: Create xtask Directory

```bash
# From your project root
mkdir -p crates/xtask
cd crates/xtask
```

### 1.2: Create Cargo.toml

Create `crates/xtask/Cargo.toml`:

```toml
[package]
name = "xtask"
version = "0.1.0"
edition = "2024"

[dependencies]
clap = { version = "4.5", features = ["derive"] }
anyhow = "1.0"
colored = "3.0"
serde = { version = "1.0", features = ["derive"] }
serde_yaml = { package = "serde_yaml_ng", version = "0.10" }
serde_json = "1.0"
quick-xml = "0.38"
glob = "0.3"
regex = "1.12"
walkdir = "2.5"
ignore = "0.4"
once_cell = "1.21"
```

### 1.3: Create xtask Binary

Create `crates/xtask/src/main.rs`:

```rust
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::process::Command;

mod commands;

/// xtask: Development and CI orchestration tool
#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Development and CI orchestration", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all checks: fmt, clippy, tests
    Check,
    /// Run BDD acceptance tests
    Bdd,
    /// Generate AC status report
    AcStatus,
    /// Test Rego policies with conftest
    PolicyTest,
    /// Generate LLM context bundle
    Bundle {
        /// Task name from .llm/contextpack.yaml
        task: String,
    },
    /// Run full self-test suite
    Selftest,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check => commands::check::run(),
        Commands::Bdd => commands::bdd::run(),
        Commands::AcStatus => commands::ac_status::run(),
        Commands::PolicyTest => commands::policy_test::run(),
        Commands::Bundle { task } => commands::bundle::run(&task),
        Commands::Selftest => commands::selftest::run(),
    }
}

/// Helper to run a command and propagate failures
pub fn run_cmd(cmd: &mut Command) -> Result<()> {
    let output = cmd.output()
        .with_context(|| format!("Failed to execute: {:?}", cmd))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Command failed: {}", stderr);
    }

    Ok(())
}
```

### 1.4: Create Command Modules

Create `crates/xtask/src/commands/mod.rs`:

```rust
pub mod check;
pub mod bdd;
pub mod ac_status;
pub mod policy_test;
pub mod bundle;
pub mod selftest;
```

Create `crates/xtask/src/commands/check.rs`:

```rust
use anyhow::Result;
use colored::Colorize;
use std::process::Command;

pub fn run() -> Result<()> {
    println!("{}", "Running format check...".blue());
    crate::run_cmd(Command::new("cargo")
        .args(["fmt", "--all", "--", "--check"]))?;

    println!("{}", "Running clippy...".blue());
    crate::run_cmd(Command::new("cargo")
        .args(["clippy", "--all-targets", "--all-features", "--", "-D", "warnings"]))?;

    println!("{}", "Running tests...".blue());
    crate::run_cmd(Command::new("cargo")
        .args(["test", "--workspace"]))?;

    println!("{}", "✓ All checks passed".green());
    Ok(())
}
```

Create `crates/xtask/src/commands/bdd.rs`:

```rust
use anyhow::Result;
use colored::Colorize;

pub fn run() -> Result<()> {
    println!("{}", "BDD tests not yet configured".yellow());
    println!("See Step 2 to initialize acceptance tests");
    Ok(())
}
```

Create `crates/xtask/src/commands/ac_status.rs`:

```rust
use anyhow::Result;
use colored::Colorize;

pub fn run() -> Result<()> {
    println!("{}", "AC status not yet configured".yellow());
    println!("See Step 2 to initialize spec ledger");
    Ok(())
}
```

Create `crates/xtask/src/commands/policy_test.rs`:

```rust
use anyhow::Result;
use colored::Colorize;

pub fn run() -> Result<()> {
    println!("{}", "Policy tests not yet configured".yellow());
    println!("See Step 2 to initialize policies");
    Ok(())
}
```

Create `crates/xtask/src/commands/bundle.rs`:

```rust
use anyhow::Result;
use colored::Colorize;

pub fn run(task: &str) -> Result<()> {
    println!("{}", format!("LLM bundler not yet configured for task: {}", task).yellow());
    println!("See Step 2 to initialize context packs");
    Ok(())
}
```

Create `crates/xtask/src/commands/selftest.rs`:

```rust
use anyhow::Result;
use colored::Colorize;

pub fn run() -> Result<()> {
    println!("{}", "Running basic selftest...".blue());

    // Just run check for now
    crate::commands::check::run()?;

    println!("{}", "✓ Basic checks passed!".green());
    println!("Run Step 2 to enable full governance suite");
    Ok(())
}
```

### 1.5: Add to Workspace

Edit your root `Cargo.toml` to add xtask to workspace members:

```toml
[workspace]
members = [
    "crates/xtask",
    # ... your existing crates
]
```

### 1.6: Test xtask

```bash
# From project root
cargo run -p xtask -- check
```

**Expected output:**
```
Running format check...
Running clippy...
Running tests...
✓ All checks passed
```

**If this fails:**
- Check that `crates/xtask/` path is correct
- Verify all files were created
- Run `cargo build -p xtask` to see compilation errors

---

## Step 2: Initialize Governance Structure (15 min)

Now add the governance files that xtask will orchestrate.

### 2.1: Create Specification Ledger

Create `specs/spec_ledger.yaml`:

```yaml
# Acceptance Criteria Ledger
# Maps user stories → requirements → acceptance criteria → tests

stories:
  # Start with existing functionality
  - id: US-CORE-001
    title: "Core Service Capabilities"
    requirements:
      - id: REQ-CORE-HEALTH
        title: "Health Check"
        acceptance_criteria:
          - id: AC-CORE-001
            text: "Service responds to health checks"
            tests: []  # Add your existing tests here

  # Add your domain-specific stories below
  # Example:
  # - id: US-AUTH-001
  #   title: "User Authentication"
  #   requirements:
  #     - id: REQ-AUTH-LOGIN
  #       title: "User Login"
  #       acceptance_criteria:
  #         - id: AC-AUTH-001
  #           text: "Users can log in with email and password"
  #           tests: [{ type: bdd, tag: "@AC-AUTH-001" }]
```

**Tips:**
- Start with 3-5 core ACs from your existing functionality
- Don't try to document everything at once
- You can add more incrementally

### 2.2: Create Policy Directory

Create basic governance policies:

**Create `policy/ledger.rego`:**

```rego
package main

deny[msg] {
    # Ensure every AC has tests defined
    ac := input.stories[_].requirements[_].acceptance_criteria[_]
    not has_tests(ac)
    msg := sprintf("AC '%s' must have a non-empty 'tests' array.", [ac.id])
}

has_tests(ac) {
    is_array(ac.tests)
    count(ac.tests) > 0
}
```

**Create `policy/testdata/ledger_valid.json`:**

```json
{
  "stories": [
    {
      "id": "US-001",
      "title": "Example",
      "requirements": [
        {
          "id": "REQ-001",
          "title": "Example Requirement",
          "acceptance_criteria": [
            {
              "id": "AC-001",
              "text": "Example criterion",
              "tests": [{ "type": "bdd", "tag": "@AC-001" }]
            }
          ]
        }
      ]
    }
  ]
}
```

**Create `policy/testdata/ledger_invalid.json`:**

```json
{
  "stories": [
    {
      "id": "US-001",
      "title": "Example",
      "requirements": [
        {
          "id": "REQ-001",
          "title": "Example",
          "acceptance_criteria": [
            {
              "id": "AC-001",
              "text": "Missing tests",
              "tests": []
            }
          ]
        }
      ]
    }
  ]
}
```

### 2.3: Create LLM Context Pack Configuration

Create `.llm/contextpack.yaml`:

```yaml
tasks:
  implement_feature:
    max_bytes: 300000
    include:
      - specs/spec_ledger.yaml
      - crates/**/src/**/*.rs
      - Cargo.toml
    description: "Context for implementing features: specs and code"

  debug_tests:
    max_bytes: 200000
    include:
      - crates/**/tests/**/*.rs
      - crates/**/src/**/*.rs
    description: "Context for debugging tests"
```

Create `.llm/.llmignore` (uses gitignore syntax):

```gitignore
# Exclude large/generated files from LLM context
target/
.git/
*.lock
*.png
*.jpg
*.pdf
node_modules/
```

### 2.4: Create Acceptance Test Structure (Optional but Recommended)

If you want BDD acceptance tests:

```bash
# Create acceptance test crate
cargo new --lib crates/acceptance
```

Edit `crates/acceptance/Cargo.toml`:

```toml
[package]
name = "acceptance"
version = "0.1.0"
edition = "2024"

[dependencies]
cucumber = "0.21"
tokio = { version = "1", features = ["full"] }
```

Create `crates/acceptance/tests/acceptance.rs`:

```rust
use cucumber::{World, WorldInit};

#[derive(Debug, WorldInit)]
pub struct TestWorld;

#[tokio::main]
async fn main() {
    TestWorld::cucumber()
        .run_and_exit("specs/features")
        .await;
}
```

Create your first feature file `specs/features/core.feature`:

```gherkin
Feature: Core Service Health
  As a platform operator
  I want to check service health
  So that I can monitor system availability

  @AC-CORE-001
  Scenario: Health check endpoint responds
    Given the service is running
    When I send GET request to "/health"
    Then I receive 200 status code
```

### 2.5: Update Policy Test Command

Update `crates/xtask/src/commands/policy_test.rs` to actually run conftest:

```rust
use anyhow::{Context, Result};
use colored::Colorize;
use std::process::Command;

pub fn run() -> Result<()> {
    // Check if conftest is available
    if !is_conftest_available() {
        println!("{}", "⚠ conftest not found on PATH".yellow());
        println!("Install: brew install conftest (macOS) or use nix develop");
        return Ok(());
    }

    println!("{}", "Testing Rego policies...".blue());

    // Test ledger policy
    println!("\n{}", "Ledger Policy:".bold());
    test_policy("policy/ledger.rego", "policy/testdata/ledger_valid.json", true)?;
    test_policy("policy/ledger.rego", "policy/testdata/ledger_invalid.json", false)?;

    println!("\n{}", "✓ All policy tests passed!".green());
    Ok(())
}

fn is_conftest_available() -> bool {
    Command::new("conftest")
        .arg("--version")
        .output()
        .is_ok()
}

fn test_policy(policy: &str, fixture: &str, should_pass: bool) -> Result<()> {
    let output = Command::new("conftest")
        .args(["test", "-p", policy, fixture])
        .output()
        .with_context(|| format!("Failed to run conftest on {}", fixture))?;

    let passed = output.status.success();

    if passed == should_pass {
        println!("  {} {} (correctly {})",
            "✓".green(),
            fixture,
            if should_pass { "passed" } else { "failed" }
        );
        Ok(())
    } else {
        anyhow::bail!(
            "Policy test failed: {} should have {} but {}",
            fixture,
            if should_pass { "passed" } else { "failed" },
            if passed { "passed" } else { "failed" }
        );
    }
}
```

---

## Step 3: Run Selftest (5 min)

Verify everything is wired up correctly.

### 3.1: Run the Suite

```bash
cargo run -p xtask -- selftest
```

### 3.2: Interpret Results

**✓ All checks passed:**
```
Running basic selftest...
Running format check...
Running clippy...
Running tests...
✓ All checks passed!
✓ Basic checks passed!
```

Your governance foundation is installed correctly!

**Common issues:**

#### Format check fails
```
✗ Format check failed
```

**Fix:**
```bash
cargo fmt --all
```

#### Clippy warnings
```
✗ Clippy failed: warnings found
```

**Fix:**
```bash
cargo clippy --all-targets --all-features --fix
```

#### Tests fail
```
✗ Tests failed
```

**Fix:**
- Review test output
- Fix failing tests in your existing codebase
- Or temporarily exclude failing crates with `--exclude`

### 3.3: Run Individual Commands

Test each command separately:

```bash
# Check code quality
cargo run -p xtask -- check

# Test policies (if conftest installed)
cargo run -p xtask -- policy-test

# Generate AC status (after Step 2)
cargo run -p xtask -- ac-status
```

---

## Step 4: Wire CI Integration (10 min)

Add governance checks to your CI pipeline.

### 4.1: GitHub Actions Example

Create `.github/workflows/governance.yml`:

```yaml
name: Governance Checks
on:
  push:
    branches: [main]
  pull_request:

jobs:
  checks:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          toolchain: stable

      - name: Run code quality checks
        run: cargo run -p xtask -- check

      - name: Run policy tests
        run: |
          # Install conftest
          wget https://github.com/open-policy-agent/conftest/releases/download/v0.56.0/conftest_0.56.0_Linux_x86_64.tar.gz
          tar xzf conftest_0.56.0_Linux_x86_64.tar.gz
          sudo mv conftest /usr/local/bin/

          # Run tests
          cargo run -p xtask -- policy-test

      - name: Generate AC status report
        run: cargo run -p xtask -- ac-status
        continue-on-error: true  # Don't fail if no tests yet

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: governance-reports
          path: |
            docs/feature_status.md
```

### 4.2: GitLab CI Example

Create `.gitlab-ci.yml`:

```yaml
stages:
  - governance

governance:checks:
  stage: governance
  image: rust:latest
  before_script:
    - rustup component add rustfmt clippy
  script:
    - cargo run -p xtask -- check

governance:policies:
  stage: governance
  image: rust:latest
  before_script:
    - wget -O /usr/local/bin/conftest https://github.com/open-policy-agent/conftest/releases/download/v0.56.0/conftest_0.56.0_Linux_x86_64
    - chmod +x /usr/local/bin/conftest
  script:
    - cargo run -p xtask -- policy-test
```

### 4.3: Pre-commit Hook (Optional)

Create `.git/hooks/pre-commit`:

```bash
#!/bin/bash
# Run checks before allowing commit

echo "Running pre-commit checks..."
cargo run -p xtask -- check

if [ $? -ne 0 ]; then
    echo "❌ Pre-commit checks failed. Fix issues before committing."
    exit 1
fi

echo "✅ Pre-commit checks passed!"
```

Make it executable:
```bash
chmod +x .git/hooks/pre-commit
```

---

## Step 5: Customize Policies (15 min)

Add project-specific governance rules.

### 5.1: Add Feature Flag Policy

If your project uses feature flags, enforce ownership:

Create `flags/registry.yaml`:

```yaml
flags:
  - key: new_ui
    owner: frontend-team
    default: false
    expires_at: 2026-06-30
    description: "New UI redesign rollout"
```

Create `policy/flags.rego`:

```rego
package main

deny[msg] {
    flag := input.flags[_]
    not flag.owner
    msg := sprintf("Flag '%s' must have an owner", [flag.key])
}

deny[msg] {
    flag := input.flags[_]
    not flag.expires_at
    msg := sprintf("Flag '%s' must have an expiration date", [flag.key])
}
```

### 5.2: Add Privacy Policy

For projects handling PII:

Create `specs/privacy.yaml`:

```yaml
fields:
  - path: user.email
    classification: PII
    owner: auth-team
    retention: "365d"
    purpose: "User authentication and communication"

  - path: user.payment_info
    classification: SENSITIVE
    owner: payments-team
    retention: "2555d"  # 7 years for compliance
    purpose: "Payment processing"
```

Create `policy/privacy.rego`:

```rego
package main

deny[msg] {
    field := input.fields[_]
    field.classification == "PII"
    not field.owner
    msg := sprintf("PII field '%s' must have an owner", [field.path])
}

deny[msg] {
    field := input.fields[_]
    not field.retention
    msg := sprintf("Field '%s' must specify retention period", [field.path])
}
```

### 5.3: Test Policies Locally

```bash
# Test individual policy
conftest test -p policy/flags.rego flags/registry.yaml

# Test all via xtask
cargo run -p xtask -- policy-test
```

**Expected output:**
```
Testing Rego policies...

Ledger Policy:
  ✓ ledger_valid.json (correctly passed)
  ✓ ledger_invalid.json (correctly failed)

✓ All policy tests passed!
```

---

## Step 6: Map Existing Tests to ACs (Optional, 20 min)

Connect your existing tests to acceptance criteria for traceability.

### 6.1: Identify Core Functionality

List 5-10 core capabilities your service provides:

Example for a payment service:
- Process payments
- Refund transactions
- Check transaction status
- Validate payment methods
- Handle webhooks

### 6.2: Create ACs for Each

Update `specs/spec_ledger.yaml`:

```yaml
stories:
  - id: US-PAYMENTS-001
    title: "Payment Processing"
    requirements:
      - id: REQ-PAY-PROCESS
        title: "Process Payment Requests"
        acceptance_criteria:
          - id: AC-PAY-001
            text: "Service accepts valid payment requests and returns transaction ID"
            tests: [{ type: unit, path: "crates/core/tests/payments.rs::test_process_payment" }]

          - id: AC-PAY-002
            text: "Service rejects invalid payment amounts"
            tests: [{ type: unit, path: "crates/core/tests/payments.rs::test_invalid_amount" }]

      - id: REQ-PAY-REFUND
        title: "Refund Processed Payments"
        acceptance_criteria:
          - id: AC-PAY-003
            text: "Service can refund completed transactions"
            tests: [{ type: unit, path: "crates/core/tests/refunds.rs::test_full_refund" }]
```

### 6.3: Generate Status Report

Implement AC status mapping in `crates/xtask/src/commands/ac_status.rs`:

```rust
use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Deserialize)]
struct Ledger {
    stories: Vec<Story>,
}

#[derive(Deserialize)]
struct Story {
    id: String,
    title: String,
    requirements: Vec<Requirement>,
}

#[derive(Deserialize)]
struct Requirement {
    id: String,
    title: String,
    acceptance_criteria: Vec<AcceptanceCriterion>,
}

#[derive(Deserialize)]
struct AcceptanceCriterion {
    id: String,
    text: String,
    tests: Vec<Test>,
}

#[derive(Deserialize)]
struct Test {
    #[serde(rename = "type")]
    test_type: String,
    #[serde(default)]
    path: String,
}

pub fn run() -> Result<()> {
    println!("{}", "Generating AC status report...".blue());

    // Load ledger
    let ledger_yaml = fs::read_to_string("specs/spec_ledger.yaml")
        .context("Failed to read specs/spec_ledger.yaml")?;
    let ledger: Ledger = serde_yaml::from_str(&ledger_yaml)
        .context("Failed to parse ledger YAML")?;

    // Generate markdown report
    let mut report = String::from("# Feature Status Report\n\n");
    report.push_str("Generated from `specs/spec_ledger.yaml`\n\n");
    report.push_str("| AC ID | Description | Test Coverage | Status |\n");
    report.push_str("|-------|-------------|---------------|--------|\n");

    for story in &ledger.stories {
        for req in &story.requirements {
            for ac in &req.acceptance_criteria {
                let status = if ac.tests.is_empty() {
                    "❓ Unknown"
                } else {
                    "✅ Covered"
                };

                let test_count = ac.tests.len();
                report.push_str(&format!(
                    "| {} | {} | {} test(s) | {} |\n",
                    ac.id,
                    truncate(&ac.text, 50),
                    test_count,
                    status
                ));
            }
        }
    }

    // Write report
    fs::create_dir_all("docs")?;
    fs::write("docs/feature_status.md", report)?;

    println!("{}", "✓ Generated docs/feature_status.md".green());
    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}
```

Run it:
```bash
cargo run -p xtask -- ac-status
cat docs/feature_status.md
```

---

## Troubleshooting

### Issue: "cargo run -p xtask" fails

**Symptom:**
```
error: package `xtask` is not a member of the workspace
```

**Fix:**
- Check `Cargo.toml` has `"crates/xtask"` in `workspace.members`
- Verify directory structure: `crates/xtask/Cargo.toml` exists
- Run `cargo metadata` to verify workspace structure

### Issue: Format check fails with "file not formatted"

**Symptom:**
```
✗ Format check failed
Diff in /path/to/file.rs
```

**Fix:**
```bash
# Auto-format all code
cargo fmt --all

# Then retry
cargo run -p xtask -- check
```

### Issue: Policy tests fail "conftest not found"

**Symptom:**
```
⚠ conftest not found on PATH
```

**Fix Option 1 (Nix):**
```bash
nix develop  # Provides conftest automatically
cargo run -p xtask -- policy-test
```

**Fix Option 2 (Manual install):**
```bash
# macOS
brew install conftest

# Linux
wget https://github.com/open-policy-agent/conftest/releases/download/v0.56.0/conftest_0.56.0_Linux_x86_64.tar.gz
tar xzf conftest_0.56.0_Linux_x86_64.tar.gz
sudo mv conftest /usr/local/bin/
```

### Issue: AC status shows all "Unknown"

**Symptom:**
```
All ACs show ❓ Unknown status
```

**Fix:**
- Verify `specs/spec_ledger.yaml` has `tests` arrays populated
- Check test paths/tags are correct
- ACs without tests will show as Unknown (this is expected initially)

### Issue: Clippy warnings block commit

**Symptom:**
```
error: this could be written with `if let`
  --> crates/core/src/lib.rs:10:5
```

**Fix Option 1 (Auto-fix):**
```bash
cargo clippy --all-targets --fix
```

**Fix Option 2 (Adjust warning level):**

In `crates/xtask/src/commands/check.rs`, change:
```rust
.args(["clippy", "--all-targets", "--all-features", "--", "-D", "warnings"])
```

To:
```rust
.args(["clippy", "--all-targets", "--all-features", "--", "-W", "clippy::all"])
```

### Issue: BDD tests don't run

**Symptom:**
```
BDD tests not yet configured
```

**Fix:**
- Complete Step 2.4 to add acceptance test crate
- Update `crates/xtask/src/commands/bdd.rs` to run cucumber tests
- Create feature files in `specs/features/`

---

## Next Steps

### Immediate (Do Today)
1. ✅ **Commit your changes:**
   ```bash
   git add .
   git commit -m "feat: add governance infrastructure (xtask + policies)"
   git push
   ```

2. ✅ **Share with team:**
   - Document in team wiki
   - Demo `cargo run -p xtask -- check` workflow
   - Get buy-in for AC-first development

### Short Term (This Week)
1. **Add 5-10 core ACs** to `specs/spec_ledger.yaml`
2. **Map existing tests** to ACs
3. **Add CI integration** (Step 4)
4. **Write first BDD scenario** (if using acceptance tests)

### Medium Term (This Month)
1. **Expand AC coverage** to 80% of features
2. **Add custom policies** for your domain (flags, privacy, etc.)
3. **Train team** on AC-first workflow
4. **Integrate LLM bundler** for AI-assisted development

### Recommended Reading
- **[AC-First Development](../tutorials/first-ac-change.md)** - Learn the AC workflow
- **[xtask Commands Reference](../reference/xtask-commands.md)** - All commands explained
- **[Use LLM Bundles](use-llm-bundles.md)** - AI-assisted development with safety
- **[Branch Protection Profiles](../reference/branch-protection-profiles.md)** - CI requirements

---

## What This Enables

After completing this guide, your brownfield project now has:

### ✅ Single Command Interface
```bash
cargo run -p xtask -- check      # Before every commit
cargo run -p xtask -- selftest   # Before every push
cargo run -p xtask -- ac-status  # Check AC coverage
```

### ✅ Policy Enforcement
- Every AC must have tests (enforced by Rego)
- Feature flags require ownership
- PII fields have retention policies
- Validated in CI on every PR

### ✅ Traceability
- User stories → Requirements → ACs → Tests
- Generate coverage reports automatically
- Track which features are tested

### ✅ LLM-Safe Development
- Bounded context bundles (prevents leaking entire codebase)
- Policy validation before AI suggestions applied
- Governance-first, move fast second

---

## Frequently Asked Questions

### Q: Do I need to use Nix?

**A:** No, but recommended. Nix provides all tools (conftest, rustfmt, etc.) automatically.

Without Nix:
- Install Rust via rustup
- Install conftest manually
- Install other tools as needed

### Q: Can I keep my existing test structure?

**A:** Yes! The governance layer sits alongside your existing tests.

- Keep unit tests in `crates/*/tests/`
- Keep integration tests wherever they are
- Add BDD tests optionally in `crates/acceptance/`
- Map existing tests to ACs in ledger

### Q: Do I have to use BDD/Gherkin?

**A:** No. BDD is optional but recommended for acceptance criteria.

Alternatives:
- Map existing integration tests to ACs
- Use `tests` field to reference unit test paths
- Skip `bdd` command, focus on `check` and `policy-test`

### Q: What if my team doesn't want AC-first workflow?

**A:** Start with just policy enforcement:

Minimal adoption:
1. Install xtask (`cargo run -p xtask -- check`)
2. Use in CI for code quality
3. Skip AC ledger and policies initially
4. Adopt incrementally as team sees value

### Q: How do I upgrade to newer template versions?

**A:** Since this is brownfield, you don't track upstream. Instead:

1. Watch template releases: https://github.com/your-org/rust-template/releases
2. Read changelogs for improvements
3. Manually copy changes you want (new policies, commands, etc.)

See [Adoption Patterns](../explanation/adoption-patterns.md) for alternatives.

### Q: Can I customize the xtask commands?

**A:** Absolutely! The xtask code is yours to modify:

- Add custom commands (e.g., `Deploy`, `Migrate`)
- Change check strictness
- Add project-specific validations
- Wire into your existing tooling

### Q: What's the performance impact?

**A:** Negligible for development, seconds in CI:

| Command | Local | CI |
|---------|-------|-----|
| `check` | ~10s | ~30s |
| `policy-test` | ~1s | ~2s |
| `ac-status` | <1s | <1s |
| `selftest` | ~15s | ~45s |

Runs only when you invoke it, no background processes.

---

## Summary

**Time investment:** ~45-60 minutes initial setup + 30-60 minutes customization

**What you added:**
- ✅ xtask orchestrator for unified CLI
- ✅ Specification ledger for AC tracking
- ✅ Policy enforcement for governance
- ✅ LLM context bundler for safe AI development
- ✅ CI integration for automated checks

**What you kept:**
- ✅ All existing code unchanged
- ✅ Current architecture preserved
- ✅ Existing tests still run
- ✅ Team workflow intact

**Next actions:**
1. Commit changes
2. Share with team
3. Add 5-10 ACs to ledger
4. Wire CI integration
5. Start using `cargo run -p xtask -- check`

**Questions?** See:
- [xtask Commands Reference](../reference/xtask-commands.md)
- [Adoption Patterns](../explanation/adoption-patterns.md)
- [Template Architecture](../explanation/architecture.md)

---

**You're ready to use governance-heavy tooling while moving fast with LLMs!** 🦀
