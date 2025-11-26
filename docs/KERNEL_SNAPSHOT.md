# Kernel Snapshot v3.3.3

**Date:** 2025-11-26 | **Version:** v3.3.3-kernel

## Executive Summary

This is the frozen kernel baseline (v3.3.3-kernel) for the Rust-as-Spec platform template. All 65 acceptance criteria pass. All 8 selftest gates pass. Day-0 commands work as documented. This is a stable, forkable baseline.

**Note:** "Selftest green" means the template meets its own specifications. It does not mean every use case has been validated in production. See [ROADMAP.md](./ROADMAP.md) for known gaps.

---

## Statistics

| Category | Count | Status |
|----------|-------|--------|
| **Total ACs** | 65 | All passing |
| **Kernel ACs** | 48 | All passing |
| **Non-kernel ACs** | 17 | All passing |
| **Selftest Gates** | 8 | All passing |

---

## Key Capabilities

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

## Verification

```bash
cargo xtask doctor       # Environment validated
cargo xtask selftest     # 8/8 gates pass
cargo xtask ac-status    # 65/65 PASS, 0 FAIL, 0 UNKNOWN
cargo run -p app-http    # Listening on :8080
```

**Detailed AC statuses:** `docs/feature_status.md`

---

## Fork Readiness

The template is ready to fork. Services inheriting from v3.3.3-kernel get:

- Runtime, APIs, and UI that pass their ACs
- DevEx tooling for agents and humans
- Governed workflows with BDD acceptance tests
- Continuous governance validation via selftest
- Agent-friendly documentation and bundler

**Known gaps** (documented in ROADMAP.md):

- Branch protection not configured (manual GitHub setting required)
- No IDP positioning documentation
- No brownfield migration guide
- Template not yet validated by a second service

The first real fork will likely discover friction. Capture it in `FRICTION_LOG.md` and consider feeding systematic issues back to the kernel.

---

## End of Kernel Snapshot
