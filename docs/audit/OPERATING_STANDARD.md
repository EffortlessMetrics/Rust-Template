---
id: GUIDE-OPERATING-STANDARD-001
title: Operating Standard for Trusted Change
doc_type: guide
status: published
audience: contributors, reviewers, agents
tags: [audit, trust, governance, receipts, devlt, quality, maintainability, historian]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-DOCS-CONSISTENCY]
acs: [AC-PLT-009, AC-PLT-010]
adrs: []
last_updated: 2026-01-07
---

# Operating Standard for Trusted Change

This is the canonical reference for "how to do this right" in an AI-native repo.

---

## 1. What We're Building

Not "pretty PRs." We're building a **publishing system for trusted change**.

- **Generation is cheap and noisy.**
- **Trust is expensive and scarce.**
- The PR surface makes trust **legible, reproducible, and hard to lie about**.

The unit of work isn't "code written." It's:

> **A decision-ready change with an evidence pack.**

---

## 2. Two Core Truths

### A) GitHub's UI Is the User

If the PR body says "green" but the observable surface reads "red/unknown," trust is lost immediately.

Pick a posture and be consistent:

- **Local gate canonical**: Show the local reproduction path and receipts.
- **GitHub checks canonical**: Emit at least one authoritative check run matching gate outcome.

Mixing these is a credibility leak.

### B) DevLT = Decision Time

In AI-native repos, dev effort doesn't correlate with LOC. It correlates with:

- Decisions surfaced
- Constraints set
- Corrections made
- Acceptance granted

**DevLT = human decision minutes per trusted change**, not "hours coding."

Because some decisions happen in Claude Code UX (not GitHub), treat DevLT as:

- **measured** when possible
- **estimated** when necessary
- **always labeled** with a basis

---

## 3. Truth Surface Architecture

### Layering (this matters more than prose quality)

| Layer | Purpose |
|-------|---------|
| **PR Cover Sheet** | Human interface — short, stable, review-oriented, idempotent |
| **Receipts** | Audit interface — structured JSON, validated against schemas, proof anchor |
| **Thread / commits** | Flight recorder — valuable, but not the entry point |

This separation keeps the repo honest *and* readable.

---

## 4. Non-Negotiable Rules

### Rule 1: No Claim Without a Pointer

If you say "tests passed," link to:

- `receipts/gate.json` or equivalent
- A check run linking to receipts

If you can't point, you don't claim. Say "unknown" or "not captured."

### Rule 2: Wrongness Is First-Class

Don't hide mistakes — **serialize them**.

Cover sheets have an **Errata** section:

- What was wrong
- How it was detected
- How it was fixed
- What prevention was added

This turns "AI got it wrong" into "the factory caught it."

### Rule 3: Idempotent Updates or Drift

PR updates must be bounded:

- Replace only the cover sheet block between markers
- Leave everything else untouched
- Save a version-controlled copy under `docs/audit/EXHIBITS/PR-<n>.md`

### Rule 4: Measured vs Estimated Must Be Explicit

Every human-time and compute metric needs:

- **value** (optional)
- **confidence** (measured/estimated/unknown)
- **basis** (where the estimate came from)

Without this, your "ledger" becomes vibe-based performance theatre.

### Rule 5: Templates Are Guidance; Gates Are Enforcement

PR/issue templates should be lightweight and helpful.

Gates enforce:

- Receipts exist (or PR declares why not)
- Schemas validate
- Docs/contract checks pass

Don't proxy "people filled out headings" for correctness.

---

## 5. Concrete Artifact Set

### Docs

| Path | Purpose |
|------|---------|
| `docs/audit/AUDIT_PATH.md` | 15-minute verification route |
| `docs/audit/PROVENANCE.md` | Automated vs human; trust model |
| `docs/audit/PR_COVER_SHEET.md` | Canonical cover sheet format |
| `docs/audit/RECEIPTS.md` | Receipt types + meaning + schemas |
| `docs/audit/FAILURE_MODES.md` | Taxonomy of "how things go wrong" |
| `docs/audit/CASEBOOK.md` | Curated exhibits |
| `docs/audit/EXHIBITS/` | Version-controlled PR cover sheets |

### Receipts + Schemas

Receipts are JSON, validated by JSON Schema, and treated as the proof anchor:

| Receipt | Schema | Purpose |
|---------|--------|---------|
| `receipts/gate.json` | `specs/schemas/gate.schema.json` | What ran, pass/fail, durations, environment |
| `receipts/economics.json` | `specs/schemas/economics.schema.json` | DevLT + compute + iterations + confidence |
| `receipts/dossier.json` | `specs/schemas/dossier.schema.json` | Scope map, findings, exhibit score |
| `receipts/quality.json` | `specs/schemas/quality.schema.json` | Contract changes, boundary integrity, verification depth, risks |
| `receipts/telemetry.json` | `specs/schemas/telemetry.schema.json` | Normalized hard probe outputs (tool measurements) |
| `receipts/timeline.json` | `specs/schemas/timeline.schema.json` | Temporal topology, friction zones, convergence |

Receipt validation automatically maps `<name>.json` to `<name>.schema.json`. Any new receipt type just needs a corresponding schema file.

### Ephemeral vs Permanent

| Location | Retention | Purpose |
|----------|-----------|---------|
| `.runs/` | Ephemeral (gitignored) | Working artifacts during PR |
| `docs/audit/EXHIBITS/` | Permanent (committed) | Curated history |

---

## 6. Canonical Workflow

The minimum viable "truth surface loop":

```bash
# 1. Run the gate → write receipts/gate.json
cargo xtask receipts-gate --pr <n>

# 2. Record DevLT/compute → write receipts/economics.json
cargo xtask receipts-economics --pr <n> \
  --author-minutes 30 --author-confidence estimated \
  --compute-usd 5.00 --compute-confidence estimated

# 3. Generate quality metrics → write receipts/quality.json
cargo xtask receipts-quality --pr <n>

# 4. Generate telemetry (hard probes) → write receipts/telemetry.json
cargo xtask receipts-telemetry --pr <n> --profile full

# 5. Generate timeline (temporal topology) → write receipts/timeline.json
cargo xtask receipts-timeline --pr <n>

# 6. Validate all receipts against schemas
cargo xtask receipts-validate --dir .runs/current

# 7. Generate cover sheet from receipts
cargo xtask pr-cover --pr <n>

# 8. Update PR body with bounded replacement
cargo xtask pr-update --pr <n>

# 9. Save the cover sheet into exhibits
cargo xtask pr-update --pr <n> --save-exhibit
```

When this is routine, archaeology stops being artisanal.

---

## 7. PR Archaeology (Structured Scan)

"Read the diff" doesn't scale. Use a structured scan:

### Pass 1: Scope Map

- Directory histogram
- Hotspots
- Additive vs cross-cutting

### Pass 2: Claim Integrity

- Contradictions between prose and artifacts
- Perf "multipliers" / measurement drift
- Docs that look executable but aren't

### Pass 3: Proof Pack

- Do receipts exist?
- Do schemas validate?
- Are claims derived from receipts?

### Pass 4: Convergence + Prevention

- What broke
- What caught it
- What changed to prevent recurrence

Output = dossier + cover sheet + (optionally) a backlog item if the failure mode repeats.

---

## 8. Measurement Integrity

"× faster" claims are radioactive unless semantics are pinned.

**Prefer:**

- **Absolute metrics** (p50/p95, units)
- **Pinned baselines** (dataset hash, semantics version, reference commit)
- **Denominator drift as incident** (instrumentation issue, not feature)
- **Prevention gates** (rather than arguing about the number)

This applies to: perf, coverage, "AC counts," any derived ratio.

---

## 9. Docs-as-Spec Governance

The system's strength is also its constraint:

- `specs/doc_index.yaml` ↔ doc frontmatter must match
- Doc policies can require design docs for kernel REQs
- Version linting flags hardcoded versions in example payloads

**Posture:**

- Treat doc metadata as part of the kernel
- Keep examples version-agnostic (`vX.Y.Z`, `1.xx.0`)
- Wire every "kernel requirement" to at least one design/plan doc

---

## 10. Economic Framing

The mistake framing: "machine vs human."

The correct framing:

> **Total cost of trusted change = DevLT (dominant) + compute spend (lever) + rework risk (avoided).**

**Publicly emphasize:**

- DevLT bands + confidence
- Proof completeness
- Design alignment

**Privately track:**

- Compute bands + confidence
- Iteration counts
- Recurring failure modes
- Prevention ROI (which gate reduced future DevLT)

---

## 11. What "Doing It Right" Looks Like

A cold reviewer can:

- Open a PR and understand scope in 60 seconds
- Validate trust in 5–10 minutes by following receipt links
- See "what went wrong" when it did, and how prevention was added
- Reproduce the gate locally with one command
- Never be asked to believe prose

That's the whole point.

---

## 12. LLM Analysis Contract (Historian)

When LLM agents (Historian, casebook generators, PR analyzers) produce analysis, they must follow this contract: **speak broadly, conclude precisely**.

> **Narrative comes first.** Tags/anchors are for load-bearing claims, not for every sentence. The model writes naturally; tagging is for the claims that affect conclusions and recommendations.

### The Rule: "LLMs May Infer; Artifacts Must Disclose"

Every claim in a dossier, cover sheet, or analysis artifact should be explicitly labeled:

| Tag | Meaning |
|-----|---------|
| **[OBS]** | Observed — directly measured from inputs (diff stats, test runs, receipts) |
| **[DER]** | Derived — computed from observed (ratios, deltas, histograms) |
| **[INF]** | Inferred — LLM judgment (design alignment, boundary integrity, likely issues) |
| **[REC]** | Recommended — suggested follow-ups, refactors, tests |

The model writes naturally, but tags **load-bearing statements** and anchors them.

### Evidence Anchoring

Every inferred or derived claim should include:

- **`evidence_pointers`**: paths, functions, receipts, commit IDs, hunk references
- **`assumptions`**: what the model assumed to reach the conclusion
- **`confidence`**: high / medium / low with brief justification

Example:
> The refactor reduces coupling by moving persistence concerns behind `FooRepo`. **[INF]**
> *(Pointers: `src/service/foo.rs:42`, `src/repo/foo_repo.rs`; confidence: medium)*

### Bounded Estimates, Never "Unknown"

For core metrics (DevLT, compute band, quality delta):

- Always output a **range** (`lb_minutes`, `ub_minutes`)
- Widen the range when evidence is weak
- Explicitly state what coverage was missing (e.g., "no session trace")

The failure mode becomes "low confidence, wide bounds" — not "unknown."

### Quality-First Metric Stack

Quality is the primary measure. Economics are tuning signals.

**Hard metrics (tooling; repeatable):**

| Category | Metrics |
|----------|---------|
| Change surface | churn, files changed, hotspots, modules touched |
| Contract surface | schema diffs, public API deltas, CLI surface changes |
| Safety markers | `unsafe` delta, concurrency primitives, dependency delta |
| Verification | test delta, coverage (only if actually measured) |

**Semantic metrics (LLM; structured):**

| Category | What to assess |
|----------|---------------|
| Design alignment | Does implementation respect ADR/plan constraints? Where does it drift? |
| Boundary integrity | Are adapters at edges? Is domain leaking to IO? Are modules cohesive? |
| Test depth | Do tests assert behavior or just presence? Are error paths exercised? |
| Doc drift | Are docs updated atomically? Any claims now false? |

### Quality Receipt Shape

```json
{
  "quality": {
    "contract": {
      "public_api": { "changed": false, "breaking": false, "evidence": ["..."] },
      "schema": { "changed": true, "breaking": false, "evidence": ["..."] }
    },
    "boundaries": {
      "modules_touched": 4,
      "hotspots": ["src/a.rs", "src/b/mod.rs"],
      "llm_assessment": {
        "rating": "improved|neutral|degraded",
        "notes": ["..."],
        "confidence": "medium",
        "evidence": ["..."]
      }
    },
    "verification": {
      "tests_added_loc": 320,
      "impl_added_loc": 110,
      "test_density_delta": 2.9,
      "llm_test_depth": {
        "rating": "hardened|mixed|shallow",
        "notes": ["..."],
        "confidence": "medium"
      }
    },
    "risks": {
      "unsafe_delta": { "added": 0, "removed": 0 },
      "deps_added": ["..."]
    }
  }
}
```

### Historian Output Structure

1. **Narrative report** (freeform, high-signal)
   - Executive summary, maintainability, design alignment, verification depth, risks, recommendations
   - Full paragraphs welcome — don't be terse

2. **Evidence & assumptions ledger** (short, explicit)
   - Key pointers: 5–15 anchors (file paths, functions, receipts, commits)
   - Assumptions: only those that materially affect conclusions
   - Confidence notes

3. **Structured appendix** (machine-friendly JSON)
   - Index for downstream automation and dashboarding

### Versioning

Re-analysis is expected. Every analysis should carry:

- `method_id`: e.g., `devlt_est_v2:decision_weighted`
- `method_version`: version of the estimation algorithm
- `analysis_run_id`: unique identifier for this run

Prior analyses are appended, not overwritten, so "we re-analyzed and tightened the bounds" is visible and auditable.

### Probe Profiles

Probes are organized into three profiles to balance thoroughness vs speed:

| Profile | Use Case | Probes Included |
|---------|----------|-----------------|
| `fast` | Always-on, cheap | churn/hotspots, contract flags, existing gate/economics |
| `full` | Serious PRs | + geiger, rust-code-analysis, cargo-modules, public-api diff |
| `exhibit` | Casebook/release | + mutation testing, coverage, semver-checks, deny/advisories |

Telemetry receipts always record which probes ran and which were marked `not_run` with reasons.

---

## 13. Next Steps That Compound

1. **Dossier generation** (`pr-dossier`)
   - Structured facts + findings + references + exhibit score
   - Emits `receipts/dossier.json` validated by schema

2. **Casebook generation** (`casebook-gen`)
   - Generates/updates `docs/audit/CASEBOOK.md` from dossiers
   - Makes exhibit selection mechanical, not taste-driven

3. **Backlog extraction**
   - Aggregates failure modes across dossiers
   - Creates issues/friction entries for top recurring prevention targets

4. **Optional GitHub truth surface**
   - Single "Swarm Gate" check-run posting receipt validation result
   - Only if GitHub UI should reflect canonical truth

---

## Related Docs

- [Audit Path (15 Minutes)](AUDIT_PATH.md) — Quick verification guide
- [Receipts Schema and Usage](RECEIPTS.md) — Schema reference
- [PR Cover Sheet Format](PR_COVER_SHEET.md) — Canonical format
- [Casebook](CASEBOOK.md) — Curated exhibits
- [Failure Modes](FAILURE_MODES.md) — What went wrong and how we hardened
- [Provenance](PROVENANCE.md) — Trust model and claim-backing rules
