# Questions Directory

This directory contains structured question artifacts emitted by flows when they encounter ambiguity or missing information.

## Purpose

Instead of failing or stalling when faced with unclear specifications, duplicate IDs, or missing dependencies, flows in this platform emit **question artifacts** that can be reviewed and resolved asynchronously.

## Schema

Questions follow the schema defined in `specs/questions_schema.yaml`. Each question includes:

- **Unique ID**: Pattern `Q-{COMPONENT}-{NUMBER}` (e.g., `Q-TPL-001`)
- **Context**: Which flow generated it and why
- **Related IDs**: Task, requirement, and AC IDs involved
- **Options**: Possible resolution paths with risk levels
- **Recommendation**: Suggested option with rationale
- **Status**: `open`, `answered`, `resolved`, or `obsolete`

## File Format

Questions are YAML files named `Q-{COMPONENT}-{NUMBER}.yaml`:

```yaml
id: Q-BUNDLE-001
task_id: implement_ac
req_ids: [REQ-TPL-SUGGEST-NEXT]
ac_ids: [AC-TPL-SUGGEST-NEXT-CLI]
summary: "Bundle flow found multiple ACs for requirement - unclear priority"
context:
  flow: bundle
  phase: ac_selection
  description: "Detailed explanation of the ambiguity..."
  files_involved: ["specs/spec_ledger.yaml"]
options:
  - label: "Option A"
    description: "..."
    risk: low
    reversible: true
recommendation:
  option_label: "Option A"
  rationale: "..."
  confidence: medium
created_by: flow
created_at: "2025-11-26T00:00:00Z"
status: open
```

## Lifecycle

1. **Creation**: Flows automatically create questions when encountering ambiguity
2. **Review**: Questions surfaced via `cargo xtask questions-list` or `/platform/questions`
3. **Resolution**: Resolved via CLI, HTTP API, or manual YAML editing
4. **Cleanup**: Resolved questions remain for audit trail; obsolete ones can be archived

## Commands

- `cargo xtask questions-list` - List all questions (with optional status filter)
- `cargo xtask question-resolve <ID> --option <label>` - Resolve a question
- `cargo xtask status` - Shows count of open questions

## HTTP Endpoints

- `GET /platform/questions` - List all questions (JSON)
- `GET /platform/questions/{id}` - Get specific question
- `POST /platform/questions/{id}/resolve` - Resolve a question
- `GET /platform/status` - Includes questions section with counts

## Integration with Flows

Questions can be emitted by:

- **bundle**: When AC selection is ambiguous or requirements are unclear
- **suggest-next**: When task dependencies are circular or missing
- **ac-new**: When duplicate AC IDs are detected
- **release-prepare**: When version conflicts or uncommitted changes exist

## Governance Impact

- Open question count is tracked in governance health metrics
- High number of open questions may indicate specification gaps
- Questions serve as input for improving specs and documentation

## Example Workflows

### Flow encounters ambiguity
```bash
# Flow detects ambiguity and emits question
cargo xtask bundle implement_ac
# Output: "⚠️  Question Q-BUNDLE-001 created: Multiple ACs found for requirement"

# Review questions
cargo xtask questions-list --status=open

# Resolve question
cargo xtask question-resolve Q-BUNDLE-001 --option "Implement AC-001 first"
```

### Agent-driven resolution
```bash
# Agent checks platform status
curl http://localhost:8080/platform/questions | jq '.questions[] | select(.status == "open")'

# Agent reviews question and chooses option
curl -X POST http://localhost:8080/platform/questions/Q-BUNDLE-001/resolve \
  -H "Content-Type: application/json" \
  -d '{"option_label": "Implement AC-001 first", "resolved_by": "agent"}'
```

## See Also

- `specs/questions_schema.yaml` - Full schema definition
- `docs/AGENT_GUIDE.md` - How agents should handle questions
- `CLAUDE.md` - Section on handling ambiguity and decisions
