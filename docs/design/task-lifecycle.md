---
id: DESIGN-TPL-TASK-LIFECYCLE-001
title: Task Status Transitions
author: governance-system
doc_type: design_doc
date: 2025-11-22
status: draft
stories: [US-TPL-PLATFORM-V3]
requirements: [REQ-TPL-TASK-LIFECYCLE]
tags: [platform, governance, domain]
acs: [AC-TPL-TASK-TRANSITIONS]
adrs: [ADR-0001]
---

# Task Status Transitions

## Problem

Without enforced state transitions, task status can change arbitrarily (e.g., Done -> Todo, or skipping InProgress). This breaks workflow tracking and makes task history meaningless.

## Solution

Define a finite state machine for task status with explicit allowed transitions. The domain model enforces these transitions, preventing invalid state changes.

**Valid States**:
- `Todo`: Not started
- `InProgress`: Actively being worked on
- `Review`: Ready for review/QA
- `Done`: Completed and validated

**Allowed Transitions**:

```
Todo -> InProgress
InProgress -> Review
Review -> Done
Review -> InProgress (backward for rework)
InProgress -> Todo (backward for re-planning)
```

## Implementation Approach

**Domain Model**: `crates/business-core/src/lib.rs` (governance module):

```rust
pub enum TaskStatus {
    Todo,
    InProgress,
    Review,
    Done,
}

impl TaskStatus {
    pub fn can_transition_to(&self, target: &TaskStatus) -> bool {
        use TaskStatus::*;
        match (self, target) {
            (Todo, InProgress) => true,
            (InProgress, Review) => true,
            (Review, Done) => true,
            (Review, InProgress) => true, // Backwards allowed for rework
            (InProgress, Todo) => true,   // Backwards allowed for re-planning
            _ => false,
        }
    }
}
```

**Validation**: `TaskService::move_task()` checks `current.can_transition_to(new)` before persisting. Returns `GovernanceError::InvalidTransition` if the transition is not allowed.

**Testing**: Unit tests in `crates/business-core/src/lib.rs` (governance module tests) covering all valid and invalid transitions.

**Benefits**: Enforces workflow integrity, prevents invalid status changes, makes task history reliable for reporting and auditing.
