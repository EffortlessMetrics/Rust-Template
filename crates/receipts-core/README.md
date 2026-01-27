# receipts-core

Receipt schema types and serialization rules.

## Purpose

This crate provides core receipt types for governance and CI/CD evidence. It focuses on types and serialization - IO should be handled elsewhere.

## Design Philosophy

Receipts should be:
- **Structured**: Use typed fields for all data
- **Serializable**: Support JSON/YAML for evidence storage
- **Validatable**: Include validation rules where applicable
- **Minimal**: No IO, no HTTP, just types and validation

## Receipt Types

### Gate Receipt

Captures gate execution results (fmt, clippy, tests, selftest).

```rust
use receipts_core::{GateReceipt, GateStatus, GateResult};

let receipt = GateReceipt {
    schema_version: "1.0".to_string(),
    run_id: "2026-01-07T14:32Z-pr209".to_string(),
    commit: "abc123".to_string(),
    started_at: "2026-01-07T14:32:00Z".parse().unwrap(),
    finished_at: "2026-01-07T14:35:42Z".parse().unwrap(),
    gates: vec![/* gate results */],
    overall_status: GateStatus::Pass,
    repo_version: "v3.3.14".to_string(),
    environment: Environment {
        os: "linux".to_string(),
        rust_version: "1.83.0".to_string(),
        nix_shell: true,
    },
    pr: Some(209),
};
```

### Economics Receipt

Tracks developer time, compute spend, iteration counts, and value delivered.

```rust
use receipts_core::{EconomicsReceipt, Confidence, DevLtMinutes};

let receipt = EconomicsReceipt {
    schema_version: "1.0".to_string(),
    pr: 209,
    run_id: "2026-01-07T14:32Z-pr209".to_string(),
    devlt_minutes: DevLtMinutes {
        author: Some(25),
        author_confidence: Confidence::Estimated,
        ..Default::default()
    },
    compute: ComputeSpend::default(),
    iterations: Iterations::default(),
    value_delivered: ValueDelivered::default(),
};
```

### Dossier

Structured PR analysis for casebook generation.

```rust
use receipts_core::{Dossier, Scope, ExhibitScore};

let dossier = Dossier {
    schema_version: "1.0".to_string(),
    pr: 209,
    title: "Add pagination error contract BDD scenarios".to_string(),
    merged_at: "2026-01-07T15:00:00Z".parse().unwrap(),
    scope: Scope::default(),
    intent: Intent::default(),
    findings: Vec::new(),
    errata: Vec::new(),
    exhibit_score: ExhibitScore::default(),
    factory_delta: FactoryDelta::default(),
};
```

### Timeline Receipt

Captures PR evolution, friction zones, and convergence patterns.

```rust
use receipts_core::{TimelineReceipt, Topology};

let receipt = TimelineReceipt {
    schema_version: "1.0".to_string(),
    pr: Some(123),
    run_id: "test-run".to_string(),
    wall_clock: WallClock {
        first_commit: "2026-01-07T10:00:00Z".parse().unwrap(),
        last_commit: "2026-01-07T14:00:00Z".parse().unwrap(),
        ..Default::default()
    },
    sessions: Vec::new(),
    friction_zones: Vec::new(),
    oscillations: Vec::new(),
    convergence: None,
    topology: Topology::Linear,
    ..Default::default()
};
```

### Quality Receipt

Tracks contract changes, boundary integrity, verification depth, and risk indicators.

```rust
use receipts_core::{QualityReceipt, Quality};

let receipt = QualityReceipt {
    schema_version: "1.0".to_string(),
    pr: Some(123),
    run_id: Some("test-run".to_string()),
    quality: Quality::default(),
    meta: None,
};
```

### Telemetry Receipt

Captures normalized hard probe outputs - tool measurements, change surface, and verification coverage.

```rust
use receipts_core::{TelemetryReceipt, ProbeProfile, ProbeStatus};

let receipt = TelemetryReceipt {
    schema_version: "1.0".to_string(),
    pr: Some(123),
    run_id: "test-run".to_string(),
    profile: Some(ProbeProfile::Full),
    change_surface: ChangeSurface::default(),
    contracts: None,
    safety: None,
    structure: None,
    verification: None,
    probes: vec![ProbeResult {
        name: "cargo-clippy".to_string(),
        version: Some("0.1.0".to_string()),
        status: ProbeStatus::Run,
        reason: None,
        duration_ms: Some(1234),
        artifact_path: None,
    }],
    not_run: vec![],
    meta: None,
};
```

### Meta Provenance

Optional metadata header for receipts with method versioning and evidence pointers.

```rust
use receipts_core::{ReceiptMeta, MetaConfidence};

let meta = ReceiptMeta::builder()
    .method_id("telemetry-v1")
    .method_version(1)
    .analysis_run_id("2026-01-07T14-32-00Z-pr209")
    .input("git_diff")
    .input("git_log")
    .assumption("base branch is origin/main")
    .confidence(MetaConfidence::Medium)
    .evidence("crates/gov-receipts/src/lib.rs")
    .evidence("63da971")
    .build();
```

## Validation

All receipt types support validation:

```rust
use receipts_core::{Receipt, ReceiptError, validate_schema_version};

// Validate schema version
let result = validate_schema_version("1.0", "1.0");
assert!(result.is_ok());

// Serialize to JSON
let json = receipt.to_json()?;

// Parse from JSON
let parsed = Receipt::from_json(&json)?;
```

## Error Handling

```rust
use receipts_core::{ReceiptError, ReceiptResult};

fn process_receipt() -> ReceiptResult<()> {
    Err(ReceiptError::InvalidSchemaVersion("2.0".to_string()))
}

// Handle errors
match process_receipt() {
    Ok(_) => println!("Valid"),
    Err(ReceiptError::InvalidSchemaVersion(v)) => println!("Invalid version: {}", v),
    Err(e) => println!("Error: {}", e),
}
```

## License

Apache-2.0 OR MIT
