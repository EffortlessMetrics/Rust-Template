Feature: Refunds
  @smoke @AC-123
  Scenario: Create a refund
    Given an order "ORD-1" totalling 5000 cents
    When I POST /refunds with { "orderId": "ORD-1", "amountCents": 5000 }
    Then I receive 201 with a "refundId"
