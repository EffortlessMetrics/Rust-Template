# Template Version: v3.3.1
# Schema: spec_ledger.yaml v1.0
# Last Updated: 2025-11-22

Feature: Governance Write Capability
  As a platform engineer or agent
  I want to persist task status changes to the file system
  So that the platform remembers state across restarts and commits

  @AC-TPL-GOV-WRITE-TASK-STATUS-200
  Scenario: Task status is persisted via governance repository
    Given a task "TASK-TPL-GOV-WRITE-001" exists
    When the system updates its status to "InProgress"
    Then specs/tasks_state.yaml should contain that task with status "InProgress"
