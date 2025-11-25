Feature: Configuration validation

  Background:
    Given the platform is running

  @AC-TPL-CONFIG-VALIDATION
  Scenario: Invalid configuration fails fast
    Given the config file "config/local.yaml" contains:
      """
      env: dev
      settings:
        http.port: "not-a-number"
      secrets:
        db.url: "postgres://postgres:postgres@localhost:5432/app"
        auth.jwt_signing_key: "dev-secret-key"
      """
    When I validate the configuration against the schema
    Then the configuration validation should fail
    And the validation error should contain "http.port"
