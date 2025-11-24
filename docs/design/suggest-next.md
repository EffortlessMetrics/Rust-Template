---
doc_type: design_doc
id: DESIGN-TPL-SUGGEST-NEXT-001
title: "Task-Aware Next-Step Suggestions"
stories: ["US-TPL-PLT-001"]
requirements: ["REQ-TPL-SUGGEST-NEXT"]
acs: ["AC-TPL-SUGGEST-NEXT-CLI", "AC-TPL-SUGGEST-NEXT-HTTP"]
adrs: ["ADR-0001"]
status: draft
last_reviewed: 2025-11-22
owner: "platform"
---

# Task-Aware Next-Step Suggestions

## 1. Problem

Developers and AI agents need guidance on what to do next when working on a task. Currently, the platform defines tasks in `specs/tasks.yaml` and workflows in `specs/devex_flows.yaml`, but there's no bridge between them. Users must manually determine which commands to run and in what order, leading to missed steps, workflow violations, and inefficient onboarding.

## 2. Solution

Provide a `suggest-next` feature that reads a task's `recommended_flows` field from `tasks.yaml`, looks up the corresponding flow in `devex_flows.yaml`, and returns a structured sequence of commands and edits tailored to that task's context. This creates a "guided mode" where the platform tells you exactly what to do next.

## 3. Implementation Approach

**CLI Command**: `cargo xtask suggest-next --task <ID>`
- Loads task from `specs/tasks.yaml`
- Reads `recommended_flows` field (e.g., `["ac_first"]`)
- Looks up flow definition in `specs/devex_flows.yaml`
- Renders a structured step-by-step guide with commands and expected outcomes

**HTTP Endpoint**: `GET /platform/tasks/suggest-next?task=<ID>`
- Returns JSON payload with task metadata and `recommended_sequence` array
- Each step includes: command, description, expected output, next action
- Enables web UI and agent consumption

**Data Flow**:
1. Task defines `recommended_flows: ["ac_first"]`
2. Flow `ac_first` maps to commands: `ac-new`, `bundle implement_ac`, `bdd`, `selftest`
3. Suggest-next interpolates task-specific values (AC IDs, requirement IDs) into command templates
4. Returns contextualized sequence: "Run `cargo xtask ac-new AC-TPL-SUGGEST-NEXT-CLI ...`"

## 4. Edge Cases & Validation

- Task with no `recommended_flows`: suggest default discovery flow (`help-flows`, `tasks-list`)
- Task with multiple flows: present first flow by default, allow `--flow` override
- Invalid task ID: return helpful error with `tasks-list` suggestion
- Missing flow definition: warn and fall back to manual guidance

## 5. Open Questions / Future Work

- Should suggest-next track progress within a flow (e.g., mark steps as completed)?
- Could this support flow branching (if X succeeds, do Y; else do Z)?
- Integration with task status updates (auto-transition Todo -> InProgress when flow starts)?
