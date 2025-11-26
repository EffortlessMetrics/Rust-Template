# Kernel Snapshot v3.3.3

**Date:** 2025-11-26 | **Version:** v3.3.3-kernel

## Executive Summary

This is the frozen hyper-green kernel baseline (v3.3.3-kernel) for the Rust-as-Spec platform template. **All 65 acceptance criteria pass unconditionally.** All 8 selftest gates pass. Day-0 commands work smoothly with zero exceptions. This is a production-ready, forkable baseline with better UX than human-wired service development. **Ready to fork for any service.**

---

## Statistics

| Category | Count | Status |
|----------|-------|--------|
| **Total ACs** | 65 | **100% passing** |
| **Kernel ACs** | 48 | All passing |
| **Non-kernel ACs** | 17 | All passing |
| **Selftest Gates** | 8 | All passing |

---

## Key Capabilities (All Passing)

**Runtime & APIs:**
- Service health, version, metrics endpoints
- Platform introspection APIs (/platform/graph, /platform/devex/flows, /platform/docs/index, /platform/schema)
- Agent hints API (/platform/agent/hints) with task prioritization
- Platform UI dashboard with graph visualization and flows view
- Configuration validation and IAC alignment (Docker Compose, Kubernetes, Terraform)
- Task lifecycle and governance write operations

**DevEx CLI:**
- Development flows: `doctor`, `help-flows`, `check`, `test-changed`, `ac-status`, `ac-coverage`
- Bundler & agent tools: `bundle`, `suggest-next`
- Governance: `adr-new`, `ac-new`, `docs-check`, `graph-export`, `selftest`
- Release management: `release-prepare`, `release-bundle`
- Operational: `audit`, `sbom-local`, `ci-local`, `status`

**Governance:**
- BDD acceptance tests (110 scenarios passing, 65 ACs covered)
- Graph invariants for REQ/AC/test/doc relationships
- Policy tests (22/22 passing)
- Pre-commit hooks and markdown hygiene
- AC/ADR bidirectional mapping

---

## Verification at v3.3.3-kernel

```bash
cargo xtask doctor       # ✅ Environment validated
cargo xtask selftest     # ✅ 8/8 gates pass
cargo xtask ac-status    # ✅ 65/65 PASS, 0 FAIL, 0 UNKNOWN
cargo run -p app-http    # ✅ Listening on :8080
```

**Detailed AC statuses:** `docs/feature_status.md`

---

## Fork Readiness

**Unconditionally ready.** All 65 ACs pass, all selftest gates pass, day-0 commands work smoothly. This template delivers a better developer experience than human-wired service development. Fork it, run the commands, everything is green. No escape hatches, no "yeah-but"s, no @ci-only caveats.

Services forking from v3.3.3-kernel inherit:
- Production-ready runtime, APIs, and UI
- Complete DevEx tooling for agents and humans
- Governed workflows with BDD acceptance tests
- Continuous governance validation via selftest
- Agent-friendly documentation and bundler

This is the kernel at its most stable, most forkable state.

---

**End of Kernel Snapshot v3.3.3**
