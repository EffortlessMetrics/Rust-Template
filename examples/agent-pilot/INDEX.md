# Agent Pilot Harness Index
<!-- doclint:disable orphan-version -->

**Version:** 1.0.0 (aligned with template v3.3.5)
**Created:** 2025-12-01
**Purpose:** Structured framework for evaluating autonomous agent capabilities

---

## Overview

The agent pilot harness enables controlled, time-boxed experiments to test autonomous agents on real development tasks within the Rust-as-Spec template governance framework.

**Key Value:**
- **Learn what works** – Discover which tasks agents handle well
- **Capture friction** – Systematically document DevEx issues
- **Validate governance** – Test whether specs/flows/APIs enable autonomy
- **Improve iteratively** – Use learnings to refine template and workflows

---

## Quick Navigation

| Document | Purpose | When to Use |
|----------|---------|-------------|
| **[QUICKSTART.md](QUICKSTART.md)** | Get started in 10 minutes | First time running a pilot |
| **[README.md](README.md)** | Comprehensive documentation | Understanding pilot structure |
| **[pilot-plan-template.yaml](pilot-plan-template.yaml)** | Blank pilot configuration | Creating a new pilot |
| **[pilot-plan-example.yaml](pilot-plan-example.yaml)** | Concrete example pilot | Learning by example |
| **[friction-template.yaml](friction-template.yaml)** | DevEx friction capture | Recording friction during pilot |
| **[adr-template.md](adr-template.md)** | Design decision template | Documenting non-trivial choices |
| **[pilot-notes.md](pilot-notes.md)** | Findings summary template | Capturing outcomes and learnings |
| **[.claude/agents/pilot-agent.md](.claude/agents/pilot-agent.md)** | Pilot execution agent | Reference for agent behavior |

---

## File Structure

```
examples/agent-pilot/
├── INDEX.md                          # This file (navigation hub)
├── README.md                         # Comprehensive documentation
├── QUICKSTART.md                     # 10-minute quick start guide
│
├── pilot-plan-template.yaml          # Blank pilot configuration
├── pilot-plan-example.yaml           # Concrete example pilot (AC-TPL-001)
├── pilot-notes.md                    # Template for findings summary
│
├── friction-template.yaml            # Template for DevEx friction capture
├── adr-template.md                   # Template for design decisions
│
├── .claude/
│   └── agents/
│       └── pilot-agent.md            # Autonomous pilot execution agent
│
├── friction-entries/                 # Captured friction (ephemeral)
│   └── .gitkeep
├── adrs/                             # Design decisions (ephemeral)
│   └── .gitkeep
├── evidence/                         # Logs, screenshots, bundle outputs (ephemeral)
│   └── .gitkeep
│
└── .gitignore                        # Exclude ephemeral pilot artifacts
```

**Note:** Directories marked "ephemeral" contain pilot-specific artifacts that should not be committed unless they're generalizable examples or template improvements.

---

## Pilot Lifecycle

### 1. Plan

- Copy `pilot-plan-template.yaml`
- Define objective, scope, time-box, phases, success criteria
- Review with team if needed

### 2. Execute

- **Setup:** Bootstrap environment, query platform APIs, generate bundle
- **Execute:** Implement/fix following governed workflows
- **Review:** Run selftest, document outcomes

### 3. Capture

- **Friction:** Use `friction-template.yaml` for DevEx issues
- **Decisions:** Use `adr-template.md` for design choices
- **Notes:** Use `pilot-notes.md` for summary

### 4. Learn

- Review friction patterns across pilots
- Identify template improvements
- Refine flows, skills, docs, APIs
- Share learnings with team

---

## Common Pilot Types

### Type 1: AC Implementation Pilot

**Objective:** Implement a single AC autonomously
**Time-box:** 2-4 hours
**Skill:** `governed-feature-dev`
**Bundle:** `implement_ac`

**Example:** See `pilot-plan-example.yaml`

**Success Criteria:**
- AC implemented correctly per spec
- BDD scenario passes
- Selftest green
- Friction captured

---

### Type 2: Maintenance Pilot

**Objective:** Fix failing test or resolve friction
**Time-box:** 1-2 hours
**Skill:** `governed-maintenance`
**Bundle:** `debug_tests` or `implement_ac`

**Success Criteria:**
- Issue resolved
- No regressions
- AC status improves
- Friction captured

---

### Type 3: Governance Debugging Pilot

**Objective:** Understand and fix selftest failure
**Time-box:** 1-2 hours
**Skill:** `governed-governance-debug`
**Bundle:** Custom or `implement_feature`

**Success Criteria:**
- Selftest passes
- Root cause documented in ADR
- Process improvement identified
- Friction captured

---

## Key Concepts

### Guardrails

Rules that pilots must follow:

- **Must query** `/platform/agent/hints` for prioritized work
- **Must use bundles** for focused context (avoid scanning entire repo)
- **Must capture friction** immediately (don't accumulate pain)
- **Must run validation ladder** (check → test-ac → selftest)
- **Must time-box** work (stop at limit and document progress)
- **Must NOT push** to remote without explicit instruction

### Success Criteria

Pilots define clear, measurable success criteria:

- AC-level: Does the AC pass?
- Test-level: Do tests pass without regressions?
- Governance-level: Is selftest green?
- Process-level: Was friction captured?
- Autonomy-level: Was manual intervention needed?

### Friction Capture

Systematic DevEx issue tracking:

- **Categories:** api, docs, tooling, specs, test, workflow, bundle, agent
- **Severities:** low, medium, high, critical
- **Structured format:** YAML with id, date, summary, description, impact
- **Immediate capture:** Don't wait until end of pilot

### Platform APIs

Introspection endpoints for agents:

- `/platform/status` – Governance health and AC coverage
- `/platform/agent/hints` – Prioritized next work
- `/platform/graph` – Full governance graph
- `/platform/tasks` – Task list with filtering
- `/platform/friction` – Development friction log

See `docs/AGENT_GUIDE.md` for full API reference.

---

## Metrics to Track

### Quantitative

- Time taken vs. time-box
- Commands executed
- Platform API queries
- Files modified (code/tests/docs/specs)
- Validation attempts and results
- Friction entries created

### Qualitative (1-5 scale)

- Spec/AC clarity
- Bundle quality and focus
- Platform API usefulness
- Documentation clarity
- Tooling effectiveness
- Friction capture ease
- Overall autonomy achieved

---

## Integration with Template

### Governance

- Pilot agent is validated by `cargo xtask agents-lint`
- Follows AGENTS_GOVERNANCE.md and AGENTS_TEMPLATE.md rules
- Uses least-privilege tool set (Read, Grep, Glob, Edit, Write, Bash)
- Operates in `acceptEdits` permission mode with justification

### Skills

- Pilot agent references: `governed-feature-dev`, `governed-maintenance`, `governed-governance-debug`
- Pilots validate whether skills provide sufficient guidance
- Friction captures skill gaps or ambiguities

### Workflows

- Pilots follow `specs/devex_flows.yaml` patterns
- Validate AC-first methodology (understand AC → implement → validate)
- Use validation ladder (check → test-ac → selftest)

### Specs

- Pilots operate on REQ/AC IDs from `specs/spec_ledger.yaml`
- Validate whether specs are clear enough for autonomous work
- Capture spec ambiguities in ADRs or friction entries

---

## Contributing

### Adding a New Pilot

1. Copy `pilot-plan-template.yaml` → `pilot-plan-<NAME>.yaml`
2. Fill in objective, scope, phases, success criteria
3. Run pilot following phases
4. Capture friction and learnings
5. Summarize in `pilot-notes-<NAME>.md`
6. **Only commit if generalizable** (e.g., new pilot type example)

### Improving the Harness

If you discover harness issues during a pilot:

1. **Capture as friction:** Use `friction-template.yaml`
2. **File GitHub issue:** Link friction entry and pilot ID
3. **Propose fix:** Update templates, docs, or agent
4. **Validate fix:** Run another pilot with improvements
5. **Commit improvements:** Update templates, README, or agent

### Promoting Learnings

When friction or decisions are generalizable:

- **Friction → Template improvement:** Update docs, skills, flows, APIs
- **ADR → Production ADR:** Move from `adrs/` to `/docs/adr/`
- **Pattern → Skill:** Codify successful workflow as reusable skill
- **Pilot → Example:** Promote successful pilot to reference example

---

## FAQ

**Q: Should I commit my pilot-plan and pilot-notes?**
A: Only if they're generalizable examples. Most pilots are ephemeral experiments and should stay local or be summarized in issues/ADRs.

**Q: What if my pilot "fails" (doesn't meet success criteria)?**
A: That's still valuable! The goal is to learn. Document what blocked progress, capture friction, and use that to improve the template.

**Q: Can I modify the templates?**
A: Yes! If the templates don't work well, capture that as friction and propose improvements. The harness should evolve based on usage.

**Q: How do I know which bundle to use?**
A: See `.llm/contextpack.yaml` for available bundles. For AC implementation, use `implement_ac`. For debugging, use `debug_tests`. For broad features, use `implement_feature`.

**Q: Should I use the pilot-agent or a different agent?**
A: The pilot-agent is designed for pilots, but you can use any agent. Just reference it in your `pilot-plan.yaml`. You can also create a custom agent for specialized pilots.

**Q: How do I run a pilot with a different agent?**
A: In `pilot-plan.yaml`, set `agent: your-agent-name` in the Execute phase. The agent must exist in `.claude/agents/`.

**Q: Can I run multiple pilots in parallel?**
A: Yes, but use separate branches and pilot-plan files to avoid conflicts. Merge learnings back to main via friction entries, ADRs, or issues.

---

## See Also

- `docs/AGENT_GUIDE.md` – Platform API reference and agent patterns
- `docs/AGENTS_GOVERNANCE.md` – Agent design and validation rules
- `docs/SKILLS_GOVERNANCE.md` – Understanding governed workflows
- `CLAUDE.md` – Core workflows and validation ladder
- `docs/how-to/use-llm-bundles.md` – Bundle usage and customization
- `.llm/contextpack.yaml` – Available bundle tasks

---

## Versioning

This harness follows semantic versioning:

- **Major:** Breaking changes to pilot structure or agent interface
- **Minor:** New features (templates, examples, pilot types)
- **Patch:** Bug fixes, doc improvements, clarifications

**Current Version:** 1.0.0

**Changelog:**
- 1.0.0 (2025-12-01): Initial release with example pilot, templates, and pilot-agent

---

**Feedback:** Capture harness friction using `friction-template.yaml` with category `workflow` and file a GitHub issue.

**Maintainer:** Template governance team
**License:** Same as parent repo
