# acceptance

BDD acceptance test framework for governance validation.

## What It Is

`acceptance` provides Cucumber-based acceptance testing infrastructure for Rust-as-Spec platform. It includes:

- **Step definitions**: Gherkin step implementations for test scenarios
- **Test world setup**: Isolated test environment with real HTTP router
- **AC coverage tracking**: Validates Acceptance Criteria against executable tests

This crate uses cucumber-rs for BDD testing and validates that platform behavior matches documented requirements.

### What It Owns

| Module | Responsibility |
|--------|----------------|
| `world.rs` | Test world state, HTTP router, temp directory management |
| `coverage_writer.rs` | AC coverage writer that streams test results to JSONL |
| `steps/` | Gherkin step implementations (HTTP, CLI, governance) |

### What It Is Not

- **Not a test runner**: Use `cargo test --test acceptance` to run tests
- **Not a mocking library**: Tests use real components (in-process HTTP router)
- **Not production code**: This crate is for tests only

## Quick Start

### Running Acceptance Tests

```bash
# Run all acceptance tests
cargo test --test acceptance

# Run specific feature file
cargo test --test acceptance -- features/health.feature

# Run scenarios with specific tags
cargo test --test acceptance -- --tags "@smoke"
```

### Writing Scenarios

Create `.feature` files in `tests/features/`:

```gherkin
Feature: Health Endpoint
  As a platform user
  I want to check system health
  So that I can verify the platform is running

  Scenario: Health endpoint returns 200 OK
    Given the platform is running
    When I send a GET request to "/health"
    Then the response status should be 200
    And the response body should contain "OK"
```

### Tagging for AC Coverage

Tag scenarios with AC IDs to enable coverage tracking:

```gherkin
@AC-KERN-001 @smoke
Scenario: Health endpoint returns 200 OK
  Given the platform is running
  When I send a GET request to "/health"
  Then the response status should be 200
```

## Test World

The `World` struct provides isolated test state:

```rust
pub struct World {
    pub app: Router,              // Real HTTP router (in-process, no network)
    pub last_response: Option<Response>,
    pub request_headers: HeaderMap,
    pub _temp_dir: Arc<tempfile::TempDir>,
    pub cli_exit_code: Option<i32>,
    pub cli_stdout: String,
    pub cli_stderr: String,
    pub platform_auth_mode: Option<String>,
    pub platform_auth_token: Option<String>,
}
```

### Key Features

- **Real HTTP router**: Uses `app-http`'s router for in-process testing
- **Isolated temp directory**: Each scenario gets its own isolated workspace copy
- **Per-scenario auth**: Platform auth configuration is isolated between parallel scenarios
- **Git worktree support**: RAII guard for temporary worktree cleanup

### Using the World

Steps access world state via cucumber's `World` trait:

```rust
#[given(expr = "the platform is running")]
async fn platform_running(world: &mut World) {
    // World already has app router initialized
    // Additional setup here...
}

#[when(expr = "I send a GET request to {path}")]
async fn send_get_request(world: &mut World, path: &str) {
    let response = world.app
        .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .unwrap();

    world.last_response = Some(Response {
        status: response.status().as_u16(),
        body: serde_json::from_slice(&hyper::body::to_bytes(response.into_body()).unwrap()).unwrap(),
        headers: response.headers().clone(),
        raw_body: String::new(),
    });
}
```

## AC Coverage Writer

The `AcCoverageWriter` streams test results to JSONL format:

### Output Format

Each scenario produces one line per AC ID:

```json
{"ac_id":"AC-KERN-001","status":"passed","feature":"specs/features/health.feature","scenario":"Health endpoint returns 200 OK","tags":["smoke","AC-KERN-001"]}
```

### Atomic Write Pattern

The writer uses atomic writes to prevent corruption:

1. Writes go to `{path}.tmp` during execution
2. Each line is flushed immediately for crash safety
3. On successful completion, temp file is renamed to target path
4. If process crashes, temp file is left behind

### Usage

```rust
use acceptance::AcCoverageWriter;

let writer = AcCoverageWriter::new("coverage.jsonl")?;

// Writer automatically handles cucumber events
// Call finalize() after all scenarios complete
writer.finalize()?;
```

## Step Categories

### HTTP Steps

Steps for testing HTTP endpoints:

```gherkin
Given the platform is running
When I send a GET request to "/health"
Then the response status should be 200
And the response body should contain "OK"
```

### CLI Steps

Steps for testing xtask commands:

```gherkin
When I run xtask command "doctor"
Then the command should succeed
And the output should contain "All checks passed"
```

### Governance Steps

Steps for testing governance artifacts:

```gherkin
Given I have a friction entry with id "FRICTION-TOOL-001"
When I list friction entries
Then the response should contain the friction entry
```

## AC Coverage

### Generating Coverage Reports

```bash
# Run acceptance tests with coverage writer
cargo test --test acceptance

# Coverage data written to artifacts/coverage.jsonl

# Generate human-readable report
cargo xtask ac-status
```

### Coverage Status

| Status | Description |
|---------|-------------|
| `passed` | All scenarios with this AC passed |
| `failed` | At least one scenario with this AC failed |
| `unknown` | No scenarios tagged with this AC |

### Filtering Coverage

```bash
# Show only ACs with unknown status (coverage backlog)
cargo xtask ac-coverage --todo

# Filter to kernel ACs only
cargo xtask ac-coverage --todo --must-have

# Show all ACs grouped by requirement
cargo xtask ac-coverage
```

## Test Isolation

The framework provides several isolation mechanisms:

### Per-Scenario Temp Directories

Each scenario gets its own isolated workspace copy:

```rust
impl Default for World {
    fn default() -> Self {
        let temp_dir = Arc::new(tempfile::tempdir().unwrap());

        // Copy workspace specs, config, docs to temp dir
        copy_dir_recursive(&workspace_specs, &temp_specs_dir);

        // World uses isolated paths
        Self { _temp_dir: temp_dir, ... }
    }
}
```

### Per-Scenario Auth Configuration

Platform auth is isolated between parallel scenarios:

```rust
impl World {
    pub fn reload_app(&mut self) {
        // Use EnvVarGuard to safely set env vars
        let guard = EnvVarGuard::new(&["PLATFORM_AUTH_MODE", "PLATFORM_AUTH_TOKEN"]);

        match self.platform_auth_mode.as_deref() {
            Some(mode) => guard.set("PLATFORM_AUTH_MODE", mode),
            None => guard.remove("PLATFORM_AUTH_MODE"),
        }

        // Rebuild app with isolated auth config
        self.app = build_app_with_auth(...);

        // Guard drops here, restoring original env var state
    }
}
```

### Git Worktree Cleanup

RAII guard for temporary worktree cleanup:

```rust
pub struct TempWorktree {
    repo_root: PathBuf,
    worktree_path: PathBuf,
}

impl Drop for TempWorktree {
    fn drop(&mut self) {
        // Remove worktree and prune stale metadata
        Command::new("git")
            .args(["worktree", "remove", "--force", &self.worktree_path])
            .status();

        Command::new("git")
            .args(["worktree", "prune", "--expire", "now"])
            .status();
    }
}
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                  Test Execution                      │
│  ┌────────────────────────────────────────────┐       │
│  │  cucumber-rs (test runner)         │       │
│  │  ┌────────────────────────────────┐   │       │
│  │  │  Test Scenarios (.feature) │   │       │
│  │  └──────────┬───────────────────┘   │       │
│  │             │                          │       │
│  │  ┌──────────▼──────────────────┐   │       │
│  │  │  Step Implementations        │   │       │
│  │  │  (this crate)             │   │       │
│  │  │  ┌────────────────────┐      │   │       │
│  │  │  │  World State     │      │   │       │
│  │  │  │  (isolated env)  │      │   │       │
│  │  │  └────────┬──────────┘      │   │       │
│  │  │           │                  │   │       │
│  │  │  ┌────────▼────────────┐   │   │       │
│  │  │  │  Real HTTP Router  │   │   │       │
│  │  │  │  (app-http)      │   │   │       │
│  │  │  └────────────────────┘   │   │       │
│  │  └───────────────────────────────┘   │       │
│  └────────────────────────────────────────────┘       │
│                                                │
│  ┌────────────────────────────────────────────┐       │
│  │  AC Coverage Writer               │       │
│  │  (streams to coverage.jsonl)       │       │
│  └────────────────────────────────────────────┘       │
└─────────────────────────────────────────────────────────────┘
```

## Consumers

This crate is used by:

| Consumer | Usage |
|----------|-------|
| CI/CD | Automated AC coverage validation |
| Developers | Manual BDD testing of platform behavior |
| `xtask` | AC coverage reporting via `ac-status` command |

## See Also

- [`app-http/README.md`](../app-http/README.md) - HTTP router used in tests
- [`ac-kernel/README.md`](../ac-kernel/README.md) - AC coverage types
- [`testing/README.md`](../testing/README.md) - Test utilities used for isolation
- [`docs/how-to/write-bdd-scenarios.md`](../../docs/how-to/write-bdd-scenarios.md) - Guide for writing scenarios
