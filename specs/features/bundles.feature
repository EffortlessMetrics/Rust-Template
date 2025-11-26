Feature: LLM Bundle Structure
  As a developer or AI agent
  I want to generate context bundles with a predictable structure
  So that I can provide consistent, reliable context for LLM-assisted development

  Background:
    Given the xtask CLI is available
    And the spec ledger contains requirements and ACs

  @AC-TPL-BUNDLE-LAYOUT
  Scenario: Bundle implement_ac creates expected structure
    When I run "cargo xtask bundle implement_ac"
    Then the bundle directory should exist under ".llm/bundle/"
    And the bundle should contain "bundle.json"
    And bundle.json should have field "task_id"
    And bundle.json should have field "requirement_ids"
    And bundle.json should have field "ac_ids"
    And bundle.json should have field "files"
    And the bundle should contain at least one context file

  @AC-TPL-BUNDLE-LAYOUT
  Scenario: Bundle JSON contains valid metadata
    Given a bundle has been generated for an AC
    When I read the bundle.json file
    Then the task_id field should be a non-empty string
    And the requirement_ids field should be an array
    And the ac_ids field should be an array
    And the files field should be an array of file paths
    And at least one file in the bundle should be a markdown context file

  @AC-TPL-BUNDLE-LAYOUT
  Scenario: Bundle includes human-readable context
    Given a bundle has been generated for an AC
    When I list the bundle contents
    Then I should find at least one markdown file
    And the markdown file should contain contextual information about the task
    And the markdown file should reference the relevant requirements and ACs
