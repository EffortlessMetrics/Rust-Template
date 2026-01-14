---
id: DESIGN-TPL-GOV-WRITE-001
title: Task Status Persistence
author: governance-system
doc_type: design_doc
date: 2025-11-22
status: draft
stories: [US-TPL-PLATFORM-V3]
requirements: [REQ-TPL-GOV-WRITE-001]
tags: [platform, governance, write-layer]
acs: [AC-TPL-GOV-WRITE-TASK-STATUS-200]
adrs: [ADR-0001]
---

# Task Status Persistence

## Problem

The platform can read governance state (tasks, requirements, ACs) from `specs/*.yaml` files, but cannot persist state changes (e.g., task status transitions) in a durable way. This prevents autonomous agent loops where agents update task status as they complete work.

## Solution

Provide a write-layer API that persists task status changes to machine-managed state files, separate from human-authored YAML. The governance graph reflects both sources (human specs + machine state), ensuring agents can update status without corrupting human comments or structure.

## Implementation Approach

**Storage**: Create `state/tasks.json` for machine-managed task status:

```json
{
  "TASK-001": {"status": "InProgress", "updated_at": "2025-11-22T10:00:00Z"},
  "TASK-002": {"status": "Done", "updated_at": "2025-11-22T09:00:00Z"}
}
```

**API**: Add function `set_task_status(task_id: &str, status: TaskStatus) -> Result<()>`:
- Validates task exists in `specs/tasks.yaml`
- Validates status transition (Todo -> InProgress -> Done)
- Writes to `state/tasks.json`
- Returns error if invalid transition or task not found

**Graph Integration**: Update `spec_runtime::graph::build_graph()` to merge:
1. Task definitions from `specs/tasks.yaml`
2. Status overrides from `state/tasks.json`

**Benefits**: Durable state, agents can update task status, preserves human-authored YAML, enables autonomous workflows.
