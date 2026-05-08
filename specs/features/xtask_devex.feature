# Template Version: v3.3.8
# Schema: spec_ledger.yaml v1.0
# Last Updated: 2025-12-01

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
    And the output should contain "Pre-commit hooks"

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
    And the output should contain "http://localhost:8080/ui"

  @AC-PLT-018 @devup
  Scenario: dev-up checks Docker availability
    When I run "cargo xtask dev-up" with low-resource mode
    Then the command should succeed
    And the output should mention Docker status

  @AC-PLT-018 @devup
  Scenario: dev-up succeeds in clean environment
    Given the pre-commit hook does not exist
    When I run "cargo xtask dev-up" with low-resource mode
    Then the command should succeed
    And the pre-commit hook should be installed
    And the output should contain "dev-up complete"

  @AC-PLT-001
  Scenario: doctor validates the environment and prints next steps
    Given I am in a clean workspace
    When I run "cargo xtask doctor"
    Then the command should succeed
    And the output should contain "environment"
    And the output should contain "Rust"
    And the output should contain "Recommendations"

  @AC-PLT-ENV-ABI-CHECK
  Scenario: doctor shows structured environment sections
    Given I am in a clean workspace
    When I run "cargo xtask doctor"
    Then the command should succeed
    And the output should contain "Environment:"
    And the output should contain "ABI Compatibility:"
    And the output should contain "Build Configuration:"
    And the output should contain "Required Tools:"

  @AC-PLT-ENV-ABI-CHECK
  Scenario: doctor detects environment type
    Given I am in a clean workspace
    When I run "cargo xtask doctor"
    Then the command should succeed
    And the output should contain "Environment type"
    And the output should mention either "Nix devshell" or "Native"

  @AC-PLT-ENV-ABI-CHECK
  Scenario: doctor checks toolchain ABI compatibility
    Given I am in a clean workspace
    When I run "cargo xtask doctor"
    Then the command should succeed
    And the output should contain "Toolchain ABI"
    And the output should show ABI check result

  @AC-PLT-ENV-ABI-CHECK
  Scenario: doctor checks glibc version on Linux
    Given I am in a clean workspace
    When I run "cargo xtask doctor"
    Then the command should succeed
    And the output should contain "glibc compatibility"
    And the output should show glibc status

  @AC-PLT-ENV-ABI-CHECK
  Scenario: doctor checks libz.so.1 availability
    Given I am in a clean workspace
    When I run "cargo xtask doctor"
    Then the command should succeed
    And the output should contain "libz.so.1 available"

  @AC-PLT-ENV-SCCACHE-WARN
  Scenario: doctor reports sccache status
    Given I am in a clean workspace
    When I run "cargo xtask doctor"
    Then the command should succeed
    And the output should contain "sccache status"

  @AC-PLT-ENV-SCCACHE-WARN
  Scenario: doctor provides TROUBLESHOOTING.md reference when warnings found
    Given I am in a clean workspace
    When I run "cargo xtask doctor"
    Then the command should succeed
    And the output should contain "Recommendations"
    And if warnings exist then output should mention "TROUBLESHOOTING.md"

  @AC-PLT-ENV-ABI-CHECK
  Scenario: doctor reports exit code status
    Given I am in a clean workspace
    When I run "cargo xtask doctor"
    Then the command should succeed
    And the output should contain "Exit code:"

  @AC-PLT-002
  Scenario: help-flows renders DevEx flows grouped by category
    When I run "cargo xtask help-flows"
    Then the command should succeed
    And the output should contain "Onboarding"
    And the output should contain "Design & Acceptance Criteria"
    And the output should contain "Release Management"
    And the output should contain "ac-new"
    And the output should contain "selftest"

  @AC-PLT-003
  Scenario: check runs fmt, clippy, and tests as a fast dev loop
    Given I am in a clean workspace
    When I run "cargo xtask check"
    Then the command should succeed
    And the output should contain "format"
    And the output should contain "clippy"
    And the output should contain "tests"

  @AC-PLT-003 @selective_testing @ci-only
  Scenario: test-changed builds tag expression for changed features
    # Note: Marked @ci-only because git worktree operations can flake
    # when VS Code Git extension or other tools access .git concurrently
    Given a temporary git worktree for test-changed
    And I add a selective test-changed feature file
    When I run "cargo xtask test-changed" in plan-only mode
    Then the command should succeed
    And the output should contain "Plan-only mode enabled"
    And the output should contain "@AC-PLT-003"
    And the output should contain "CUCUMBER_TAG_EXPRESSION=\"@AC-PLT-003\""
    And I clean up the selective testing worktree

  @AC-PLT-004
  Scenario: adr-new creates numbered ADR from template with next steps
    When I run "cargo xtask adr-new test-adr-scaffolding"
    Then the command should succeed
    And the output should contain "Created"
    And the output should contain "docs/adr"
    And the output should contain "Next steps"
    And the output should contain "Edit docs/adr"
    And the output should contain "spec_ledger.yaml"
    And a new ADR file should exist
    And the ADR file should have the correct title format
    And the ADR file should contain all required sections
    And the new ADR number should be sequential
    And I clean up the test ADR file

  @AC-PLT-005
  Scenario: ac-new generates YAML AC entry with Gherkin template and next steps
    When I run "cargo xtask ac-new --story US-TPL-001 --requirement REQ-TPL-HEALTH AC-TEST-SCAFFOLD test-ac-scaffold"
    Then the command should succeed
    And the output should contain "Prepared AC entry"
    And the output should contain "AC-TEST-SCAFFOLD"
    And the output should contain "Next steps"
    And the output should contain "spec_ledger.yaml"
    And the output should contain "Given"
    And the output should contain "When"
    And the output should contain "Then"

  @AC-PLT-006
  Scenario: audit runs supply-chain vulnerability check
    When I run "cargo xtask audit"
    Then the output should contain "cargo-audit"
    And the output should contain "cargo-deny"
    And the output should contain "Summary"

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
  Scenario: docs-check validates documentation consistency
    When I run "cargo xtask docs-check"
    Then the output should contain "Doc"
    And the output should contain "policies"
    And the output should contain "Skills definitions"

  @AC-PLT-010
  Scenario: docs-check reports issues when found
    When I run "cargo xtask docs-check"
    Then the output should contain "issue"
    And the output should contain "To fix"

  @AC-PLT-011
  Scenario: release-prepare updates version files
    Given the current version is "3.3.0"
    When I run "cargo xtask release-prepare 3.3.1"
    Then the command should succeed
    And "specs/spec_ledger.yaml" should contain "3.3.1"
    And "README.md" should contain "3.3.1"
    And "CLAUDE.md" should contain "3.3.1"

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

  # Versioning Engine Scenarios (v3.3.6)
  @AC-TPL-VERSION-MANIFEST
  Scenario: version manifest declares all version-bearing files
    Given I am in a clean workspace
    When I check for "specs/version_manifest.yaml"
    Then the file should exist
    And the file should contain version pattern definitions
    And the file should reference "spec_ledger.yaml"
    And the file should reference "README.md"
    And the file should reference "CHANGELOG.md"

  @AC-TPL-VERSION-DRYRUN
  Scenario: release-prepare dry-run shows changes without modifying files
    Given the current spec_ledger version is "3.3.6"
    When I run "cargo xtask release-prepare 3.3.7 --dry-run"
    Then the command should succeed
    And the output should contain "Dry run"
    And the output should contain "changes that would be made"
    And the output should contain "3.3.6"
    And the output should contain "3.3.7"
    And "specs/spec_ledger.yaml" should still contain "3.3.6"

  @AC-TPL-VERSION-ATOMIC
  Scenario: release-prepare updates all version files atomically
    Given the version manifest lists multiple files
    When I run "cargo xtask release-prepare" with a valid version
    Then either all files are updated or none are
    And the command should report all files modified on success

  @AC-PLT-014
  Scenario: devex_flows.yaml defines canonical flows
    Given I am in a clean workspace
    When I check for "specs/devex_flows.yaml"
    Then the file should exist
    And the file should contain flow definitions
    And the file should define "onboarding" flows
    And the file should define "design" flows
    And the file should define "release" flows

  @AC-PLT-015 @ci-only
  Scenario: selftest enforces devex contract
    # Note: Marked @ci-only to avoid recursive selftest within selftest BDD step
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
    And the output should suggest the UI URL "http://localhost:8080/ui"

  @AC-PLT-019 @ci-only
  Scenario: selftest displays condensed summary with 8 steps
    # Note: Marked @ci-only to avoid recursive selftest within selftest BDD step
    When I run "cargo xtask selftest"
    Then the output should show clear pass/fail indicators
    And the output should summarize all 8 steps
    And each step should have a status indicator

  @AC-PLT-019 @ci-only
  Scenario: selftest summary shows all step names
    # Note: Marked @ci-only to avoid recursive selftest within selftest BDD step
    When I run "cargo xtask selftest"
    Then the selftest summary should contain "Core checks"
    And the selftest summary should contain "BDD acceptance tests"
    And the selftest summary should contain "AC/ADR mapping"
    And the selftest summary should contain "LLM bundler"
    And the selftest summary should contain "Policy tests"
    And the selftest summary should contain "DevEx contract"
    And the selftest summary should contain "Graph invariants"
    And the selftest summary should contain "AC coverage"

  @AC-PLT-019 @ci-only
  Scenario: selftest summary shows pass/fail status for each step
    # Note: Marked @ci-only to avoid recursive selftest within selftest BDD step
    When I run "cargo xtask selftest"
    Then each step in the summary should show either "OK" or "FAIL"
    And the summary should display step numbers 1 through 8

  @AC-PLT-019 @ci-only
  Scenario: selftest shows actionable error messages on failure
    # Note: Marked @ci-only to avoid recursive selftest within selftest BDD step
    Given a selftest step has failed
    When I run "cargo xtask selftest"
    Then the output should contain "Next actions:"
    And the output should provide specific recovery commands
    And recovery commands should include runnable xtask commands

  @AC-PLT-020 @ci-only
  Scenario: selftest respects XTASK_LOW_RESOURCES environment variable
    # Note: Marked @ci-only to avoid recursive selftest within selftest BDD step
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
  Scenario: skills-fmt formats Skills files
    Given a SKILL.md file exists in ".claude/skills"
    When I run "cargo xtask skills-fmt"
    Then the command should succeed

  @AC-TPL-SKILLS-LINT
  Scenario: skills-lint validates all SKILL.md files silently when valid
    Given Agent Skills exist in .claude/skills/
    When I run "cargo xtask skills-lint"
    Then the command should succeed

  @AC-TPL-SKILLS-LIFECYCLE-DOCS
  Scenario: Skills governance docs describe lifecycle and template contracts
    Given I am in the actual workspace
    Then the file "docs/SKILLS_GOVERNANCE.md" should exist
    And the file "docs/SKILLS_GOVERNANCE.md" should contain "create, maintain, retire"
    And the file "docs/SKILLS_GOVERNANCE.md" should contain "ADR-0020"
    And the file "docs/SKILLS_TEMPLATE.md" should exist
    And the file "docs/SKILLS_TEMPLATE.md" should contain "When to Use"
    And the file "docs/SKILLS_TEMPLATE.md" should contain "allowed-tools"

  @AC-TPL-SKILLS-GOVERNANCE-001
  Scenario: Skills governance doc covers spec, validation rules, and ADR-0020
    Given I am in the actual workspace
    Then the file "docs/SKILLS_GOVERNANCE.md" should exist
    And the file "docs/SKILLS_GOVERNANCE.md" should contain "ADR-0020"
    And the file "docs/SKILLS_GOVERNANCE.md" should contain "validation"

  @AC-TPL-SKILLS-GOVERNANCE-003
  Scenario: Skills template doc provides creation checklist
    Given I am in the actual workspace
    Then the file "docs/SKILLS_TEMPLATE.md" should exist
    And the file "docs/SKILLS_TEMPLATE.md" should contain "checklist"
    And the file "docs/SKILLS_TEMPLATE.md" should contain "description"

  @AC-TPL-SKILLS-GOVERNANCE-002
  Scenario: skills-lint enforces description quality, tool safety, and flow mapping
    Given I am in the actual workspace
    Given low resources are disabled
    When I run "cargo xtask skills-lint"
    Then the command should succeed
    And the output should contain "[SKILL LINT]"
    And the file "docs/SKILLS_TEMPLATE.md" should contain "WHAT (capability) and WHEN (triggers/context)"
    And the file "docs/SKILLS_TEMPLATE.md" should contain "allowed-tools follows least-privilege principle"
    And the file "docs/SKILLS_TEMPLATE.md" should contain "references devex_flows or xtask commands"

  @AC-TPL-SKILLS-DESCRIPTION-QUALITY
  Scenario: skills-lint validates description WHAT and WHEN criteria
    Given I am in the actual workspace
    Given low resources are disabled
    When I run "cargo xtask skills-lint"
    Then the command should succeed
    And the file "docs/SKILLS_TEMPLATE.md" should contain "WHAT (capability) and WHEN (triggers/context)"

  @AC-TPL-SKILLS-ALLOWED-TOOLS-SAFETY
  Scenario: skills-lint enforces allowed-tools least-privilege
    Given I am in the actual workspace
    Given low resources are disabled
    When I run "cargo xtask skills-lint"
    Then the command should succeed
    And the file "docs/SKILLS_TEMPLATE.md" should contain "allowed-tools follows least-privilege principle"

  @AC-TPL-SKILLS-FLOW-MAPPING
  Scenario: skills-lint validates flow mapping references
    Given I am in the actual workspace
    Given low resources are disabled
    When I run "cargo xtask skills-lint"
    Then the command should succeed
    And the file "docs/SKILLS_TEMPLATE.md" should contain "references devex_flows or xtask commands"

  @AC-TPL-AGENTS-LIFECYCLE-DOCS
  Scenario: Agent governance docs describe lifecycle responsibilities
    Given I am in the actual workspace
    Then the file "docs/AGENTS_GOVERNANCE.md" should exist
    And the file "docs/AGENTS_GOVERNANCE.md" should contain "Lifecycle"
    And the file "docs/AGENTS_GOVERNANCE.md" should contain "Create an Agent"
    And the file "docs/AGENTS_GOVERNANCE.md" should contain "Maintain an Agent"
    And the file "docs/AGENTS_GOVERNANCE.md" should contain "Retire an Agent"

  @AC-TPL-AGENTS-DESCRIPTION-QUALITY
  Scenario: agents-lint validates description, model, permission, and skill references
    Given I am in the actual workspace
    When I run "cargo xtask agents-lint"
    Then the command should succeed
    And the output should contain "[AGENT LINT]"
    And the file "docs/AGENTS_GOVERNANCE.md" should contain "description non-empty"
    And the file "docs/AGENTS_GOVERNANCE.md" should contain "permissionMode"
    And the file "docs/AGENTS_GOVERNANCE.md" should contain "model"
    And the file "docs/AGENTS_GOVERNANCE.md" should contain "skills"

  @AC-TPL-AGENTS-TOOLS-PERMISSION-SAFETY
  Scenario: agents-lint validates tools and permissionMode safety
    Given I am in the actual workspace
    When I run "cargo xtask agents-lint"
    Then the command should succeed
    And the file "docs/AGENTS_GOVERNANCE.md" should contain "permissionMode"
    And the file "docs/AGENTS_GOVERNANCE.md" should contain "least-privilege"

  @AC-TPL-AGENTS-MODEL-POLICY
  Scenario: agents-lint validates model policy compliance
    Given I am in the actual workspace
    When I run "cargo xtask agents-lint"
    Then the command should succeed
    And the file "docs/AGENTS_GOVERNANCE.md" should contain "model"

  @AC-TPL-AGENTS-SKILLS-REFERENCES
  Scenario: agents-lint validates skill references exist
    Given I am in the actual workspace
    When I run "cargo xtask agents-lint"
    Then the command should succeed
    And the file "docs/AGENTS_GOVERNANCE.md" should contain "skills"

  @AC-PLT-AC-DEMOTION-GOVERNED
  Scenario: AC demotion workflow is documented as ADR-governed change
    Given I am in the actual workspace
    Then the file "docs/how-to/change-acceptance-criterion.md" should exist
    And the file "docs/how-to/change-acceptance-criterion.md" should contain "Demote Kernel AC"
    And the file "docs/how-to/change-acceptance-criterion.md" should contain "must_have_ac: true"
    And the file "docs/how-to/change-acceptance-criterion.md" should contain "must_have_ac: false"
    And the file "docs/how-to/change-acceptance-criterion.md" should contain "ADR or issue filed"

  @AC-TPL-XTASK-SPEC-ROOT
  Scenario: tasks-list honors SPEC_ROOT override when set
    Given the environment variable "SPEC_ROOT" is set to "/tmp/does-not-exist"
    Given low resources are disabled
    When I run "cargo xtask tasks-list"
    Then the command should fail
    And the output should contain "/tmp/does-not-exist/specs/tasks.yaml"

  @AC-TPL-TS-CONFIG-VALIDATION
  Scenario: ts-config validator catches deprecated settings contract
    Given I am in the actual workspace
    When I run "cargo xtask validate-ts-config"
    Then the command should succeed
    And the output should contain "Validating TypeScript configuration standards"
    And the output should contain "pass validation"

  @AC-TPL-SKILLS-NAME-FORMAT
  Scenario: skills-lint enforces skill name format in the template
    Given I am in the actual workspace
    When I run "cargo xtask skills-lint"
    Then the command should succeed

  @AC-TPL-AGENTS-NAME-FORMAT
  Scenario: agents-lint enforces agent name format in the template
    Given I am in the actual workspace
    When I run "cargo xtask agents-lint"
    Then the command should succeed

  @AC-PLT-DOC-INDEX-FRONTMATTER
  Scenario: docs-check enforces doc_index frontmatter invariants
    Given I am in the actual workspace
    When I run "cargo xtask docs-check"
    Then the command should succeed

  @AC-TPL-AGENTS-GOVERNANCE-001
  Scenario: agents-lint validates all agent files when valid
    Given Claude Code agents exist in .claude/agents/
    When I run "cargo xtask agents-lint"
    Then the command should succeed
    And the output should contain "Agents governance check passed" or no agents found

  @AC-TPL-AGENTS-GOVERNANCE-001
  Scenario: agents-lint reports errors for invalid agent names
    Given an agent file with invalid name "My-Invalid-Agent.md"
    When I run "cargo xtask agents-lint"
    Then the command should fail
    And the output should contain "ERRORS"
    And the output should contain "must match"

  @AC-TPL-AGENTS-GOVERNANCE-002
  Scenario: agents-lint validates agent requirement alignment
    Given agents are defined in .claude/agents/
    When I check the spec_ledger for agent REQs
    Then each agent should have a corresponding REQ-*-AGENTS-* requirement
    And each REQ should have at least one AC

  @AC-TPL-AGENTS-TEMPLATE-DOC
  Scenario: AGENTS_TEMPLATE.md exists and provides authoring guidance
    Given I check the docs directory
    When I look for AGENTS_TEMPLATE.md
    Then the file should exist
    And the file should contain frontmatter examples
    And the file should contain step-by-step creation workflow
    And the file should contain governance checklist

  @AC-TPL-AGENT-SKILLS
  Scenario: Skills directory contains required skill definitions
    Given Agent Skills exist in .claude/skills/
    When I check the skills directory structure
    Then the skills directory should contain "bootstrap-dev-env"
    And the skills directory should contain "governed-feature-dev"
    And the skills directory should contain "governed-release"
    And the skills directory should contain "governed-maintenance"
    And each skill should have a valid SKILL.md file
    And each SKILL.md should have proper frontmatter with name, description, and allowed-tools

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

  # FIXME: Race condition with parallel scenarios modifying same workspace files
  @AC-PLT-021 @wip
  Scenario: service-init updates service branding
    Given a clean git working directory
    When I run service-init with id "test-service" name "Test Service" and description "A test service"
    Then the command should succeed
    And "specs/service_metadata.yaml" should contain "service_id: test-service"
    And "specs/service_metadata.yaml" should contain "display_name: Test Service"
    And "specs/service_metadata.yaml" should contain "description: A test service"
    And "README.md" should contain "# Test Service"
    And "README.md" should contain "A test service"
    And "CLAUDE.md" should contain "# CLAUDE.md – Test Service"
    And the output should contain "Service initialization complete"
    And I clean up the service-init test files

  @AC-PLT-021
  Scenario: service-init is idempotent
    Given service metadata has been initialized
    When I run service-init with the same parameters twice
    Then both runs should succeed
    And the second run should report "No changes needed"
    And I clean up the service-init test files

  @AC-PLT-021
  Scenario: service-init validates service ID format
    When I run service-init with an invalid service ID "MyService"
    Then the command should fail
    And the output should contain "kebab-case"

  # FIXME: Race condition with parallel scenarios modifying same workspace files
  @AC-PLT-021 @wip
  Scenario: service-init updates metadata and README for new service identity
    Given a clean git working directory
    When I run service-init with id "my-new-service" name "My New Service" and description "A new test service"
    Then the command should succeed
    And "specs/service_metadata.yaml" should contain "service_id: my-new-service"
    And "specs/service_metadata.yaml" should contain "display_name: My New Service"
    And "specs/service_metadata.yaml" should contain "description: A new test service"
    And "README.md" should contain "# My New Service"
    And "README.md" should contain "A new test service"
    And "CLAUDE.md" should contain "# CLAUDE.md – My New Service"
    And I clean up the service-init test files

  # FIXME: Race condition with parallel scenarios modifying same workspace files
  @AC-PLT-021 @wip
  Scenario: platform status reflects service identity after service-init
    Given service-init has been run with custom identity
    Then "specs/service_metadata.yaml" should contain "service_id: platform-test-service"
    And "specs/service_metadata.yaml" should contain "display_name: Platform Test Service"
    And "specs/service_metadata.yaml" should contain "description: Service for testing platform status"
    And I clean up the service-init test files

  @AC-TPL-KERNEL-CONTRACT-EMITTED
  Scenario: release-bundle emits kernel contract JSON
    Given I am in the actual workspace
    When I run "cargo xtask release-bundle 0.0.0-test"
    Then the command should succeed
    And the file "release_evidence/kernel_contract.v0.0.0-test.json" should exist
    And "release_evidence/kernel_contract.v0.0.0-test.json" should contain "commands"
    And "release_evidence/kernel_contract.v0.0.0-test.json" should contain "platform_endpoints"
    And "release_evidence/kernel_contract.v0.0.0-test.json" should contain "governance_schemas"

  @example_fork_ci @AC-TPL-EXAMPLE-FORK-BUILDS
  Scenario: example fork builds and passes selftest
    Given the example fork exists at "examples/fork-customization/"
    When the example fork selftest runs in CI
    Then the fork selftest should pass
    And the fork should demonstrate customization patterns

  @AC-TPL-CLI-JSON-OUTPUT
  Scenario: version command supports JSON output
    Given I am in the actual workspace
    When I run "cargo xtask version --json"
    Then the command should succeed
    And the output should be valid JSON
    And the JSON should include "kernel_version" field

  @AC-TPL-CLI-JSON-OUTPUT
  Scenario: ac-status supports JSON output format
    # ac-status auto-regenerates JUnit if missing and outputs JSON
    # Exit code depends on AC status (fail if any ACs fail)
    Given I am in the actual workspace
    When I run "cargo xtask ac-status --json"
    Then the output should be valid JSON
    And the JSON should include "timestamp" field
    And the JSON should include "acs" field

  @AC-TPL-CLI-JSON-OUTPUT
  Scenario: friction-list supports JSON output
    Given I am in the actual workspace
    When I run "cargo xtask friction-list --json"
    Then the command should succeed
    And the output should be valid JSON

  @AC-TPL-CLI-JSON-OUTPUT
  Scenario: questions-list supports JSON output
    Given I am in the actual workspace
    When I run "cargo xtask questions-list --json"
    Then the command should succeed
    And the output should be valid JSON

  @AC-TPL-CLI-JSON-OUTPUT
  Scenario: fork-list supports JSON output
    Given I am in the actual workspace
    When I run "cargo xtask fork-list --json"
    Then the command should succeed
    And the output should be valid JSON

  @AC-TPL-CLI-JSON-CORE
  Scenario: version command produces stable JSON for tooling
    Given I am in the actual workspace
    When I run "cargo xtask version --json"
    Then the command should succeed
    And the output should be valid JSON
    And the JSON should include "kernel_version" field

  @AC-TPL-CLI-JSON-CORE
  Scenario: ac-status command produces JSON output for tooling
    # ac-status outputs valid JSON with stable top-level structure (schema v2.0)
    # Uses must_have_ac metadata instead of prefix-based categorization
    # Exit code depends on whether any ACs are failing
    Given I am in the actual workspace
    When I run "cargo xtask ac-status --json"
    Then the output should be valid JSON
    And the JSON should include "schema_version" field
    And the JSON should include "timestamp" field
    And the JSON should include "must_have_acs" field
    And the JSON should include "optional_acs" field
    And the JSON should include "coverage_percent" field
    And the JSON should include "acs" field

  @AC-TPL-XTASK-NONINTERACTIVE
  Scenario: doctor runs non-interactively with CI=1
    Given the environment variable "CI" is set to "1"
    When I run "cargo xtask doctor"
    Then the command should succeed
    And the command should not prompt for input
    And the exit code should be 0 on success

  @AC-TPL-XTASK-NONINTERACTIVE @ci-only
  Scenario: selftest runs non-interactively with XTASK_NONINTERACTIVE=1
    # Note: Marked @ci-only to avoid recursive selftest within selftest BDD step
    Given the environment variable "XTASK_NONINTERACTIVE" is set to "1"
    When I run "cargo xtask selftest"
    Then the command should succeed
    And the command should not prompt for input
    And the exit code should be 0 on success

  @AC-TPL-XTASK-NONINTERACTIVE
  Scenario: check runs non-interactively in CI mode
    Given the environment variable "CI" is set to "1"
    When I run "cargo xtask check"
    Then the command should not prompt for input
    And the exit code should reflect success or failure

  @AC-TPL-XTASK-NONINTERACTIVE
  Scenario: ac-status runs non-interactively in automation mode
    Given the environment variable "XTASK_NONINTERACTIVE" is set to "1"
    When I run "cargo xtask ac-status"
    Then the command should not prompt for input
    And the exit code should reflect command success or failure

  @AC-TPL-XTASK-NONINTERACTIVE
  Scenario: bundle command runs non-interactively with CI=1
    Given the environment variable "CI" is set to "1"
    When I run "cargo xtask bundle implement_ac"
    Then the command should not prompt for input
    And the exit code should reflect command success or failure

  @AC-TPL-XTASK-NONINTERACTIVE
  Scenario: version command runs non-interactively
    Given a clean development environment
    And the environment variable "XTASK_NONINTERACTIVE" is set to "1"
    When I run "cargo xtask version"
    Then the command should not prompt for input
    And the exit code should reflect command success or failure

  @AC-TPL-XTASK-NONINTERACTIVE
  Scenario: friction-list runs non-interactively in CI mode
    Given the environment variable "CI" is set to "1"
    When I run "cargo xtask friction-list"
    Then the command should not prompt for input
    And the exit code should be 0 on success

  @AC-TPL-XTASK-NONINTERACTIVE
  Scenario: exit codes reflect operation success in automation mode
    Given automation mode is enabled
    When commands succeed in non-interactive mode
    Then they should exit with code 0
    And when commands fail in non-interactive mode
    Then they should exit with non-zero codes

  @AC-TPL-IDP-SNAPSHOT
  Scenario: idp-snapshot emits valid JSON with governance health
    When I run "cargo xtask idp-snapshot"
    Then the command should succeed
    And the output should be valid JSON
    And the JSON should contain field "timestamp"
    And the JSON should contain field "template_version"
    And the JSON should contain field "governance_health"
    And the JSON should contain field "documentation"
    And the JSON should contain field "task_hints"

  @AC-TPL-IDP-SNAPSHOT-VALID-JSON
  Scenario: idp-snapshot JSON is parseable and complete
    When I run "cargo xtask idp-snapshot --pretty"
    Then the command should succeed
    And the output should be valid JSON
    And the JSON field "governance_health" should have "status"
    And the JSON field "governance_health" should have "ac_coverage"

  @AC-TPL-IDP-SNAPSHOT
  Scenario: idp-snapshot writes to file when --output specified
    When I run "cargo xtask idp-snapshot --output /tmp/idp-test.json"
    Then the command should succeed
    And the file "/tmp/idp-test.json" should exist
    And the file should contain valid JSON

  @AC-TPL-IDP-CELL-SMOKE
  Scenario: idp-snapshot output matches IDP Cell Contract shape
    When I run "cargo xtask idp-snapshot --pretty"
    Then the command should succeed
    And the output should be valid JSON
    And the JSON should contain field "timestamp"
    And the JSON should contain field "template_version"
    And the JSON should contain field "service_id"
    And the JSON should contain field "governance_health"
    And the JSON field "governance_health" should have "status"
    And the JSON field "governance_health" should have "ac_coverage"
    And the JSON field "governance_health" should have "spec_counts"
    And the JSON should contain field "documentation"
    And the JSON should contain field "task_hints"

  @AC-TPL-IDP-CELL-SMOKE
  Scenario: kernel-status reports IDP surfaces as healthy
    Given I am in the actual workspace
    When I run "cargo xtask kernel-status"
    Then the output should contain "IDP Surfaces"
    And the output should contain "idp-snapshot:"

  # Docs Governance Scenarios (Lane 1 hardening)
  @AC-PLT-009
  Scenario: docs-check validates version alignment across all consumer files
    When I run "cargo xtask docs-check"
    Then the output should contain "Version alignment"
    And the output should verify "README.md"
    And the output should verify "CLAUDE.md"
    And the output should verify "spec_ledger.yaml"

  @AC-PLT-009
  Scenario: docs-check provides actionable fix hints on version mismatch
    When I run "cargo xtask docs-check"
    And version mismatches are detected
    Then the output should contain "To fix"
    And the output should contain "release-prepare"

  @AC-PLT-010
  Scenario: docs-check validates AC status regeneration
    When I run "cargo xtask docs-check"
    Then the output should contain "AC status consistency"
    And the output should indicate sync status

  @AC-PLT-010
  Scenario: docs-check provides fix hints for AC status out of sync
    When I run "cargo xtask docs-check"
    And AC status is out of sync
    Then the output should contain "cargo xtask ac-status"

  @AC-TPL-AC-STATUS-CONSISTENCY
  Scenario: feature_status.md contains governance semantics cross-reference
    Given the file "docs/feature_status.md" exists
    Then the file should contain "ac-kernel/README.md"
    And the file should contain "ac-governance-semantics"
    And the file should contain "must_have_ac"

  @AC-TPL-AC-STATUS-CONSISTENCY
  Scenario: ac-status regeneration includes governance link
    When I run "cargo xtask ac-status"
    Then the command should succeed
    And the file "docs/feature_status.md" should contain "For formal definitions of"
    And the file "docs/feature_status.md" should contain "ac-kernel/README.md"

  @AC-PLT-DOC-INDEX-FRONTMATTER
  Scenario: docs-check validates bidirectional front-matter alignment
    When I run "cargo xtask docs-check"
    Then the output should contain "Doc index & front-matter"
    And the check should validate both index-to-doc and doc-to-index

  @AC-PLT-DOC-INDEX-FRONTMATTER
  Scenario: docs-check provides fix hints for front-matter issues
    When I run "cargo xtask docs-check"
    And front-matter mismatches are detected
    Then the output should contain "docs-frontmatter-sync"
    And the output should explain bidirectional alignment

  @AC-PLT-009
  Scenario: docs-check validates doc policies
    When I run "cargo xtask docs-check"
    Then the output should contain "Doc policies"
    And policy violations should provide actionable hints

  @AC-PLT-009
  Scenario: docs-check validates Skills definitions
    When I run "cargo xtask docs-check"
    Then the output should contain "Skills definitions"
    And skill issues should suggest "skills-fmt"

  @AC-PLT-009
  Scenario: docs-check validates Service policies
    When I run "cargo xtask docs-check"
    Then the output should contain "Service policies"
    And service policy issues should reference service_metadata.yaml
