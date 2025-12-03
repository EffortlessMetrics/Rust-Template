# Plan: AC-PLT-021 - service-init Command

**Date:** 2025-12-02
**Status:** Ready for Implementation
**Related ACs:** AC-PLT-021

## Scope

**Files in scope:**
- `crates/xtask/src/commands/service_init.rs` - Add CLAUDE.md update function
- `specs/devex_flows.yaml` - Register service-init command in flows spec
- `crates/acceptance/src/steps/xtask_devex.rs` - Add CLAUDE.md backup/restore in BDD steps
- `specs/features/xtask_devex.feature` - Add CLAUDE.md assertion to test scenarios

**What's already working (80% complete):**
- Command exists: `cargo xtask service-init`
- Updates `service_metadata.yaml` with service_id, display_name, description
- Updates `README.md` with new service name and description
- ID validation (kebab-case regex)
- Idempotent operation (safe to run multiple times)
- Comprehensive unit tests (6 test functions)
- BDD test infrastructure (5 scenarios)

## Goals

1. Complete the AC-PLT-021 implementation by adding CLAUDE.md updates
2. Register service-init command in devex_flows.yaml (governance contract)
3. Update BDD tests to verify CLAUDE.md changes
4. Achieve 100% AC-PLT-021 coverage with all tests passing

## Implementation Steps

1. **Add CLAUDE.md update function** (`crates/xtask/src/commands/service_init.rs`)
   - Create `update_claude()` function similar to existing `update_readme()` pattern
   - Extract and replace service name in CLAUDE.md
   - Preserve version suffix in template name references (e.g., "v3.3.6")
   - Handle edge cases (file missing, multiple matches, etc.)

2. **Integrate CLAUDE.md update into orchestration**
   - Call `update_claude()` in the `run()` function (around line 51)
   - Add to the orchestration sequence: metadata → README → CLAUDE.md
   - Ensure error handling matches existing pattern

3. **Register command in devex_flows.yaml**
   - Add `service-init:` entry under `commands:` section
   - Set category: `onboarding`
   - Set summary: "Initialize service branding (ID, name, description in metadata, README, CLAUDE.md)"
   - Set `required: true`
   - Configure docs visibility: `readme_table: true`, `contributing_flow: false`, `claude_golden_path: true`

4. **Update BDD test infrastructure** (`crates/acceptance/src/steps/xtask_devex.rs`)
   - Add CLAUDE.md backup tracking in service-init setup steps (around line 2050)
   - Add CLAUDE.md cleanup/restoration in teardown logic (around line 2144 restore logic)
   - Ensure CLAUDE.md is restored to original state after tests

5. **Add CLAUDE.md assertions to BDD scenarios** (`specs/features/xtask_devex.feature`)
   - Update scenario "Basic branding update" (around line 575-577)
   - Add assertion: `And "CLAUDE.md" should contain "My New Service"`
   - Ensure all 5 scenarios verify CLAUDE.md updates

## Verification Commands

```bash
# Run targeted BDD tests
CUCUMBER_TAG_EXPRESSION="@AC-PLT-021" cargo test -p acceptance --test acceptance

# Run AC-specific tests (unit + BDD)
cargo xtask ac-tests AC-PLT-021

# Verify AC status
cargo xtask ac-status | grep AC-PLT-021

# Full validation
cargo xtask selftest
```

## Definition of Done

- [ ] `update_claude()` function implemented in service_init.rs
- [ ] `update_claude()` called in `run()` orchestration function
- [ ] CLAUDE.md backup/restore implemented in BDD steps (xtask_devex.rs)
- [ ] `service-init` registered in devex_flows.yaml with correct metadata
- [ ] BDD scenarios include CLAUDE.md assertions (all 5 scenarios)
- [ ] All 5 @AC-PLT-021 BDD scenarios pass
- [ ] `cargo xtask ac-status` shows AC-PLT-021 as PASS
- [ ] `cargo xtask selftest` passes (devex contract satisfied)
- [ ] No other ACs flip to FAIL
