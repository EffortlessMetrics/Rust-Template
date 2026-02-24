# Template Version: v3.3.8
# Schema: spec_ledger.yaml v1.0
# Last Updated: 2025-11-22

Feature: Local Runtime Infrastructure
  As a developer
  I want reproducible local runtime dependencies
  So that I can develop and test without external cloud dependencies

  Background:
    Given I am in the actual workspace

  @AC-TPL-LOCAL-DOCKER
  Scenario: docker-compose.yaml provides Postgres and Jaeger
    Then the file "docker-compose.yaml" should exist
    And "docker-compose.yaml" should contain "postgres:16"
    And "docker-compose.yaml" should contain "jaegertracing/all-in-one"
    And "docker-compose.yaml" should contain "HTTP_PORT"
    And "docker-compose.yaml" should contain "OTEL_EXPORTER_OTLP_ENDPOINT"

  @AC-TPL-IAC-COMPOSE-ALIGN
  Scenario: docker-compose ports align with config schema defaults
    Then the file "docker-compose.yaml" should exist
    And "docker-compose.yaml" should contain "8080"
    And the file "specs/config_schema.yaml" should exist
    And the file "specs/config_schema.yaml" should contain "8080"

  @AC-TPL-IAC-K8S-ALIGN
  Scenario: Kubernetes dev manifests align with config defaults
    Then the file "infra/k8s/dev/deployment.yaml" should exist
    And the file "infra/k8s/dev/deployment.yaml" should contain "containerPort: 8080"
    And the file "infra/k8s/dev/configmap.yaml" should contain "PORT:"
    And the file "infra/k8s/dev/service.yaml" should contain "targetPort: http"

  @AC-TPL-IAC-TF-ALIGN
  Scenario: Terraform variables mirror governed config keys
    Then the file "infra/tf/main.tf" should exist
    And the file "infra/tf/main.tf" should contain "http_port"
    And the file "infra/tf/main.tf" should contain "telemetry_otlp_endpoint"
    And the file "infra/tf/main.tf" should contain "platform_auth_mode"
    And the file "specs/config_schema.yaml" should contain "http.port"
