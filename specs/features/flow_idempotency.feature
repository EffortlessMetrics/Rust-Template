# Template Version: v3.3.1
# Schema: spec_ledger.yaml v1.0
# Last Updated: 2025-11-26

Feature: Flow Idempotency
  As a developer or agent
  I want platform flows to be safe to rerun
  So that running them multiple times does not corrupt state or create duplicates

  Background:
    Given I have a clean workspace with no uncommitted changes

  @AC-TPL-FLOW-IDEMPOTENT @selftest @idempotent @ci-only
  Scenario: selftest produces stable output when run twice
    # Note: Marked @ci-only to avoid recursive selftest within selftest BDD step
    When I run "cargo xtask selftest" and capture the output
    And I run "cargo xtask selftest" again and capture the output
    Then both selftest outputs should have the same pass/fail status
    And both selftest outputs should report the same number of checks
    And both selftest outputs should have identical summary sections

  @AC-TPL-FLOW-IDEMPOTENT @selftest @idempotent @ci-only
  Scenario: selftest does not create duplicate artifacts
    # Note: Marked @ci-only to avoid recursive selftest within selftest BDD step
    Given I record the workspace state before selftest
    When I run "cargo xtask selftest"
    And I record the workspace state after first selftest
    And I run "cargo xtask selftest" again
    And I record the workspace state after second selftest
    Then no new files should be created by the second run
    And the generated documentation files should be identical

  @AC-TPL-FLOW-IDEMPOTENT @suggest-next @idempotent
  Scenario: suggest-next produces identical results when run twice
    When I run "cargo xtask suggest-next --task implement_ac" and capture the output
    And I run "cargo xtask suggest-next --task implement_ac" again and capture the output
    Then both suggest-next outputs should be identical
    And the recommended command sequence should be the same

  @AC-TPL-FLOW-IDEMPOTENT @suggest-next @idempotent
  Scenario: suggest-next does not create artifacts
    Given I record the workspace state before suggest-next
    When I run "cargo xtask suggest-next --task implement_ac"
    And I record the workspace state after suggest-next
    Then no new files should be created by suggest-next
    And no existing files should be modified by suggest-next

  @AC-TPL-FLOW-IDEMPOTENT @selftest @low-resources @ci-only
  Scenario: selftest idempotency works in low-resource mode
    # Note: Marked @ci-only to avoid recursive selftest within selftest BDD step
    When I run "cargo xtask selftest" with "XTASK_LOW_RESOURCES=1" and capture the output
    And I run "cargo xtask selftest" with "XTASK_LOW_RESOURCES=1" again and capture the output
    Then both selftest outputs should have the same pass/fail status
    And the low-resource mode should be consistently applied
