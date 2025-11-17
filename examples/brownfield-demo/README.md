# Brownfield Demo: Rust IaC Integration

This example demonstrates how to integrate `rust_iac_xtask_core` into an existing (brownfield) Rust project. It shows the end-to-end workflow for adding Infrastructure-as-Code governance tooling to a project that already has a codebase and tests.

## Overview

This demo contains:

- **`server/`**: A simple HTTP server (existing brownfield codebase)
  - Basic CRUD operations for items
  - Existing tests
  - No Rust IaC tooling initially

- **`xtask/`**: A minimal xtask binary that delegates to `rust_iac_xtask_core`
  - Provides `init` command to scaffold Rust IaC structure
  - Provides `selftest` command to verify the setup

- **`rust_iac_xtask_core`**: Shared library (located in `../../rust_iac_xtask_core/`)
  - Core functionality for initialization and validation
  - Can be used by multiple projects

## Quick Start

### 1. Clone This Example

If you're using this as a template for your own brownfield project:

```bash
# Copy the example to your project
cp -r examples/brownfield-demo my-brownfield-project
cd my-brownfield-project

# Update Cargo.toml workspace name and dependencies
# Update xtask/Cargo.toml to point to your rust_iac_xtask_core location
```

### 2. Run the Server (Before Adding Rust IaC)

The server is a simple HTTP API demonstrating a typical brownfield project:

```bash
# Run the server
cargo run -p server

# In another terminal, test the endpoints
curl http://127.0.0.1:3000/health
curl http://127.0.0.1:3000/items
curl -X POST http://127.0.0.1:3000/items \
  -H "Content-Type: application/json" \
  -d '{"name": "New Item", "description": "Test item"}'

# Run existing tests
cargo test -p server
```

### 3. Initialize Rust IaC Structure

Add Rust IaC governance tooling to your brownfield project:

```bash
# Initialize in brownfield mode
cargo run -p xtask -- init --mode=brownfield

# This creates:
# - RUST_IAC.toml (project configuration)
# - specs/ directory with spec_ledger.yaml
# - policy/ directory with example.rego
# - .llm/ directory with contextpack.yaml
```

**What to expect:**

```
=> Initializing Rust IaC structure in brownfield mode...
  Setting up brownfield project structure...
  ✓ Created RUST_IAC.toml
  ✓ Created specs/spec_ledger.yaml
  ✓ Created policy/example.rego
  ✓ Created .llm/contextpack.yaml
  ✓ Created directory structure
✓ Rust IaC initialization complete!
```

### 4. Run Self-Test

Verify that the Rust IaC structure is set up correctly:

```bash
cargo run -p xtask -- selftest
```

**What to expect:**

```
=> Running Rust IaC self-test...
  ✓ Configuration file exists
  ✓ Specifications directory exists
  ✓ Policy directory exists
  ✓ LLM context directory exists
✓ Configuration is valid
  ✓ Specification ledger exists
✓ All self-tests passed!
```

### 5. Review Generated Files

After initialization, examine the generated structure:

#### `RUST_IAC.toml`

Project configuration file:

```toml
[project]
name = "rust-iac-project"
version = "0.1.0"
mode = "brownfield"

[specs]
ledger = "specs/spec_ledger.yaml"
features_dir = "specs/features"

[policy]
dir = "policy"
tests_dir = "policy/tests"

[llm]
contextpack = ".llm/contextpack.yaml"
```

#### `specs/spec_ledger.yaml`

Tracks relationships between user stories, requirements, and acceptance criteria:

```yaml
user_stories: []
  # - id: US-001
  #   title: Example User Story
  #   description: As a user, I want to...
  #   acceptance_criteria:
  #     - AC-001

requirements: []
  # - id: REQ-001
  #   title: Example Requirement
  #   description: System shall...
  #   user_story: US-001
  #   priority: high

acceptance_criteria: []
  # - id: AC-001
  #   title: Example Acceptance Criterion
  #   description: When... Then...
  #   requirement: REQ-001
  #   priority: must-have
```

#### `policy/example.rego`

Sample Rego policy template:

```rego
package policies.example

deny[msg] {
    false  # Placeholder
    msg := "Example deny message"
}

allow {
    true
}
```

#### `.llm/contextpack.yaml`

Defines task-specific context bundles for LLM assistance:

```yaml
tasks:
  - name: selftest
    description: "Context for running self-tests"
    includes:
      - "specs/**/*.yaml"
      - "specs/**/*.feature"
      - "policy/**/*.rego"
      - "RUST_IAC.toml"
```

## Project Structure

After initialization, your project structure will look like:

```
brownfield-demo/
├── Cargo.toml              # Workspace manifest
├── README.md               # This file
├── RUST_IAC.toml          # Rust IaC configuration (generated)
│
├── server/                 # Your existing brownfield codebase
│   ├── Cargo.toml
│   └── src/
│       └── main.rs         # Simple HTTP server
│
├── xtask/                  # Xtask automation
│   ├── Cargo.toml
│   └── src/
│       └── main.rs         # Delegates to rust_iac_xtask_core
│
├── specs/                  # Specifications (generated)
│   ├── spec_ledger.yaml    # Requirements tracking
│   └── features/           # BDD feature files (empty initially)
│
├── policy/                 # Policies (generated)
│   ├── example.rego        # Sample policy
│   └── tests/              # Policy tests (empty initially)
│
└── .llm/                   # LLM context (generated)
    └── contextpack.yaml    # Context pack definitions
```

## How It Works

### Xtask Delegation Pattern

The `xtask/src/main.rs` is minimal and delegates to `rust_iac_xtask_core`:

```rust
use anyhow::Result;
use clap::{Parser, Subcommand};
use rust_iac_xtask_core::{commands, InitMode};

#[derive(Parser)]
#[command(name = "xtask")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(long, default_value = "brownfield")]
        mode: String,
    },
    Selftest,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { mode } => {
            let init_mode: InitMode = mode.parse()?;
            commands::init::init(init_mode, None)?;
        }
        Commands::Selftest => {
            commands::selftest::selftest(None)?;
        }
    }

    Ok(())
}
```

This pattern allows:
- **Shared core logic** in `rust_iac_xtask_core`
- **Project-specific customization** in your `xtask/` binary
- **Easy updates** by updating the core library version

### Path Dependency

The `xtask/Cargo.toml` uses a path dependency for development:

```toml
[dependencies]
rust_iac_xtask_core = { path = "../../../rust_iac_xtask_core" }
```

For production use, you would:
1. Publish `rust_iac_xtask_core` to crates.io
2. Update to: `rust_iac_xtask_core = "0.1.0"`

Or use a git dependency:

```toml
[dependencies]
rust_iac_xtask_core = { git = "https://github.com/yourorg/rust-template", tag = "v0.1.0" }
```

## Next Steps

After successfully initializing and running selftest:

### 1. Add Acceptance Criteria

Define your first user story and acceptance criteria in `specs/spec_ledger.yaml`:

```yaml
user_stories:
  - id: US-001
    title: List Items
    description: As a user, I want to list all items
    acceptance_criteria:
      - AC-001

requirements:
  - id: REQ-001
    title: Items Endpoint
    description: System shall provide GET /items endpoint
    user_story: US-001
    priority: high

acceptance_criteria:
  - id: AC-001
    title: List Items Returns JSON
    description: GET /items returns JSON array of items
    requirement: REQ-001
    priority: must-have
```

### 2. Create BDD Scenarios

Create `specs/features/items.feature`:

```gherkin
Feature: Item Management
  As a user
  I want to manage items
  So I can track my inventory

  @AC-001
  Scenario: List all items
    Given the server is running
    When I send GET to "/items"
    Then the response status is 200
    And the response is a JSON array
```

### 3. Add Policies

Create governance policies in `policy/`:

```rego
# policy/api_standards.rego
package policies.api_standards

deny[msg] {
    endpoint := input.endpoints[_]
    not startswith(endpoint.path, "/api/v1")
    msg := sprintf("Endpoint %v must start with /api/v1", [endpoint.path])
}
```

### 4. Extend Xtask

Add more commands to `xtask/src/main.rs`:

```rust
enum Commands {
    Init { mode: String },
    Selftest,
    // Add your custom commands
    Bdd,           // Run BDD tests
    PolicyTest,    // Run policy tests
    Deploy { env: String },
}
```

## Friction Points & Lessons Learned

### What Works Well

- **Minimal integration**: Only need to add `xtask/` directory
- **Non-invasive**: Doesn't modify existing code
- **Clear separation**: Specs and policies in dedicated directories
- **Testable**: Can verify structure with `selftest`

### Potential Friction

- **Path dependencies**: Need to manage relative paths during development
- **Workspace configuration**: Need to exclude from parent workspace
- **Initial setup**: Need to understand the directory structure

### Recommended Workflow

1. Start with a working brownfield project
2. Add minimal `xtask/` directory
3. Run `init --mode=brownfield`
4. Verify with `selftest`
5. Incrementally add specs and policies
6. Extend xtask commands as needed

## Testing This Demo

Run the full test suite:

```bash
# Build everything
cargo build

# Run server tests
cargo test -p server

# Run core library tests
cd ../../../rust_iac_xtask_core
cargo test
cd -

# Initialize (if not already done)
cargo run -p xtask -- init --mode=brownfield

# Run selftest
cargo run -p xtask -- selftest

# Start the server
cargo run -p server
```

## Using This in Your Project

To use this pattern in your own brownfield project:

### Option 1: Copy the Pattern

```bash
# 1. Copy xtask directory to your project
cp -r examples/brownfield-demo/xtask /path/to/your/project/

# 2. Add xtask to your workspace Cargo.toml
# members = ["your-crate", "xtask"]

# 3. Update rust_iac_xtask_core dependency
# - Either use path dependency (development)
# - Or use git/crates.io dependency (production)

# 4. Run init
cd /path/to/your/project
cargo run -p xtask -- init --mode=brownfield

# 5. Verify
cargo run -p xtask -- selftest
```

### Option 2: Use as Git Submodule

```bash
# Add as submodule (if using monorepo)
git submodule add https://github.com/yourorg/rust-template vendor/rust-template

# Reference rust_iac_xtask_core from submodule
# rust_iac_xtask_core = { path = "vendor/rust-template/rust_iac_xtask_core" }
```

### Option 3: Fork and Customize

```bash
# Fork the template repository
# Clone your fork
# Modify rust_iac_xtask_core for your organization's needs
# Publish to internal registry or use as git dependency
```

## Comparison: Brownfield vs Greenfield

| Aspect | Brownfield (This Demo) | Greenfield |
|--------|------------------------|------------|
| **Starting Point** | Existing codebase | New project |
| **Init Mode** | `--mode=brownfield` | `--mode=greenfield` |
| **Generated Structure** | Minimal (specs, policy, .llm) | Full scaffold (may include app skeleton) |
| **Existing Tests** | Preserves all existing tests | Generates test templates |
| **Migration Path** | Incremental adoption | Full structure from start |
| **Risk** | Low (non-invasive) | Low (clean slate) |

## Troubleshooting

### Issue: Workspace conflicts

**Error**: `current package believes it's in a workspace when it's not`

**Solution**: Add to parent `Cargo.toml`:

```toml
[workspace]
exclude = ["examples/brownfield-demo"]
```

### Issue: Path dependency not found

**Error**: `could not find rust_iac_xtask_core`

**Solution**: Verify relative path in `xtask/Cargo.toml`:

```toml
# From examples/brownfield-demo/xtask/Cargo.toml
rust_iac_xtask_core = { path = "../../../rust_iac_xtask_core" }
```

### Issue: Init already run

**Warning**: Files already exist, skipping

**Solution**: This is expected behavior. Init is idempotent and won't overwrite existing files.

## Related Documentation

- [rust_iac_xtask_core API](../../../rust_iac_xtask_core/README.md) (if exists)
- [Main Template README](../../README.md)
- [Pilot Project Plan](../../docs/PILOT-PROJECT-PLAN.md)

## License

This example is part of the Rust Template project and is dual-licensed under Apache-2.0 OR MIT.
