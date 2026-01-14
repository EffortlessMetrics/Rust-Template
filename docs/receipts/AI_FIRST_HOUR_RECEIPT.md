<!-- doclint:disable orphan-version -->

# AI Agent First-Hour Receipt — Template v3.3.14 (Kernel v3.3.9-kernel)

> **Template:** Copy this file and rename to `AI_FIRST_HOUR_YYYY-MM-DD.md` to record
> your AI agent's first-hour onboarding results. The goal is to prove that an AI agent
> can autonomously orient and begin work using the platform's structured APIs.

**Agent Type:** [Claude Code | Custom Agent | Other]
**Source kernel:** EffortlessMetrics/Rust-Template@v3.3.9-kernel
**Date:** YYYY-MM-DD
**Session ID:** [optional session identifier]

---

## Summary

- [ ] Environment bootstrap succeeded (`cargo xtask dev-up`)
- [ ] Platform status queried (`/platform/status`)
- [ ] Agent hints retrieved (`/platform/agent/hints`)
- [ ] Context bundle generated (`cargo xtask bundle`)
- [ ] Validation loop executed (`cargo xtask check`)
- [ ] No blocking issues requiring human intervention

**Overall result:** [PASS | PASS WITH ISSUES | FAIL]

---

## Step 1: Environment Bootstrap

**Command:** `cargo xtask dev-up`

**Start time:** HH:MM
**End time:** HH:MM
**Duration:** X minutes

**Outcome:** [OK | ISSUES]

**Substep results:**
- [ ] `doctor` – Environment validated
- [ ] `install-hooks` – Pre-commit hooks installed
- [ ] `kernel-smoke` – Baseline validation passed
- [ ] `ac-status --summary` – AC coverage retrieved

**Environment details:**

```
Rust version: [rustc --version output]
Nix version: [nix --version output]
Platform: [Linux/macOS/Windows]
```

**Notes:**
- [Any warnings or issues during bootstrap]

---

## Step 2: Context Acquisition

### 2.1 Platform Status

**Command:** `curl -s http://localhost:8080/platform/status | jq '.governance'`

**Response (key fields):**

```json
{
  "selftest_status": "[pass|fail]",
  "ac_pass": [number],
  "ac_total": [number],
  "policies_pass": [number],
  "policies_total": [number]
}
```

**Outcome:** [OK | ISSUES]

### 2.2 Agent Hints

**Command:** `curl -s http://localhost:8080/platform/agent/hints | jq '.hints | length'`

**Work items discovered:** [number]

**Top 3 hints:**
1. [Task ID] - [Title] - [Priority]
2. [Task ID] - [Title] - [Priority]
3. [Task ID] - [Title] - [Priority]

**Outcome:** [OK | ISSUES]

### 2.3 Governance Graph

**Command:** `curl -s http://localhost:8080/platform/graph | jq '.governance | {stories: .stories | length, requirements: .requirements | length, acs: .acceptance_criteria | length}'`

**Governance counts:**

```json
{
  "stories": [number],
  "requirements": [number],
  "acs": [number]
}
```

**Outcome:** [OK | ISSUES]

---

## Step 3: Context Bundle Generation

**Command:** `cargo xtask bundle implement_ac`

**Bundle location:** `bundle/implement_ac/`

**Bundle contents:**
- [ ] `bundle.yaml` – Manifest with task_id, requirement_ids, ac_ids
- [ ] `context.md` – Bundled files in markdown format

**Bundle scope:**
- Task ID: [from bundle.yaml]
- Requirement IDs: [list]
- AC IDs: [list]

**Outcome:** [OK | ISSUES]

**Notes:**
- [Any issues with bundle generation]
- [Was the bundle scope appropriate?]

---

## Step 4: Validation Loop

**Command:** `cargo xtask check`

**Substep results:**
- [ ] `fmt` – Format check passed
- [ ] `clippy` – Lint check passed
- [ ] `tests` – Unit tests passed

**Duration:** X minutes

**Outcome:** [OK | ISSUES]

**Notes:**
- [Any test failures or warnings]

---

## Step 5: Decision Capture (if applicable)

**Decisions made during session:**

| # | Decision | Type | Artifact Created |
|---|----------|------|------------------|
| 1 | [Decision description] | [ADR | Issue | Friction] | [Link or ID] |

**Questions encountered:**

| # | Question | Resolution |
|---|----------|------------|
| 1 | [Question] | [How resolved or escalated] |

---

## Friction Encountered

List any friction points that should be fed back:

| # | Friction Description | Severity | Category | Recommendation |
|---|---------------------|----------|----------|----------------|
| 1 | [e.g., API returned unexpected shape] | [low\|medium\|high] | [api\|docs\|tooling] | [Specific fix] |
| 2 | [e.g., Bundle missing expected file] | [low\|medium\|high] | [bundle\|scope] | [Specific fix] |

---

## Metrics

| Metric | Value |
|--------|-------|
| **Total time to first work item** | X minutes |
| **API calls made** | [number] |
| **Bundle generation time** | X seconds |
| **Validation (check) time** | X minutes |
| **Human interventions required** | [0 | number with descriptions] |

---

## Agent Observations

### What worked well

- [List things that worked smoothly]

### What could be improved

- [List areas for improvement]

### Comparison to expected workflow

- [Did the agent follow docs/how-to/ai-first-hour.md?]
- [Any deviations and why?]

---

## Success Criteria

The first-hour onboarding is **successful** when:

- [ ] Agent can reach `cargo xtask check` green without human help
- [ ] Agent discovers work via `/platform/agent/hints`
- [ ] Agent generates focused context via `cargo xtask bundle`
- [ ] Agent understands governance state via `/platform/status`
- [ ] Total time < 30 minutes for full orientation

**This run confirms:** [All criteria met | Criteria X not met]

---

## Follow-up Actions

- [ ] File friction entries for any issues discovered
- [ ] Update agent hints if priority was unclear
- [ ] Report kernel issues if APIs behaved unexpectedly
- [ ] Add to docs/receipts/ for future reference

---

## Signatures

**Agent:** [Agent identifier or session ID]
**Validated by:** [Human reviewer, if applicable]
**Date:** YYYY-MM-DD
