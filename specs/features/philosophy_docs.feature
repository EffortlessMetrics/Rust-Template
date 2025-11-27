# Template Version: v3.3.3
# Schema: spec_ledger.yaml v1.0
# Last Updated: 2025-11-26

@philosophy @docs
Feature: Template Philosophy Documentation
  As a fork maintainer
  I want the template's opinionated defaults to be clearly documented
  So I can understand and customize them for my needs

  # Note: These scenarios verify documentation exists and contains key content
  # They run against the actual workspace, not a temp copy

  @AC-TPL-OPINIONS-DOCUMENTED
  Scenario: QUICKSTART.md contains Defaults & Opinions section
    Given I am in the actual workspace
    Then the file "docs/QUICKSTART.md" should exist
    And "docs/QUICKSTART.md" should contain "Defaults & Opinions"
    And "docs/QUICKSTART.md" should contain "environment"
    And "docs/QUICKSTART.md" should contain "selftest"

  @AC-TPL-OPINIONS-DOCUMENTED
  Scenario: ROADMAP.md contains Philosophy section
    Given I am in the actual workspace
    Then the file "docs/ROADMAP.md" should exist
    And "docs/ROADMAP.md" should contain "Philosophy"
    And "docs/ROADMAP.md" should contain "Opinionated"

  @AC-TPL-OVERRIDE-DOC
  Scenario: Override guide exists with recommended steps
    Given I am in the actual workspace
    Then the file "docs/how-to/change-template-opinion.md" should exist
    And "docs/how-to/change-template-opinion.md" should contain "Step 1"
    And "docs/how-to/change-template-opinion.md" should contain "spec_ledger"
    And "docs/how-to/change-template-opinion.md" should contain "selftest"

  @AC-TPL-OVERRIDE-DOC
  Scenario: Override guide provides concrete examples
    Given I am in the actual workspace
    And "docs/how-to/change-template-opinion.md" should contain "Example"
    And "docs/how-to/change-template-opinion.md" should contain "must_have_ac"

  @AC-TPL-OVERRIDE-TRACEABLE
  Scenario: Override doc is registered in doc_index
    Given I am in the actual workspace
    Then the file "specs/doc_index.yaml" should exist
    And "specs/doc_index.yaml" should contain "change-template-opinion"
    And "specs/doc_index.yaml" should contain "override_path"
