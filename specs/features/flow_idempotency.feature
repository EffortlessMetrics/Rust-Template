# Template Version: v3.3.1
# Schema: spec_ledger.yaml v1.0
# Last Updated: 2025-11-26
#
# NOTE: Scenarios tagged @ci-only require a pristine git working tree.
# These are excluded from local development runs to avoid blocking progress on
# typical dirty working trees. CI runs the full suite including @ci-only scenarios.

Feature: Flow Idempotency
  As a developer or agent
  I want platform flows to be safe to rerun
  So that running them multiple times does not corrupt state or create duplicates

  Background:
    Given I have a clean workspace with no uncommitted changes

  @AC-TPL-FLOW-IDEMPOTENT @selftest @idempotent
  Scenario: selftest produces stable output when run twice
    When I run "cargo xtask selftest" and capture the output
    And I run "cargo xtask selftest" again and capture the output
    Then both selftest outputs should have the same pass/fail status
    And both selftest outputs should report the same number of checks
    And both selftest outputs should have identical summary sections

  @AC-TPL-FLOW-IDEMPOTENT @selftest @idempotent
  Scenario: selftest does not create duplicate artifacts
    Given I record the workspace state before selftest
    When I run "cargo xtask selftest"
    And I record the workspace state after first selftest
    And I run "cargo xtask selftest" again
    And I record the workspace state after second selftest
    Then no new files should be created by the second run
    And the generated documentation files should be identical

  @AC-TPL-FLOW-IDEMPOTENT @selftest @idempotent @ci-only
  Scenario: selftest is deterministic with unchanged codebase
    Given the git working tree is clean
    When I run "cargo xtask selftest"
    Then the git working tree should still be clean
    And no uncommitted changes should exist

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

  @AC-TPL-FLOW-IDEMPOTENT @suggest-next @idempotent @ci-only
  Scenario: suggest-next is deterministic without side effects
    Given the git working tree is clean
    When I run "cargo xtask suggest-next --task implement_ac"
    Then the git working tree should still be clean
    And no uncommitted changes should exist

  @AC-TPL-FLOW-IDEMPOTENT @selftest @low-resources
  Scenario: selftest idempotency works in low-resource mode
    When I run "cargo xtask selftest" with "XTASK_LOW_RESOURCES=1" and capture the output
    And I run "cargo xtask selftest" with "XTASK_LOW_RESOURCES=1" again and capture the output
    Then both selftest outputs should have the same pass/fail status
    And the low-resource mode should be consistently applied
