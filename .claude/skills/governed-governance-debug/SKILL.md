---
name: governed-governance-debug
description: |
  Diagnose and fix selftest failures in the Rust-as-Spec platform cell. Use when cargo xtask selftest fails, when user reports "governance broken", or when policy violations are detected. Systematically isolates and resolves issues across the 7 selftest steps.
allowed-tools:
- Read
- Grep
- Glob
- Edit
- Write
- Bash
---

# Governed Governance Debug

## When to Use

Use this Skill when:
- `cargo xtask selftest` fails
- User reports "governance is broken"
- Policy violations detected
- Graph invariants violated
- AC mapping issues
- User says "fix selftest", "debug governance", or "why is selftest failing?"

## Prerequisites

- Repository is checked out
- Basic Rust toolchain available
- Platform may or may not be running (we'll check)

## Workflow

Selftest has 7 steps. We diagnose which step failed, isolate the issue, fix it, and re-validate.

### Step 0: Identify the failure

```bash
# Run selftest with verbose output
cargo xtask selftest -v
```

**Example output:**

```
[1/7] Running core checks (fmt + clippy + tests)... ✅
[2/7] Running BDD acceptance tests... ✅
[3/7] Checking AC mapping and coverage... ✗ FAILED
  Error: AC-TPL-123 has no tests
[4/7] Validating LLM bundler... (skipped)
[5/7] Running policy tests... (skipped)
[6/7] Validating DevEx flows... (skipped)
[7/7] Checking graph invariants... (skipped)
```

**Identify:** Step 3 failed (AC mapping)

### Step 1: Isolate the specific issue

Run the isolated command for the failing step:

| Step | Isolated Command | What it checks |
|------|------------------|----------------|
| 1 | `cargo xtask check` | fmt, clippy, tests |
| 2 | `cargo xtask bdd` | BDD scenarios |
| 3 | `cargo xtask ac-status` | AC-to-test mapping |
| 4 | `cargo xtask bundle implement_ac` | LLM bundler works |
| 5 | `cargo xtask policy-test` | Rego policies pass |
| 6 | `cargo xtask help-flows` | DevEx flows valid |
| 7 | `curl http://localhost:3000/platform/graph` | Graph invariants |

**Example:**

```bash
# Step 3 failed, run AC status check
cargo xtask ac-status

# Output:
# AC-TPL-123: "Feature X returns Y"
#   Status: ❌ No tests found
#   Expected: @AC-TPL-123 tag in BDD or unit tests
```

### Step 2: Fix the root cause

Based on the failing step:

#### Step 1 failures (Core checks)

```bash
# Run individual checks
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all

# Fix:
cargo fmt --all  # Auto-fix formatting
# Fix clippy warnings manually
# Fix test failures
```

#### Step 2 failures (BDD)

```bash
cargo xtask bdd

# Common issues:
# - Missing step definitions: Check crates/acceptance/src/steps/
# - Feature syntax errors: Validate Gherkin syntax in specs/features/*.feature
# - Scenario not tagged: Add @AC-XXX tag
```

**Fix:**
- Add missing step definitions
- Fix Gherkin syntax
- Ensure scenarios are tagged with ACs

#### Step 3 failures (AC mapping)

```bash
cargo xtask ac-status

# AC without tests:
# - Add BDD scenario with @AC-ID tag in specs/features/
# - OR add unit test with AC reference in code comments
```

**Fix:**
Edit `specs/spec_ledger.yaml` and add test reference:

```yaml
acceptance_criteria:
  - id: AC-TPL-123
    text: "Feature X returns Y"
    tests:
      - { type: bdd, tag: "@AC-TPL-123" }  # Add this
```

Then create matching scenario in `specs/features/`:

```gherkin
@AC-TPL-123
Scenario: Feature X returns Y
  Given preconditions
  When action
  Then expected outcome
```

#### Step 4 failures (LLM bundler)

```bash
cargo xtask bundle implement_ac

# Common issues:
# - YAML syntax errors in spec files
# - Missing referenced docs
```

**Fix:**
- Validate YAML syntax: `yamllint specs/*.yaml`
- Check that all referenced docs exist

#### Step 5 failures (Policy tests)

```bash
cargo xtask policy-test

# Common issues:
# - Policy violations in specs
# - Rego syntax errors
```

**Fix:**
- Read policy error messages carefully
- Update specs to comply (e.g., add missing must_have_ac)
- Or update policies in `policies/*.rego` if policy is wrong

#### Step 6 failures (DevEx flows)

```bash
cargo xtask help-flows

# Common issues:
# - Command in flow not defined in devex_flows.yaml
# - Flow references missing command
```

**Fix:**
Edit `specs/devex_flows.yaml`:

```yaml
commands:
  missing-command:  # Add missing command definition
    category: onboarding
    summary: "Description"
    required: true
```

#### Step 7 failures (Graph invariants)

```bash
# Check if platform is running
curl http://localhost:3000/platform/status

# If not running:
cargo run -p app-http &
sleep 5

# Get graph data
curl http://localhost:3000/platform/graph | jq
```

**Common issues:**
- Requirements with `must_have_ac: true` but no ACs
- Commands in flows but not in commands section
- Unreachable nodes (orphaned requirements)

**Fix:**
- Add missing ACs to requirements
- Define commands properly in `devex_flows.yaml`
- Link orphaned requirements to user stories

### Step 3: Re-run selftest

```bash
cargo xtask selftest
```

**Expected:** All 7 steps pass ✅

### Step 4: Document the fix (if non-trivial)

If the issue revealed a gap in documentation or a common mistake:

```bash
# Update friction log
echo "- $(date +%Y-%m-%d): Fixed AC mapping for AC-TPL-123 (missing BDD tag)" >> docs/friction_log.md

# Or update AGENT_GUIDE.md with new troubleshooting tip
```

## Exit Criteria

Debugging complete when:
- ✅ `cargo xtask selftest` passes all 7 steps
- ✅ Root cause identified and documented
- ✅ No new regressions introduced
- ✅ Friction log updated (if applicable)

## Error Recovery

### If you can't identify the failure

```bash
# Get maximum verbosity
RUST_LOG=debug cargo xtask selftest -v 2>&1 | tee selftest-debug.log

# Share log with team or review carefully
```

### If fix causes new failures

```bash
# Revert changes
git checkout -- <files>

# Re-run selftest to confirm it's back to original state
cargo xtask selftest

# Try a more targeted fix
```

### If platform won't start (Step 7)

```bash
# Check port availability
lsof -i :3000

# Kill existing process
kill -9 <PID>

# Restart platform
cargo run -p app-http
```

## Examples

### Example 1: AC has no tests

```bash
# Run selftest
cargo xtask selftest
# Output: [3/7] AC mapping... ✗ AC-TPL-NEWFEATURE-001 has no tests

# Isolate
cargo xtask ac-status | grep AC-TPL-NEWFEATURE-001
# AC-TPL-NEWFEATURE-001: "New feature works"
#   Status: ❌ No tests

# Fix: Create BDD scenario
cat >> specs/features/new_feature.feature <<'EOF'
@AC-TPL-NEWFEATURE-001
Scenario: New feature works
  When I use the new feature
  Then it should work as expected
EOF

# Update spec_ledger.yaml
# Add: tests: [{ type: bdd, tag: "@AC-TPL-NEWFEATURE-001" }]

# Re-run
cargo xtask selftest
# ✅ All pass
```

### Example 2: Graph invariant violation

```bash
# Selftest fails at step 7
cargo xtask selftest
# [7/7] Graph invariants... ✗ REQ-TPL-XYZ has must_have_ac=true but no ACs

# Fix: Add AC to requirement in spec_ledger.yaml
acceptance_criteria:
  - id: AC-TPL-XYZ-001
    text: "Requirement XYZ is met"
    tests: [{ type: manual, tag: "xyz_manual_test" }]

# Re-run
cargo xtask selftest
# ✅ All pass
```

### Example 3: Policy violation

```bash
# Selftest fails at step 5
cargo xtask policy-test
# ✗ Policy violated: requirement_must_have_owner
#   REQ-TPL-ABC has no owner field

# Fix: Add owner to spec_ledger.yaml
- id: REQ-TPL-ABC
  title: "..."
  owner: team-platform  # Add this
  must_have_ac: true

# Re-run
cargo xtask selftest
# ✅ All pass
```

## Success Criteria

Governance debug complete when:
- ✅ Selftest passes (11/11 steps)
- ✅ Root cause documented
- ✅ Fix is minimal and targeted
- ✅ No new policy violations introduced

## References

- **Selftest implementation:** `crates/xtask/src/tasks/selftest.rs`
- **Policy definitions:** `policies/*.rego`
- **Graph invariants:** `docs/explanation/governance-graph.md`
- **Operational guide:** `docs/AGENT_GUIDE.md`
- **Platform API:** <http://localhost:3000/platform/graph>

## Notes

- **Selftest is the contract:** If it passes, your work is valid
- **Don't bypass governance:** Fix the issue, don't disable the check
- **Policies are code:** They can have bugs too; propose changes if policy is wrong
- **Graph invariants are structural:** They ensure traceability and discoverability
