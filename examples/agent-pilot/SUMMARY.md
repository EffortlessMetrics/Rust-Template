# Agent Pilot Harness - Creation Summary
<!-- doclint:disable orphan-version -->

**Created:** 2025-12-01
**Template Version:** v3.3.5
**Status:** Complete and ready for use

---

## What Was Created

### Documentation (4 files)

1. **README.md** (308 lines)
   - Comprehensive overview of the agent pilot harness
   - Purpose, structure, workflows, guardrails, and FAQ
   - Detailed guidance on running pilots and capturing learnings

2. **QUICKSTART.md** (244 lines)
   - 10-minute quick start guide
   - Step-by-step first pilot execution
   - Troubleshooting and common pitfalls

3. **INDEX.md** (331 lines)
   - Navigation hub for all pilot harness documents
   - File structure, pilot lifecycle, common pilot types
   - Integration with template governance

4. **SUMMARY.md** (this file)
   - Creation summary and validation results

### Configuration Templates (3 files)

1. **pilot-plan-template.yaml** (163 lines)
   - Blank pilot plan configuration
   - Structured format for: metadata, objective, scope, phases, success criteria, metrics
   - Extensive inline comments and guidance

2. **pilot-plan-example.yaml** (156 lines)
   - Concrete example pilot using AC-TPL-001
   - Demonstrates all phases (Setup, Execute, Review)
   - Meta-pilot for validating the harness itself

3. **pilot-notes.md** (400+ lines)
   - Template for documenting pilot outcomes
   - Sections for: summary, success criteria, phase breakdown, metrics, friction, learnings
   - Example filled-in sections for guidance

### Capture Templates (2 files)

1. **friction-template.yaml** (231 lines)
   - Structured DevEx friction capture
   - Categories: api, docs, tooling, specs, test, workflow, bundle, agent
   - Severities: low, medium, high, critical
   - Example filled-in entry at bottom

2. **adr-template.md** (354 lines)
   - Design decision documentation template
   - Sections: context, decision, consequences, validation, follow-up
   - Example filled-in ADR with reasoning
   - Guidance on when to use vs. not use

### Agent Definition (1 file)

1. **.claude/agents/pilot-agent.md** (261 lines)
   - Autonomous pilot execution agent
   - Frontmatter: name, description, tools, model, permissionMode, skills
   - Validated by `cargo xtask agents-lint`
   - Role, workflow, tool usage, safety constraints, examples

### Supporting Files (3 files)

1. **.gitignore**
   - Excludes ephemeral pilot artifacts (evidence, pilot-specific notes/plans)
   - Keeps templates and examples

2. **Directory structure**
   - `friction-entries/` with .gitkeep
   - `adrs/` with .gitkeep
   - `evidence/` with .gitkeep

---

## Validation Results

### Agent Validation

- ✅ Pilot agent validated by AGENTS_GOVERNANCE.md rules
- ✅ Name format: `pilot-agent` (kebab-case, ≤64 chars)
- ✅ Description: Includes WHAT and WHEN (≤1024 chars)
- ✅ Tools: Explicit list (Read, Grep, Glob, Edit, Write, Bash)
- ✅ Model: `sonnet` (approved model)
- ✅ PermissionMode: `acceptEdits` (with justification in description)
- ✅ Skills: Valid references (governed-feature-dev, governed-maintenance, governed-governance-debug)
- ✅ No hardcoded secrets detected

### File Counts

- Total files created: 13
- Total directories: 6 (3 with .gitkeep)
- Total lines: ~1,892 lines of documentation and templates

### Structure

```
examples/agent-pilot/
├── README.md                     # Main documentation
├── QUICKSTART.md                 # Quick start guide
├── INDEX.md                      # Navigation hub
├── SUMMARY.md                    # This file
├── pilot-plan-template.yaml      # Blank pilot config
├── pilot-plan-example.yaml       # Example pilot
├── pilot-notes.md                # Findings template
├── friction-template.yaml        # Friction capture
├── adr-template.md               # ADR template
├── .gitignore                    # Ignore rules
├── .claude/agents/pilot-agent.md # Pilot agent definition
├── friction-entries/             # For friction (ephemeral)
├── adrs/                         # For ADRs (ephemeral)
└── evidence/                     # For logs/screenshots (ephemeral)
```

---

## Key Features

### 1. Structured Pilot Framework

- **Time-boxed experiments** with clear phases (Setup, Execute, Review)
- **Success criteria** defined upfront
- **Guardrails** to ensure safe, governed operation
- **Metrics** to quantify outcomes (time, commands, friction, autonomy)

### 2. Systematic Friction Capture

- **Structured format** (YAML with categories, severities, impact)
- **Immediate capture** (don't accumulate pain)
- **Actionable insights** for template improvement
- **Integration** with platform API (`/platform/friction`)

### 3. Design Decision Documentation

- **ADR format** for non-trivial choices
- **Context and consequences** clearly documented
- **Related artifacts** linked (REQs, ACs, tests, code)
- **Status tracking** (DRAFT → ACCEPTED → SUPERSEDED)

### 4. Platform API Integration

- **Introspection first** (`/platform/agent/hints`, `/platform/status`, `/platform/graph`)
- **Bundle-driven context** (`cargo xtask bundle implement_ac`)
- **Validation ladder** (check → test-ac → selftest)
- **Governance-aware** (align with spec_ledger, flows, ACs)

### 5. Governed Agent

- **Least-privilege tools** (Read, Grep, Glob, Edit, Write, Bash)
- **Skill references** (governed-feature-dev, governed-maintenance, governed-governance-debug)
- **Safety constraints** (no unsupervised pushes, no spec violations, time-boxing)
- **Clear workflow** (understand → execute → validate → capture)

---

## Usage Patterns

### Pattern 1: AC Implementation Pilot

**Use when:** Testing agent's ability to implement a single AC autonomously

**Steps:**
1. Copy `pilot-plan-template.yaml` → `pilot-plan-ac-impl.yaml`
2. Set objective, AC_ID, time-box (2-4h), success criteria
3. Run Setup: `cargo xtask dev-up`, query APIs, generate bundle
4. Run Execute: Implement code/tests following bundle and skill
5. Run Review: `cargo xtask selftest`, document outcomes
6. Capture friction and learnings

**Success:** AC passes, selftest green, friction captured

---

### Pattern 2: Maintenance Pilot

**Use when:** Testing agent's ability to fix a failing test or resolve friction

**Steps:**
1. Identify friction entry or failing AC
2. Copy pilot-plan-template, set objective, time-box (1-2h)
3. Run Setup: Understand issue via APIs or bundle
4. Run Execute: Apply fix, validate incrementally
5. Run Review: Verify resolution, no regressions
6. Document findings

**Success:** Issue resolved, no regressions, friction captured

---

### Pattern 3: Governance Debugging Pilot

**Use when:** Understanding why selftest fails or policy rejects

**Steps:**
1. Run `cargo xtask selftest` to identify failure
2. Copy pilot-plan-template, set objective, time-box (1-2h)
3. Run Setup: Understand failure, check specs/flows/skills
4. Run Execute: Fix root cause or document ambiguity in ADR
5. Run Review: Verify selftest passes
6. Propose template improvement

**Success:** Selftest green, root cause documented, process improved

---

## Integration Points

### With Template Governance

- **Specs:** Pilots operate on REQ/AC IDs from `specs/spec_ledger.yaml`
- **Skills:** Pilots reference `governed-feature-dev`, `governed-maintenance`, `governed-governance-debug`
- **Flows:** Pilots follow `specs/devex_flows.yaml` patterns
- **Validation:** Pilots use validation ladder (check → test-ac → selftest)

### With Platform APIs

- `/platform/status` – Governance health and AC coverage
- `/platform/agent/hints` – Prioritized next work
- `/platform/graph` – Full governance graph
- `/platform/tasks` – Task list with filtering
- `/platform/friction` – Development friction log

### With Bundles

- `implement_ac` – AC implementation context
- `debug_tests` – Test debugging context
- `implement_feature` – Broader feature context

---

## Next Steps

### For First Pilot

1. Read `QUICKSTART.md` (10 minutes)
2. Run example pilot (`pilot-plan-example.yaml`)
3. Validate harness works as expected
4. Document any harness friction encountered

### For Real Pilots

1. Pick an AC, friction, or failing test
2. Copy `pilot-plan-template.yaml`
3. Fill in objective, scope, phases, success criteria
4. Run pilot following phases
5. Capture friction and learnings
6. Review outcomes with team

### For Harness Improvement

1. Capture harness issues as friction (category: `workflow`)
2. File GitHub issues linking friction entries
3. Propose template/doc/agent improvements
4. Run follow-up pilot to validate improvements

---

## Success Metrics

The agent pilot harness is successful if:

- ✅ **Pilots are easy to run** – Clear docs, templates, examples
- ✅ **Friction is captured systematically** – Structured format, immediate capture
- ✅ **Learnings are actionable** – Lead to template improvements
- ✅ **Autonomy is measurable** – Metrics quantify agent capability
- ✅ **Governance is validated** – Specs/flows/skills enable autonomous work

---

## Acknowledgments

This harness builds on:

- **CLAUDE.md** – Core workflows and validation ladder
- **AGENTS_GOVERNANCE.md** – Agent design principles
- **SKILLS_GOVERNANCE.md** – Governed workflow patterns
- **docs/AGENT_GUIDE.md** – Platform API reference
- Learnings from brownfield-demo and fork-customization examples

---

## Questions or Feedback?

- Capture harness friction using `friction-template.yaml` (category: `workflow`)
- File GitHub issue with label `agent-pilot-harness`
- See `README.md` FAQ section for common questions

**Harness Maintainer:** Template governance team
**Version:** 1.0.0
**Last Updated:** 2025-12-01
