---
id: GUIDE-TPL-BROWNFIELD-001
title: Brownfield Adoption Guide
doc_type: guide
status: published
audience: developers, platform-engineers, team-leads
tags: [adoption, brownfield, migration, governance, onboarding]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-ONBOARDING, REQ-PLT-DOC-TEMPLATES]
acs: [AC-PLT-001, AC-PLT-008]
adrs: [ADR-0003, ADR-0005]
last_updated: 2025-11-26
---

# Brownfield Adoption Guide

> **Note**: This is a high-level strategy guide. For step-by-step technical instructions, see **[how-to/add-governance-to-existing-repo.md](../how-to/add-governance-to-existing-repo.md)**.

**You have an existing Rust service. You want governance without rewriting.**

This guide helps you choose the right adoption strategy and links you to detailed resources.

---

## When to Use This Guide

Use this if you have:
- An existing Rust codebase in production or near-production
- Tests you want to keep
- A desire to add AC-first development and policy enforcement
- No appetite for a full rewrite

---

## Decision Tree: Which Approach?

Choose based on your current situation and governance goals:

| If you want... | Use this profile | Time to adopt |
|----------------|------------------|---------------|
| **Just code quality checks** | Minimal | Week 1 |
| **AC tracking + policy** | Standard | Weeks 1-3 |
| **Full governance suite** | Strict | Weeks 1-5 |

### Minimal Profile

- **Just want:** fmt, clippy, test gates
- **Skip:** AC ledger, BDD, policy tests
- **Best for:** Small teams, low-risk services

### Standard Profile (Recommended)

- **Adds:** AC ledger, spec tracking, basic policies
- **Coverage:** 60-80% of production features
- **Best for:** Most production services

### Strict Profile

- **Adds:** All contract gates, privacy policies, full traceability
- **Coverage:** 90%+ of features
- **Best for:** Regulated industries, compliance-heavy systems

See [ADOPTION.md](../../ADOPTION.md#1-profiles) for full profile definitions.

---

## Phased Rollout (5-Week Plan)

| Week | What to Add | Detailed Guide | Validation |
|------|-------------|----------------|------------|
| **1** | xtask orchestrator (check, test) | [add-governance-to-existing-repo.md](../how-to/add-governance-to-existing-repo.md#step-1-install-the-xtask-orchestrator-10-min) | `cargo xtask check` passes |
| **2** | Policy tests (Rego) + basic ledger | [add-governance-to-existing-repo.md](../how-to/add-governance-to-existing-repo.md#step-2-initialize-governance-structure-15-min) | `cargo xtask policy-test` passes |
| **3** | AC ledger + map existing tests | [add-governance-to-existing-repo.md](../how-to/add-governance-to-existing-repo.md#step-6-map-existing-tests-to-acs-optional-20-min) | AC coverage report generated |
| **4** | BDD acceptance tests (optional) | [add-governance-to-existing-repo.md](../how-to/add-governance-to-existing-repo.md#24-create-acceptance-test-structure-optional-but-recommended) | First BDD scenario passes |
| **5** | Full selftest + CI gates | [add-governance-to-existing-repo.md](../how-to/add-governance-to-existing-repo.md#step-4-wire-ci-integration-10-min) | `cargo xtask selftest` green |

**Total time investment:** 45-60 min setup + 2-4 hours customization across 5 weeks.

See [ADOPTION.md](../../ADOPTION.md#3-brownfield-adoption-existing-services) for the strategic 6-stage sequence.

---

## Two Integration Patterns

### Pattern A: Clone and Detach (Full Control)

- **Use when:** Building 1-2 services, want full ownership
- **Pros:** No merge conflicts, full customization freedom
- **Cons:** No automatic template updates
- **See:** [adoption-patterns.md](../explanation/adoption-patterns.md#pattern-a-clone-and-detach-single-service)

### Pattern B: Template as Upstream (Stay Current)

- **Use when:** Building 3-10 services, want ongoing improvements
- **Pros:** Get template updates automatically
- **Cons:** Merge conflicts, requires discipline
- **See:** [adoption-patterns.md](../explanation/adoption-patterns.md#pattern-b-template-as-upstream)

Most brownfield teams use **Pattern A** (detach) since existing services are already customized.

---

## Working Example

See the [brownfield-demo](../../examples/brownfield-demo/) for a complete working example:
- Existing HTTP server (pre-governance)
- Minimal xtask integration
- Step-by-step initialization
- Policy and spec scaffolding

Run the demo:

```bash
cd examples/brownfield-demo
cargo run -p xtask -- init --mode=brownfield
cargo run -p xtask -- selftest
```

---

## Library Mode (Advanced)

If your project is **highly customized** or you only want selective governance features:

Use the core library without the full template structure:

```toml
[dependencies]
rust_iac_xtask_core = { git = "https://github.com/yourorg/rust-template" }
```

Build your own xtask binary using the shared core.

See [MISSING_MANUAL.md](../MISSING_MANUAL.md#1-the-brownfield-adoption-path-library-mode) for details.

---

## Troubleshooting

### Existing tests don't map to ACs

**Solution:** Start with `AC-LEGACY-*` prefixes. Incrementally refactor as you add new features.

### Too many lint warnings blocking progress

**Solution:** Use `#[allow(...)]` temporarily. Set clippy to warn-only until you clean up legacy code.

### Coverage too low to meet gates

**Solution:** Start with Minimal profile (no coverage gate). Ratchet up to Standard once baseline improves.

### Team resists AC-first workflow

**Solution:** Adopt incrementally. Start with just `xtask check` in CI. Add AC ledger only for new features.

See [add-governance-to-existing-repo.md](../how-to/add-governance-to-existing-repo.md#troubleshooting) for detailed troubleshooting.

---

## Success Criteria

You've successfully adopted brownfield governance when:

1. ✅ `cargo xtask selftest` passes (even if coverage is low initially)
2. ✅ Team can add **new** features via AC-first workflow
3. ✅ Friction is captured in `FRICTION_LOG.md` for continuous improvement
4. ✅ CI enforces at least Minimal profile checks on main branch

**Don't aim for perfection on day 1.** Start with Minimal, iterate to Standard.

---

## See Also

- **[ADOPTION.md](../../ADOPTION.md)** - Strategic adoption profiles (Minimal/Standard/Strict)
- **[adoption-patterns.md](../explanation/adoption-patterns.md)** - Pattern A vs B vs C (clone, upstream, generator)
- **[add-governance-to-existing-repo.md](../how-to/add-governance-to-existing-repo.md)** - 60-min step-by-step technical guide
- **[brownfield-demo](../../examples/brownfield-demo/)** - Working code example
- **[MISSING_MANUAL.md](../MISSING_MANUAL.md)** - Library mode and advanced topics
