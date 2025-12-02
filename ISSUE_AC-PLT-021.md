# Fix AC-PLT-021: service-init command

## AC Definition

From `specs/spec_ledger.yaml` (lines 135-144):

```yaml
AC-PLT-021:
  text: "`cargo xtask service-init` updates service_metadata.yaml,
         README, and CLAUDE.md with a new service ID, name, and
         description, and `/platform/status` reflects the new identity."
  adr: ADR-0022
  tags: [kernel]
  must_have_ac: true
  tests:
    - { type: integration, tag: "@AC-PLT-021", file: "specs/features/xtask_devex.feature" }
```

## Current Status

**Implementation: 80% complete**

### ✅ What Works
- Command exists: `cargo xtask service-init`
- Updates `service_metadata.yaml` with service_id, display_name, description
- Updates `README.md` with new service name and description
- ID validation (kebab-case regex)
- Idempotent operation (safe to run multiple times)
- Comprehensive unit tests (6 test functions)
- BDD test infrastructure (5 scenarios)

### ❌ What's Missing

1. **CLAUDE.md update NOT implemented** (spec requirement)
   - Spec says: "updates README, and CLAUDE.md"
   - Current state: NO CODE implements CLAUDE.md updates
   - File: `crates/xtask/src/commands/service_init.rs` missing `update_claude()` function

2. **Command NOT registered in devex_flows.yaml** (governance contract violation)
   - All other commands are listed in `specs/devex_flows.yaml`
   - `service-init` is missing from the flows spec
   - Selftest will flag this as "devex contract not satisfied"

## BDD Test Scenarios

From `specs/features/xtask_devex.feature` (lines 567-612):

1. ✅ Basic branding update (metadata + README)
2. ✅ Idempotency test (second run returns "No changes needed")
3. ✅ ID format validation (rejects non-kebab-case IDs)
4. ✅ Service identity update for new services
5. ⚠️ Platform status reflection (will fail on CLAUDE.md assertion)

## Files Requiring Changes

### Priority 1 (Required)

1. **`crates/xtask/src/commands/service_init.rs`**
   - Add `update_claude()` function (similar to `update_readme()` pattern)
   - Extract and replace service name in CLAUDE.md
   - Handle version suffix preservation
   - Call it in `run()` orchestration (around line 51)

2. **`specs/devex_flows.yaml`**
   - Add `service-init:` entry under `commands:` section
   - Example:
     ```yaml
     service-init:
       category: service_setup
       summary: "Initialize service branding (ID, name, description)"
       required: true
       docs:
         readme_table: true
         contributing_flow: false
         claude_golden_path: true
     ```

3. **`crates/acceptance/src/steps/xtask_devex.rs`**
   - Add CLAUDE.md backup tracking in service-init setup steps
   - Add cleanup for CLAUDE.md restoration (around line 2144 restore logic)

4. **`specs/features/xtask_devex.feature`**
   - Add CLAUDE.md assertion to scenarios (around line 575-577):
     ```gherkin
     And "CLAUDE.md" should contain "My New Service"
     ```

## Verification Commands

```bash
# Run targeted BDD tests
cargo xtask bdd --tags @AC-PLT-021

# Run AC-specific tests
cargo xtask ac-tests AC-PLT-021

# Verify AC status
cargo xtask ac-status | grep AC-PLT-021

# Full governance check
cargo xtask selftest
```

## Acceptance Criteria

- [ ] `update_claude()` function implemented in service_init.rs
- [ ] `update_claude()` called in `run()` function
- [ ] CLAUDE.md backup/restore in BDD steps
- [ ] `service-init` registered in devex_flows.yaml
- [ ] BDD scenario includes CLAUDE.md assertion
- [ ] All 5 BDD scenarios pass
- [ ] `cargo xtask selftest` passes AC-PLT-021

## Related Files

- Implementation: `crates/xtask/src/commands/service_init.rs` (336 lines)
- Spec: `specs/spec_ledger.yaml` (lines 135-144)
- BDD: `specs/features/xtask_devex.feature` (lines 567-612)
- Steps: `crates/acceptance/src/steps/xtask_devex.rs` (lines 2050-2300)
- Flows: `specs/devex_flows.yaml` (392 lines, missing service-init)
- ADR: `docs/adr/0022-platform-metadata-and-test-isolation.md`

## Labels

`kernel`, `ac-fail`, `must-fix`, `xtask`, `governance`
