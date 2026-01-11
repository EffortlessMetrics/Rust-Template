# Semantic-Only Merge Rule (Historian Contract)

LLM agents may only populate semantic fields via appendix. Deterministic telemetry
is never overwritten by LLM output.

## Principle

The Historian and similar analysis agents follow "speak broadly, conclude precisely":

- **Narrative can be expansive**: Rich analysis, context, and interpretation
- **Appendix is structured**: Machine-parseable JSON with explicit fields
- **Telemetry is deterministic**: LOC, churn, coverage — computed by tools, not LLMs

## Field Categories

### Semantic Fields (LLM may populate)

| Field | Type | Description |
|-------|------|-------------|
| `boundary_rating` | enum | `improved` / `neutral` / `degraded` |
| `boundary_notes` | string[] | Design alignment observations |
| `test_depth_rating` | enum | `hardened` / `mixed` / `shallow` |
| `test_depth_notes` | string[] | Verification quality observations |
| `risk_notes` | string[] | Potential issues and concerns |
| `assumptions` | string[] | What was assumed in analysis |
| `evidence_pointers` | string[] | `path:`, `commit:`, `function:` refs |
| `confidence` | enum | `high` / `medium` / `low` |

### Telemetry Fields (never LLM-populated)

| Field | Source | Description |
|-------|--------|-------------|
| `loc_delta` | tokei/scc | Lines of code change |
| `files_changed` | git | Number of files modified |
| `unsafe_delta` | cargo-geiger | Unsafe block count change |
| `coverage_pct` | tarpaulin/llvm-cov | Test coverage percentage |
| `churn_score` | git log | Historical modification frequency |
| `mutation_score` | cargo-mutants | Mutation testing score |

## Merge Behavior

When `receipts-quality --llm` or similar tools merge LLM analysis:

1. **Parse appendix** between `<!-- historian:appendix:start -->` and `<!-- historian:appendix:end -->`
2. **Extract semantic fields only** from the JSON
3. **Preserve existing telemetry** — never overwrite with LLM values
4. **Validate enum values** — reject invalid ratings
5. **Record provenance** — mark fields as `llm_populated: true`

## Fallback Behavior

If appendix is missing or invalid:

- Emit valid receipt with `confidence: low`
- Record assumption: "LLM analysis unavailable or malformed"
- Never fabricate semantic ratings — use `null` for unknown

## Example Appendix

```markdown
<!-- historian:appendix:start -->
{
  "boundary_rating": "improved",
  "boundary_notes": ["[INF] Async handlers now delegate blocking I/O to spawn_blocking"],
  "test_depth_rating": "hardened",
  "test_depth_notes": ["[INF] Regression tests verify no executor starvation"],
  "risk_notes": [],
  "assumptions": ["Tokio runtime is multi-threaded"],
  "evidence_pointers": ["path:crates/app-http/src/tasks.rs:45", "commit:83e5647"],
  "confidence": "high"
}
<!-- historian:appendix:end -->
```

## Why This Matters

This separation prevents:

- **Hallucinated metrics**: LLMs cannot claim coverage percentages they didn't measure
- **Evidence contamination**: Telemetry remains reproducible by re-running tools
- **Audit confusion**: Clear provenance for every field in quality receipts
