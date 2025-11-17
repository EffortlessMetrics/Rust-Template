# Optional Kubernetes Modules - Examples

This directory contains **optional** Kubernetes resources that are **NOT** included in the base template by default. These examples are provided for you to add to your deployment when you need them.

## Why These Are Optional

The template includes a **minimal, portable foundation** that works on any Kubernetes cluster:
- Deployment
- Service
- Environment overlays (dev/staging/prod)

The resources in this directory depend on:
- **Cluster-specific capabilities** (e.g., CNI with NetworkPolicy support)
- **External infrastructure** (e.g., Ingress controllers, metrics-server)
- **Operational maturity** (e.g., knowing your scaling patterns before adding HPA)

**Philosophy**: Start simple, add complexity only when needed.

## Available Examples

| Resource | File | When to Use | Prerequisites |
|----------|------|-------------|---------------|
| **HorizontalPodAutoscaler** | `hpa-example.yaml` | Variable load, cost optimization | metrics-server installed |
| **NetworkPolicy** | `networkpolicy-example.yaml` | Multi-tenant, compliance, zero-trust | CNI with NetworkPolicy support (Calico, Cilium, etc.) |
| **Ingress** | `ingress-example.yaml` | External traffic routing, TLS termination | Ingress controller (nginx, Traefik, ALB, etc.) |
| **PodDisruptionBudget** | `pdb-example.yaml` | High availability, 2+ replicas | None (works on any cluster) |

## Quick Reference

### HorizontalPodAutoscaler (HPA)

**What it does**: Automatically scales pod count based on CPU/memory/custom metrics.

**Use when**:
- Your service has variable load (not constant traffic)
- You want to optimize costs by scaling down during low traffic
- You have metrics infrastructure and know your scaling thresholds

**Don't use when**:
- Your service has constant load (use fixed replicas)
- You're in dev/staging (use fixed replicas for predictability)
- Your cluster doesn't have metrics-server

**Example**:
```yaml
# Basic CPU-based scaling
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: app-http
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: app-http
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
```

**See**: `hpa-example.yaml` for complete examples with extensive comments.

---

### NetworkPolicy

**What it does**: Acts as firewall rules for pods, controlling ingress/egress traffic.

**Use when**:
- You have multi-tenant clusters (need isolation)
- You have compliance requirements (PCI-DSS, HIPAA)
- You want defense in depth (zero-trust networking)

**Don't use when**:
- Your CNI doesn't support NetworkPolicy (kubenet, basic Flannel)
- You're in local dev (adds friction, no security benefit)
- You haven't mapped service dependencies yet (you'll break things)

**Example**:
```yaml
# Deny-all baseline + allow specific traffic
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: app-http
spec:
  podSelector:
    matchLabels:
      app: app-http
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx
    ports:
    - protocol: TCP
      port: 8080
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          name: kube-system
      podSelector:
        matchLabels:
          k8s-app: kube-dns
    ports:
    - protocol: UDP
      port: 53
```

**See**: `networkpolicy-example.yaml` for complete examples with deny-all baseline and common patterns.

---

### Ingress

**What it does**: Exposes HTTP/HTTPS routes from outside the cluster to services within.

**Use when**:
- You need external access to your service
- You want SSL/TLS termination at the edge
- You need to consolidate multiple services under one domain/IP

**Don't use when**:
- Your service is internal-only (use Service type: ClusterIP)
- You need TCP/UDP protocols (use Gateway API or LoadBalancer)
- Your cluster doesn't have an Ingress controller

**Example**:
```yaml
# Basic HTTPS Ingress with cert-manager
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: app-http
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/force-ssl-redirect: "true"
spec:
  ingressClassName: nginx
  tls:
  - hosts:
    - myapp.example.com
    secretName: app-http-tls
  rules:
  - host: myapp.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: app-http
            port:
              number: 80
```

**See**: `ingress-example.yaml` for nginx, Traefik, AWS ALB, GCP GCE examples with TLS setup.

---

### PodDisruptionBudget (PDB)

**What it does**: Ensures minimum availability during voluntary disruptions (node drains, cluster upgrades).

**Use when**:
- You're running production with 2+ replicas
- You have strict availability requirements (SLA-driven)
- Your cluster has frequent node maintenance

**Don't use when**:
- You have single replica (PDB has no effect or blocks drains)
- You're in dev/staging (no availability requirements)

**Example**:
```yaml
# Keep at least 2 pods running during disruptions
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: app-http
spec:
  minAvailable: 2
  selector:
    matchLabels:
      app: app-http
```

**See**: `pdb-example.yaml` for complete examples including HPA considerations.

---

## How to Use These Examples

### Step 1: Verify Prerequisites

Each optional module has different requirements:

**For HPA**:
```bash
# Check if metrics-server is installed
kubectl get deployment metrics-server -n kube-system

# Test metrics availability
kubectl top pods -n default
```

**For NetworkPolicy**:
```bash
# Check if your CNI supports NetworkPolicy
kubectl get pods -n kube-system | grep -E 'calico|cilium|weave'

# Test enforcement (create test policy and verify it blocks traffic)
# See networkpolicy-example.yaml for testing instructions
```

**For Ingress**:
```bash
# Check if Ingress controller is installed
kubectl get ingressclass

# Check Ingress controller pods
kubectl get pods -n ingress-nginx  # Or your controller's namespace
```

**For PDB**:
```bash
# No prerequisites - works on any cluster
# But verify you have 2+ replicas
kubectl get deployment app-http -n default
```

### Step 2: Copy Relevant Example

```bash
# Copy the example you need to your environment overlay
cp infra/k8s/examples/hpa-example.yaml infra/k8s/prod/hpa.yaml
cp infra/k8s/examples/networkpolicy-example.yaml infra/k8s/prod/networkpolicy.yaml
cp infra/k8s/examples/ingress-example.yaml infra/k8s/prod/ingress.yaml
cp infra/k8s/examples/pdb-example.yaml infra/k8s/prod/pdb.yaml
```

### Step 3: Customize for Your Service

Edit the copied file:

1. **Update metadata**: Change `name`, `namespace`, `labels` to match your service
2. **Update selectors**: Match your Deployment's pod labels
3. **Update configuration**: Adjust values based on your requirements
   - HPA: `minReplicas`, `maxReplicas`, `averageUtilization`
   - NetworkPolicy: Add ingress/egress rules for your dependencies
   - Ingress: Update `host`, `secretName`, annotations for your Ingress controller
   - PDB: Set `minAvailable` or `maxUnavailable` based on your replica count

### Step 4: Add to Kustomization

Reference the new resource in your environment's `kustomization.yaml`:

```yaml
# infra/k8s/prod/kustomization.yaml
resources:
  - ../dev
  - hpa.yaml             # Add this
  - networkpolicy.yaml   # Add this
  - ingress.yaml         # Add this
  - pdb.yaml             # Add this
```

### Step 5: Deploy and Test

**Deploy to staging first** (always):

```bash
kubectl apply -k infra/k8s/staging
```

**Test the new resource**:

```bash
# For HPA
kubectl get hpa -n my-service-staging -w
# Generate load and watch scaling

# For NetworkPolicy
kubectl exec -it <pod> -n my-service-staging -- sh
curl http://external-service  # Should work or fail based on policy

# For Ingress
curl https://myapp.staging.example.com/health

# For PDB
kubectl get pdb -n my-service-staging
kubectl drain <node> --dry-run=client  # Test drain simulation
```

**If tests pass, deploy to production**:

```bash
kubectl apply -k infra/k8s/prod
```

## Common Integration Patterns

### Pattern 1: Gradual Adoption (Recommended)

Add optional modules **one at a time**, testing thoroughly between each:

1. Start with base template (Deployment, Service)
2. Add PDB (simplest, no dependencies)
3. Add Ingress (if you need external access)
4. Add NetworkPolicy (once you've mapped dependencies)
5. Add HPA (once you've profiled resource usage)

### Pattern 2: Environment-Specific

Use different optional modules per environment:

```yaml
# dev/kustomization.yaml
resources:
  - deployment.yaml
  - service.yaml
# No optional modules in dev

# staging/kustomization.yaml
resources:
  - ../dev
  - pdb.yaml  # Test PDB behavior

# prod/kustomization.yaml
resources:
  - ../dev
  - hpa.yaml           # Scale for production traffic
  - networkpolicy.yaml # Security for production
  - ingress.yaml       # External access
  - pdb.yaml           # High availability
```

### Pattern 3: Feature Flags in Kustomization

Make it explicit what's enabled:

```yaml
# infra/k8s/prod/kustomization.yaml
resources:
  - ../dev

  # Optional modules (uncomment to enable):
  # - hpa.yaml
  # - networkpolicy.yaml
  - ingress.yaml         # Enabled
  - pdb.yaml             # Enabled
```

## Extending Policy Validation

When you add optional modules, **extend the policy** to enforce safe configurations.

Create new policy files in `policy/` directory:

```bash
# Create policy for each optional module
touch policy/k8s_hpa.rego
touch policy/k8s_networkpolicy.rego
touch policy/k8s_ingress.rego
touch policy/k8s_pdb.rego
```

**Example: HPA policy** (`policy/k8s_hpa.rego`):

```rego
package main

import future.keywords.in

# Require minReplicas >= 3 in production
deny[msg] {
    input.kind == "HorizontalPodAutoscaler"
    environment == "production"
    min_replicas := to_number(input.spec.minReplicas)
    min_replicas < 3
    msg := sprintf("HPA %s must have minReplicas >= 3 in production (found: %d)",
                   [input.metadata.name, min_replicas])
}

# Prevent CPU targets < 50% (too aggressive)
deny[msg] {
    input.kind == "HorizontalPodAutoscaler"
    metric := input.spec.metrics[_]
    metric.type == "Resource"
    metric.resource.name == "cpu"
    target := metric.resource.target.averageUtilization
    target < 50
    msg := sprintf("HPA %s CPU target %d%% is too aggressive (use >= 50%%)",
                   [input.metadata.name, target])
}
```

**Test your policies**:

```bash
# Add test cases in policy/testdata/
# policy/testdata/k8s_hpa_invalid.yaml
# policy/testdata/k8s_hpa_valid.yaml

# Run policy tests
cargo run -p xtask -- policy-test
```

See `docs/explanation/infra-modules.md` for complete policy examples.

## Troubleshooting

### HPA Not Scaling

```bash
# Check HPA status
kubectl get hpa -n default
# Look for "unable to get metrics"

# Check metrics-server
kubectl get deployment metrics-server -n kube-system
kubectl logs -n kube-system -l k8s-app=metrics-server

# Verify resource requests are set
kubectl get deployment app-http -o yaml | grep -A 5 resources:
```

### NetworkPolicy Blocking Traffic

```bash
# Check if CNI enforces NetworkPolicy
kubectl get networkpolicy -n default

# Test connectivity
kubectl exec -it <pod> -n default -- sh
curl http://other-service

# Check CNI logs for drops
# Cilium:
kubectl -n kube-system exec -it cilium-xxxxx -- cilium monitor --type drop

# Calico:
kubectl logs -n kube-system -l k8s-app=calico-node | grep -i drop
```

### Ingress Not Reachable

```bash
# Check Ingress status
kubectl get ingress -n default
kubectl describe ingress app-http -n default

# Check Ingress controller logs
kubectl logs -n ingress-nginx -l app.kubernetes.io/name=ingress-nginx

# Verify DNS
nslookup myapp.example.com
# Should return Ingress controller's external IP

# Check certificate (if using TLS)
kubectl get certificate -n default
kubectl describe certificate app-http-tls -n default
```

### PDB Blocking Node Drain

```bash
# Check PDB status
kubectl get pdb -n default
# If "ALLOWED DISRUPTIONS: 0", drain will block

# Check why
kubectl describe pdb app-http -n default
# Look at Current/Desired/Expected pods

# Fix: Scale up Deployment to allow disruptions
kubectl scale deployment app-http --replicas=4 -n default
# Now PDB might allow 1 disruption
```

## Best Practices

1. **Always test in staging first** - Don't deploy optional modules directly to production
2. **Add incrementally** - One module at a time, not all at once
3. **Monitor after adding** - Watch metrics, logs, and events for issues
4. **Document your choices** - Update your README with which modules you use and why
5. **Review periodically** - Remove modules you're not actually using

## Further Reading

- **Comprehensive guide**: `docs/explanation/infra-modules.md`
  - Detailed explanation of why each module is optional
  - Trade-offs and when to use each
  - Policy extension examples
  - Common mistakes and debugging tips

- **Core infrastructure**: `../README.md`
  - What's included in the base template
  - How to customize deployments
  - Kustomize overlay structure

- **Policy validation**: `../../policy/k8s.rego`
  - Existing policies for Deployment/Service
  - How to extend for optional modules

## Summary

| Module | Complexity | Risk | Benefit | Recommendation |
|--------|-----------|------|---------|----------------|
| **PDB** | Low | Low | High (for HA) | Add early for production 2+ replicas |
| **Ingress** | Medium | Medium | High (for external access) | Add when you need external routing |
| **HPA** | Medium | Medium | Medium (for variable load) | Add after profiling resource usage |
| **NetworkPolicy** | High | High | High (for security) | Add when you have compliance requirements |

**Start simple, add complexity only when you have a clear need.**

## Contributing

If you add a new optional module example:

1. Create well-commented YAML with multiple examples
2. Add entry to this README
3. Add policy validation (if applicable)
4. Add test cases in `policy/testdata/`
5. Update `docs/explanation/infra-modules.md`

Examples of other optional modules you might add:
- ServiceMonitor (for Prometheus Operator)
- VirtualService (for Istio)
- ConfigMap/Secret generators
- CronJob for periodic tasks
- Custom metrics for HPA
