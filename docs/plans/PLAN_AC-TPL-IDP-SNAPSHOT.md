# Plan: AC-TPL-IDP-SNAPSHOT - idp-snapshot Command

**Date:** 2025-12-02
**Status:** Ready for Implementation
**Related ACs:** AC-TPL-IDP-SNAPSHOT, AC-TPL-IDP-SNAPSHOT-VALID-JSON

## Scope

**Crates/modules:**
- `crates/xtask/src/commands/idp_snapshot.rs` (new file) - Core command implementation
- `crates/xtask/src/commands/mod.rs` - Register new command
- `crates/xtask/src/main.rs` - Add CLI subcommand
- `specs/features/xtask_devex.feature` - BDD scenarios for idp-snapshot
- `crates/acceptance/src/steps/xtask_devex.rs` - BDD step implementations

**Feature:** New xtask command for IDP consumption

## Goals

1. Implement `cargo xtask idp-snapshot` command that emits stable JSON
2. Include governance health, AC coverage, documentation metrics, and task hints
3. Provide machine-readable snapshot suitable for IDP tile/dashboard consumption
4. Validate JSON output structure and completeness via BDD tests

## Implementation Steps

1. **Create idp_snapshot.rs command module**
   - Define `IdpSnapshot` struct with required fields:
     - `timestamp`: ISO 8601 formatted timestamp
     - `template_version`: from spec_ledger.yaml metadata
     - `service_id`: from service_metadata.yaml
     - `governance_health`: nested object with status, ac_coverage
     - `documentation_metrics`: nested object with doc counts
     - `task_hints`: array of hints for pending/in_progress tasks
   - Implement `run()` function to gather data from platform APIs
   - Add `--output <file>` flag to write JSON to file (optional, defaults to stdout)

2. **Integrate with existing platform APIs**
   - Reuse `spec-runtime` crate APIs:
     - `governance_repo.load_spec_ledger()` for template version
     - `governance_repo.load_service_metadata()` for service_id
     - `governance_repo.compute_ac_status()` for AC coverage
     - `load_tasks()` and `generate_hints()` for task hints
   - Reuse doc_index parsing for documentation metrics
   - Aggregate into IdpSnapshot struct

3. **Add JSON serialization**
   - Derive `Serialize` for IdpSnapshot and nested structs
   - Use `serde_json::to_string_pretty()` for human-readable output
   - Ensure stable field ordering (use `#[serde(skip_serializing_if = "Option::is_none")]` for optional fields)

4. **Register command in xtask CLI**
   - Add `IdpSnapshot` variant to Commands enum in `main.rs`
   - Wire up command execution in match statement
   - Add help text: "Generate IDP-consumable governance snapshot (JSON)"

5. **Write BDD scenarios** (`specs/features/xtask_devex.feature`)
   - Scenario 1: "idp-snapshot emits valid JSON with all required fields"
     - Tag: `@AC-TPL-IDP-SNAPSHOT`
     - Steps: run command, verify JSON parseable, check top-level keys
   - Scenario 2: "idp-snapshot output matches schema"
     - Tag: `@AC-TPL-IDP-SNAPSHOT-VALID-JSON`
     - Steps: validate nested object structure, required fields present

6. **Implement BDD steps** (`crates/acceptance/src/steps/xtask_devex.rs`)
   - Add step: `When I run "cargo xtask idp-snapshot"`
   - Add step: `Then the output should be valid JSON`
   - Add step: `And the JSON should have field "timestamp"`
   - Add step: `And the JSON should have field "template_version"`
   - Add step: `And the JSON should have field "governance_health"`
   - Reuse existing JSON validation helpers

## Verification Commands

```bash
# Run targeted BDD tests
CUCUMBER_TAG_EXPRESSION="@AC-TPL-IDP-SNAPSHOT" cargo test -p acceptance --test acceptance

# Test command manually
cargo xtask idp-snapshot | jq .

# Verify JSON structure
cargo xtask idp-snapshot | jq 'keys | sort'

# Verify AC status
cargo xtask ac-status | grep AC-TPL-IDP-SNAPSHOT

# Full validation
cargo xtask selftest
```

## Definition of Done

- [ ] `idp_snapshot.rs` command module created with IdpSnapshot struct
- [ ] Command registered in xtask main.rs CLI
- [ ] JSON output includes all required fields (timestamp, template_version, service_id, governance_health, documentation_metrics, task_hints)
- [ ] BDD scenarios written and tagged @AC-TPL-IDP-SNAPSHOT
- [ ] BDD steps implemented in xtask_devex.rs
- [ ] Both AC-TPL-IDP-SNAPSHOT BDD tests pass
- [ ] `cargo xtask ac-status` shows AC-TPL-IDP-SNAPSHOT as PASS
- [ ] Manual test: `cargo xtask idp-snapshot | jq .` produces valid JSON
- [ ] No other ACs flip to FAIL
