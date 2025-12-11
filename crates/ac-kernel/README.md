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
| `model` | Core types: `AcStatus`, `AcSource`, `AcEvidence`, `Scenario`, `TestMapping` |
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
│   ├── ac/
│   │   └── coverage.jsonl   # BDD coverage output
│   └── junit/
│       └── acceptance.xml   # JUnit test results
└── artifacts/
    └── ac-status/           # Historical ac-status snapshots
```

### Customizing the Layout

Use `SpecLayoutBuilder` to override individual paths while keeping defaults for the rest:

```rust
use ac_kernel::SpecLayout;
use std::path::Path;

let layout = SpecLayout::builder(Path::new("/my/repo"))
    .with_ledger("/custom/path/ledger.yaml")
    .with_features_dir("/custom/features")
    .build();

// Other paths remain at defaults:
// - coverage_file: /my/repo/target/ac/coverage.jsonl
// - junit_file: /my/repo/target/junit/acceptance.xml
// - history_dir: /my/repo/artifacts/ac-status
```

**Path semantics**: Paths passed to `with_*` methods are used **as-is**. Pass absolute paths, or handle root-joining yourself. The builder does not interpret relative paths.

For full control, construct `SpecLayout` directly:

```rust
use ac_kernel::SpecLayout;
use std::path::PathBuf;

let layout = SpecLayout {
    ledger: PathBuf::from("/custom/path/ledger.yaml"),
    coverage_file: PathBuf::from("/custom/path/coverage.jsonl"),
    junit_file: PathBuf::from("/custom/path/junit.xml"),
    history_dir: PathBuf::from("/custom/path/history"),
    features_dir: PathBuf::from("/custom/path/features"),
};
```

## AC Governance Semantics

### `must_have_ac` Field

The `must_have_ac` field controls whether an AC is part of the **kernel contract** (hard gate) or just **tracked** (soft gate):

| `must_have_ac` | Effect |
|----------------|--------|
| `true` (default) | AC is **mandatory**. Tests must exist and pass for `selftest` to be green. |
| `false` | AC is **tracked** but non-blocking. Shows in `ac-status` / `ac-coverage` but doesn't fail `selftest`. |

This applies at two levels:

- **On a REQ**: Controls whether the requirement must have at least one AC with mapped tests.
- **On an AC**: Controls whether that specific AC's tests gate selftest.

### AC Status Values

Each AC can be in one of these states:

| Status | Meaning |
|--------|---------|
| `Pass` | All mapped tests exist and pass |
| `Fail` | At least one mapped test exists and fails |
| `Unknown` | No test mapping or coverage data found |

### Selftest Behavior

`cargo xtask selftest` only treats ACs with `must_have_ac: true` as mandatory:

- If a kernel AC (`must_have_ac: true`) is `Fail` or `Unknown` → **selftest fails**
- If an optional AC (`must_have_ac: false`) is `Fail` or `Unknown` → **selftest warns but passes**

This allows incremental adoption: add new ACs as `must_have_ac: false`, implement tests, then flip to `true` when ready.

## AcEvidence Model (ADR-0024)

The `AcEvidence` struct provides a unified view of test evidence for an AC:

```rust
use ac_kernel::AcEvidence;

let mut ev = AcEvidence::new("AC-KERN-001", true); // kernel AC
ev.unit_mapped = 2;  // 2 unit tests mapped in spec_ledger.yaml
ev.bdd_passed = 1;   // 1 BDD scenario passed

// Compute status from evidence
match ev.status() {
    AcStatus::Pass => println!("AC is covered"),
    AcStatus::Fail => println!("AC has failing tests"),
    AcStatus::Unknown => println!("AC has no test evidence"),
}
```

### Evidence Sources

| Source | Field | Meaning |
|--------|-------|---------|
| `spec_ledger.yaml` | `unit_mapped` | Unit tests declared for this AC |
| `spec_ledger.yaml` | `bdd_mapped` | BDD/integration tests declared |
| `coverage.jsonl` | `bdd_passed` | BDD scenarios that passed at runtime |
| `coverage.jsonl` | `bdd_failed` | BDD scenarios that failed at runtime |

### Status Classification Rules

The `status()` method follows ADR-0024:

1. **FAIL**: `bdd_failed > 0` (any BDD scenario failed)
2. **PASS**: `bdd_passed > 0 OR unit_mapped > 0` (evidence exists)
3. **UNKNOWN**: No evidence

**Why unit_mapped counts as PASS**: Unit tests are presumed to run because `cargo xtask check` (which runs before the AC coverage gate in selftest) executes all unit tests via `cargo test --workspace`.

### Kernel Coverage Gate

Environment variables control the enforcement level:

| Variable | Effect |
|----------|--------|
| `XTASK_STRICT_AC_COVERAGE=1` | No unknown kernel ACs allowed (budget=0) |
| `KERNEL_UNKNOWN_BUDGET=N` | Allow at most N unknown kernel ACs |
| Neither set | Unlimited unknowns (backward compatible) |

See **ADR-0024** for the full specification of the evidence model and gate semantics.

## See Also

- `docs/reference/ac-status-json-schema.md` - Full JSON schema documentation
- `docs/reference/xtask-commands.md` - CLI commands using this library
