<!-- doclint:disable orphan-version -->
# How-To: Report Fork Feedback to the Kernel

**Audience:** Developers maintaining a service forked from this template who encounter bugs, friction, or missing features.

This guide helps you report issues back to the kernel repository so that improvements can benefit all forks.

---

## Quick Reference

Choose your path based on what you found:

| What You Found | Where to Report | Template |
|----------------|----------------|----------|
| **Bug in kernel AC** | Kernel repo issue | `.github/ISSUE_TEMPLATE/kernel-bug.md` |
| **Missing feature or friction** | Kernel repo issue | `.github/ISSUE_TEMPLATE/kernel-feedback.md` |
| **Process/tooling friction** | Your fork's `FRICTION_LOG.md` first | N/A |

---

## 1. Reporting a Kernel Bug

### When to Use This Path

- A kernel acceptance criterion (AC) is failing or behaving incorrectly
- Core functionality like `/platform/*` endpoints, `xtask` commands, or governance validation is broken
- The issue exists in the template itself, not in your domain-specific code

### Steps

1. **Verify it's a kernel issue**
   - Check that the bug reproduces in a clean fork without your domain customizations
   - Confirm the affected AC is listed in the kernel's `docs/KERNEL_SNAPSHOT.md`

2. **Check for existing issues**
   - Search the kernel repository's issues for duplicates
   - Filter by label `kernel-bug`

3. **Create issue using the template**
   - Navigate to kernel repo → Issues → New Issue → "Kernel Bug"
   - Fill in all required fields:
     - **AC ID:** e.g., `AC-PLT-015`, `AC-TPL-003`
     - **Reproduction steps:** Exact commands to trigger the bug
     - **Expected behavior:** What the AC definition says should happen (reference `specs/spec_ledger.yaml`)
     - **Actual behavior:** What actually happens (include error messages, logs)
     - **Environment:** Template version, platform (Linux/macOS/Windows), Nix usage

4. **Link from your fork**
   - If you've created a workaround or investigation in your fork, link to it
   - This helps kernel maintainers understand the real-world impact

---

## 2. Requesting a Kernel Improvement

### When to Use This Path

- You hit a workflow pain point that would help other forks too
- A feature is missing that multiple services need
- Documentation is unclear or incomplete

**Not for:** Domain-specific features unique to your service. Keep those in your fork.

### Steps

1. **Document in your fork's FRICTION_LOG.md first**

   ```bash
   # In your fork
   cargo xtask friction-new \
     --id FRICTION-MYSERV-001 \
     --category devex \
     --severity medium \
     --summary "Bundle filtering by REQ/AC doesn't support wildcards"
   ```

   - This captures context in your environment
   - See your fork's `FRICTION_LOG.md` for format and categories

2. **Determine if it's generic or domain-specific**
   - **Generic:** Helps all forks (e.g., better error messages, new `xtask` command, `/platform/*` endpoint improvement)
   - **Domain-specific:** Only helps your service (e.g., business logic, domain models)
   - If domain-specific, solve it in your fork and stop here

3. **Create issue using the feedback template**
   - Navigate to kernel repo → Issues → New Issue → "Kernel Feedback"
   - Fill in all fields:
     - **Which fork:** Name of your service
     - **Pain point:** Clear description of what's difficult or missing
     - **Generic vs domain-specific:** Explain why this helps all forks
     - **Proposed solution:** How you'd like the kernel improved
     - **Affected kernel contracts:** Which ACs or endpoints would change (see `docs/KERNEL_SNAPSHOT.md`)
     - **Fork context:** Link to your friction log entry, PR, or issue

4. **Optionally prototype in your fork**
   - If you've implemented a solution locally, share it
   - Kernel maintainers can help genericize it for backporting

---

## 3. What Happens Next

### Triage Process

- Kernel maintainers review fork feedback issues **monthly**
- Issues are labeled:
  - `kernel-bug` – Broken behavior, high priority
  - `kernel-doc` – Documentation gaps
  - `kernel-ergonomics` – UX/DX improvements
  - `kernel-feature` – New capabilities
  - `fork-feedback` – All issues from forks

### Decision Criteria

See `KERNEL_MAINTENANCE.md` for full governance details. In summary:

- **Bugs:** Fixed immediately if they break kernel ACs
- **Improvements:** Accepted if they solve demonstrated pain in real forks
- **New features:** Require justification and testing in at least one fork
- **Breaking changes:** Require ADR and coordination with all active forks

### If Accepted

1. Kernel maintainers implement the fix/feature
2. You may be asked to test the fix in your fork before it merges
3. The change is validated with tests and `cargo xtask selftest`
4. Once merged, you can backport it to your fork (see §4)

### If Declined

- Not all feedback will be backported to the kernel
- Domain-specific features belong in forks
- Speculative features without demonstrated need are declined
- Maintainers will explain the decision in the issue

---

## 4. Backporting Kernel Fixes to Your Fork

When a kernel improvement lands, pull it into your fork:

```bash
# One-time setup
cd /path/to/your-fork
git remote add template https://github.com/org/rust-template.git

# Pull kernel updates
git fetch template
git checkout main
git merge template/main --no-commit

# Review changes carefully
git status
git diff --cached

# Resolve conflicts if needed
# Focus on preserving your domain customizations

# Test before committing
cargo xtask doctor
cargo xtask selftest

# Commit the merge
git commit -m "chore: merge kernel updates from template v3.4.0"
git push origin main
```

**Important:**
- Always review changes before committing the merge
- Kernel updates may conflict with your domain code
- Test thoroughly in your fork's environment
- Document any manual adjustments in your fork's ADRs or friction log

---

## 5. Examples

### Example: Reporting a Bug

**Scenario:** `/platform/agent/hints` returns 500 error when `tasks_state.yaml` is missing.

**Steps:**
1. Reproduce in clean fork → confirms it's a kernel issue
2. Check `docs/KERNEL_SNAPSHOT.md` → endpoint is covered by `AC-PLT-020`
3. Create issue using "Kernel Bug" template
4. Include: AC ID, `curl` command that fails, expected JSON response, actual error
5. Link to your fork's investigation if you dug into the code

**Outcome:** Kernel maintainers fix in next patch release, you backport when ready.

---

### Example: Requesting a Feature

**Scenario:** `cargo xtask bundle` doesn't filter by REQ ID, only AC ID. This makes scoping bundles harder.

**Steps:**
1. Add to your fork's `FRICTION_LOG.md`: "FRICTION-MYSERV-003: Bundle filtering requires listing all child ACs instead of parent REQ"
2. Determine it's generic (helps all forks doing REQ-level planning)
3. Create issue using "Kernel Feedback" template
4. Propose: Add `--req` flag to `xtask bundle` command
5. Link to your friction log entry showing real-world pain

**Outcome:** Kernel maintainers implement `--req` flag, add tests, document in `TEMPLATE-CONTRACTS.md`, release in next minor version.

---

## 6. Additional Resources

- **Kernel governance:** `KERNEL_MAINTENANCE.md` in kernel repo
- **Issue templates:** `.github/ISSUE_TEMPLATE/` in kernel repo
- **Kernel capabilities:** `docs/KERNEL_SNAPSHOT.md` in kernel repo
- **Friction log schema:** `specs/friction_schema.yaml` in your fork
- **Template contracts:** `TEMPLATE-CONTRACTS.md` in kernel repo

---

## Summary

1. **Bugs in kernel ACs** → Use kernel-bug template, include reproduction steps
2. **Generic improvements** → Document in your FRICTION_LOG.md first, then use kernel-feedback template
3. **Domain-specific needs** → Solve in your fork, don't report to kernel
4. **Backport kernel fixes** → Use `git merge template/main`, test before committing

The kernel evolves based on **real fork needs**. Your feedback makes the template better for everyone.
