# Pilot Notes Template

**Pilot ID:** PILOT-XXX-XXX
**Date:** YYYY-MM-DD
**Pilot Objective:** [From pilot-plan.yaml]
**Time Budget:** [Time limit from plan]
**Actual Time:** [How long it actually took]

---

## Summary

Brief 2-3 sentence summary of what this pilot accomplished (or attempted).

Example: "This pilot tested autonomous implementation of AC-EXAMPLE-001 (GET /api/todos endpoint) using the governed-feature-dev workflow. The agent successfully implemented code and tests in 1h 45min, surfacing one medium-severity friction issue with OpenAPI documentation."

---

## Success Criteria Assessment

For each criterion in pilot-plan.yaml, assess whether it was met:

| ID | Criterion | Met? | Notes |
|----|-----------|------|-------|
| SC-001 | [Description] | ✅ / ❌ / ⚠️ | [Details] |
| SC-002 | [Description] | ✅ / ❌ / ⚠️ | [Details] |
| ... | ... | ... | ... |

Legend:
- ✅ Fully met
- ⚠️ Partially met
- ❌ Not met

---

## Phase Breakdown

### Setup Phase

**Time:** [X minutes]

**Activities:**
- [What was done]
- [Commands run]
- [APIs queried]

**Outcomes:**
- [What was learned or prepared]
- [Checkpoints met/not met]

**Friction:**
- [Link to friction entries, if any]

---

### Execute Phase

**Time:** [X minutes]

**Activities:**
- [Implementation work]
- [Tests written]
- [Validation attempts]

**Outcomes:**
- [Code changes]
- [Test results]
- [AC status change]

**Friction:**
- [Link to friction entries]
- [Time lost to friction]

**Design Decisions:**
- [Link to ADRs, if any]

---

### Review Phase

**Time:** [X minutes]

**Activities:**
- [Validation commands]
- [Documentation]
- [Artifact collection]

**Outcomes:**
- [Selftest result]
- [AC status final]
- [Learnings captured]

**Friction:**
- [Final friction entries]

---

## Metrics

### Quantitative

| Metric | Value |
|--------|-------|
| Total time | [X hours Y minutes] |
| Commands executed | [Count] |
| Platform API queries | [Count] |
| Files modified | [Count (code/tests/docs/specs)] |
| Validation attempts | [check: X, test-ac: Y, selftest: Z] |
| Friction entries created | [Count] |
| ADRs created | [Count] |

### Qualitative

Rate on scale of 1-5 (1=poor, 5=excellent):

| Aspect | Rating | Notes |
|--------|--------|-------|
| Spec/AC clarity | X/5 | [Details] |
| Bundle quality | X/5 | [Was context sufficient?] |
| Platform API usefulness | X/5 | [Did APIs surface needed info?] |
| Documentation clarity | X/5 | [Was pilot harness easy to follow?] |
| Tooling effectiveness | X/5 | [Did xtask commands work as expected?] |
| Friction capture ease | X/5 | [Was template helpful?] |
| Overall autonomy | X/5 | [Could this run without human intervention?] |

---

## Friction Summary

Total friction entries: [Count]

### By Severity

- Critical: [Count]
- High: [Count]
- Medium: [Count]
- Low: [Count]

### By Category

- API: [Count]
- Docs: [Count]
- Tooling: [Count]
- Specs: [Count]
- Test: [Count]
- Workflow: [Count]
- Bundle: [Count]
- Agent: [Count]

### Top 3 Friction Points

1. **[FRICTION-PILOT-XXX]** (severity: [X]): [One-line summary]
   - Impact: [Time lost / blockers]
   - Status: [open/resolved]

2. **[FRICTION-PILOT-YYY]** (severity: [X]): [One-line summary]
   - Impact: [Time lost / blockers]
   - Status: [open/resolved]

3. **[FRICTION-PILOT-ZZZ]** (severity: [X]): [One-line summary]
   - Impact: [Time lost / blockers]
   - Status: [open/resolved]

---

## Design Decisions

List any ADRs created during this pilot:

### ADR-PILOT-XXX: [Title]

- **Status**: [DRAFT/ACCEPTED]
- **Context**: [Brief summary of the decision]
- **Impact**: [High/Medium/Low]
- **Follow-up needed**: [Yes/No - what?]

---

## Platform API Usage

**APIs Queried:**

| Endpoint | Times Queried | Usefulness | Notes |
|----------|---------------|------------|-------|
| /platform/status | X | High/Med/Low | [What info was useful/missing?] |
| /platform/agent/hints | X | High/Med/Low | [Was prioritized work clear?] |
| /platform/graph | X | High/Med/Low | [Did graph help understand scope?] |
| /platform/tasks | X | High/Med/Low | [Were tasks discoverable?] |
| /platform/friction | X | High/Med/Low | [Was existing friction visible?] |
| ... | ... | ... | ... |

**Findings:**

- What APIs were most useful?
- What information was missing from APIs?
- Any API issues encountered?

---

## Bundle Evaluation

**Bundle Used:** [Task name from .llm/contextpack.yaml]

**Bundle Quality:**

- **Completeness**: Did bundle include all needed files? [Yes/Mostly/No]
- **Focus**: Was bundle scoped appropriately? [Too broad/Right size/Too narrow]
- **Size**: [Actual size in KB vs. max_bytes limit]
- **Missing files**: [List files that should have been in bundle but weren't]
- **Unnecessary files**: [List files in bundle that weren't needed]

**Recommendations:**

- Should bundle include additional patterns (e.g., `specs/openapi/**/*.yaml`)?
- Should bundle exclude certain files to reduce size?
- Should a new bundle task be created for this use case?

---

## Learnings

### What Worked Well

1. [Thing that worked]
   - Why it worked
   - How it helped

2. [Another thing]
   - ...

### What Didn't Work

1. [Thing that didn't work]
   - Why it didn't work
   - What could be better

2. [Another thing]
   - ...

### Surprises

- [Unexpected findings, both positive and negative]

### Unknowns Discovered

- [Questions or ambiguities that arose]
- [Areas needing more investigation]

---

## Recommendations

### For Template Improvements

1. **[Category: docs/tooling/specs/etc.]**: [Specific recommendation]
   - **Rationale**: [Why this would help]
   - **Effort**: [Low/Medium/High]
   - **Priority**: [Low/Medium/High]

2. [Another recommendation]
   - ...

### For Pilot Harness Improvements

1. **[Improvement]**: [Description]
   - **Would help**: [What friction this would eliminate]

2. [Another improvement]
   - ...

### For Future Pilots

1. **[Advice]**: [What to do differently next time]

2. [More advice]
   - ...

---

## Follow-Up Actions

### Immediate (within 1-2 days)

- [ ] File GitHub issue for [friction/improvement]
- [ ] Promote ADR-PILOT-XXX to accepted (or close as superseded)
- [ ] Update [docs/flows/skills] based on learnings
- [ ] Share pilot findings with team

### Short-term (within 1-2 weeks)

- [ ] Address high-severity friction discovered
- [ ] Implement recommended template improvements
- [ ] Run follow-up pilot with adjustments

### Long-term

- [ ] Monitor for patterns across multiple pilots
- [ ] Consider promoting pilot patterns to standard flows/skills
- [ ] Update governance based on agent pilot learnings

---

## Attachments

- `friction-entries/` – Structured friction entries
- `adrs/` – Design decisions made during pilot
- `evidence/` – Logs, screenshots, bundle outputs

---

## Example Filled-In Section (delete when using template):

### Summary Example

"This pilot tested autonomous implementation of AC-EXAMPLE-001 (GET /api/todos endpoint) using the governed-feature-dev workflow. The agent successfully implemented code and tests in 1h 45min (under 2h budget), surfacing one medium-severity friction issue with unclear OpenAPI documentation. Selftest passed, demonstrating the governed workflow is viable for agent-driven AC implementation."

### Phase Breakdown Example

#### Execute Phase

**Time:** 60 minutes

**Activities:**
- Read `bundle/implement_ac/context.md` to understand AC requirements
- Implemented handler in `crates/app-http/src/handlers/todos.rs`
- Added BDD scenario in `specs/features/todos.feature` with `@AC-EXAMPLE-001` tag
- Added step definitions in `crates/acceptance/src/steps/todo_steps.rs`
- Ran validation: `cargo xtask test-changed` (pass), `cargo xtask ac-tests AC-EXAMPLE-001` (pass)

**Outcomes:**
- Code implements AC requirements (handler returns JSON array of todos)
- BDD scenario passes (Green: 1, Red: 0)
- Unit tests pass (crates/app-http: 12/12)
- No clippy violations

**Friction:**
- FRICTION-PILOT-001 (medium): Unclear where to add OpenAPI spec
  - Time lost: 15 minutes grepping for openapi files
  - Workaround: Found examples in existing handlers

**Design Decisions:**
- ADR-PILOT-001: Use dedicated handler module vs. generic CRUD handler
  - Chose dedicated handler for clarity and consistency with existing patterns

---

**Pilot Notes Template Version:** 1.0.0 (aligned with template v3.3.5)
