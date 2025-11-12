@AC-123
Feature: Refund API
  Scenario: Returns ID within 200ms
    Given an unpaid invoice exists
    When I request a refund
    Then I get a 200 with a refund_id
