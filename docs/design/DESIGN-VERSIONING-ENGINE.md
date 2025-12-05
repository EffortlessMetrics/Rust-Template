---
id: DESIGN-TPL-VERSIONING-ENGINE-001
title: "Single Versioning Engine for release-prepare"
author: platform-team
doc_type: design_doc
date: 2025-12-01
status: published
stories: [US-TPL-PLT-001]
requirements: [REQ-TPL-VERSIONING-ENGINE]
tags: [platform, release, versioning, devex]
acs: [AC-TPL-VERSION-MANIFEST, AC-TPL-VERSION-DRYRUN, AC-TPL-VERSION-ATOMIC]
adrs: [ADR-0005]
---

# Single Versioning Engine for release-prepare

## Status

**In Progress** - Implementation underway for v3.3.6 kernel. Core components ready: <!-- doclint:disable orphan-version -->
- ✅ Version manifest (`specs/version_manifest.yaml`) - declares all 10+ version-bearing files
- ✅ Versioning module (`crates/xtask/src/commands/versioning.rs`) - schema-aligned structs
- 🔲 Unit tests for engine (TASK-VERS-TEST-001)
- 🔲 Integration with release-prepare (TASK-VERS-INTEG-001)

## Problem Statement

Currently `cargo xtask release-prepare` updates only 4 files, but `docs-check` validates 8+ version-bearing files. This creates manual drift and version misalignment.

**Files updated by release-prepare (4):**
- `specs/spec_ledger.yaml`
- `README.md`
- `CLAUDE.md`
- `CHANGELOG.md`

**Files validated by docs-check but NOT updated (4+):**
- `docs/ROADMAP.md`
- `docs/KERNEL_SNAPSHOT.md`
- `docs/explanation/TEMPLATE-CONTRACTS.md`
- `specs/service_metadata.yaml`
- `specs/doc_index.yaml`
- `docs/feature_status.md`

## Proposed Solution

### 1. Declarative Version Manifest

Create `specs/version_manifest.yaml`:

```yaml
source_of_truth: specs/spec_ledger.yaml
version_path: $.metadata.template_version
date_path: $.metadata.last_updated

targets:
  - file: README.md
    patterns:
      - marker: "# Rust-as-Spec Platform Cell"
        format: "# Rust-as-Spec Platform Cell (v{version})"
      - marker: "Current Template Version"
        format: "**Current Template Version:** v{version}"

  - file: CLAUDE.md
    patterns:
      - marker: "# CLAUDE.md"
        format: "# CLAUDE.md – Rust-as-Spec Platform Cell (v{version})"
      - marker: "Template Version:"
        format: "**Template Version:** v{version}"

  # ... etc for all 8+ files
```

### 2. Unified Versioning Module

```
crates/xtask/src/versioning/
├── mod.rs           # Public API
├── engine.rs        # VersionEngine: read/update/validate
├── manifest.rs      # Load and parse version_manifest.yaml
├── validator.rs     # Shared validation (replaces docs-check duplicates)
└── format.rs        # Pattern matching and replacement
```

### 3. Key Types

```rust
struct VersionInfo {
    current: String,      // "3.3.5" <!-- doclint:disable orphan-version -->
    previous: Option<String>,
    date: String,         // "2025-12-01"
}

struct VersionManifest {
    source_of_truth: PathBuf,
    targets: Vec<VersionTarget>,
}

struct VersionTarget {
    file: PathBuf,
    patterns: Vec<VersionPattern>,
}

impl VersionEngine {
    fn update_all(&self, new_version: &str) -> Result<UpdateReport>;
    fn validate_all(&self) -> Result<ValidationReport>;
    fn dry_run(&self, new_version: &str) -> Result<DryRunReport>;
}
```

### 4. Integration Points

**release-prepare:**
```rust
let engine = VersionEngine::from_manifest("specs/version_manifest.yaml")?;
engine.update_all(&new_version)?;
```

**docs-check:**
```rust
let engine = VersionEngine::from_manifest("specs/version_manifest.yaml")?;
let report = engine.validate_all()?;
if !report.is_aligned() {
    issues += 1;
}
```

## Benefits

1. **Single source of truth** for version locations
2. **DRY** - No duplicated regex patterns between commands
3. **Dry-run support** - Preview changes before applying
4. **Transactional** - All-or-nothing updates with rollback
5. **Extensible** - Add new versioned files via manifest, not code

## Gaps Addressed

| Gap | Solution |
|-----|----------|
| File coverage (4 vs 8+) | Manifest declares all targets |
| Duplicated validation | Shared `VersionValidator` |
| No dry-run | `engine.dry_run()` method |
| No transactional safety | Write to temp, atomic rename |
| v-prefix inconsistency | Manifest specifies format per file |
| Manual date handling | Date from manifest `date_path` |

## Implementation Estimate

- Manifest schema + loader: ~2 hours
- VersionEngine core: ~4 hours
- Migrate release-prepare: ~2 hours
- Migrate docs-check validation: ~2 hours
- Tests: ~2 hours

Total: ~12 hours (1.5 dev days)

## Decision

Implementing for v3.3.6 kernel. This enables manifest-driven releases with dry-run support. <!-- doclint:disable orphan-version -->

## Progress Tracking

| Task | Status |
|------|--------|
| ✅ Create `specs/version_manifest.yaml` | Complete - 10+ files declared |
| ✅ Implement `crates/xtask/src/commands/versioning.rs` | Complete - schema-aligned |
| 🔲 Add unit tests (manifest loading, plan generation) | TASK-VERS-TEST-001 |
| 🔲 Integrate into `release-prepare` command | TASK-VERS-INTEG-001 |
| 🔲 Add `--dry-run` flag to release-prepare | Part of TASK-VERS-INTEG-001 |
| 🔲 Promote versioning ACs | TASK-VERS-RELEASE-001 |

## Related Work

- **AC-TPL-REL-SEMVER**: Version format validation (already enforced)
- **AC-TPL-VERSION-ALIGN**: Cross-file version consistency (this design fixes the gap)
- **docs-check**: Current validation logic to be replaced by `VersionValidator`
- **release-prepare**: Current update logic to be replaced by `VersionEngine`

## Open Questions

1. Should `version_manifest.yaml` support JSONPath for nested YAML updates?
2. Should we validate version ordering (3.3.6 > 3.3.5) at update time? <!-- doclint:disable orphan-version -->
3. Should the manifest support conditional patterns (e.g., only update if marker exists)?
4. Should we add a `--force` flag to override validation failures?

## References

- Current implementation: `crates/xtask/src/commands/release_prepare.rs`
- Validation logic: `crates/xtask/src/commands/docs_check.rs`
- Spec ledger: `specs/spec_ledger.yaml` (source of truth for current version)
