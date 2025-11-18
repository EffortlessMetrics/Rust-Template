# v[X.Y.Z] Release Plan

**Created**: YYYY-MM-DD
**Target Release**: YYYY-MM-DD (if known)
**Status**: [Planning / In Progress / Complete]

---

## Release Theme

[One sentence describing the focus of this release]

---

## Scope

### Epics

**Epic 1: [Epic Name]**
- **Description**: [Brief description of what this epic delivers]
- **Value**: [Why this is important]
- **Acceptance Criteria**: AC-[XXX]-[YYY] (link to `specs/spec_ledger.yaml`)

**Epic 2: [Epic Name]** (optional - prefer single epic)
- **Description**: [Brief description]
- **Value**: [Why this is important]
- **Acceptance Criteria**: AC-[XXX]-[YYY]

### Out of Scope

Explicitly list what will NOT be included in this release:

- [ ] Feature X (deferred to v[X.Y+1.Z])
- [ ] Breaking change Y (stability priority)
- [ ] Refactoring Z (not user-facing)

---

## Guardrails

**Locked** (CANNOT change in this release):

- ❌ API surface of `crate-name-1` (stability commitment)
- ❌ Breaking changes to `crate-name-2` (user facing)
- ❌ Dependency upgrades (unless security critical)
- ❌ [Other protected areas]

**Allowed** (CAN change):

- ✅ New optional features (behind feature flags)
- ✅ Documentation improvements
- ✅ Internal refactoring (non-breaking)
- ✅ [Other safe changes]

---

## Implementation Roadmap

### Phase 1: Research & Planning
- [ ] Research library X version Y compatibility
- [ ] Review upstream documentation for Z
- [ ] Identify breaking changes in dependencies
- [ ] Document decision log (see below)

### Phase 2: Implementation
- [ ] Task 1: [Brief description]
- [ ] Task 2: [Brief description]
- [ ] Task 3: [Brief description]

### Phase 3: Testing
- [ ] Unit tests for new functionality
- [ ] Integration tests for adapters (if applicable)
- [ ] BDD scenarios for user-facing behavior
- [ ] Manual testing with [tool/environment]

### Phase 4: Documentation
- [ ] Create `docs/how-to/[feature].md` guide
- [ ] Update `crates/[crate]/README.md`
- [ ] Update CHANGELOG.md
- [ ] Update `specs/spec_ledger.yaml` with new ACs

### Phase 5: Release
- [ ] Run `cargo fmt --all -- --check`
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Run `cargo test --workspace`
- [ ] Run `cargo run -p xtask -- selftest`
- [ ] Test feature flag matrix (if applicable)
- [ ] Create git tag `v[X.Y.Z]`
- [ ] Push to origin

---

## Exit Criteria

Release is complete when:

1. **Functionality**:
   - [ ] All epics implemented and tested
   - [ ] AC IDs in `spec_ledger.yaml` marked as complete

2. **Quality Gates**:
   - [ ] `cargo fmt` passes
   - [ ] `cargo clippy --all-features` passes (zero warnings)
   - [ ] `cargo test --workspace` passes
   - [ ] `xtask selftest` passes (fmt, clippy, test, BDD, AC mapping, policy)

3. **Documentation**:
   - [ ] CHANGELOG.md updated with v[X.Y.Z] entry
   - [ ] How-to guides created (if new features)
   - [ ] Crate READMEs updated (if API changes)

4. **Release**:
   - [ ] Git tag `v[X.Y.Z]` created and pushed
   - [ ] GitHub release visible
   - [ ] CI passes (if applicable)

---

## Decision Log

**Created**: YYYY-MM-DD

### Decision 1: [Decision Title] (YYYY-MM-DD)

**Decision**: [What was decided]

**Rationale**:
- [Reason 1]
- [Reason 2]
- [Reason 3]

**Alternatives Considered**:
- **Option A**: [Brief description] - Rejected because [reason]
- **Option B**: [Brief description] - Rejected because [reason]

**Implementation Notes**:
- [Technical detail 1]
- [Technical detail 2]

---

### Decision 2: [Next Decision] (YYYY-MM-DD)

[Repeat structure above for each major decision]

---

## Risks & Mitigations

### Risk 1: [Risk Description]
- **Likelihood**: [Low / Medium / High]
- **Impact**: [Low / Medium / High]
- **Mitigation**: [How to reduce or handle]

### Risk 2: [Next Risk]
- **Likelihood**: [Low / Medium / High]
- **Impact**: [Low / Medium / High]
- **Mitigation**: [How to reduce or handle]

---

## Dependencies

### External Dependencies
- [ ] Library X version Y (research status: [Not Started / In Progress / Complete])
- [ ] Tool Z availability (e.g., `conftest` for policy tests)

### Internal Dependencies
- [ ] Crate A must be updated before Crate B
- [ ] Feature flag X must be implemented before Y

---

## Timeline (Optional)

| Phase                | Start Date | End Date   | Status        |
|----------------------|------------|------------|---------------|
| Planning             | YYYY-MM-DD | YYYY-MM-DD | [Not Started] |
| Implementation       | YYYY-MM-DD | YYYY-MM-DD | [Not Started] |
| Testing              | YYYY-MM-DD | YYYY-MM-DD | [Not Started] |
| Documentation        | YYYY-MM-DD | YYYY-MM-DD | [Not Started] |
| Release              | YYYY-MM-DD | YYYY-MM-DD | [Not Started] |

**Note**: Dates are estimates and may shift based on complexity.

---

## Post-Release

### Immediate Actions
- [ ] Verify tag on GitHub
- [ ] Close `v[X.Y.Z]` milestone (if using)
- [ ] Announce release (if applicable)

### Deferred to Next Release
- [ ] Task X (too large for this release)
- [ ] Feature Y (awaiting upstream dependency)
- [ ] Refactoring Z (lower priority)

### Pilot Projects (Before Next Release)
- [ ] Run greenfield pilot with v[X.Y.Z]
- [ ] Maintain friction log (see `docs/templates/FRICTION_LOG.md`)
- [ ] Gather feedback before planning v[X.Y+1.Z]

---

## Notes

[Any additional context, learnings, or observations]

---

## References

- **Acceptance Criteria**: `specs/spec_ledger.yaml`
- **Previous Release**: [docs/v[X.Y-1.Z]-plan.md](v[X.Y-1.Z]-plan.md)
- **Release Playbook**: [docs/RELEASE_PLAYBOOK.md](../RELEASE_PLAYBOOK.md)
- **Friction Log Template**: [docs/templates/FRICTION_LOG.md](FRICTION_LOG.md)

---

**End of Release Plan**
