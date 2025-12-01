@bundles @platform @devex
Feature: LLM Bundle Structure
  As a developer or AI agent
  I want to generate context bundles with a predictable structure
  So that I can provide consistent, reliable context for LLM-assisted development

  @AC-TPL-BUNDLE-LAYOUT
  Scenario: Bundle creates directory structure with manifest and context
    Given I am in the actual workspace
    When I run "cargo xtask bundle implement_ac"
    Then the command should succeed
    And the output should contain "Generated"
    And the output should contain "bundle/implement_ac"
    And the file "bundle/implement_ac/bundle.yaml" should exist
    And the file "bundle/implement_ac/context.md" should exist

  @AC-TPL-BUNDLE-MANIFEST
  Scenario: Bundle manifest contains required fields
    Given I am in the actual workspace
    When I run "cargo xtask bundle implement_ac"
    Then the command should succeed
    And "bundle/implement_ac/bundle.yaml" should contain "bundle_version:"
    And "bundle/implement_ac/bundle.yaml" should contain "task_id:"
    And "bundle/implement_ac/bundle.yaml" should contain "git_sha:"
    And "bundle/implement_ac/bundle.yaml" should contain "timestamp:"
    And "bundle/implement_ac/bundle.yaml" should contain "specs:"
    And "bundle/implement_ac/bundle.yaml" should contain "docs:"

  @AC-TPL-BUNDLE-MANIFEST
  Scenario: Manifest task_id matches requested task name
    Given I am in the actual workspace
    When I run "cargo xtask bundle implement_ac"
    Then the command should succeed
    And "bundle/implement_ac/bundle.yaml" should contain "task_id: implement_ac"

  @AC-TPL-BUNDLE-LAYOUT
  Scenario: Context markdown is readable and includes files
    Given I am in the actual workspace
    When I run "cargo xtask bundle implement_ac"
    Then the command should succeed
    And "bundle/implement_ac/context.md" should contain "# LLM Context Bundle"
    And "bundle/implement_ac/context.md" should contain "# FILE:"
    And "bundle/implement_ac/context.md" should contain "spec_ledger.yaml"

  @AC-TPL-BUNDLE-MANIFEST-LINKED
  Scenario: Manifest populates requirement and AC IDs from tasks.yaml
    Given I am in the actual workspace
    When I run "cargo xtask bundle implement_ac"
    Then the command should succeed
    And "bundle/implement_ac/bundle.yaml" should contain "requirement_ids:"
    And "bundle/implement_ac/bundle.yaml" should contain "REQ-TPL-SUGGEST-NEXT"
    And "bundle/implement_ac/bundle.yaml" should contain "ac_ids:"
    And "bundle/implement_ac/bundle.yaml" should contain "AC-TPL-SUGGEST-NEXT-CLI"

  @AC-TPL-BUNDLE-MANIFEST-LINKED
  Scenario: Manifest populates tests from spec_ledger for linked ACs
    Given I am in the actual workspace
    When I run "cargo xtask bundle implement_ac"
    Then the command should succeed
    And "bundle/implement_ac/bundle.yaml" should contain "tests:"
    And "bundle/implement_ac/bundle.yaml" should contain "type:"
    And "bundle/implement_ac/bundle.yaml" should contain "tag:"

  @AC-TPL-BUNDLE-MANIFEST
  Scenario: Manifest includes bundle_version 1 for current schema
    Given I am in the actual workspace
    When I run "cargo xtask bundle implement_ac"
    Then the command should succeed
    And "bundle/implement_ac/bundle.yaml" should contain "bundle_version: 1"
