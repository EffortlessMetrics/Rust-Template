---
id: DESIGN-TPL-AGENTS-GOVERNANCE-001
title: Agents Governance Framework
author: governance-system
doc_type: design_doc
date: 2025-11-28
status: published
stories: [US-TPL-PLT-001]
requirements: [REQ-TPL-AGENTS-GOVERNANCE]
tags: [platform, agent, governance, devex]
acs: [AC-TPL-AGENTS-LINT, AC-TPL-AGENTS-FRONTMATTER]
adrs: [ADR-0004, ADR-0020, ADR-0021]
---

# Agents Governance Framework

## Problem

Agents (in `.claude/agents/`) are specialized tools that extend platform capabilities. Without governance, agents may be improperly configured, expose secrets, or reference non-existent tools, creating safety and consistency issues.

## Solution

Implement a comprehensive lint framework that validates:
1. **Frontmatter integrity**: All agents have valid YAML front-matter (name, model, permissionMode, etc.)
2. **Enum validation**: Fields like `model` and `permissionMode` use defined enums
3. **Reference integrity**: All referenced skills, custom tools, and MCP servers exist
4. **Secret detection**: No hardcoded API keys, tokens, or passwords
5. **Tool manifest**: Optional; validates tool references if present

## Implementation Approach

- **Lint tool**: `cargo xtask agents-lint` performs structural and semantic checks
- **Integration**:
  - Runs in `cargo xtask selftest`
  - Runs in `cargo xtask precommit` when agents change
  - Integrated into CI via `.github/workflows/ci-agents.yml`
- **Error levels**:
  - **Hard errors**: Frontmatter malformed, missing required fields, secrets detected, dangling references
  - **Warnings**: Best-practice hints (doc completeness, tool count heuristics)
- **Heuristics**: Optional (disabled by default); quality guidance in AGENTS_VALIDATION.md, not in the lint

## Notes

- Agents are powerful; lint enforces safety-critical checks only
- See `.claude/agents/*/AGENT.md` for examples
- Opinionated heuristics about tool count or model choice are guidance, not rules
