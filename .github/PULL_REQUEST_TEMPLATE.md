## Summary

<!-- 1-3 bullet points describing what this PR does -->

-

## Linked ACs / Issues

<!-- Reference any ACs or issues this addresses. Use "Closes #X" to auto-close issues. -->

- AC:
- Closes #

## Changes

<!-- Brief description of what changed. Group by area if multiple concerns. -->

### Code Changes

-

### Documentation Changes

-

## Test Plan

<!-- How was this tested? Include commands to reproduce. -->

- [ ] `cargo xtask check` passes
- [ ] `cargo xtask selftest` passes (if touching governance)
- [ ] New tests added for changed behavior
- [ ] Manual testing performed (describe below)

**Manual testing steps:**

```bash
# Commands to reproduce / verify
```

## Evidence & Verification

<!-- For significant changes, include verification evidence -->

**CI Status:** <!-- CI active / CI disabled; local gate canonical -->

**Reproduce locally:**

```bash
cargo xtask selftest
```

## Checklist

- [ ] Code follows project conventions (`cargo fmt`, `cargo clippy`)
- [ ] Documentation updated if behavior changed
- [ ] No secrets or sensitive data included
- [ ] Commit messages follow conventional format (`feat:`, `fix:`, `docs:`)

---

<!-- For kernel changes (see CONTRIBUTING.md §9): -->
<!-- - [ ] ADR created for architectural decisions -->
<!-- - [ ] Version bump if changing kernel contracts -->
