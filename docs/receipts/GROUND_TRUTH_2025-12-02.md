<!-- doclint:disable orphan-version -->
# Ground Truth - December 2, 2025

## Critical Infrastructure Fix

**Issue**: JUnit XML file was empty due to premature process termination
**Root Cause**: `crates/acceptance/tests/acceptance.rs:179` called `std::process::exit(0)` before JUnit writer could flush buffers
**Fix Applied**: Removed explicit exit to allow clean shutdown and buffer flushing
**Result**: JUnit XML now properly written (191K, contains test results)

## Verified AC Status (from cargo xtask ac-status)

**Command**: `cargo xtask ac-status`
**JUnit Source**: `target/junit/acceptance.xml` (191K, properly formatted)
**Timestamp**: 2025-12-02 03:32 UTC

### Summary Statistics
- **Total ACs**: 112
- **Scenarios Found**: 203
- **ACs with Results**: 91
- **Unit Tests Captured**: 239
- **Passing ACs**: 87
- **Failing ACs**: 4

### Failing ACs

1. **AC-PLT-021** - `service-init` command updates service branding
2. **AC-TPL-IDP-SNAPSHOT** - `idp-snapshot` command generates JSON
3. **AC-TPL-AGENT-HINTS** - Agent hints API returns recommendations
4. **AC-TPL-XTASK-NONINTERACTIVE** - Commands work in non-interactive mode

### What the Agent Campaign Accomplished

**Exploration (9 agents)**: ✅ Comprehensive analysis of all failing ACs
**Implementation (12 agents)**: ⚠️ Made changes but verification blocked by JUnit infrastructure bug
**Documentation**: ✅ Created extensive guides and summaries

**Key Finding**: The agents' reports claiming "ACs fixed and passing" were **unverified hypotheses** because:
- JUnit XML was empty throughout the campaign
- ac-status couldn't read test results
- Agents based conclusions on command output, not actual test framework results

### Infrastructure Bug Impact

The `std::process::exit(0)` bug has been present since the test harness was created. This means:
- AC status reports have been unreliable
- Test results weren't being captured
- Governance validation was incomplete

**This bug is now FIXED** and AC status can be trusted going forward.

### Next Steps

1. **Verify the 4 failing ACs** with targeted BDD runs
2. **Update agent summaries** to reflect ground truth
3. **Slice work into PRs** based on actual status, not agent claims
4. **Re-run selftest** to get full governance validation

### Receipts Location

- `docs/receipts/ac-status_2025-12-02-final.log` - Full AC status output
- `target/junit/acceptance.xml` - Test results (191K, valid XML)
- This file - Ground truth summary
