package k8s

# Kubernetes Security and Best Practices Policy
# Enforces security best practices for Kubernetes deployments

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
