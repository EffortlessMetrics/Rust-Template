package main

# Kubernetes Security and Best Practices Policy
# Enforces security best practices for Kubernetes deployments
#
# Environment-specific rules:
# - dev: Relaxed rules for faster iteration
# - staging: Moderate rules, balancing safety and flexibility
# - production: Strict rules for maximum safety
#
# ============================================================================
# SECRETS AND CONFIGURATION PATTERN (Milestone 3.1)
# ============================================================================
#
# This policy enforces the secure handling of secrets in Kubernetes manifests.
# This is a CRITICAL LLM-safety feature that prevents accidental exposure of
# sensitive credentials in version control or generated manifests.
#
# PATTERN ENFORCED:
# 1. Non-sensitive config -> ConfigMap via envFrom.configMapRef
# 2. Sensitive values -> Secret via envFrom.secretRef or env.valueFrom.secretKeyRef
# 3. DENY literal values for sensitive environment variable names
#
# SENSITIVE PATTERNS (must use secretRef):
# - *PASSWORD, *_PASSWORD, PASSWORD_*
# - *TOKEN, *_TOKEN, TOKEN_*
# - *KEY, *_KEY, KEY_* (except non-sensitive keys like SORT_KEY)
# - *SECRET, *_SECRET, SECRET_*
# - DATABASE_URL, DB_URL, *_DATABASE_URL
# - *API_KEY, API_KEY_*
# - *PRIVATE_KEY, PRIVATE_KEY_*
# - *CREDENTIALS, CREDENTIALS_*
# - *AUTH*, *OAUTH*
#
# SECURITY BENEFIT:
# - Prevents secrets from being committed to git
# - Forces use of K8s Secrets (encrypted at rest when configured)
# - Makes it obvious in code review when sensitive data is mishandled
# - Reduces attack surface for credential theft from manifest files
# - Enables LLM-assisted code generation to be safer by default
#
# ============================================================================

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

# Observability: Require Prometheus scrape annotations for prod/staging Services
deny[msg] {
    input.kind == "Service"
    service_environment := service_env
    service_environment == "production"
    not has_prometheus_annotations
    msg := sprintf("Production Service %s must have Prometheus scrape annotations (prometheus.io/scrape, prometheus.io/port, prometheus.io/path)", [input.metadata.name])
}

deny[msg] {
    input.kind == "Service"
    service_environment := service_env
    service_environment == "staging"
    not has_prometheus_annotations
    msg := sprintf("Staging Service %s must have Prometheus scrape annotations (prometheus.io/scrape, prometheus.io/port, prometheus.io/path)", [input.metadata.name])
}

has_prometheus_annotations {
    input.metadata.annotations["prometheus.io/scrape"] == "true"
    input.metadata.annotations["prometheus.io/port"]
    input.metadata.annotations["prometheus.io/path"]
}

service_env := env {
    # Detect from app.kubernetes.io/environment label on service
    env := input.metadata.labels["app.kubernetes.io/environment"]
}

service_env := "dev" {
    # Default to dev if no environment label
    not input.metadata.labels["app.kubernetes.io/environment"]
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

# Require liveness and readiness probes in staging (not just warn)
# Staging should mirror prod probes for realistic testing
deny[msg] {
    input.kind == "Deployment"
    environment == "staging"
    container := input.spec.template.spec.containers[_]
    not container.livenessProbe
    msg := sprintf("Staging deployment %s container %s must have a liveness probe (staging should mirror prod)", [input.metadata.name, container.name])
}

deny[msg] {
    input.kind == "Deployment"
    environment == "staging"
    container := input.spec.template.spec.containers[_]
    not container.readinessProbe
    msg := sprintf("Staging deployment %s container %s must have a readiness probe (staging should mirror prod)", [input.metadata.name, container.name])
}

# PodDisruptionBudget policies
# Note: The requirement for PDB in production is documented here
# In practice, PDB is a separate resource and would be validated via:
# 1. Checking that a PDB resource exists with matching selector
# 2. This can be done in CI/CD by validating all manifests together
# 3. Or by using a tool like Gatekeeper/Kyverno in-cluster
#
# For now, we validate the PDB resource itself below when it's present

# Validate PodDisruptionBudget configuration
deny[msg] {
    input.kind == "PodDisruptionBudget"
    has_min_available
    min_available := to_number(input.spec.minAvailable)
    min_available < 1
    msg := sprintf("PodDisruptionBudget %s minAvailable must be at least 1 (found: %d)", [input.metadata.name, min_available])
}

deny[msg] {
    input.kind == "PodDisruptionBudget"
    has_max_unavailable
    max_unavailable := to_number(input.spec.maxUnavailable)
    max_unavailable < 1
    msg := sprintf("PodDisruptionBudget %s maxUnavailable must be at least 1 (found: %d)", [input.metadata.name, max_unavailable])
}

# Warn if PDB minAvailable seems too high (should be less than expected replica count)
warn[msg] {
    input.kind == "PodDisruptionBudget"
    has_min_available
    min_available := to_number(input.spec.minAvailable)
    # For a typical 3-replica deployment, minAvailable should be 2
    # For 2-replica, minAvailable should be 1
    # Warn if minAvailable >= 3 (very restrictive)
    min_available >= 3
    msg := sprintf("PodDisruptionBudget %s has high minAvailable: %d (this may prevent voluntary disruptions)", [input.metadata.name, min_available])
}

has_min_available {
    input.spec.minAvailable
}

has_max_unavailable {
    input.spec.maxUnavailable
}

# ============================================================================
# SECRETS ENFORCEMENT RULES (Milestone 3.1)
# ============================================================================

# List of sensitive environment variable patterns
# These must NEVER have literal values - must use secretRef or secretKeyRef
sensitive_env_patterns := [
    "PASSWORD",
    "TOKEN",
    "SECRET",
    "KEY",
    "DATABASE_URL",
    "DB_URL",
    "API_KEY",
    "PRIVATE_KEY",
    "CREDENTIALS",
    "AUTH",
    "OAUTH",
]

# Check if an environment variable name contains sensitive patterns
is_sensitive_env_name(name) {
    pattern := sensitive_env_patterns[_]
    contains(upper(name), pattern)
    # Exclude known non-sensitive patterns
    not is_non_sensitive_exception(name)
}

# Exceptions for non-sensitive environment variables that contain sensitive keywords
is_non_sensitive_exception(name) {
    # Allow these specific patterns that are not actually sensitive
    upper_name := upper(name)
    exceptions := [
        "SORT_KEY",
        "PARTITION_KEY",
        "CACHE_KEY_PREFIX",
        "LOG_LEVEL",  # Not a KEY in the credential sense
    ]
    exception := exceptions[_]
    upper_name == exception
}

# Deny deployments with literal values for sensitive environment variables
deny[msg] {
    input.kind == "Deployment"
    container := input.spec.template.spec.containers[_]
    env_var := container.env[_]
    is_sensitive_env_name(env_var.name)
    # Has a literal value (not valueFrom)
    env_var.value
    msg := sprintf(
        "SECURITY: Deployment %s container %s has literal value for sensitive env var '%s'. Must use secretRef or secretKeyRef from a Secret resource.",
        [input.metadata.name, container.name, env_var.name]
    )
}

# Deny deployments with configMapKeyRef for sensitive environment variables
deny[msg] {
    input.kind == "Deployment"
    container := input.spec.template.spec.containers[_]
    env_var := container.env[_]
    is_sensitive_env_name(env_var.name)
    # Uses configMapKeyRef instead of secretKeyRef
    env_var.valueFrom.configMapKeyRef
    msg := sprintf(
        "SECURITY: Deployment %s container %s uses configMapKeyRef for sensitive env var '%s'. Must use secretKeyRef from a Secret resource instead.",
        [input.metadata.name, container.name, env_var.name]
    )
}

# Recommend using envFrom for cleaner configuration management
warn[msg] {
    input.kind == "Deployment"
    container := input.spec.template.spec.containers[_]
    # Has individual env vars but no envFrom
    container.env
    not container.envFrom
    count(container.env) > 5
    msg := sprintf(
        "BEST PRACTICE: Deployment %s container %s has %d individual env vars. Consider using envFrom with ConfigMap/Secret for cleaner config management.",
        [input.metadata.name, container.name, count(container.env)]
    )
}

# Require that deployments with envFrom use both ConfigMap and Secret
# This encourages proper separation of sensitive and non-sensitive config
warn[msg] {
    input.kind == "Deployment"
    container := input.spec.template.spec.containers[_]
    container.envFrom
    has_config_map_ref := has_configmap_ref(container)
    has_secret_ref := has_secret_ref(container)
    # Warn if only one type is used (should use both for complete config)
    has_config_map_ref
    not has_secret_ref
    msg := sprintf(
        "BEST PRACTICE: Deployment %s container %s uses ConfigMap but no Secret. Consider using Secret for sensitive values.",
        [input.metadata.name, container.name]
    )
}

has_configmap_ref(container) {
    container.envFrom[_].configMapRef
}

has_secret_ref(container) {
    container.envFrom[_].secretRef
}
