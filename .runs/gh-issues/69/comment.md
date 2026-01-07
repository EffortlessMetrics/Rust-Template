## Investigation Report: Issue #69 - Testing Coverage

### Status
**Status:** triaged / fix-ready
**Local gates:** `cargo xtask ac-status`, `cargo xtask ac-coverage`, `cargo xtask bdd`, `cargo xtask selftest`

### Evidence

**Coverage Metrics:**
- **AC mapping:** 105/134 ACs passing (78.4%), 29 unknown (all non-kernel/CI-only)
- **Kernel ACs:** 73/73 passing (100%) - all `must_have_ac=true` requirements satisfied
- **BDD scenario count:** 249 scenarios total across 27 feature files
- **Scenario-to-AC mapping:** 252 scenarios with @AC-* tags
- **Unit tests:** 562+ unit tests mapped via spec_ledger.yaml
- **Integration tests:** 14 integration test files with 13+ test targets

**Test Infrastructure Health:**
- ✅ BDD framework (Cucumber/Gherkin) - operational, all 249 scenarios passing
- ✅ Acceptance testing - passing (7,313 LOC in crates/acceptance/src)
- ✅ Unit tests - 240+ tests passing with 0 failures
- ✅ Policy tests - 22/22 passing
- ✅ CI/DevOps contracts - satisfied

---

### Current Gaps

**1. Crates with Zero Unit Tests (540 LOC total):**

| Crate | Lines | Status | Issue |
|-------|-------|--------|-------|
| `model` | 55 | 0 tests | #50 - Core domain types untested |
| `business-core` | 128 | 0 tests | #50 - Port traits & use cases untested |
| `telemetry` | 357 | 0 tests | #53 - Observability infrastructure untested |

**2. BDD Scenarios Missing AC Tags:**
- ✅ All 249 BDD scenarios ARE properly tagged with @AC-* identifiers
- No unmapped scenarios detected

**3. Non-Kernel ACs Lacking Test Evidence (18 unknown):**
These are intentionally CI-only or meta-governance ACs (not blocking):
- AC-TPL-AGENTS-* (5 items) - agent governance validation via `agents-lint`
- AC-TPL-SKILLS-* (7 items) - skill governance validation via `skills-lint`
- AC-TPL-GRAPH-INVARIANTS - graph structure validation
- Others are CI-only validation

**4. Code Quality - Panic Usage in Acceptance Tests (#57):**
- Found 15+ `panic!()` calls in acceptance test steps
- Location: `crates/acceptance/src/steps/` (7,313 LOC)
- Impact: Tests work but use non-idiomatic Rust assertion patterns

**5. Module Coverage Gaps:**
- 5 crates with NO test modules: model, business-core, telemetry, adapters-grpc, rust_iac_*

---

### Impact

**What's at Risk:**
1. **Regression Safety** - Core domain logic not protected by unit tests
2. **Tracing Reliability** - Observability infrastructure untested
3. **Refactoring Confidence** - 540 LOC has zero test coverage

**Why This Matters:**
- Template enforces **AC-first development**: spec_ledger.yaml defines contract
- `cargo xtask ac-status` validates coverage: currently 73/73 kernel ACs passing
- The 540 LOC in untested crates are **not currently mapped to any AC**

---

### Plan

**Minimal Fix (Tier 1 - Visibility):**

1. **Bind untested crates to ACs in spec_ledger.yaml:**
   - Create REQ-TPL-MODEL-TESTS for `model` crate
   - Create REQ-TPL-BUSINESS-CORE-TESTS for `business-core` crate
   - Create REQ-TPL-TELEMETRY-TESTS for `telemetry` crate

2. **Add BDD scenarios for visibility:**
   ```bash
   cargo xtask ac-suggest-scenarios AC-TPL-MODEL-SERIALIZATION
   cargo xtask ac-suggest-scenarios AC-TPL-TELEMETRY-INIT
   ```

**Follow-ups (Tier 2 - Close gaps):**

1. **Implement unit tests:**
   - `crates/model/src/lib.rs`: Serialization/deserialization tests
   - `crates/business-core/src/lib.rs`: Use case tests
   - `crates/telemetry/src/lib.rs`: OTLP fallback, env var parsing

2. **Improve assertion quality (issue #57):**
   - Replace 15+ `panic!()` calls with `assert!()` / `assert_eq!()`
   - Add `#[track_caller]` to test helper functions

3. **Integration test coverage (issue #28):**
   - Expand `crates/adapters-grpc/tests/smoke.rs`
   - Add tests for adapter implementations

**Test Plan:**
```bash
# Verify baseline
cargo xtask ac-status
cargo xtask ac-coverage

# After adding unit tests
cargo test --lib
cargo xtask test-changed

# Verify governance closure
cargo xtask selftest
```

---

### Decision / Next Action

**Recommendation:** KEEP OPEN with priority labels

**Justification:**
1. **Already Healthy:** Selftest green (100% kernel AC coverage), 0 test failures
2. **Governance Gap:** 540 LOC in untested crates invisible to spec_ledger.yaml
3. **Not Blocking:** Non-kernel ACs are informational only; template is production-ready
4. **Bounded Work:** Clear sub-issues (#50, #53, #57, #28)

**Gate Status:**
- ✅ Selftest: PASS (73/73 kernel ACs + 43/61 non-kernel ACs)
- ✅ BDD: PASS (249 scenarios, 252 tagged with @AC-*)
- ✅ Unit tests: 240+ tests, 0 failures
- ⚠️ Unmapped coverage: 540 LOC in 3 crates lack both tests AND AC bindings

**Next Steps:**
1. Create ACs for test coverage requirements
2. Implement tests for model, business-core, telemetry
3. Replace panic!() with assertions in acceptance tests
