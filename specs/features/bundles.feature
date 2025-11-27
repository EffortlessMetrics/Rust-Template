@bundles @platform @devex
Feature: LLM Bundle Structure
  As a developer or AI agent
  I want to generate context bundles with a predictable structure
  So that I can provide consistent, reliable context for LLM-assisted development

  @AC-TPL-BUNDLE-LAYOUT
  Scenario: Bundle implement_ac creates expected structure
    Given I am in the actual workspace
    When I run "cargo xtask bundle implement_ac"
    Then the command should succeed
    And the output should contain "bundle"
