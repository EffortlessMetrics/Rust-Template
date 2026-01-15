## Summary

<!-- 1-3 bullet points describing what this PR does -->

-

## Scope

<!-- Required for danger-zone changes (specs, policy, CI, CLAUDE.md) -->
<!-- Advisory for other PRs - helps reviewers understand intent -->

Type: <!-- mechanical | behavior | governance | release | docs -->
Intent: <!-- 1-2 sentences: what is this change trying to accomplish? -->
Touchpoints: <!-- directories or key files this PR modifies -->
Evidence: <!-- selftest green, specific test coverage, etc. -->
Non-goals: <!-- what this PR intentionally does NOT change (optional) -->

## Test plan

<!-- How was this tested? What should reviewers verify? -->

- [ ] `cargo xtask selftest` passes
- [ ] <!-- Additional verification steps -->

## Evidence & Verification

<!-- For CI-active repos: CI workflow ran successfully -->
<!-- For CI-disabled repos: local gate is canonical -->

```
cargo xtask selftest
# paste summary or link to gate receipt
```

---

<!-- Optional sections below -->

<details>
<summary>Checklist (click to expand)</summary>

- [ ] Code follows project patterns
- [ ] Tests added/updated for behavior changes
- [ ] Documentation updated if needed
- [ ] No secrets or credentials committed
- [ ] Commit messages are clear and descriptive

</details>
