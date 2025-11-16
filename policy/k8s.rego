package main

# Kubernetes Security and Best Practices Policy
# Enforces security best practices for Kubernetes deployments
#
# Environment-specific rules:
# - dev: Relaxed rules for faster iteration
# - staging: Moderate rules, balancing safety and flexibility
# - production: Strict rules for maximum safety

import future.keywords.in

# Deny containers running as root
deny[msg] {
    input.kind == "Deployment"
    not containers_run_as_nonroot
    msg := sprintf("Deployment %s must set securityContext.runAsNonRoot: true at pod or container level", [input.metadata.name])
}

containers_run_as_nonroot {
    # Check pod-level securityContext
    input.spec.template.spec.securityContext.runAsNonRoot == true
}

containers_run_as_nonroot {
    # Check all containers have runAsNonRoot set
    container := input.spec.template.spec.containers[_]
    container.securityContext.runAsNonRoot == true
}

# Require specific labels on deployments
required_labels := ["app", "version", "component"]

deny[msg] {
    input.kind == "Deployment"
    missing_label := missing_required_label
    msg := sprintf("Deployment %s is missing required label: %s", [input.metadata.name, missing_label])
}

missing_required_label := label {
    label := required_labels[_]
    not input.metadata.labels[label]
}

# Require resource limits on all containers
deny[msg] {
    input.kind == "Deployment"
    container := input.spec.template.spec.containers[_]
    not container.resources.limits
    msg := sprintf("Deployment %s container %s must have resource limits defined", [input.metadata.name, container.name])
}

deny[msg] {
    input.kind == "Deployment"
    container := input.spec.template.spec.containers[_]
    not container.resources.limits.cpu
    msg := sprintf("Deployment %s container %s must have CPU limit defined", [input.metadata.name, container.name])
}

deny[msg] {
    input.kind == "Deployment"
    container := input.spec.template.spec.containers[_]
    not container.resources.limits.memory
    msg := sprintf("Deployment %s container %s must have memory limit defined", [input.metadata.name, container.name])
}

# Require resource requests on all containers
deny[msg] {
    input.kind == "Deployment"
    container := input.spec.template.spec.containers[_]
    not container.resources.requests
    msg := sprintf("Deployment %s container %s must have resource requests defined", [input.metadata.name, container.name])
}

deny[msg] {
    input.kind == "Deployment"
    container := input.spec.template.spec.containers[_]
    not container.resources.requests.cpu
    msg := sprintf("Deployment %s container %s must have CPU request defined", [input.metadata.name, container.name])
}

deny[msg] {
    input.kind == "Deployment"
    container := input.spec.template.spec.containers[_]
    not container.resources.requests.memory
    msg := sprintf("Deployment %s container %s must have memory request defined", [input.metadata.name, container.name])
}

# Best practice: Recommend liveness and readiness probes (warnings only)
warn[msg] {
    input.kind == "Deployment"
    container := input.spec.template.spec.containers[_]
    not container.livenessProbe
    msg := sprintf("Deployment %s container %s should have a liveness probe", [input.metadata.name, container.name])
}

warn[msg] {
    input.kind == "Deployment"
    container := input.spec.template.spec.containers[_]
    not container.readinessProbe
    msg := sprintf("Deployment %s container %s should have a readiness probe", [input.metadata.name, container.name])
}

# Best practice: Recommend dropping all capabilities
warn[msg] {
    input.kind == "Deployment"
    container := input.spec.template.spec.containers[_]
    not has_dropped_all_capabilities(container)
    msg := sprintf("Deployment %s container %s should drop all capabilities", [input.metadata.name, container.name])
}

has_dropped_all_capabilities(container) {
    container.securityContext.capabilities.drop[_] == "ALL"
}

# Services policy: Require labels
deny[msg] {
    input.kind == "Service"
    missing_label := missing_required_label
    msg := sprintf("Service %s is missing required label: %s", [input.metadata.name, missing_label])
}

# Environment detection
environment := env {
    # Detect from app.kubernetes.io/environment label
    env := input.metadata.labels["app.kubernetes.io/environment"]
}

environment := env {
    # Detect from commonLabels in Kustomize output
    env := input.spec.template.metadata.labels["app.kubernetes.io/environment"]
}

environment := "dev" {
    # Default to dev if no environment label
    not input.metadata.labels["app.kubernetes.io/environment"]
    not input.spec.template.metadata.labels["app.kubernetes.io/environment"]
}

# Production-specific policies
# Require minimum replica count for HA
deny[msg] {
    input.kind == "Deployment"
    environment == "production"
    replicas := to_number(input.spec.replicas)
    replicas < 3
    msg := sprintf("Production deployment %s must have at least 3 replicas for HA (found: %d)", [input.metadata.name, replicas])
}

# Require pod anti-affinity in production
deny[msg] {
    input.kind == "Deployment"
    environment == "production"
    not has_pod_antiaffinity
    msg := sprintf("Production deployment %s must have pod anti-affinity for spreading across nodes", [input.metadata.name])
}

has_pod_antiaffinity {
    input.spec.template.spec.affinity.podAntiAffinity
}

# Require zero-downtime rolling updates in production
deny[msg] {
    input.kind == "Deployment"
    environment == "production"
    strategy := input.spec.strategy
    strategy.type == "RollingUpdate"
    max_unavailable := strategy.rollingUpdate.maxUnavailable
    max_unavailable != 0
    msg := sprintf("Production deployment %s must have maxUnavailable: 0 for zero-downtime updates", [input.metadata.name])
}

# Require production labels (team, cost-center)
production_required_labels := ["team", "cost-center"]

deny[msg] {
    input.kind == "Deployment"
    environment == "production"
    label := production_required_labels[_]
    not input.metadata.labels[label]
    msg := sprintf("Production deployment %s is missing required label: %s", [input.metadata.name, label])
}

# Require liveness and readiness probes in production (not just warn)
deny[msg] {
    input.kind == "Deployment"
    environment == "production"
    container := input.spec.template.spec.containers[_]
    not container.livenessProbe
    msg := sprintf("Production deployment %s container %s must have a liveness probe", [input.metadata.name, container.name])
}

deny[msg] {
    input.kind == "Deployment"
    environment == "production"
    container := input.spec.template.spec.containers[_]
    not container.readinessProbe
    msg := sprintf("Production deployment %s container %s must have a readiness probe", [input.metadata.name, container.name])
}

# Staging-specific policies
# Require at least 2 replicas in staging (for light load testing)
deny[msg] {
    input.kind == "Deployment"
    environment == "staging"
    replicas := to_number(input.spec.replicas)
    replicas < 2
    msg := sprintf("Staging deployment %s should have at least 2 replicas for testing (found: %d)", [input.metadata.name, replicas])
}
