# Truth Labels (required for load-bearing statements)

**Principle:** LLMs may infer; artifacts must disclose.

For any load-bearing statement in a cover sheet, dossier, PR body, or agent handoff,
tag the claim with its provenance:

| Tag | Meaning | Requirements |
|-----|---------|--------------|
| **[OBS]** | Observed | Directly measured from inputs (paths, receipts, test outputs, diff stats) |
| **[DER]** | Derived | Computed from observed (ratios, deltas, histograms). Include method. |
| **[INF]** | Inferred | LLM judgment (design alignment, boundary integrity, risk assessment). Include confidence + evidence pointers. |
| **[REC]** | Recommended | Suggested action. Include rationale + where to apply. |

## Evidence Anchoring

Every inferred ([INF]) or derived ([DER]) claim must include:

- **evidence_pointers**: paths, functions, receipts, commit IDs, hunk references
- **assumptions**: what was assumed to reach the conclusion
- **confidence**: high / medium / low with brief justification

## Bounded Estimates

For core metrics (DevLT, compute band, quality delta):

- Always output a **range** (lower bound, upper bound)
- Widen the range when evidence is weak
- Explicitly state what coverage was missing
- Never output "unknown" — use "low confidence, wide bounds" instead

## Narrative vs Conclusions

- **Narrative can be freeform**: Rich context, analysis, and explanation are encouraged.
- **Conclusions must be structured**: Tagged and anchored for machine parsing and audit.

This prevents both:
- Tiny comments without context (agents write real analysis)
- Eloquent wrongness (load-bearing claims are anchored and verifiable)

## Examples

### Good

```markdown
[OBS] 15 files changed, 412 insertions, 89 deletions across crates/app-http and crates/gov-http.

[DER] ~82% of changes are in async handler paths (ratio derived from file histogram).

[INF] Boundary integrity improved by extracting sync I/O to spawn_blocking.
Confidence: high. Evidence: path:crates/app-http/src/tasks.rs:45-67, commit:83e5647.

[REC] Add integration test for concurrent task operations to verify no executor starvation.
```

### Bad

```markdown
This PR is great and improves performance significantly.
(No tag, no evidence, no confidence — unverifiable claim)
```
