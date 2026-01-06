@platform @issues @AC-GOV-025
Feature: Platform Issues Endpoint
  As a governance consumer
  I want a stable, predictable /platform/issues endpoint
  So that I can reliably query friction, questions, and tasks in a unified view

  Background:
    Given the service is running
    And the workspace has a clean issues state

  # ============================================================================
  # Schema Stability
  # ============================================================================

  @AC-GOV-025 @schema
  Scenario: Response schema is stable
    Given a friction entry "FRICTION-EX-001" with status "open" and severity "high"
    And a question entry "Q-EX-001" with status "open"
    When I request "/platform/issues" as JSON
    Then the response status should be 200
    And each issue has required fields: kind, id, status, native_status, summary, priority
    And optional fields when present are not null: created_at, category, owner
    And array fields are arrays not null: refs, labels

  @AC-GOV-025 @schema
  Scenario: Status is normalized across all issue types
    Given a friction entry "FRICTION-001" with native status "investigating"
    And a question entry "Q-001" with native status "answered"
    And a task "TASK-001" with native status "InProgress"
    When I request "/platform/issues" as JSON
    Then the response status should be 200
    And each issue status is one of: "open", "in_progress", "resolved"
    And native_status preserves the original value

  # ============================================================================
  # Ordering Stability (Contract)
  # ============================================================================

  @AC-GOV-025 @ordering
  Scenario: Ordering is deterministic - priority first
    Given a friction entry "LOW-001" with priority 3 and date "2025-01-01"
    And a friction entry "HIGH-001" with priority 1 and date "2025-01-01"
    And a friction entry "MED-001" with priority 2 and date "2025-01-01"
    When I request "/platform/issues" as JSON
    Then the response status should be 200
    And the issues are ordered: "HIGH-001", "MED-001", "LOW-001"

  @AC-GOV-025 @ordering
  Scenario: Ordering is deterministic - date second, newest first
    Given a friction entry "OLD-001" with priority 2 and date "2025-01-01"
    And a friction entry "NEW-001" with priority 2 and date "2025-01-15"
    When I request "/platform/issues" as JSON
    Then the response status should be 200
    And the issues are ordered: "NEW-001", "OLD-001"

  @AC-GOV-025 @ordering
  Scenario: Ordering is deterministic - ID as tiebreaker
    Given a friction entry "C-001" with priority 2 and date "2025-01-01"
    And a friction entry "A-001" with priority 2 and date "2025-01-01"
    And a friction entry "B-001" with priority 2 and date "2025-01-01"
    When I request "/platform/issues" as JSON
    Then the response status should be 200
    And the issues are ordered: "A-001", "B-001", "C-001"

  # ============================================================================
  # Filtering
  # ============================================================================

  @AC-GOV-025 @filtering
  Scenario: Filter by kind returns only that kind
    Given a friction entry "FRICTION-001" with status "open"
    And a question entry "Q-001" with status "open"
    And a task "TASK-001" with status "Todo"
    When I request "/platform/issues?kind=friction" as JSON
    Then the response status should be 200
    And every issue kind is "friction"
    And the summary shows total of 1

  @AC-GOV-025 @filtering
  Scenario: Filter by status returns only that status
    Given a friction entry "OPEN-001" with status "open"
    And a friction entry "RESOLVED-001" with status "resolved"
    When I request "/platform/issues?status=open" as JSON
    Then the response status should be 200
    And every issue status is "open"

  @AC-GOV-025 @filtering
  Scenario: Text search matches id, summary, and category
    Given a friction entry "FRICTION-CI-001" with summary "Build failure" and category "ci_cd"
    And a friction entry "FRICTION-DEV-001" with summary "IDE crash" and category "tooling"
    When I request "/platform/issues?q=CI" as JSON
    Then the response status should be 200
    And the first issue id is "FRICTION-CI-001"

  # ============================================================================
  # Cursor Pagination
  # ============================================================================

  @AC-GOV-025 @pagination @cursor
  Scenario: Cursor pagination returns stable pages
    Given 5 friction entries with IDs "A-001" through "A-005"
    When I request "/platform/issues?limit=2" as JSON
    Then I get 2 issues and a next_cursor
    When I request "/platform/issues?limit=2&cursor=<next_cursor>" as JSON
    Then I get 2 issues
    And page 1 and page 2 have no overlapping ids

  @AC-GOV-025 @pagination @cursor
  Scenario: Cursor pagination exhausts all items
    Given 5 friction entries with IDs "A-001" through "A-005"
    When I paginate through all issues with limit 2
    Then I receive exactly 5 unique issues
    And the final page has no next_cursor

  # ============================================================================
  # Offset Pagination (Fallback)
  # ============================================================================

  @AC-GOV-025 @pagination @offset
  Scenario: Offset pagination works with page and per_page
    Given 5 friction entries with IDs "A-001" through "A-005"
    When I request "/platform/issues?page=1&per_page=2" as JSON
    Then I get 2 issues
    And pagination shows page 1 of 3
    And pagination shows total_items of 5

  @AC-GOV-025 @pagination @offset
  Scenario: Offset pagination includes next_cursor for migration
    Given 3 friction entries
    When I request "/platform/issues?page=1&per_page=2" as JSON
    Then I get 2 issues
    And next_cursor is present for consumers who want cursor pagination

  # ============================================================================
  # Summary Counts
  # ============================================================================

  @AC-GOV-025 @summary
  Scenario: Summary reflects filtered items before pagination
    Given 3 friction entries with status "open"
    And 2 question entries with status "open"
    When I request "/platform/issues?limit=2" as JSON
    Then I get 2 issues
    And summary shows total of 5
    And summary shows friction count of 3
    And summary shows question count of 2

  @AC-GOV-025 @summary
  Scenario: Summary counts by status are accurate
    Given a friction entry with status "open"
    And a friction entry with status "resolved"
    And a friction entry with status "investigating"
    When I request "/platform/issues" as JSON
    Then summary shows open count of 1
    And summary shows in_progress count of 1
    And summary shows resolved count of 1

  # ============================================================================
  # Pagination Error Cases (400s)
  # ============================================================================

  @AC-GOV-025 @pagination @error
  Scenario: Mixed pagination params are rejected
    When I request "/platform/issues?cursor=abc&page=2" as JSON
    Then the response status should be 400
    And the error message mentions "cursor" and "page"

  @AC-GOV-025 @pagination @error
  Scenario: Invalid cursor encoding is rejected
    When I request "/platform/issues?cursor=!!!invalid-base64!!!" as JSON
    Then the response status should be 400
    And the error message mentions "cursor"

  @AC-GOV-025 @pagination @error
  Scenario: Cursor exceeding 4KB is rejected
    Given a cursor value of 4097 characters
    When I request "/platform/issues" with that cursor as JSON
    Then the response status should be 400
    And the error message mentions "cursor" and "length"

  @AC-GOV-025 @pagination @error
  Scenario: Unknown cursor version is rejected
    Given a cursor with version 99
    When I request "/platform/issues" with that cursor as JSON
    Then the response status should be 400
    And the error message mentions "version"
