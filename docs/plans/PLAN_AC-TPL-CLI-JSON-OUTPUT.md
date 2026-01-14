# Plan: AC-TPL-CLI-JSON-OUTPUT - JSON Output for Core Commands

**Date:** 2025-12-02
**Status:** Ready for Implementation
**Related ACs:** AC-TPL-CLI-JSON-OUTPUT, AC-TPL-CLI-JSON-CORE

## Scope

**Commands in scope:**
- `ac-status` - AC coverage/status report
- `version` - Build information
- `friction-list` - DevEx friction entries
- `questions-list` - Design questions/ambiguities
- `fork-list` - Fork registry entries

**Files to modify:**
- `crates/xtask/src/commands/ac_status.rs` - Add --json flag
- `crates/xtask/src/commands/version.rs` - Add --json flag (verify existing)
- `crates/xtask/src/commands/friction.rs` - Add --json flag to list subcommand
- `crates/xtask/src/commands/questions.rs` - Add --json flag to list subcommand
- `crates/xtask/src/commands/forks.rs` - Add --json flag to list subcommand
- `specs/features/xtask_devex.feature` - BDD scenarios for JSON output
- `crates/acceptance/src/steps/xtask_devex.rs` - BDD step implementations

## Goals

1. Add `--json` flag to 5 core reporting commands
2. Emit single valid JSON document on stdout when `--json` is passed
3. Use stable top-level shape for each command's JSON output
4. Maintain stable exit codes: 0 on success, non-zero on failure
5. Validate JSON output structure and completeness via BDD tests

## Implementation Steps

1. **Define JSON output structs for each command**

   **ac-status:**

   ```rust
   #[derive(Serialize)]
   struct AcStatusJson {
       timestamp: String,
       summary: AcSummary,
       details: Vec<AcDetail>,
   }
   ```

   **version:**

   ```rust
   #[derive(Serialize)]
   struct VersionJson {
       version: String,
       git_sha: String,
       build_date: String,
       template_version: String,
   }
   ```

   **friction-list:**

   ```rust
   #[derive(Serialize)]
   struct FrictionListJson {
       entries: Vec<FrictionEntry>,
       total_open: usize,
       total_resolved: usize,
   }
   ```

   **questions-list:**

   ```rust
   #[derive(Serialize)]
   struct QuestionsListJson {
       questions: Vec<Question>,
       total: usize,
   }
   ```

   **fork-list:**

   ```rust
   #[derive(Serialize)]
   struct ForkListJson {
       forks: Vec<Fork>,
       total: usize,
   }
   ```

2. **Add --json flag to each command's Args struct**

   ```rust
   #[derive(Parser)]
   pub struct AcStatusArgs {
       /// Output in JSON format
       #[arg(long)]
       json: bool,
   }
   ```

3. **Implement JSON output in each command's run() function**
   - Check `args.json` flag
   - If true: serialize to JSON, print to stdout, return
   - If false: use existing human-readable format
   - Example pattern:

     ```rust
     if args.json {
         let json_output = AcStatusJson { /* ... */ };
         println!("{}", serde_json::to_string_pretty(&json_output)?);
         return Ok(());
     }
     // existing human-readable output
     ```

4. **Verify version command** (may already have --json)
   - Check if `version.rs` already implements --json
   - If yes: verify it matches AC requirements (stable shape, exit codes)
   - If no: implement following the pattern above

5. **Write unit tests for JSON shape stability**
   - Add test `ac_status_json_shape_is_stable` in `commands/ac_status.rs`
   - Add test `version_json_shape_is_stable` in `commands/version.rs`
   - Add test `friction_list_json_shape_is_stable` in `commands/friction.rs`
   - Add test `questions_list_json_shape_is_stable` in `commands/questions.rs`
   - Add test `fork_list_json_shape_is_stable` in `commands/forks.rs`
   - Each test: serialize sample data, parse JSON, verify required keys present

6. **Write BDD scenarios** (`specs/features/xtask_devex.feature`)
   - Scenario 1: "ac-status --json emits valid JSON"
     - Tag: `@AC-TPL-CLI-JSON-OUTPUT`
     - Steps: run command with --json, verify valid JSON, check keys
   - Scenario 2: "version --json emits valid JSON"
     - Tag: `@AC-TPL-CLI-JSON-CORE`
     - Steps: run command with --json, verify structure
   - Scenario 3: "friction-list --json emits valid JSON"
   - Scenario 4: "questions-list --json emits valid JSON"
   - Scenario 5: "fork-list --json emits valid JSON"

7. **Implement BDD steps** (`crates/acceptance/src/steps/xtask_devex.rs`)
   - Add step: `When I run "cargo xtask ac-status --json"`
   - Add step: `Then the output should be valid JSON`
   - Add step: `And the JSON should have field "summary"`
   - Reuse existing JSON validation helpers from other tests

## Verification Commands

```bash
# Manual verification
cargo xtask ac-status --json | jq .
cargo xtask version --json | jq .
cargo xtask friction-list --json | jq .
cargo xtask questions-list --json | jq .
cargo xtask fork-list --json | jq .

# Verify JSON structure
cargo xtask ac-status --json | jq 'keys | sort'

# Run targeted BDD tests
CUCUMBER_TAG_EXPRESSION="@AC-TPL-CLI-JSON-OUTPUT" cargo test -p acceptance --test acceptance

# Run unit tests
cargo test -p xtask json_shape_is_stable

# Verify AC status
cargo xtask ac-status | grep AC-TPL-CLI-JSON-OUTPUT

# Full validation
cargo xtask selftest
```

## Definition of Done

- [ ] JSON output structs defined for all 5 commands
- [ ] `--json` flag added to each command's Args struct
- [ ] JSON output implemented in each command's run() function
- [ ] version command verified (may already exist)
- [ ] Unit tests added for JSON shape stability (5 test functions)
- [ ] BDD scenarios written and tagged @AC-TPL-CLI-JSON-OUTPUT / @AC-TPL-CLI-JSON-CORE
- [ ] BDD steps implemented in xtask_devex.rs
- [ ] All @AC-TPL-CLI-JSON-OUTPUT tests pass
- [ ] All @AC-TPL-CLI-JSON-CORE tests pass
- [ ] Manual test: `cargo xtask ac-status --json | jq .` produces valid JSON
- [ ] Exit codes remain stable: 0 on success, non-zero on failure
- [ ] `cargo xtask ac-status` shows AC-TPL-CLI-JSON-OUTPUT as PASS
- [ ] No other ACs flip to FAIL

## Notes

- **Pattern Consistency:** Use same --json flag pattern across all commands
- **Backward Compatibility:** Default output remains human-readable; --json is opt-in
- **Testing Strategy:** Unit tests for shape stability + BDD for E2E validation
- **IDP/Agent Integration:** JSON output enables machine consumption by agents and IDPs
