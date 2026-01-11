# Governance Rules

This directory contains governance rules that guide agent behavior in this platform cell.
Rules are numbered for ordering and grouped by concern.

## Rule Categories

| Range | Category | Description |
|-------|----------|-------------|
| 00-19 | Core principles | Fundamental operating stance |
| 20-39 | Evidence & truth | How claims are made and verified |
| 40-59 | CI & gates | Verification infrastructure behavior |
| 60-79 | Artifacts & outputs | How agents produce work products |
| 80-99 | Safety & boundaries | Constraints on agent actions |

## Current Rules

- **35-truth-labels.md** — Required claim labeling for load-bearing statements
- **40-ci-posture.md** — How to handle CI disabled/unavailable states
- **45-semantic-only-merge.md** — LLM vs deterministic field separation in receipts

## Operating Principles (not yet codified as rules)

The following principles guide this rule system:

### Autonomy inside containment

- Default-allow work plane for exploration and implementation
- Strict boundary gates at PR/merge/release points
- Evidence required at exits, not every step

### Quality is the product

- PR maintainability and verification depth are the north star
- Time/cost efficiency are tuning signals, not goals
- Optimize for cold reviewer can validate in 5-10 minutes

### LLMs are non-deterministic narrators

- Treat model output as untrusted unless anchored
- Narrative is welcome; conclusions must be tagged and verifiable
- Artifacts disclose provenance; the kernel is the sensor suite

### Speak broadly, conclude precisely

- Rich analysis and context are encouraged
- Conclusions are structured, tagged, and machine-parseable
- Never output "unknown" — use bounded ranges with confidence

## Adding New Rules

1. Choose appropriate number range for the category
2. Use descriptive kebab-case filename
3. Include clear title, principle statement, and concrete examples
4. Reference existing patterns (ADRs, historian contract, etc.)
5. Run `cargo xtask selftest` to ensure rules don't conflict with governance

## Relationship to Other Artifacts

- **CLAUDE.md** — High-level operating instructions (references these rules)
- **Historian agent** — Implements truth-labels and semantic-merge contracts
- **PR commands** — Reference CI-posture rules for verification sections
- **Receipts** — Conform to semantic-only-merge field separation
