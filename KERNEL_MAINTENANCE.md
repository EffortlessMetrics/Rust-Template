# KERNEL_MAINTENANCE.md – Governance for the Rust-as-Spec Kernel

This document defines the rules for maintaining this kernel/template repository going forward, ensuring it remains a stable, minimal, and fork-friendly foundation rather than a dumping ground for speculative features.

## 1. Purpose

This repository is a **productized kernel and template**, not a research lab or sandbox.

- Changes should be driven by **real needs demonstrated by forks** (e.g., Knowledge Hub, other downstream projects).
- The kernel should remain **minimal, stable, and comprehensible** so that forks can easily adapt it to their domain.
- All features must have clear value across multiple fork scenarios or solve critical pain points in the kernel itself.

## 2. What Changes Are Allowed

### Always Allowed (No Discussion Required)

- **Bug fixes** to failing kernel ACs or broken functionality
- **Documentation corrections** and clarity improvements
- **Dependency updates** (security patches, critical fixes)
- **CI/tooling fixes** that unblock development

**Process:** Open PR, link to issue or failing AC. Merge when tests pass.

### Allowed with Justification (GitHub Issue + Fork Link)

- **New kernel ACs** (when forks demonstrate a pattern or a clear gap)
- **New `/platform/*` endpoints** (when multiple forks would measurably benefit)
- **New xtask commands** (when they improve the fork development workflow)
- **Enhancements to existing features** (better error messages, performance, UX improvements)

**Process:**
1. Open GitHub issue: "Kernel improvement: [feature] (from [fork name])"
2. Link to the fork or user story demonstrating the need
3. Propose the change as generic and reusable
4. Implement with tests; validate in requesting fork
5. Document in TEMPLATE-CONTRACTS.md if introducing a new contract

### Requires ADR

- **New required fields** in `spec_ledger.yaml`, `config_schema.yaml`, or `tasks.yaml`
- **Breaking changes** to `/platform/*` API contracts
- **New mandatory selftest steps** or validation rules
- **Changes to the governance model** or validation ladder

**Process:**
1. Draft ADR in `docs/adr/` explaining the change and rationale
2. Discuss in GitHub issue with rationale
3. Implement with backwards-compatibility where possible
4. Update TEMPLATE-CONTRACTS.md
5. Ensure selftest remains green

### Not Allowed

- **Domain-specific features** (these belong in forks, not the kernel)
- **Speculative "nice to have" features** with no fork asking for them
- **Experimental ACs without governance** (unless marked `tags: [future]` and `must_have_ac: false`)
- **Convenience shortcuts** that bypass the governance model

## 3. Process for Kernel Improvements

### Discovered from Forks (Backporting)

1. **Identify the pain point** in a fork (e.g., "Knowledge Hub needs better bundle filtering")
2. **Create GitHub issue** in kernel repo: `Kernel improvement: [feature] (from [fork name])`
3. **Propose as generic/reusable** – avoid fork-specific logic
4. **Implement in kernel** with tests aligned to kernel ACs
5. **Validate in the requesting fork** – confirm it solves the problem
6. **Document in TEMPLATE-CONTRACTS.md** if it's a new public contract or endpoint
7. **Merge to main** when tests pass and fork maintainers sign off

### Proactive Improvements (Kernel Originated)

1. **Draft ADR** explaining why this helps all forks (not just this repo)
2. **Tag the AC as `tags: [future]`** and set `must_have_ac: false` initially
3. **Get tests green** in this repo
4. **Test in at least one fork** to validate the approach
5. **Promote to `must_have_ac: true`** when proven and adopted
6. **Keep ADR in repo** as permanent record of decision

## 4. Labels and Issue Templates

Use these labels for kernel issues:

- **kernel-bug:** Broken behavior or failing AC in the kernel
- **kernel-doc:** Documentation gaps, errors, or clarity issues
- **kernel-ergonomics:** UX/DX improvements for template consumers and fork authors
- **kernel-feature:** New capability requested by or validated with forks
- **fork-feedback:** Issues, pain points, or improvements bubbled up from active forks

## 5. Who Decides

- **Trivial changes** (bug fixes, doc corrections, tooling fixes): Just do it, open PR
- **New features or enhancements:** Open GitHub issue, discuss, link to fork demonstrating need, implement with tests
- **Breaking changes or new contracts:** Require ADR, consensus from maintainers, and coordination with affected forks

## 6. Review Cadence

- **Per PR:** Ensure tests pass, linked to issue or AC, documented if needed
- **Monthly:** Review open kernel issues; triage fork-feedback and decide on prioritization
- **Quarterly:** Audit fork feedback for common patterns worth backporting to kernel
- **Per fork milestone:** Proactively check if fork discovered kernel gaps worth generalizing

## 7. The One Rule

> **If a feature doesn't either (a) fix a kernel AC, (b) improve kernel stability/docs, or (c) solve a demonstrated pain point in an active fork, it doesn't belong here.**

Forks are free to customize and extend. The kernel exists to be stable, clear, and easy to fork from. Keep it that way.
