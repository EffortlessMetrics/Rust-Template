# Kubernetes Infrastructure

This directory contains Kubernetes manifests organized by environment using Kustomize overlays.

## Structure

```
k8s/
├── dev/                    # Base manifests (also used for local dev)
│   ├── deployment.yaml
│   └── service.yaml
├── staging/                # Staging overlay
│   ├── kustomization.yaml
│   └── deployment-patch.yaml
├── prod/                   # Production overlay
│   ├── kustomization.yaml
│   └── deployment-patch.yaml
└── README.md
```

## Environments

### Dev

**Purpose**: Local development and testing

**Location**: `dev/`

**Characteristics**:
- Minimal resource limits
- Single replica
- Relaxed security (runs as non-root but less strict)
- Debug-friendly (verbose logging)
- No readiness/liveness probes (faster iteration)

**Deploy**:

```bash
cargo run -p xtask -- deploy --env dev
# Or directly:
kubectl apply -k infra/k8s/dev
```

### Staging

**Purpose**: Pre-production testing, integration tests

**Location**: `staging/`

**Characteristics**:
- Inherits from `dev/` via Kustomize
- 2 replicas (tests load balancing)
- Moderate resource limits
- Info-level logging
- ConfigMap for environment-specific settings

**Deploy**:

```bash
cargo run -p xtask -- deploy --env staging
# Or directly:
kubectl apply -k infra/k8s/staging
```

**What it tests**:
- Multi-replica behavior
- Rolling updates
- ConfigMap injection
- Namespace isolation

### Production

**Purpose**: Live user traffic

**Location**: `prod/`

**Characteristics**:
- Inherits from `dev/` via Kustomize
- 3+ replicas (HA)
- Strict resource limits
- Warn-level logging
- Zero-downtime rolling updates
- Pod anti-affinity (spread across nodes)
- Stricter readiness/liveness probes
- Prometheus annotations
- Production labels (team, cost-center)

**Deploy**:

```bash
cargo run -p xtask -- deploy --env prod
# Or directly:
kubectl apply -k infra/k8s/prod
```

**Safety features**:
- `maxUnavailable: 0` ensures zero-downtime
- Pod anti-affinity spreads pods across nodes
- Tighter probe timeouts catch issues fast
- Resource limits prevent runaway containers

## Customization

### Per-Service Customization

When using this template for a new service:

1. **Search and replace** `my-service` with your service name across all files
2. **Adjust namespaces** in `staging/kustomization.yaml` and `prod/kustomization.yaml`
3. **Tune resource limits** in `staging/deployment-patch.yaml` and `prod/deployment-patch.yaml` based on actual usage
4. **Update labels** in `prod/kustomization.yaml` (team, cost-center, etc.)

### Adding Secrets

Add secrets via Kustomize's `secretGenerator`:

```yaml
# In staging/kustomization.yaml or prod/kustomization.yaml
secretGenerator:
  - name: app-secrets
    literals:
      - DATABASE_URL=postgres://...
    files:
      - tls.crt
      - tls.key
```

Then reference in `deployment-patch.yaml`:

```yaml
env:
  - name: DATABASE_URL
    valueFrom:
      secretKeyRef:
        name: app-secrets
        key: DATABASE_URL
```

### Adding New Resources

To add more K8s resources (ConfigMap, Secret, Ingress, etc.):

1. Add the base manifest in `dev/` (e.g., `ingress.yaml`)
2. Reference it in `staging/kustomization.yaml` and `prod/kustomization.yaml` under `resources:`
3. Add environment-specific patches if needed

Example:

```yaml
# dev/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: my-service
spec:
  rules:
    - host: my-service.local
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: my-service
                port:
                  number: 80
```

```yaml
# staging/kustomization.yaml
resources:
  - ../dev
  - ingress-patch.yaml  # Override host for staging

# staging/ingress-patch.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: my-service
spec:
  rules:
    - host: my-service.staging.example.com
```

## Policy Validation

All manifests are validated via Rego policies in `policy/k8s.rego`. Run validation:

```bash
cargo run -p xtask -- policy-test
```

Policies enforce:
- Security: no root containers, read-only filesystem where possible
- Labels: required labels present
- Resources: limits defined
- Readiness: health check endpoints exist

## Troubleshooting

### Kustomize build fails

```bash
# Test Kustomize build without applying
kubectl kustomize infra/k8s/staging
kubectl kustomize infra/k8s/prod
```

### Deployment stuck in rollout

```bash
# Check events
kubectl describe deployment my-service -n my-service-staging

# Check pod logs
kubectl logs -f deployment/my-service -n my-service-staging

# Rollback if needed
kubectl rollout undo deployment/my-service -n my-service-staging
```

### Resource limits too tight

If pods are OOMKilled or CPU-throttled:

1. Check actual usage: `kubectl top pods -n my-service-staging`
2. Adjust limits in `deployment-patch.yaml`
3. Redeploy: `cargo run -p xtask -- deploy --env staging`

## References

- Kustomize docs: <https://kustomize.io/>
- K8s deployment docs: <https://kubernetes.io/docs/concepts/workloads/controllers/deployment/>
- Template policy docs: `../../policy/README.md`
- Deploy command: `../../docs/how-to/deploy-dev.md`
