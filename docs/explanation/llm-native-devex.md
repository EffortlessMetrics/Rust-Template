# Explanation: LLM-Native Developer Experience

This document explains how the Rust-as-Spec platform cell is designed to make LLM-native development safe and effective.

---

## What is LLM-Native DevEx?

LLM-native developer experience (DevEx) means designing repository structure, tooling, and contracts specifically to support AI agents as first-class development participants alongside human developers.

Key principles:
- **Bounded context**: AI agents work within well-defined scopes with explicit boundaries
- **Structured APIs**: Machine-readable endpoints and CLI outputs for agent consumption
- **Verifiable changes**: Every change can be validated against acceptance criteria
- **Clear contracts**: Explicit specifications prevent hallucination-driven drift

---

## Key Components

### 1. Context Bundling (`.llm/contextpack.yaml`)

The bundler generates focused context packages for specific development tasks:

```bash
cargo xtask bundle implement_ac    # Context for implementing an AC
cargo xtask bundle fix_failing_ac  # Context for debugging a failing test
cargo xtask bundle explore         # General exploration context
```

**Why bundling matters:**
- Prevents context window overflow with irrelevant files
- Focuses agent attention on the relevant code paths
- Includes spec references, test files, and related documentation

### 2. Agent Hints API (`/platform/agent/hints`)

The platform exposes prioritized work suggestions:

```bash
curl http://localhost:8080/platform/agent/hints | jq '.hints[:3]'
```

Hints are ranked by:
- Task priority (failing tests > pending work > improvements)
- Dependency ordering (blocked tasks deprioritized)
- Scope clarity (well-specified tasks ranked higher)

### 3. Governance Graph (`/platform/graph`)

The full governance graph is queryable:

```bash
curl http://localhost:8080/platform/graph | jq '.governance.stories | length'
```

This enables agents to:
- Navigate from stories → requirements → ACs → tests
- Understand what's tested and what's not
- Find related documentation for any AC

### 4. CLI JSON Outputs

All xtask commands support `--json` for machine-readable output:

```bash
cargo xtask ac-status --json          # AC coverage as JSON
cargo xtask friction-list --json      # Friction entries as JSON
cargo xtask suggest-next --format json # Suggested tasks as JSON
```

---

## Repo Layout for LLM Safety

The repository structure is designed to make scope boundaries explicit:

```
specs/
├── spec_ledger.yaml     # Stories, REQs, ACs - the specification
├── features/            # BDD scenarios - the executable spec
└── platform_schema.yaml # API contracts

crates/
├── core/                # Pure domain logic (no I/O)
├── app-http/            # HTTP handlers (thin layer)
└── acceptance/          # BDD step definitions

.llm/
├── contextpack.yaml     # Bundle definitions
└── bundles/             # Generated context files
```

**Key insight:** An agent implementing an AC should primarily touch:
1. The spec ledger (to understand requirements)
2. Feature files (to write/update scenarios)
3. The relevant crate (to implement behavior)

This prevents agents from making changes that cross architectural boundaries.

---

## Validation Loop

The AC-first workflow provides natural checkpoints:

1. **Define**: Add AC to `spec_ledger.yaml` (agent or human)
2. **Specify**: Write BDD scenario with `@AC-XXX` tag
3. **Implement**: Write code to satisfy the scenario
4. **Verify**: `cargo xtask test-ac AC-XXX` validates the change
5. **Gate**: `cargo xtask selftest` ensures nothing broke

Agents can run these commands and interpret their output to self-correct.

---

## Agent Skills Integration

The `.claude/skills/` directory contains governed workflow recipes that agents can invoke:

- `bootstrap-dev-env` - Environment setup
- `governed-feature-dev` - AC-first feature implementation
- `governed-maintenance` - Platform upkeep tasks

Skills provide guardrails that prevent common agent mistakes.

---

## Best Practices for Agent Development

1. **Start with hints**: Query `/platform/agent/hints` before diving into code
2. **Use bundles**: Generate context before implementing (`cargo xtask bundle`)
3. **Test incrementally**: Run `cargo xtask test-ac` after each change
4. **Respect boundaries**: Don't modify governance files unless that's the task
5. **Capture friction**: If something is unclear, log it via `cargo xtask friction-new`

---

## See Also

- **[AGENT_GUIDE.md](../AGENT_GUIDE.md)** - Operational guide for LLMs
- **[how-to/use-llm-bundles.md](../how-to/use-llm-bundles.md)** - Bundle generation guide
- **[how-to/ai-first-hour.md](../how-to/ai-first-hour.md)** - Agent onboarding walkthrough
- **[reference/xtask-commands.md](../reference/xtask-commands.md)** - CLI reference
