# Template Version: v3.0.0
# Schema: spec_ledger.yaml v1.0
# Last Updated: 2025-11-22

Feature: Git Hooks Installation and Governance
  As a developer
  I want automated git hooks that enforce governance checks
  So that I can catch issues before committing code

  Background:
    Given a clean development environment

  @AC-TPL-HOOKS-INSTALL @hooks
  Scenario: install-hooks creates executable pre-commit hook on Unix
    Given I am on a Unix platform
    And the pre-commit hook does not exist
    When I run "cargo xtask install-hooks"
    Then the command should succeed
    And file ".git/hooks/pre-commit" should exist
    And file ".git/hooks/pre-commit" should be executable
    And the output should contain "Installed .git/hooks/pre-commit"

  @AC-TPL-HOOKS-INSTALL @hooks
  Scenario: install-hooks creates pre-commit hook on Windows
    Given I am on a Windows platform
    And the pre-commit hook does not exist
    When I run "cargo xtask install-hooks"
    Then the command should succeed
    And file ".git/hooks/pre-commit" should exist
    And the output should contain "Installed .git/hooks/pre-commit"

  @AC-TPL-HOOKS-INSTALL @hooks
  Scenario: install-hooks reports success when hook already exists
    Given the pre-commit hook exists
    When I run "cargo xtask install-hooks"
    Then the command should succeed
    And file ".git/hooks/pre-commit" should exist
    And the output should contain "Installed .git/hooks/pre-commit"

  @AC-TPL-HOOKS-INSTALL @hooks
  Scenario: install-hooks fails gracefully outside git repository
    Given I am outside a git repository
    When I run "cargo xtask install-hooks"
    Then the command should succeed
    And the output should contain ".git directory not found"

  @AC-TPL-HOOKS-INSTALL @hooks
  Scenario: pre-commit hook runs governance check
    Given the pre-commit hook exists
    When I attempt to commit changes
    Then the pre-commit hook should run "cargo run -p xtask -- check"
    And the commit should succeed if checks pass
    And the commit should be blocked if checks fail

  @AC-TPL-HOOKS-INSTALL @hooks
  Scenario: pre-commit hook respects XTASK_LOW_RESOURCES environment variable
    Given XTASK_LOW_RESOURCES is set to "1"
    When I run "cargo xtask install-hooks"
    Then the command should succeed
    And the pre-commit hook should contain "export XTASK_LOW_RESOURCES=1"

  @AC-TPL-HOOKS-INSTALL @hooks
  Scenario: pre-commit hook does not include low-resource mode by default
    Given XTASK_LOW_RESOURCES is not set
    When I run "cargo xtask install-hooks"
    Then the command should succeed
    And the pre-commit hook should not contain "XTASK_LOW_RESOURCES"

  @AC-TPL-HOOKS-INSTALL @hooks
  Scenario: pre-commit hook contains governance messaging
    When I run "cargo xtask install-hooks"
    Then the command should succeed
    And the pre-commit hook should contain "Rust-as-Spec Governance Pre-Commit Hook"
    And the output should contain "cargo run -p xtask -- check"

  @AC-TPL-HOOKS-INSTALL @hooks
  Scenario: pre-commit hook can be removed by deleting file
    Given the pre-commit hook exists
    When I delete the pre-commit hook file
    Then the pre-commit hook should not exist
    And commits should proceed without governance checks

  @AC-TPL-HOOKS-INSTALL @hooks
  Scenario: install-hooks creates hooks directory if missing
    Given the .git/hooks directory does not exist
    When I run "cargo xtask install-hooks"
    Then the command should succeed
    And the .git/hooks directory should exist
    And the pre-commit hook should be installed
