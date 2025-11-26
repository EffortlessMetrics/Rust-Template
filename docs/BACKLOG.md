# BACKLOG.md – Rust-as-Spec Platform Cell

**Purpose:** Ideas that are interesting but premature for the kernel. These features are held here until forks demonstrate real need and validate the design.

**Policy:** Implement backlog items ONLY when:
1. At least one fork has a concrete use case
2. The feature is generically designed (works for any service)
3. An ADR is written justifying the addition to the kernel
4. It's implemented with `must_have_ac: false` initially
5. Once proven in 2+ forks, consider promoting to kernel

**Review:** Revisit quarterly based on fork feedback and emerging patterns.

---

## Bundle System Enhancements

### New Bundle Types
- `troubleshoot_ac` – Focus on a failing AC, include related tests/specs/docs
- `implement_task` – Gather task dependencies, related ACs, and implementation hints
- `explain_decision` – Reconstruct reasoning behind an AC or design choice from ADRs and commits
- `measure_coverage` – Show test/doc gaps for a given AC or REQ

### Bundle Quality Features
- Bundle caching and incremental updates (only regenerate changed files)
- Priority-based file inclusion (exclude low-signal artifacts if bundle exceeds size limits)
- AC coverage signals in bundles (explicit "tests this AC", "documents this AC" flags)

---

## Graph and Visualization

### Advanced Graph Rendering
- Mermaid graph with filtering (by REQ, AC status, owner, label)
- Interactive graph explorer in `/ui` (click nodes to expand/collapse)
- Dependency visualization for tasks (task → related ACs → tests → docs)
- Impact analysis (which tests/docs/ACs would fail if AC X changes)

---

## Release Evidence and Notes

### Automated Release Artifacts
- Changelog generation from evidence bundles (group changes by category)
- Automatic Keep a Changelog formatting (ADDED, CHANGED, DEPRECATED, REMOVED, FIXED, SECURITY)
- Release delta reports (diffs between version bundles, what changed and why)
- Evidence-based release notes (tie each change to REQ/AC/test coverage)

---

## Agent Hints and Suggestions

### Intelligence for Task Selection
- Effort estimates for tasks (based on AC complexity, test count, breadth of changes)
- Dependency-aware prioritization (suggest blocking tasks first)
- Context-aware bundle recommendations (suggest bundle type based on task type)
- "Next best AC to implement" algorithms (considering skill gaps, dependency chains, impact)

---

## DevEx Tooling

### Interactive Developer Experience
- `cargo xtask repl` – REPL for exploring spec_ledger, running queries (e.g., "show ACs for REQ-X")
- `cargo xtask watch` – Auto-run tests when spec changes, show live feedback
- IDE integrations (LSP for spec_ledger.yaml with completions, linting, validation)
- VS Code extension for AC navigation (inline previews, test linking, bundle shortcuts)

---

## Testing and Validation

### Advanced Test Insights
- Mutation testing for AC coverage (ensure tests would fail if AC code mutates)
- Property-based testing scaffolds (generate test data from AC assertions)
- Test gap analysis (identify ACs with low test diversity or missing edge cases)
- Flakiness detection and reporting (flag tests that pass/fail unpredictably)

---

## Documentation

### Automated and Intelligent Docs
- Auto-generated API docs from spec_ledger (keep docs fresh with spec changes)
- ADR dependency graphs (show how design decisions relate and evolve)
- "Why does this exist?" explanations for each AC (rationale, history, alternatives)
- Cross-repo template documentation portal (shared learnings across forks)

---

## How to Move Items from Backlog to Kernel

1. **Fork demonstrates need** – A downstream project (Knowledge Hub, etc.) has concrete use case
2. **Generic design** – Feature works for any Rust-as-Spec service, not domain-specific
3. **ADR written** – Architecture decision recorded with rationale and tradeoffs
4. **Pilot implementation** – Code merged with `must_have_ac: false` (optional in selftest)
5. **Validation** – Feature proven useful in 2+ independent forks
6. **Promotion** – Move to kernel as `must_have_ac: true`, document in `ROADMAP.md`

---

## Explicitly Out of Scope

These are **not** candidates for the kernel backlog:

- **Domain-specific features** – Calculators, content management, business logic
- **Service integrations** – Payment providers, auth systems, vendor SDKs
- **Application-level concerns** – UI frameworks, deployment orchestration
- **Anything that couples the kernel to a specific business domain**

Forks should implement domain features themselves; the kernel stays focused on governance, specs, and workflows.

---

**Last Updated:** 2025-11-26
**Quarterly Review Scheduled:** 2026-02-26
