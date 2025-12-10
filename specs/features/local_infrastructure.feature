# Template Version: v3.3.8
# Schema: spec_ledger.yaml v1.0
# Last Updated: 2025-11-22

Feature: Local Runtime Infrastructure
  As a developer
  I want reproducible local runtime dependencies
  So that I can develop and test without external cloud dependencies

  Background:
    Given I am in a Rust-Template workspace

  @AC-TPL-LOCAL-DOCKER
  Scenario: docker-compose.yaml provides Postgres and Jaeger
    Then the file "docker-compose.yaml" should exist
    And "docker-compose.yaml" should contain "postgres:16"
    And "docker-compose.yaml" should contain "jaegertracing/all-in-one"
