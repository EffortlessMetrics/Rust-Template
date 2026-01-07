## Investigation Report: Issue #63 - v3.5.0+ Surface Minimization Epic

### Status
**Status:** BLOCKED on v3.4.0 (#62)
**Philosophy:** "Demand-driven by real fork friction, not speculative"

### Evidence

**Epic Goals:**
1. Extract reusable crates (gov-model, gov-http, ac-kernel, versioning)
2. Publish to crates.io (not vendoring)
3. Thin template, easier fork upgrades

**Entry Criteria (from #62):**
- v3.4.0 shipped with real IDP consumer
- At least 2 active forks experiencing upgrade friction
- Clear boundary between kernel machinery and template examples

**Current State:**
- Crate infrastructure exists (gov-model, gov-http, ac-kernel)
- All crates have `publish = false`
- No fork friction data yet

### Plan

**Sequencing is correct:**
1. Complete v3.4.0 external validation first
2. Gather fork upgrade friction data from 2+ forks
3. Design crate boundaries informed by real usage
4. Then extract and publish

### Decision / Next Action

**Recommend:** No action until v3.4.0 completes. Premature extraction would risk wrong boundaries.
