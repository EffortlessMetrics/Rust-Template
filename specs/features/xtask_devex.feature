# Template Version: v2.4.0
# Schema: spec_ledger.yaml v1.0
# Last Updated: 2025-11-21

Feature: Developer Experience Commands
  As a developer
  I want discoverable and validated xtask commands
  So that I can onboard and work efficiently

  Background:
    Given a clean development environment

  @AC-PLT-018 @devup
  Scenario: dev-up installs pre-commit hooks
    Given the pre-commit hook does not exist
    When I run "cargo xtask dev-up" with low-resource mode
    Then the command should succeed
    And the pre-commit hook should be installed
    And the output should contain "Pre-commit hooks"

  @AC-PLT-018 @devup
  Scenario: dev-up skips hook installation if already present
    Given the pre-commit hook exists
    When I run "cargo xtask dev-up" with low-resource mode
    Then the command should succeed
    And the output should contain "already installed"

  @AC-PLT-018 @devup
  Scenario: dev-up runs governance check
    When I run "cargo xtask dev-up" with low-resource mode
    Then the command should succeed
    And the output should contain "governance check"
    And the output should contain "low-resource mode"

  @AC-PLT-018 @devup
  Scenario: dev-up displays next steps on success
    When I run "cargo xtask dev-up" with low-resource mode
    Then the command should succeed
    And the output should contain "dev-up complete"
    And the output should contain "Next steps"
    And the output should contain "cargo run -p app-http"
    And the output should contain "http://localhost:3000/ui"

  @AC-PLT-018 @devup
  Scenario: dev-up checks Docker availability
    When I run "cargo xtask dev-up" with low-resource mode
    Then the command should succeed
    And the output should mention Docker status

  @AC-PLT-018 @devup
  Scenario: dev-up succeeds in clean environment
    Given a completely fresh repository clone
    When I run "cargo xtask dev-up" with low-resource mode
    Then the command should succeed
    And the pre-commit hook should be installed
    And the output should contain "dev-up complete"

  @AC-PLT-001
  Scenario: doctor validates required tools
    When I run "cargo xtask doctor"
    Then the command should succeed
    And the output should check for "Rust"
    And the output should check for "git"

  @AC-PLT-002
  Scenario: help-flows renders command map
    When I run "cargo xtask help-flows"
    Then the command should succeed
    And the output should contain workflow categories
    And the output should reference "devex_flows.yaml"

  @AC-PLT-003
  Scenario: check runs fast dev loop
    When I run "cargo xtask check"
    Then the command should succeed
    And the output should indicate "fmt" was checked
    And the output should indicate "clippy" was checked
    And the output should indicate "tests" were run

  @AC-PLT-004
  Scenario: adr-new creates numbered ADR from template
    When I run "cargo xtask adr-new 'Test Decision'"
    Then the command should succeed
    And a new ADR file should be created
    And the ADR should contain metadata

  @AC-PLT-005
  Scenario: ac-new rejects duplicate IDs
    Given an AC with ID "AC-TEST-001" already exists
    When I run "cargo xtask ac-new AC-TEST-001 'Duplicate test'"
    Then the command should fail
    And the output should indicate duplicate ID error

  @AC-PLT-005
  Scenario: ac-new generates valid YAML snippet
    When I run "cargo xtask ac-new AC-TEST-999 'New test AC'"
    Then the command should succeed
    And the output should contain valid YAML
    And the YAML should include the AC ID "AC-TEST-999"

  @AC-PLT-006
  Scenario: audit runs cargo-audit and cargo-deny
    When I run "cargo xtask audit"
    Then the command should complete
    And the output should mention "cargo-audit"
    And the output should mention "cargo-deny"

  @AC-PLT-007
  Scenario: audit provides recovery guidance on failure
    Given a vulnerability exists in dependencies
    When I run "cargo xtask audit"
    Then the output should contain recovery steps
    And the recovery steps should be numbered

  @AC-PLT-008
  Scenario: sbom-local generates SPDX JSON
    When I run "cargo xtask sbom-local"
    Then the command should succeed
    And the file "target/sbom.spdx.json" should exist
    And the SBOM should be valid JSON

  @AC-PLT-009
  Scenario: docs-check validates version alignment
    When I run "cargo xtask docs-check"
    Then the command should succeed
    And the output should verify "spec_ledger" version
    And the output should verify "README" version

  @AC-PLT-010
  Scenario: docs-check regenerates feature_status
    When I run "cargo xtask docs-check"
    Then the command should succeed
    And the file "docs/feature_status.md" should be current
    And git should show no uncommitted changes to tracked files

  @AC-PLT-011
  Scenario: release-prepare updates version files
    Given the current version is "3.1.0"
    When I run "cargo xtask release-prepare 3.2.0"
    Then the command should succeed
    And "specs/spec_ledger.yaml" should contain "3.2.0"
    And "README.md" should contain "3.2.0"
    And "CLAUDE.md" should contain "3.2.0"

  @AC-PLT-012
  Scenario: release-verify runs comprehensive checks
    When I run "cargo xtask release-verify"
    Then the command should run "selftest"
    And the command should run "audit"
    And the command should run "docs-check"
    And the command should check for clean git tree

  @AC-PLT-013
  Scenario: release-verify provides git commands on success
    Given all release checks pass
    When I run "cargo xtask release-verify"
    Then the command should succeed
    And the output should contain git tag command
    And the output should contain git push command

  @AC-PLT-015
  Scenario: selftest enforces devex contract
    When I run "cargo xtask selftest"
    Then the command should validate required commands exist
    And the validation should reference "devex_flows.yaml"

  @AC-PLT-016
  Scenario: ci-local orchestrates full check suite
    When I run "cargo xtask ci-local"
    Then the command should run "doctor"
    And the command should run "selftest"
    And the command should run "audit"
    And the command should run "docs-check"

  @AC-PLT-017
  Scenario: status displays governance dashboard
    When I run "cargo xtask status"
    Then the command should succeed
    And the output should display version information
    And the output should display REQ/AC/task counts
    And the output should display selftest status
    And the output should suggest next tasks

  @AC-PLT-017
  Scenario: status shows platform server status information
    When I run "cargo xtask status"
    Then the command should succeed
    And the output should contain "Rust-as-Spec"
    And the output should contain platform version
    And the output should suggest platform server start command

  @AC-PLT-017
  Scenario: status shows git repository status context
    When I run "cargo xtask status"
    Then the command should succeed
    And the output should be formatted with visual separators
    And the output should use colors for readability

  @AC-PLT-017
  Scenario: status shows governance metrics with counts
    When I run "cargo xtask status"
    Then the command should succeed
    And the output should contain "Governance:"
    And the output should show stories count
    And the output should show requirements count
    And the output should show acceptance criteria count

  @AC-PLT-017
  Scenario: status shows task breakdown by status
    Given tasks exist in the specifications
    When I run "cargo xtask status"
    Then the command should succeed
    And the output should contain "Tasks:"
    And the output should show task counts grouped by status
    And task statuses should include "Todo", "InProgress", "Review", and "Done"

  @AC-PLT-017
  Scenario: status includes helpful next steps
    When I run "cargo xtask status"
    Then the command should succeed
    And the output should contain "Next steps:"
    And the output should suggest "cargo xtask tasks-list"
    And the output should suggest "cargo xtask selftest"
    And the output should suggest "cargo run -p app-http"
    And the output should suggest the UI URL "http://localhost:3000/ui"

  @AC-PLT-019
  Scenario: selftest displays condensed summary with 7 steps
    When I run "cargo xtask selftest"
    Then the output should show clear pass/fail indicators
    And the output should summarize all 7 steps
    And each step should have a status indicator

  @AC-PLT-019
  Scenario: selftest summary shows all step names
    When I run "cargo xtask selftest"
    Then the selftest summary should contain "Core checks"
    And the selftest summary should contain "BDD acceptance tests"
    And the selftest summary should contain "AC/ADR mapping"
    And the selftest summary should contain "LLM bundler"
    And the selftest summary should contain "Policy tests"
    And the selftest summary should contain "DevEx contract"
    And the selftest summary should contain "Graph invariants"

  @AC-PLT-019
  Scenario: selftest summary shows pass/fail status for each step
    When I run "cargo xtask selftest"
    Then each step in the summary should show either "OK" or "FAIL"
    And the summary should display step numbers 1 through 7

  @AC-PLT-019
  Scenario: selftest shows actionable error messages on failure
    Given a selftest step has failed
    When I run "cargo xtask selftest"
    Then the output should contain "Next actions:"
    And the output should provide specific recovery commands
    And recovery commands should include runnable xtask commands

  @AC-PLT-020
  Scenario: selftest respects XTASK_LOW_RESOURCES environment variable
    When I run "cargo xtask selftest" with "XTASK_LOW_RESOURCES=1"
    Then the command should complete successfully
    And the output should indicate resource-conscious execution

  @AC-PLT-020
  Scenario: XTASK_LOW_RESOURCES mode is used in CI environments
    Given the environment is CI-constrained
    When selftest runs with XTASK_LOW_RESOURCES enabled
    Then resource-intensive steps should be optimized
    And the selftest should complete within reasonable time limits

  @AC-TPL-SKILLS-FMT
  Scenario: skills-fmt normalizes SKILL.md files
    Given Agent Skills exist in .claude/skills/
    When I run "cargo xtask skills-fmt"
    Then the command should succeed
    And the output should indicate Skills were formatted
    And SKILL.md files should have consistent frontmatter formatting

  @AC-TPL-SKILLS-FMT
  Scenario: skills-fmt is idempotent
    Given Agent Skills are already formatted
    When I run "cargo xtask skills-fmt" twice
    Then both executions should succeed
    And the second run should produce identical output

  @AC-TPL-SKILLS-LINT
  Scenario: skills-lint validates required frontmatter fields
    Given a SKILL.md file with valid frontmatter
    When I run "cargo xtask skills-lint"
    Then the command should succeed
    And the output should indicate Skills passed validation

  @AC-TPL-SKILLS-LINT
  Scenario: skills-lint detects missing frontmatter fields
    Given a SKILL.md file missing required fields
    When I run "cargo xtask skills-lint"
    Then the command should fail
    And the output should indicate which fields are missing
    And the output should mention "name" or "description"

  @AC-TPL-SKILLS-LINT
  Scenario: skills-lint validates name conventions
    Given a SKILL.md file with invalid name format
    When I run "cargo xtask skills-lint"
    Then the command should fail
    And the output should indicate name convention violations
    And the output should mention "kebab-case"

  @AC-TPL-SKILLS-LINT
  Scenario: skills-lint checks description quality
    Given a SKILL.md file with vague description
    When I run "cargo xtask skills-lint"
    Then the command should fail
    And the output should indicate description needs improvement
    And the output should mention "when to use"
