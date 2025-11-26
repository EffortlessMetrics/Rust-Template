# Product Catalog Feature
# Schema: spec_ledger.yaml v1.0
# Last Updated: 2025-11-26
#
# This is an example BDD feature file for domain customization.
# Copy this pattern when creating your own domain features.

Feature: Product Catalog Management
  As a product manager
  I want to create and retrieve products
  So customers can browse and purchase items

  Background:
    Given the service is running
    And the database is clean

  @AC-PROD-001 @smoke
  Scenario: Create a new product successfully
    When I POST to "/api/products" with:
      """
      {
        "name": "Wireless Mouse",
        "description": "Ergonomic wireless mouse with USB receiver",
        "price": 29.99
      }
      """
    Then the response status is 201
    And the response body contains:
      | field       | value          |
      | name        | Wireless Mouse |
    And the response body has field "productId"
    And the X-Request-ID header is present

  @AC-PROD-002
  Scenario: Reject product creation without required fields
    When I POST to "/api/products" with:
      """
      {
        "description": "Missing name and price"
      }
      """
    Then the response status is 400
    And the response body contains:
      | field       | value            |
      | error.code  | VALIDATION_ERROR |
    And the response body contains error message mentioning "name"
    And the response body contains error message mentioning "price"

  @AC-PROD-002
  Scenario: Reject product creation with missing name
    When I POST to "/api/products" with:
      """
      {
        "price": 19.99
      }
      """
    Then the response status is 400
    And the response body contains:
      | field       | value            |
      | error.code  | VALIDATION_ERROR |
    And the response body contains error message mentioning "name"

  @AC-PROD-003
  Scenario: Reject product with negative price
    When I POST to "/api/products" with:
      """
      {
        "name": "Invalid Product",
        "price": -10.00
      }
      """
    Then the response status is 400
    And the response body contains:
      | field       | value         |
      | error.code  | INVALID_PRICE |
    And the response body contains error message mentioning "negative"

  @AC-PROD-003
  Scenario: Reject product with zero price
    When I POST to "/api/products" with:
      """
      {
        "name": "Free Product",
        "price": 0.00
      }
      """
    Then the response status is 400
    And the response body contains:
      | field       | value         |
      | error.code  | INVALID_PRICE |

  @AC-PROD-004 @smoke
  Scenario: Retrieve an existing product by ID
    Given I have created a product with:
      | name        | Mechanical Keyboard      |
      | description | RGB backlit keyboard     |
      | price       | 89.99                    |
    When I GET "/api/products/{productId}"
    Then the response status is 200
    And the response body contains:
      | field       | value                |
      | name        | Mechanical Keyboard  |
      | price       | 89.99                |
    And the response body has field "productId"
    And the response body has field "createdAt"

  @AC-PROD-005
  Scenario: Return 404 for non-existent product
    When I GET "/api/products/00000000-0000-0000-0000-000000000000"
    Then the response status is 404
    And the response body contains:
      | field       | value              |
      | error.code  | PRODUCT_NOT_FOUND  |
    And the X-Request-ID header is present

  @AC-PROD-005
  Scenario: Return 400 for malformed product ID
    When I GET "/api/products/not-a-uuid"
    Then the response status is 400
    And the response body contains:
      | field       | value            |
      | error.code  | VALIDATION_ERROR |

# Notes for Customization:
#
# 1. Scenario Tags: Use @AC-YOUR-PREFIX-XXX to link to spec_ledger.yaml
#    - @smoke for critical happy paths
#    - @AC-PROD-001, @AC-PROD-002, etc. for traceability
#
# 2. Background: Set up common preconditions
#    - Service running
#    - Clean database state
#    - Mock external dependencies
#
# 3. Given/When/Then: Follow BDD patterns
#    - Given: Set up test state
#    - When: Execute the action under test
#    - Then: Assert expected outcomes
#
# 4. Response Validation:
#    - Check status codes
#    - Verify response body structure
#    - Assert X-Request-ID header (template error handling)
#    - Validate error codes and messages
#
# 5. Test Data:
#    - Use realistic examples
#    - Cover happy paths and error cases
#    - Test boundary conditions (negative prices, empty strings, etc.)
#
# 6. Integration with Template:
#    - Your scenarios run alongside template core scenarios
#    - cargo xtask test-ac AC-PROD-XXX runs specific scenarios
#    - cargo xtask selftest runs all BDD tests
#
# 7. Step Definitions:
#    - Implement step definitions in crates/acceptance/src/steps/
#    - Reuse template core steps (HTTP helpers, request ID checks)
#    - Add domain-specific steps as needed
