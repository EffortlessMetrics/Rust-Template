# Agent Pilot Harness
<!-- doclint:disable orphan-version -->

**Purpose:** Run controlled, time-boxed pilots to evaluate autonomous agent capabilities within the Rust-as-Spec template governance framework.

---

## Overview

The agent pilot harness provides a structured environment for testing autonomous agents on real development tasks (AC implementation, maintenance, debugging) while capturing learnings about:

- **What works:** Task types, workflows, and contexts where agents excel
- **What doesn't:** Gaps, ambiguities, tool limitations, and governance friction
- **Improvements needed:** Template changes, better docs, refined flows, clearer specs

Each pilot is governed by a `pilot-plan.yaml` defining scope, time limits, phases, guardrails, and success criteria.

---

## Quick Start

### 1. Define a pilot

Copy and customize `pilot-plan-template.yaml`:

```bash
cd examples/agent-pilot
cp pilot-plan-template.yaml pilot-plan-my-test.yaml
# Edit pilot-plan-my-test.yaml: set objective, AC_IDs, time limit, phases
```

### 2. Bootstrap the pilot environment

```bash
# From repo root
cargo xtask dev-up
cargo run -p app-http &  # Start platform APIs

# From pilot directory
cd examples/agent-pilot
```

### 3. Run the pilot

**Manual mode (recommended for first pilots):**

1. Follow the phases defined in your `pilot-plan.yaml`
2. Use the agent defined in `.claude/agents/pilot-agent.md` (or your custom agent)
3. Document observations in `pilot-notes.md` as you go
4. Capture friction in `friction-entries/` using the template

**Semi-automated mode:**

```bash
# Future: cargo xtask pilot-run pilot-plan-my-test.yaml
# For now: execute commands manually and track progress
```

### 4. Capture learnings

After the pilot completes (or time-boxes out), fill in:

- **Friction entries:** Copy `friction-template.yaml` for each DevEx issue encountered
- **ADR (if needed):** Use `adr-template.md` for design decisions or ambiguities discovered
- **Pilot report:** Summarize outcomes, metrics, and recommendations in `pilot-notes.md`

---

## Pilot Structure

Each pilot follows this structure:

```
examples/agent-pilot/
├── pilot-plan-<NAME>.yaml        # Pilot configuration
├── pilot-notes.md                # Observations and findings
├── friction-entries/             # Friction discovered during pilot
│   └── FRICTION-PILOT-<N>.yaml
├── adrs/                         # Design decisions made during pilot
│   └── ADR-PILOT-<N>.md
└── evidence/                     # Artifacts (logs, screenshots, bundle outputs)
    ├── bundle-output.md
    └── selftest-results.txt
```

---

## Pilot Plan Schema

A `pilot-plan.yaml` defines:

### Metadata

- **pilot_id**: Unique identifier (e.g., `PILOT-001`)
- **objective**: High-level goal in 1-2 sentences
- **scope**: What's in/out of scope (AC IDs, time limit, tasks)

### Phases

Each phase defines:

- **name**: Phase label (Setup, Execute, Review)
- **commands**: Shell commands to run
- **agent**: Agent name to use (from `.claude/agents/`)
- **guardrails**: Rules the agent must follow
- **checkpoints**: Validation criteria before moving to next phase

### Success Criteria

List of conditions that define a successful pilot:

- AC implemented correctly
- All tests pass
- No regressions
- Selftest green
- Clear documentation of findings

---

## Example Pilots

### Pilot 1: Implement a Single AC

**Objective:** Test agent's ability to implement `AC-EXAMPLE-001` end-to-end using the `implement_ac` bundle.

**Scope:**
- 1 AC, 2-hour time limit
- Agent uses `/platform/agent/hints` for guidance
- Agent follows `governed-feature-dev` skill

**Phases:**
1. **Setup:** Generate bundle, query platform APIs, understand AC
2. **Execute:** Implement code + tests, validate incrementally
3. **Review:** Run selftest, document friction, create PR

**Success Criteria:**
- BDD scenario passes
- Selftest green
- No manual intervention required after setup

---

### Pilot 2: Maintenance Task

**Objective:** Test agent's ability to resolve friction log entry or fix failing AC.

**Scope:**
- 1 friction entry or 1 failing AC
- 1-hour time limit
- Agent uses `governed-maintenance` skill

**Phases:**
1. **Setup:** Query `/platform/friction`, understand problem
2. **Execute:** Apply fix, update tests/docs as needed
3. **Review:** Verify fix resolves issue, no new failures

**Success Criteria:**
- Friction resolved
- AC status improves
- Selftest green

---

## Platform API Usage

Agents should use introspection APIs instead of scanning files:

```bash
# Governance health and AC coverage
curl http://localhost:8080/platform/status

# Full governance graph (stories → REQs → ACs → tests → docs)
curl http://localhost:8080/platform/graph

# Prioritized next work for agents
curl http://localhost:8080/platform/agent/hints

# Development friction log
curl http://localhost:8080/platform/friction

# All tasks with filtering
curl http://localhost:8080/platform/tasks?status=Todo
```

See `docs/AGENT_GUIDE.md` for API details.

---

## Guardrails

All pilots operate under these constraints:

### Must Do

- Query `/platform/agent/hints` to understand prioritized work
- Use bundles (`cargo xtask bundle <task>`) for focused context
- Run `cargo xtask ac-status` before and after changes
- Run `cargo xtask selftest` before considering work "done"
- Capture friction as it occurs (don't accumulate pain)
- Leave repo in selftest-green state or document why not

### Must Not Do

- Push to remote without explicit instruction
- Modify specs without linking to REQ/AC
- Create new ACs without spec_ledger entry
- Skip validation steps
- Make breaking changes without ADR

---

## Friction Capture

When an agent encounters friction (unclear docs, missing tools, confusing errors):

1. **Immediate capture:**
   - Copy `friction-template.yaml`
   - Fill in: id, date, category, severity, summary, description
   - Save in `friction-entries/FRICTION-PILOT-<N>.yaml`

2. **Categories:**
   - `api`: Platform API issues
   - `docs`: Missing or unclear documentation
   - `tooling`: xtask, conftest, Nix, CI
   - `specs`: Ambiguous or conflicting specs
   - `test`: Flaky or unclear tests
   - `workflow`: Process or flow issues

3. **Severity:**
   - `low`: Annoyance, doesn't block work
   - `medium`: Slows down work, workaround exists
   - `high`: Blocks progress, requires manual intervention
   - `critical`: Complete blocker, prevents any progress

---

## ADR Capture

When an agent makes a non-trivial design decision during a pilot:

1. Copy `adr-template.md` to `adrs/ADR-PILOT-<N>.md`
2. Fill in:
   - **Context:** What problem or ambiguity was encountered?
   - **Decision:** What choice was made and why?
   - **Consequences:** What are the tradeoffs?
   - **Related ACs:** Link to relevant REQ/AC IDs
3. Mark status as `DRAFT` if incomplete or needing human review

---

## Metrics to Track

For each pilot, capture:

### Quantitative

- **Time:** Actual time vs. time-box limit
- **Commands run:** Count of xtask, cargo, curl invocations
- **API queries:** How many times agent queried platform endpoints
- **Files modified:** Count and categories (code, tests, docs, specs)
- **Validation passes:** check, test-ac, selftest attempts and results

### Qualitative

- **Clarity:** Were specs/ACs clear enough?
- **Tooling:** Did xtask/bundles provide needed context?
- **APIs:** Did platform endpoints surface the right information?
- **Friction:** What caused slowdowns or confusion?
- **Autonomy:** How much human intervention was needed?

---

## Next Steps After Pilot

1. **Review findings:** Summarize pilot-notes.md with team
2. **File issues:** Convert friction entries to GitHub issues or template improvements
3. **Update governance:** Promote DRAFT ADRs to accepted if decision is sound
4. **Refine flows:** Update `specs/devex_flows.yaml` or Skills based on learnings
5. **Iterate:** Run another pilot with adjustments

---

## FAQ

**Q: Can I run multiple pilots in parallel?**
A: Yes, but use separate branches and pilot-plan files to avoid conflicts.

**Q: What if the agent gets stuck?**
A: Time-box it, capture the friction, document what was unclear, and move on. The goal is to learn, not to force completion.

**Q: Should I commit pilot artifacts?**
A: Commit the friction entries and ADRs (if generalizable). Pilot plans and notes can stay in examples/ as documentation, but don't commit large evidence files (logs, bundle outputs).

**Q: How do I know if a pilot was successful?**
A: Check against the success_criteria in your pilot-plan.yaml. Even if not all criteria met, the pilot is valuable if it surfaces friction and learnings.

**Q: Can I use a different agent than pilot-agent.md?**
A: Yes! Create your own agent in `.claude/agents/` or use an existing one. Just reference it in your pilot-plan.yaml.

---

## See Also

- `docs/AGENT_GUIDE.md` – How to use platform APIs and introspection
- `docs/SKILLS_GOVERNANCE.md` – Understanding governed workflows
- `docs/AGENTS_GOVERNANCE.md` – Agent design and validation rules
- `CLAUDE.md` – Core workflows and validation ladder

**Pilot Template Version:** 1.0.0 (aligned with template v3.3.5)
