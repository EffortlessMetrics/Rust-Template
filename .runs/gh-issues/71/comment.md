## Investigation Report: Issue #71 - Developer Experience Epic

### Status
**Status:** IN PROGRESS - 70% of DevEx commands implemented
**Linked Issues:** #22, #24, #29, #35

### Evidence

**Implemented (✅):**
- `suggest-next`, `friction-new`, `ac-new`, `adr-new`
- `help-flows`, `ac-status`, `doctor`, `dev-up`
- Platform APIs: `/platform/agent/hints`, `/platform/status`, etc.

**Missing (❌):**
- `profile-build`, `ac-search`, `health-check --json`, `task-graph`
- Generator enforcement (technical guardrails)
- Consolidated onboarding guide
- Formal API reference documentation

**DevEx Metrics:**
| Metric | Target | Current |
|--------|--------|---------|
| Time-to-first-AC | <2 hours | ~2-3 hours |
| Commands with doc links | All | ~80% |
| Core crates doc examples | 3/3 | 0/3 |

### Plan

1. **Quick wins:** Implement `ac-search`, `health-check --json`
2. **Medium:** Generator enforcement warnings, `DAY_ONE.md` guide
3. **Documentation:** Complete stubs, add doc examples
4. **Tooling:** Fix feature_status.md determinism

### Decision / Next Action

**Recommend:** Continue at MEDIUM priority. Template is production-ready; these are polish improvements.
