# CLAUDE.md – Rust-as-Spec Platform Cell

**Template Version:** v3.3.14

You are an autonomous engineer working **inside** a Rust-as-Spec platform cell. Work confidently, validate with selftest, and leave a clear trail for async human review.

---

## Mindset

- **Specs are your brief** – `spec_ledger.yaml` defines stories → REQs → ACs → tests
- **xtask is your control surface** – every workflow has a command
- **Selftest is your referee** – the 11-step gate decides what's acceptable
- **Friction log is your feedback** – capture DevEx issues as they arise

When something is unclear:
1. Infer from existing contracts (specs, ADRs, patterns)
2. Capture reasoning in ADR, issue, or friction log
3. Keep selftest green
4. Leave a clear trail for async review

---

## Quick Start

```bash
# One-command bootstrap
cargo xtask dev-up

# Start HTTP service for platform APIs
cargo run -p app-http &

# Essential introspection
curl http://localhost:8080/platform/status        # Governance health
curl http://localhost:8080/platform/agent/hints   # Prioritized work
curl http://localhost:8080/platform/graph         # Full governance graph
```

---

## Essential Commands

| Task | Command |
|------|---------|
| Bootstrap | `cargo xtask dev-up` |
| Fast checks | `cargo xtask check` |
| AC coverage | `cargo xtask ac-status` |
| Test specific AC | `cargo xtask test-ac AC-XXX` |
| Full governance | `cargo xtask selftest` |
| Create AC | `cargo xtask ac-new AC-ID "description" --story US-X --requirement REQ-X` |
| Generate bundle | `cargo xtask bundle implement_ac` |
| List flows | `cargo xtask help-flows` |
| Friction log | `cargo xtask friction-list` |

---

## Validation Ladder

Run checks in this order, escalating only as needed:

1. **Fast** – `cargo xtask check` (fmt, clippy, unit tests)
2. **Changed** – `cargo xtask test-changed` (only affected code)
3. **AC-focused** – `cargo xtask test-ac AC-XXX`
4. **Full gate** – `cargo xtask selftest` (before PR)

---

## Sources of Truth

| Priority | Source | Purpose |
|----------|--------|---------|
| 1 | `specs/spec_ledger.yaml` | Stories → REQs → ACs → tests |
| 2 | `specs/devex_flows.yaml` | Developer workflows |
| 3 | `.claude/skills/*/SKILL.md` | Governed workflow recipes |
| 4 | `.claude/agents/*.md` | Specialized agent definitions |
| 5 | Platform APIs (`/platform/*`) | Real-time state visibility |
| 6 | `cargo xtask selftest` | Final arbiter of correctness |

---

## Handling Ambiguity

| Situation | Artifact |
|-----------|----------|
| Architectural decision | ADR (`cargo xtask adr-new "Title"`) |
| Process/tooling friction | Friction log (`cargo xtask friction-list`) |
| Spec ambiguity | Question (`cargo xtask question-new ...`) |
| Feature work | GitHub issue with REQ/AC links |

---

## Detailed Documentation

For comprehensive guidance, see:

- **First hour script**: `docs/how-to/ai-first-hour.md`
- **Full agent guide**: `docs/AGENT_GUIDE.md`
- **Governance rules**: `.claude/rules/README.md`
- **Skills governance**: `docs/SKILLS_GOVERNANCE.md`
- **Agents governance**: `docs/AGENTS_GOVERNANCE.md`

---

## Summary

Your job is to:
- Use contracts actively (specs, ADRs, patterns)
- Make reasonable, reversible decisions
- Capture reasoning as artifacts
- Finish with `cargo xtask selftest` green

The cell is designed for you to work **confidently and autonomously**. Selftest + CI keep it honest.
