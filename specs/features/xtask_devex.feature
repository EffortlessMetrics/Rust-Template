# Template Version: v3.0.0
# Schema: spec_ledger.yaml v1.0
# Last Updated: 2025-11-22

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
  Scenario: doctor validates the environment and prints next steps
    Given I am in a clean workspace
    When I run "cargo xtask doctor"
    Then the command should succeed
    And the output should contain "Checking environment"
    And the output should contain "Rust"
    And the output should contain "Next steps"

  @AC-PLT-002
  Scenario: help-flows renders DevEx flows grouped by category
    Given devex flows are defined in "specs/devex_flows.yaml"
    When I run "cargo xtask help-flows"
    Then the command should succeed
    And the output should contain "Onboarding"
    And the output should contain "Feature development"
    And the output should contain "Release"
    And the output should contain "cargo xtask ac-new"
    And the output should contain "cargo xtask selftest"

  @AC-PLT-003
  Scenario: check runs fmt, clippy, and tests as a fast dev loop
    Given I have a compilable workspace
    When I run "cargo xtask check"
    Then the command should succeed
    And the output should contain "format"
    And the output should contain "clippy"
    And the output should contain "tests"

  @AC-PLT-004
  Scenario: adr-new creates numbered ADR from template
    When I run "cargo xtask adr-new 'Comprehensive BDD Testing'"
    Then the command should succeed
    And the output should contain "✓ Created"
    And the output should contain "docs/adr"
    And the output should contain "Next steps"
    And a new ADR file should exist in docs/adr
    And the ADR file should have the correct title format
    And the ADR file should contain all required sections
    And I clean up the test ADR file

  @AC-PLT-004
  Scenario: adr-new uses next available number
    When I run "cargo xtask adr-new 'Sequential Numbering Test'"
    Then the command should succeed
    And the new ADR number should be greater than existing ADRs
    And I clean up the test ADR file

  @AC-PLT-004
  Scenario: adr-new creates valid markdown structure
    When I run "cargo xtask adr-new 'Markdown Structure Validation'"
    Then the command should succeed
    And the ADR file should contain "## Context"
    And the ADR file should contain "## Decision"
    And the ADR file should contain "## Consequences"
    And the ADR file should contain "## Compliance"
    And the ADR file should contain "## Notes"
    And I clean up the test ADR file

  @AC-PLT-004
  Scenario: adr-new provides actionable next steps
    When I run "cargo xtask adr-new 'Next Steps Guidance'"
    Then the command should succeed
    And the output should contain "Edit docs/adr"
    And the output should contain "Link this ADR in specs/spec_ledger.yaml"
    And the output should contain "cargo xtask adr-check"
    And I clean up the test ADR file

  @AC-PLT-004
  Scenario: adr-new creates a new ADR file with sequential number
    Given I am in a clean workspace
    When I run "cargo xtask adr-new 'Test Decision'"
    Then the command should succeed
    And a new file should exist in "docs/adr/" matching pattern "0*-test-decision.md"
    And the file should contain "Test Decision"
    And I clean up the test ADR file

  @AC-PLT-005
  Scenario: ac-new generates valid YAML snippet
    When I run "cargo xtask ac-new --story US-TPL-001 --requirement REQ-TPL-HEALTH AC-TEST-999 ValidYAMLGenerationTest"
    Then the command should succeed
    And the output should contain "✓ Prepared AC entry"
    And the output should contain "id: AC-TEST-999"
    And the output should contain "text: \"ValidYAMLGenerationTest\""
    And the output should contain "tests: [{ type: bdd, tag: \"@AC-TEST-999\" }]"

  @AC-PLT-005
  Scenario: ac-new rejects duplicate IDs in spec_ledger
    Given an AC with ID "AC-TPL-001" exists in spec_ledger.yaml
    When I run "cargo xtask ac-new --story US-TPL-001 --requirement REQ-TPL-HEALTH AC-TPL-001 DuplicateACTest"
    Then the command should fail
    And the output should contain "AC-TPL-001"
    And the output should contain "already exists"
    And the output should contain "spec_ledger.yaml"

  @AC-PLT-005
  Scenario: ac-new validates AC ID format
    When I run "cargo xtask ac-new --story US-TPL-001 --requirement REQ-TPL-HEALTH INVALID-FORMAT BadIDFormat"
    Then the command should fail
    And the output should contain "must start with 'AC-'"

  @AC-PLT-005
  Scenario: ac-new provides complete next steps
    When I run "cargo xtask ac-new --story US-TPL-001 --requirement REQ-TPL-HEALTH AC-TEST-998 NextStepsGuidance"
    Then the command should succeed
    And the output should contain "Next steps:"
    And the output should contain "Find requirement REQ-TPL-HEALTH"
    And the output should contain "Add the AC under acceptance_criteria:"
    And the output should contain "Add scenario to specs/features/*.feature:"
    And the output should contain "@AC-TEST-998"
    And the output should contain "Scenario: NextStepsGuidance"
    And the output should contain "cargo xtask ac-status"
    And the output should contain "cargo xtask selftest"

  @AC-PLT-005
  Scenario: ac-new outputs Gherkin template
    When I run "cargo xtask ac-new --story US-TPL-001 --requirement REQ-TPL-HEALTH AC-TEST-997 GherkinTemplateGeneration"
    Then the command should succeed
    And the output should contain "Given ..."
    And the output should contain "When ..."
    And the output should contain "Then ..."

  @AC-PLT-005
  Scenario: ac-new adds a new AC to spec_ledger.yaml
    Given I am in a clean workspace
    When I run "cargo xtask ac-new AC-TEST-001 'Test acceptance criterion' --requirement REQ-TPL-CORE-BUILD"
    Then the command should succeed
    And the output should contain "AC-TEST-001"
    And the output should contain "Test acceptance criterion"

  @AC-PLT-005
  Scenario: ac-new fails cleanly on duplicate AC IDs
    Given an AC with ID "AC-TPL-CORE-BUILD" already exists
    When I run "cargo xtask ac-new AC-TPL-CORE-BUILD 'Duplicate' --requirement REQ-TPL-CORE-BUILD"
    Then the command should fail
    And the output should contain "AC-TPL-CORE-BUILD"
    And the output should contain "already exists"

  @AC-PLT-006
  Scenario: audit runs supply-chain vulnerability check
    Given I am in a clean workspace
    When I run "cargo xtask audit"
    Then the command should complete
    And the output should match pattern "(vulnerability|advisory|Fetched|Loaded|No audit tools)"

  @AC-PLT-007
  Scenario: audit provides recovery guidance on failure
    Given a vulnerability exists in dependencies
    When I run "cargo xtask audit"
    Then the output should contain recovery steps
    And the recovery steps should be numbered

  @AC-PLT-008
  Scenario: sbom-local generates SPDX bill of materials
    Given I am in a clean workspace
    When I run "cargo xtask sbom-local"
    Then the command should succeed
    And file "target/sbom.spdx.json" should exist
    And file "target/sbom.spdx.json" should not be empty

  @AC-PLT-009
  Scenario: docs-check validates version alignment
    When I run "cargo xtask docs-check"
    Then the command should succeed
    And the output should verify "spec_ledger" version
    And the output should verify "README" version

  @AC-PLT-009
  Scenario: docs-check succeeds when documentation is consistent
    Given the documentation is up to date
    When I run "cargo xtask docs-check"
    Then the command should succeed
    And the output should contain "✓"
    And the output should contain "Documentation is consistent"

  @AC-PLT-010
  Scenario: docs-check regenerates feature_status
    When I run "cargo xtask docs-check"
    Then the command should succeed
    And the file "docs/feature_status.md" should be current
    And git should show no uncommitted changes to tracked files

  @AC-PLT-010
  Scenario: docs-check fails when documentation is outdated
    Given the README version does not match Cargo.toml
    When I run "cargo xtask docs-check"
    Then the command should fail
    And the output should contain "mismatch"
    And the output should contain "README"
    And I restore the README from backup

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

  @release_bundle_generation @AC-TPL-REL-EVIDENCE
  Scenario: release-bundle generates evidence file with all required sections
    When I run "cargo xtask release-bundle 3.1.0"
    Then the command should succeed
    And a file "release_evidence/v3.1.0.md" should be created
    And the evidence file should contain section "Tasks Completed"
    And the evidence file should contain section "Acceptance Criteria & Requirements"
    And the evidence file should contain section "Architecture Decisions"
    And the evidence file should contain section "Git Changelog"
    And the evidence file should contain section "Governance Status"
    And the evidence file should contain section "Resolved Friction"

  @release_bundle_generation @AC-TPL-REL-EVIDENCE
  Scenario: release-bundle includes done tasks from specs
    Given tasks with status "done" exist in specs/tasks.yaml
    When I run "cargo xtask release-bundle 3.1.0"
    Then the command should succeed
    And the evidence file should list all done tasks
    And each task should show its requirement ID
    And each task should show its linked ACs

  @release_bundle_generation @AC-TPL-REL-EVIDENCE
  Scenario: release-bundle includes git log since last tag
    Given a git repository with tagged releases
    When I run "cargo xtask release-bundle 3.1.0"
    Then the command should succeed
    And the evidence file should contain "Git Changelog"
    And the git section should reference the previous tag
    And the git section should include commit messages

  @release_bundle_generation @AC-TPL-REL-EVIDENCE
  Scenario: release-bundle includes selftest summary
    When I run "cargo xtask release-bundle 3.1.0"
    Then the command should succeed
    And the evidence file should contain "Selftest Status"
    And the selftest section should show pass/fail status

  @release_bundle_generation @AC-TPL-REL-EVIDENCE
  Scenario: release-bundle includes policy status
    Given policy tests have been run
    When I run "cargo xtask release-bundle 3.1.0"
    Then the command should succeed
    And the evidence file should contain "Policy Status"
    And the policy section should include status from target/policy_status.json

  @release_bundle_structure @AC-TPL-REL-CHANGELOG
  Scenario: evidence file has stable sections for LLM formatting
    When I run "cargo xtask release-bundle 3.1.0"
    Then the command should succeed
    And the evidence file should have distinct markdown sections
    And sections should be separated by "---" markers
    And the file should have a clear header with version and timestamp
    And the structure should be suitable for Keep a Changelog formatting

  @release_bundle_structure @AC-TPL-REL-CHANGELOG
  Scenario: evidence file includes linked requirements and ACs
    Given completed tasks reference requirements and ACs
    When I run "cargo xtask release-bundle 3.1.0"
    Then the command should succeed
    And the evidence file should map tasks to requirements
    And the evidence file should list all linked ACs
    And requirements should include their story context

  @release_bundle_generation @AC-TPL-REL-EVIDENCE
  Scenario: release-bundle validates version format
    When I run "cargo xtask release-bundle invalid-version"
    Then the command should fail
    And the output should indicate invalid version format
    And the output should suggest format "X.Y.Z"

  @release_bundle_generation @AC-TPL-REL-EVIDENCE
  Scenario: release-bundle creates release_evidence directory if missing
    Given the release_evidence directory does not exist
    When I run "cargo xtask release-bundle 3.1.0"
    Then the command should succeed
    And the release_evidence directory should be created
    And the evidence file should be written successfully

  @AC-TPL-SKILLS-GUIDE-001
  Scenario: AGENT_SKILLS.md provides comprehensive guidance
    Then the file "docs/AGENT_SKILLS.md" should exist
    And "docs/AGENT_SKILLS.md" should contain "Recommended Skills"

  @AC-TPL-SKILLS-ALIGN-001
  Scenario: skills-lint validates Skills alignment with workflows
    Given Agent Skills exist in .claude/skills/
    When I run "cargo xtask skills-lint"
    Then the command should succeed
