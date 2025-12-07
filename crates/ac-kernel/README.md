# ac-kernel

Core AC governance logic for the Rust-as-Spec platform.

## What It Is

`ac-kernel` is the shared library that owns the AC governance data model and core logic. It is designed to be:

- **Standalone**: No CLI, no colors, no Nix shell assumptions
- **Portable**: Pure Rust with minimal dependencies
- **Authoritative**: Single source of truth for AC-related types

### What It Owns

| Module | Responsibility |
|--------|----------------|
| `model` | Core types: `AcStatus`, `AcSource`, `Scenario`, `TestMapping` |
| `json` | JSON output schemas: `AcStatusJson`, `Ac`, `AcCategoryStats` |
| `ledger` | Ledger parsing: `parse_ledger_with_metadata`, `AcMetadata`, `AcDetails` |
| `coverage` | Coverage parsing: `AcCoverageRecord` from `coverage.jsonl` |
| `history` | History analysis: `AcHistoryReport`, `SnapshotMetric`, `SnapshotDelta` |
| `layout` | Path conventions: `SpecLayout` for locating AC artifacts |

### What It Is Not

- **No CLI**: The library provides data and logic; `xtask` provides the command-line interface
- **No repo discovery**: `spec_root()` and working directory logic stays in `xtask`
- **No CI opinions**: SLO thresholds and enforcement policies live in workflows
- **No formatting**: Colors, spinners, and pretty-printing stay in `xtask`

## Quick Start

Use the `AcKernel` facade for most use cases:

```rust
use ac_kernel::{AcKernel, SpecLayout};
use std::path::Path;

// Create a layout pointing to your repo root
let layout = SpecLayout::for_repo_root(Path::new("."));
let kernel = AcKernel::new(layout);

// Load current AC status (from ledger + coverage)
let acs = kernel.load_status()?;
for (id, ac) in &acs {
    println!("{}: {:?}", id, ac.status);
}

// Or get JSON-ready output directly
let json = kernel.load_status_json()?;
println!("Coverage: {:.1}%", json.coverage_percent);

// Load historical trends
let history = kernel.load_history()?;
println!("Analyzed {} snapshots", history.snapshot_count);
```

### Using Individual Modules

For lower-level access, use the modules directly:

```rust
use ac_kernel::{parse_ledger_with_metadata, parse_ac_coverage, AcStatus};

// Parse the ledger
let metadata = parse_ledger_with_metadata(Path::new("specs/spec_ledger.yaml"))?;

// Parse coverage results
let (scenarios, results) = parse_ac_coverage(Path::new("target/ac/coverage.jsonl"))?;

// Check specific AC status
if let Some(status) = results.get("AC-KERN-001") {
    match status {
        AcStatus::Pass => println!("AC-KERN-001 is passing"),
        AcStatus::Fail => println!("AC-KERN-001 is failing"),
        AcStatus::Unknown => println!("AC-KERN-001 has no test coverage"),
    }
}
```

## Consumers

This crate is used by:

| Crate | Usage |
|-------|-------|
| `xtask` | `ac-status`, `ac-coverage`, `ac-history` commands |
| `acceptance` | Writing `coverage.jsonl` during BDD test execution |

## Stability

The JSON output schemas are versioned and considered public API:

| Output | Schema Version | Constant |
|--------|----------------|----------|
| `ac-status --json` | 2.0 | `AC_STATUS_SCHEMA_VERSION` |
| `ac-history --json` | 1.0 | `AC_HISTORY_SCHEMA_VERSION` |

**Breaking changes** to these schemas require bumping the version constant. Shape-lock tests in the crate enforce this contract:

- `ac_status_json_shape_is_stable` (in `json.rs`)
- `ac_history_json_shape_is_stable` (in `history.rs`)

### Schema Evolution

- **v1.0 → v2.0 (ac-status)**: Changed from prefix-based categorization (`kernel_` vs `template_`) to `must_have_ac` metadata-based categorization
- **v1.0 (ac-history)**: Initial schema with snapshots, deltas, and skipped files

## Default File Layout

`SpecLayout::for_repo_root()` expects this structure:

```
<repo_root>/
├── specs/
│   ├── spec_ledger.yaml     # The spec ledger
│   └── features/            # BDD feature files
├── target/
│   └── ac/
│       └── coverage.jsonl   # BDD coverage output
└── artifacts/
    └── ac-status/           # Historical ac-status snapshots
```

Custom layouts can be created by constructing `SpecLayout` directly:

```rust
use ac_kernel::SpecLayout;
use std::path::PathBuf;

let layout = SpecLayout {
    ledger: PathBuf::from("/custom/path/ledger.yaml"),
    coverage_file: PathBuf::from("/custom/path/coverage.jsonl"),
    history_dir: PathBuf::from("/custom/path/history"),
    features_dir: PathBuf::from("/custom/path/features"),
};
```

## See Also

- `docs/reference/ac-status-json-schema.md` - Full JSON schema documentation
- `docs/reference/xtask-commands.md` - CLI commands using this library
