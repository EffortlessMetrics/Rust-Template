# ADR-PILOT-XXX: [Title of Decision]
<!-- doclint:disable orphan-version -->

**Status**: DRAFT
**Date**: YYYY-MM-DD
**Authors**: [Agent Name or Human Name]
**Pilot ID**: PILOT-XXX-XXX
**Related ACs**: [AC-XXX-YYY, AC-ZZZ-AAA]

---

## Context

**Problem or Ambiguity:**

Describe the design question, ambiguity, or decision point that arose during the pilot.

- What were you trying to accomplish?
- What was unclear or ambiguous in the specs, docs, or code?
- Why did this require a design decision (vs. following existing patterns)?

**Constraints:**

List any constraints that influenced the decision:

- Time limits (pilot time-box)
- Existing architecture or patterns
- Governance rules (selftest, ACs, specs)
- Tool limitations
- Performance, security, or maintainability requirements

**Discovery Context:**

- **Pilot phase**: [Setup | Execute | Review]
- **AC/Task in progress**: [AC-XXX-YYY or TASK-XXX-YYY]
- **When discovered**: [Timestamp or phase point]

---

## Decision

**Choice Made:**

State the decision clearly and concisely in 1-2 sentences.

Example: "We decided to implement the `/api/todos` endpoint as a new handler module rather than extending the existing generic CRUD handler."

**Reasoning:**

Explain WHY this decision was made:

- What alternatives were considered?
- What were the tradeoffs?
- Why is this choice better than the alternatives?
- How does this align with existing architecture, patterns, or governance?

**Alignment:**

- **Spec alignment**: How does this align with `spec_ledger.yaml`, `config_schema.yaml`, or `devex_flows.yaml`?
- **ADR alignment**: Does this extend or conflict with existing ADRs? (link them)
- **Pattern alignment**: Does this follow existing code patterns or introduce new ones?

---

## Consequences

**Positive:**

What are the benefits of this decision?

- Faster development
- Clearer code
- Better maintainability
- Alignment with governance
- Reduced complexity

**Negative:**

What are the downsides or risks?

- Technical debt
- Additional complexity
- Performance impact
- Future refactoring needed
- Inconsistency with other patterns

**Neutral:**

Other effects that are neither clearly positive nor negative.

---

## Validation

**How was this decision validated?**

- Tests run: [cargo xtask check, selftest, etc.]
- AC status: [Before/after]
- Code review: [Human or agent review]
- Selftest result: [Pass/Fail with details]

**Open Questions:**

List any remaining questions or uncertainties:

- What needs further investigation?
- What might need to change in the future?
- What feedback is needed from the team?

---

## Follow-Up Actions

**Immediate:**

- [ ] Document this decision in code comments (if applicable)
- [ ] Update related docs or ADRs (if applicable)
- [ ] File GitHub issue for team review (if decision is non-trivial)

**Future:**

- [ ] Revisit this decision in [timeframe] or when [condition]
- [ ] Monitor for friction or issues with this approach
- [ ] Consider promoting this pattern to a Skill or flow (if successful)

---

## Related Artifacts

**Specs:**

- `specs/spec_ledger.yaml`: [REQ-XXX-YYY, AC-XXX-YYY]
- `specs/devex_flows.yaml`: [Flow name if applicable]

**Code:**

- [File paths affected by this decision]

**Tests:**

- [Test files or BDD scenarios related to this decision]

**ADRs:**

- ADR-XXXX: [Related ADR title]
- ADR-YYYY: [Another related ADR]

**Issues/PRs:**

- GitHub Issue #XXX: [Description]
- PR #YYY: [Description]

---

## Notes

Additional context, observations, or thoughts that don't fit above.

Use this space for:

- Historical context (why the original design existed)
- Lessons learned
- Alternative approaches that were rejected (and why)
- References to external resources (RFCs, blog posts, docs)

---

## Example Filled-In ADR (delete this section when using template):

```markdown
# ADR-PILOT-001: Use Dedicated Handler Module for Todos Endpoint

**Status**: DRAFT
**Date**: 2025-12-01
**Authors**: pilot-agent
**Pilot ID**: PILOT-001
**Related ACs**: AC-EXAMPLE-001

---

## Context

**Problem or Ambiguity:**

While implementing AC-EXAMPLE-001 ("GET /api/todos returns list of todos"), it wasn't clear whether to:
1. Extend the existing generic CRUD handler in `crates/app-http/src/handlers/crud.rs`
2. Create a new dedicated handler module in `crates/app-http/src/handlers/todos.rs`

The spec_ledger.yaml and CLAUDE.md didn't provide guidance on when to use generic vs. dedicated handlers.

**Constraints:**

- Time-box: 2 hours for entire pilot
- Existing pattern: Generic CRUD handler exists but is currently unused (legacy?)
- Governance: Must maintain hexagonal architecture (ADR-0001)
- Maintainability: Code should be easy to extend for future todo features

**Discovery Context:**

- **Pilot phase**: Execute
- **AC in progress**: AC-EXAMPLE-001
- **When discovered**: 30 minutes into implementation

---

## Decision

**Choice Made:**

We decided to create a dedicated handler module (`crates/app-http/src/handlers/todos.rs`) for the todos endpoint rather than using the generic CRUD handler.

**Reasoning:**

Alternatives considered:

1. **Extend generic CRUD handler:**
   - Pro: Reuses existing code
   - Con: Generic handler is complex and not currently used
   - Con: Unclear how to specialize behavior for todos-specific validation

2. **Create dedicated handler (chosen):**
   - Pro: Clear, focused, easy to test
   - Pro: Follows existing pattern (health, version handlers are all dedicated)
   - Pro: Easy to extend with todos-specific features later
   - Con: Slightly more code

The generic CRUD handler appears to be legacy code from an earlier template version. All current endpoints (health, version, metrics) use dedicated handler modules, so this choice maintains consistency.

**Alignment:**

- **Spec alignment**: AC-EXAMPLE-001 doesn't prescribe implementation approach
- **ADR alignment**: Follows ADR-0001 (hexagonal architecture) by keeping handler logic in adapter layer
- **Pattern alignment**: Matches existing handler modules (health, version, metrics)

---

## Consequences

**Positive:**

- Clear, focused code that's easy to understand
- Consistency with existing handler patterns
- Easy to extend with todos-specific features
- Clear separation of concerns

**Negative:**

- Slightly more boilerplate than reusing generic handler
- Generic CRUD handler remains unused (technical debt)

**Neutral:**

- Handler count increases by 1

---

## Validation

**How was this decision validated?**

- Tests run: `cargo xtask ac-tests AC-EXAMPLE-001` → pass
- AC status: NotStarted → Passing
- Code review: Agent self-review (aligned with existing patterns)
- Selftest result: Pass (all 11 steps)

**Open Questions:**

- Should the generic CRUD handler be removed (technical debt)?
- Should we document "when to use generic vs. dedicated handlers" in CLAUDE.md?

---

## Follow-Up Actions

**Immediate:**

- [x] Document this decision in code comments (done in todos.rs)
- [ ] File GitHub issue for team review: "Should generic CRUD handler be removed?"
- [ ] Add guidance to CLAUDE.md or governed-feature-dev skill

**Future:**

- [ ] Revisit if we need 3+ similar endpoints (then generic handler might make sense)
- [ ] Monitor for friction with dedicated handlers (if todos features grow complex)

---

## Related Artifacts

**Specs:**

- `specs/spec_ledger.yaml`: REQ-EXAMPLE-TODOS, AC-EXAMPLE-001

**Code:**

- `crates/app-http/src/handlers/todos.rs` (new)
- `crates/app-http/src/handlers/crud.rs` (unused legacy)

**Tests:**

- `specs/features/todos.feature` (BDD scenario)
- `crates/acceptance/src/steps/todo_steps.rs` (step definitions)

**ADRs:**

- ADR-0001: Hexagonal architecture (aligned)

---

## Notes

This decision was made quickly due to pilot time constraints. A longer review might justify removing the generic CRUD handler entirely or documenting its intended use cases.

The existing handler modules (health, version, metrics) are all simple and dedicated, which suggests the generic handler is not the current standard pattern.
```

---

## Guidance for Using This Template

### When to Create a Pilot ADR

Create an ADR during a pilot when:

- Making a non-trivial design choice (not just following existing patterns)
- Specs or docs are ambiguous and you need to choose an approach
- Introducing a new pattern or deviating from existing ones
- Making a decision that affects multiple files or modules
- Tradeoffs exist and you need to document the reasoning

### When NOT to Create a Pilot ADR

Don't create an ADR for:

- Following clear, existing patterns (just do it)
- Trivial implementation choices (variable names, file organization)
- Temporary or throwaway code
- Decisions that are fully prescribed by specs or ACs

### ADR Status During Pilots

- **DRAFT**: Default status for pilot ADRs. Marks the decision as "needs review."
- **ACCEPTED**: Only use if a human reviews and approves during the pilot.
- **SUPERSEDED**: If you change your mind during the pilot, mark the old ADR superseded and create a new one.

### After the Pilot

- Pilot ADRs with status DRAFT should be reviewed by the team.
- If the decision is sound, promote to ACCEPTED and move to `/docs/adr/`.
- If the decision needs rework, file a GitHub issue and link to the pilot ADR.
- If the decision was wrong, mark as SUPERSEDED and document why.

---

**Template Version:** 1.0.0 (aligned with template v3.3.5)
