# rust_iac_xtask_core

Core library for Rust Infrastructure-as-Code xtask automation tooling.

## Overview

`rust_iac_xtask_core` provides reusable commands and utilities for managing Rust IaC projects. It implements the core functionality for:

- **Initialization**: Scaffolding project structure for brownfield and greenfield projects
- **Validation**: Self-testing to verify project setup
- **Configuration**: TOML-based project configuration
- **Governance**: Specs, policies, and LLM context management

This library is designed to be used by xtask binaries in individual projects, providing a consistent experience across all Rust IaC projects.

## Features

- **Brownfield Support**: Add governance to existing projects non-invasively
- **Greenfield Support**: Full scaffolding for new projects
- **Configuration Management**: Type-safe TOML configuration
- **Self-Validation**: Built-in self-test to verify project structure
- **Idempotent Operations**: Safe to run init multiple times

## Installation

Add to your xtask's `Cargo.toml`:

```toml
[dependencies]
rust_iac_xtask_core = { path = "../path/to/rust_iac_xtask_core" }
# or from git:
# rust_iac_xtask_core = { git = "https://github.com/yourorg/rust-template" }
# or from crates.io (when published):
# rust_iac_xtask_core = "0.1.0"
```

## Usage

### Basic Xtask Integration

Create a minimal `xtask/src/main.rs` that delegates to the core library:

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

### Initialize a Brownfield Project

```bash
cargo run -p xtask -- init --mode=brownfield
```

Creates:
- `RUST_IAC.toml` - Project configuration
- `specs/spec_ledger.yaml` - Requirements tracking
- `specs/features/` - BDD feature files directory
- `policy/` - Rego policy directory
- `policy/tests/` - Policy tests directory
- `.llm/contextpack.yaml` - LLM context definitions

### Run Self-Test

```bash
cargo run -p xtask -- selftest
```

Validates:
- ✓ Configuration file exists and is valid TOML
- ✓ Specifications directory exists
- ✓ Policy directory exists
- ✓ LLM context directory exists
- ✓ Specification ledger exists

## API

### Commands

#### `init(mode: InitMode, project_root: Option<PathBuf>) -> Result<()>`

Initialize a Rust IaC project structure.

**Parameters:**
- `mode`: Either `InitMode::Brownfield` or `InitMode::Greenfield`
- `project_root`: Optional project root path (defaults to current directory)

**Returns:** `Result<()>` - Success or error

**Example:**

```rust
use rust_iac_xtask_core::{init, InitMode};

// Initialize brownfield project in current directory
init(InitMode::Brownfield, None)?;

// Initialize greenfield project in specific directory
init(InitMode::Greenfield, Some("/path/to/project".into()))?;
```

#### `selftest(project_root: Option<PathBuf>) -> Result<()>`

Run self-test to verify project structure.

**Parameters:**
- `project_root`: Optional project root path (defaults to current directory)

**Returns:** `Result<()>` - Success or error

**Example:**

```rust
use rust_iac_xtask_core::selftest;

// Run self-test in current directory
selftest(None)?;

// Run self-test in specific directory
selftest(Some("/path/to/project".into()))?;
```

### Configuration

#### `RustIacConfig`

Main configuration structure loaded from `RUST_IAC.toml`:

```rust
pub struct RustIacConfig {
    pub project: ProjectConfig,
    pub specs: SpecsConfig,
    pub policy: PolicyConfig,
    pub llm: LlmConfig,
}
```

**Default Configuration:**

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

## Project Structure

After initialization, your project will have:

```
your-project/
├── RUST_IAC.toml          # Configuration
├── specs/
│   ├── spec_ledger.yaml   # Requirements ledger
│   └── features/          # BDD scenarios
├── policy/
│   ├── example.rego       # Sample policy
│   └── tests/             # Policy tests
└── .llm/
    └── contextpack.yaml   # LLM context packs
```

## Development

### Building

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

The test suite includes:
- Unit tests for init and selftest
- Integration tests with temporary directories
- Configuration parsing tests

### Adding New Commands

To add a new command to the core library:

1. Add the implementation to `src/lib.rs`:

```rust
pub fn my_new_command(project_root: Option<PathBuf>) -> Result<()> {
    // Implementation
    Ok(())
}
```

2. Export from `src/commands.rs`:

```rust
pub mod my_command {
    pub use crate::my_new_command;
}
```

3. Projects can then use it in their xtask:

```rust
Commands::MyCommand => {
    commands::my_command::my_new_command(None)?;
}
```

## Examples

See the [brownfield-demo](../examples/brownfield-demo/) for a complete working example.

## Brownfield vs Greenfield

| Feature | Brownfield | Greenfield |
|---------|------------|------------|
| **Use Case** | Existing projects | New projects |
| **Generated Files** | Minimal governance structure | Full scaffold |
| **Invasiveness** | Non-invasive | Full structure |
| **Migration** | Incremental adoption | Clean slate |
| **Risk** | Low | Low |

## Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_init_brownfield_creates_structure

# Run with output
cargo test -- --nocapture
```

## License

Dual-licensed under Apache-2.0 OR MIT.
