---
id: GUIDE-PROVENANCE-001
title: Provenance and Trust Model
doc_type: guide
status: published
audience: auditors, maintainers, platform-engineers
tags: [provenance, trust, ai-native, verification]
stories: []
requirements: []
acs: []
adrs: []
last_updated: 2026-01-07
---

# Provenance and Trust Model

This document explains what's automated, what's human, and what's stable in this repository.

---

## What This Repo Is

A **governed Rust service template** where:
- Specs, tests, docs, policies, and CI all agree
- The repo can prove it via `cargo xtask selftest`
- Large AI-assisted changes are normal and expected

---

## The Control Loop

```
Signal → Plan → Build → Gate → Deploy/Publish
                  ↓
              Wisdom Backlog
                  ↓
              (back to Signal)
```

Every change flows through gates. Gates produce receipts. Receipts back claims.

---

## AI-Assisted vs AI-Native

This repo is designed for **AI-native** workflows:

```
┌─────────────────────────────────────────────────────────────┐
│  AI-assisted                                                │
│    Human writes → AI helps → Human reviews diffs            │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  AI-native                                                  │
│    Machine generates → Receipts + gates → Human audits      │
│                        intent + evidence                    │
└─────────────────────────────────────────────────────────────┘
```

**Key difference:** In AI-native, you review *scope + receipts*, not individual lines.

---

## What's Automated

| Activity | Automation | Human Role |
|----------|------------|------------|
| Code formatting | `cargo fmt` (auto-staged in precommit) | None needed |
| Linting | `cargo clippy` (gate) | Fix violations |
| Tests | `cargo test` (gate) | Write tests, fix failures |
| AC status | `cargo xtask ac-status` (auto-generated) | Review coverage |
| Skills/Agents format | `xtask skills-fmt/agents-fmt` | Define content |
| Doc version alignment | `xtask docs-check` | Update if drift |
| Security scanning | CI jobs (CodeQL, gitleaks, cargo-audit) | Triage findings |
| BDD execution | `xtask bdd` | Write scenarios |

---

## What's Human

| Activity | Responsibility |
|----------|----------------|
| Intent specification | Defining what to build (REQs, ACs, stories) |
| Architecture decisions | ADRs, design docs |
| PR scope decisions | What goes in a PR |
| Errata acknowledgment | Documenting what was wrong |
| Factory improvements | Adding gates based on failure modes |

---

## What's Stable (Contracts)

| Contract | Location | Stability |
|----------|----------|-----------|
| Spec ledger schema | `specs/spec_ledger.yaml` | Kernel-frozen |
| Platform API | `/platform/*` endpoints | Kernel-frozen |
| Selftest gates | 11 steps in `xtask selftest` | Kernel-frozen |
| Config schema | `specs/config_schema.yaml` | Kernel-frozen |
| DevEx flows | `specs/devex_flows.yaml` | Kernel-frozen |

"Kernel-frozen" means: changes require ADR + version bump + explicit extension.

---

## Trust Anchors

### 1. Local Gate (`cargo xtask selftest`)

The single source of truth for "is this governed?"

### 2. Receipts (`.runs/` artifacts)

Machine-generated evidence of gate outcomes.

### 3. Spec Ledger (`specs/spec_ledger.yaml`)

The authority for what behavior is required.

### 4. Version Manifest (`specs/version_manifest.yaml`)

The authority for what version we're at.

---

## What We Don't Trust

- GitHub UI checkmarks alone (without receipt links)
- Prose claims without evidence
- Multipliers or comparative quality claims
- "Production-ready" without falsifiable criteria

---

## How to Verify This Repo

```bash
# 1. Enter the canonical environment
nix develop

# 2. Run the canonical gate
cargo xtask selftest

# 3. Check the version
cargo xtask version --json

# 4. Inspect the governance graph
cargo run -p app-http &
curl http://localhost:8080/platform/status
curl http://localhost:8080/platform/graph
```

If selftest is green and the APIs respond correctly, the governance contracts are intact.

---

## Change History Model

Changes are tracked via:

1. **Git history** — what changed when
2. **PR cover sheets** — why it changed, what was validated
3. **Casebook exhibits** — curated examples of good change
4. **Failure modes** — what went wrong and what was hardened
5. **Friction log** — DevEx issues that need attention

The goal is: any future auditor can understand *why* the code is the way it is.
