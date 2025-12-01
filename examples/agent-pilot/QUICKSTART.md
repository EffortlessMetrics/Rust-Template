# Agent Pilot Quick Start

Get up and running with your first agent pilot in 10 minutes.

---

## Prerequisites

- Nix installed (`nix develop` works)
- Repo cloned and environment bootstrapped (`cargo xtask dev-up`)
- Familiarity with `CLAUDE.md` and basic xtask commands

---

## 5-Minute Setup

### 1. Navigate to pilot directory

```bash
cd examples/agent-pilot
```

### 2. Review the example pilot

```bash
cat pilot-plan-example.yaml
```

This example pilot uses AC-TPL-001 (health check) to validate the harness itself.

### 3. Start platform APIs

```bash
# From repo root
cd ../..
cargo run -p app-http &
sleep 3
curl http://localhost:8080/platform/status
```

You should see JSON with governance health and AC coverage.

### 4. Review templates

```bash
cd examples/agent-pilot
cat friction-template.yaml    # For capturing DevEx issues
cat adr-template.md            # For design decisions
cat pilot-notes.md             # For summarizing outcomes
```

---

## Run Your First Pilot (Manual Mode)

### Phase 1: Setup (5 minutes)

```bash
# Bootstrap environment
cargo xtask dev-up

# Understand the work
curl http://localhost:8080/platform/agent/hints | jq '.'
curl http://localhost:8080/platform/status | jq '.governance'

# Generate context bundle
cargo xtask bundle implement_ac

# Baseline AC coverage
cargo xtask ac-status
```

**Checkpoint:** Platform APIs running, bundle generated, AC-TPL-001 status is "Passing"

### Phase 2: Execute (20 minutes)

For this example pilot, you're validating the workflow, not actually coding:

```bash
# Read the bundle context
cat bundle/implement_ac/context.md | less

# Review the health check implementation
cat crates/app-http/src/handlers/health.rs

# Review the BDD scenario
grep -A 10 "@AC-TPL-005" specs/features/template_core.feature

# Validate tests pass
cargo xtask ac-tests AC-TPL-001
cargo xtask check
```

**Simulate friction capture:**

```bash
# Copy friction template
cp friction-template.yaml friction-entries/FRICTION-PILOT-001.yaml

# Edit with your favorite editor to document a (real or simulated) friction point
# Example: "Bundle included too many files, making it hard to find the relevant handler"
```

**Checkpoint:** Bundle reviewed, tests pass, at least one friction entry created

### Phase 3: Review (5 minutes)

```bash
# Full governance validation
cargo xtask selftest

# Verify AC status unchanged
cargo xtask ac-status

# Document findings
cp pilot-notes.md pilot-notes-example-001.md
# Edit pilot-notes-example-001.md with summary, metrics, learnings

# Save evidence
cargo xtask selftest > evidence/selftest-results.txt
```

**Checkpoint:** Selftest green, friction captured, pilot notes documented

---

## What You Just Validated

✅ **Environment setup** – `cargo xtask dev-up` bootstraps cleanly
✅ **Platform APIs** – Introspection endpoints provide useful governance data
✅ **Bundles** – Context packing works and is scoped appropriately
✅ **Validation ladder** – `check → ac-tests → selftest` catches issues
✅ **Friction capture** – Template is usable and structured correctly
✅ **Documentation** – Pilot harness is clear enough to follow

---

## Next Steps

### Run a Real Pilot

1. Pick an AC to implement (or a friction to resolve)
2. Copy `pilot-plan-template.yaml` → `pilot-plan-my-test.yaml`
3. Fill in objective, scope, phases, success criteria
4. Run the pilot following the phases
5. Capture friction and learnings as you go
6. Review and document outcomes

### Common Pilot Types

**Type 1: AC Implementation**
- Objective: Implement a single AC autonomously
- Time-box: 2-4 hours
- Skill: `governed-feature-dev`
- Success: AC passes, selftest green, friction captured

**Type 2: Maintenance Task**
- Objective: Fix a failing test or resolve friction
- Time-box: 1-2 hours
- Skill: `governed-maintenance`
- Success: Issue resolved, no regressions, friction captured

**Type 3: Governance Debugging**
- Objective: Understand and fix a selftest failure
- Time-box: 1-2 hours
- Skill: `governed-governance-debug`
- Success: Selftest green, root cause documented

---

## Tips for Successful Pilots

### Do

- ✅ Query platform APIs before scanning files
- ✅ Use bundles for focused context
- ✅ Capture friction immediately (don't accumulate pain)
- ✅ Run validation incrementally (test-changed, then ac-tests, then selftest)
- ✅ Time-box ruthlessly (stop at limit and document progress)
- ✅ Document learnings even if pilot "fails"

### Don't

- ❌ Skip platform API queries (they're fast and authoritative)
- ❌ Try to implement multiple ACs in one pilot (keep scope small)
- ❌ Push to remote without explicit instruction
- ❌ Work beyond time-box trying to force completion
- ❌ Forget to document friction (it's half the value!)

---

## Troubleshooting

### Platform APIs not responding

```bash
# Check if service is running
curl http://localhost:8080/health

# If not, start it
cargo run -p app-http &
sleep 3
```

### Bundle command fails

```bash
# Check bundle configuration
cat .llm/contextpack.yaml

# Ensure specs exist
ls specs/spec_ledger.yaml
ls specs/features/
```

### Selftest fails unexpectedly

```bash
# Run individual checks to isolate issue
cargo xtask check
cargo xtask bdd
cargo xtask skills-lint
cargo xtask agents-lint

# Check git status (pre-commit hooks may auto-fix)
git status
```

### Friction template is unclear

See the example filled-in section at the bottom of `friction-template.yaml` for a concrete example.

---

## Questions?

- Read `README.md` for comprehensive pilot harness documentation
- See `docs/AGENT_GUIDE.md` for platform API details
- See `CLAUDE.md` for core workflows and validation ladder
- File a friction entry if you encounter pilot harness issues!

---

**Quick Start Version:** 1.0.0 (aligned with template v3.3.5)
