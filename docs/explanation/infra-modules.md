# Infrastructure Modules: Core vs Optional

## Overview

This template includes a **minimal, portable foundation** of Kubernetes resources that work across any cluster. Optional modules like HPA, NetworkPolicy, and Ingress are intentionally excluded because they depend on cluster-specific capabilities and operational decisions.

**Philosophy**: Template provides production-ready defaults; you add cluster-specific enhancements.

## Core Infrastructure (Included)

### What's In The Box

The template includes resources that work universally across all Kubernetes clusters:

1. **Deployment** (`infra/k8s/dev/deployment.yaml`)
   - Multi-environment support (dev/staging/prod)
   - Security-hardened (non-root, capabilities dropped)
   - Resource limits enforced
   - Health probes configured
   - Rolling update strategy

2. **Service** (`infra/k8s/dev/service.yaml`)
   - ClusterIP type (internal-only by default)
   - Label selectors for pod discovery
   - Named ports for flexibility

3. **Environment Overlays** (via Kustomize)
   - `dev/`: Minimal resources, fast iteration
   - `staging/`: 2 replicas, moderate resources
   - `prod/`: 3+ replicas, HA, anti-affinity

### Why These Are Core

These resources are:
- **Universal**: Every Kubernetes cluster supports them (no special features required)
- **Essential**: Required for any service to run
- **Low-risk**: Well-understood, stable APIs
- **Policy-enforced**: Validated by OPA policies in `policy/k8s.rego`

### What's Intentionally Excluded

The following are **NOT** included by default:

| Resource | Reason Excluded | Add When... |
|----------|-----------------|-------------|
| **HorizontalPodAutoscaler** | Requires metrics-server, cluster-specific tuning | You have metrics infrastructure and know your scaling patterns |
| **NetworkPolicy** | Requires CNI support (Calico, Cilium, etc.) | Your cluster has CNI with NetworkPolicy support |
| **Ingress/Gateway** | Provider-specific (nginx, Traefik, Istio, etc.) | You need external traffic routing and have an ingress controller |
| **PodDisruptionBudget** | Cluster maturity varies | You're running prod with multiple replicas and need HA guarantees |
| **ServiceMonitor** | Prometheus-specific | You're using Prometheus Operator |
| **VirtualService** | Istio-specific | You're using Istio service mesh |

## Optional Module: HorizontalPodAutoscaler

### When to Use HPA

Use HPA when:
- Your service has **variable load patterns** (not constant traffic)
- You have **metrics-server** or **custom metrics** available
- You know your **scaling thresholds** (CPU/memory targets)
- You want to **optimize costs** by scaling down during low traffic

**Don't use HPA when**:
- Your service has **constant load** (just set replicas directly)
- You're in **dev/staging** (use fixed replicas for predictability)
- You haven't **profiled** your resource usage yet (you'll set bad targets)
- Your cluster **doesn't have metrics-server** installed

### How HPA Works

```
┌──────────────┐
│ metrics-     │  Scrapes pod metrics every 15s
│ server       │  (CPU, memory, custom metrics)
└──────┬───────┘
       │
       ▼
┌──────────────────┐
│ HPA Controller   │  Compares current vs target
│                  │  Calculates desired replicas
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ Deployment       │  Scales pod count up/down
│ (replicas: N)    │
└──────────────────┘
```

### Example Manifest

See `infra/k8s/examples/hpa-example.yaml` for a complete, annotated example.

**Quick start**:
```yaml
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

### Key Configuration Options

| Field | Purpose | Recommendation |
|-------|---------|----------------|
| `minReplicas` | Floor for scaling | Set to your HA requirement (3+ for prod) |
| `maxReplicas` | Ceiling for scaling | Based on budget and cluster capacity |
| `averageUtilization` | Target percentage | 70% for CPU, 80% for memory |
| `scaleDown.stabilizationWindowSeconds` | Delay before scaling down | 300s (5 min) to avoid flapping |
| `scaleUp.stabilizationWindowSeconds` | Delay before scaling up | 0s (scale up fast) |

### Policy Considerations

If you add HPA to production, **extend the policy** to enforce safe defaults:

```rego
# In policy/k8s_hpa.rego (create this file)
package main

import future.keywords.in

# Require minimum replicas >= 3 in production
deny[msg] {
    input.kind == "HorizontalPodAutoscaler"
    environment == "production"
    min_replicas := to_number(input.spec.minReplicas)
    min_replicas < 3
    msg := sprintf("HPA %s must have minReplicas >= 3 in production (found: %d)",
                   [input.metadata.name, min_replicas])
}

# Prevent overly aggressive scaling
deny[msg] {
    input.kind == "HorizontalPodAutoscaler"
    metric := input.spec.metrics[_]
    metric.type == "Resource"
    metric.resource.name == "cpu"
    target := metric.resource.target.averageUtilization
    target < 50
    msg := sprintf("HPA %s has CPU target too low (%d%%) - risk of over-scaling",
                   [input.metadata.name, target])
}
```

### Why It's Not Default

1. **Cluster dependency**: Requires metrics-server (not always present)
2. **Premature optimization**: Most services should start with fixed replicas
3. **Tuning required**: Bad HPA settings cause thrashing (constant scaling)
4. **Complexity**: Adds operational overhead (monitoring HPA behavior)

### How to Add to Your Deployment

1. **Verify metrics-server is installed**:
   ```bash
   kubectl get deployment metrics-server -n kube-system
   ```

2. **Copy the example**:
   ```bash
   cp infra/k8s/examples/hpa-example.yaml infra/k8s/prod/hpa.yaml
   ```

3. **Customize for your service**:
   - Adjust `minReplicas`/`maxReplicas` based on capacity planning
   - Set `averageUtilization` based on load testing
   - Add custom metrics if needed (e.g., HTTP request rate)

4. **Reference in Kustomization**:
   ```yaml
   # infra/k8s/prod/kustomization.yaml
   resources:
     - ../dev
     - hpa.yaml
   ```

5. **Remove `replicas` from Deployment**:
   ```yaml
   # infra/k8s/prod/deployment-patch.yaml
   # Remove or comment out:
   # spec:
   #   replicas: 3
   ```
   HPA manages replicas; Deployment should not have `replicas` set when HPA is active.

6. **Test scaling**:
   ```bash
   # Deploy
   kubectl apply -k infra/k8s/prod

   # Watch HPA
   kubectl get hpa -n my-service-prod -w

   # Generate load (adjust for your service)
   kubectl run load-test --image=busybox --rm -it -- \
     /bin/sh -c "while true; do wget -q -O- http://app-http.my-service-prod.svc.cluster.local/health; done"

   # Observe scaling
   kubectl get pods -n my-service-prod -w
   ```

## Optional Module: NetworkPolicy

### What NetworkPolicy Provides

NetworkPolicies act as **firewall rules for pods**:
- Restrict ingress (incoming traffic) to specific sources
- Restrict egress (outgoing traffic) to specific destinations
- Enforce **zero-trust networking** (deny by default, allow explicitly)

### When to Use NetworkPolicy

Use NetworkPolicy when:
- You have **multi-tenant clusters** (need isolation between teams/services)
- You want **defense in depth** (even if network is compromised)
- You have **compliance requirements** (PCI-DSS, HIPAA, etc.)
- Your cluster has **CNI with NetworkPolicy support** (Calico, Cilium, Weave)

**Don't use NetworkPolicy when**:
- Your CNI **doesn't support it** (default kubenet, Flannel without Calico)
- You're in **local dev** (adds friction, no security benefit)
- You haven't **mapped your service dependencies** yet (you'll break things)

### CNI Support Check

```bash
# Check if your cluster supports NetworkPolicy
kubectl api-resources | grep networkpolicies

# Test with a simple policy
kubectl apply -f infra/k8s/examples/networkpolicy-example.yaml
kubectl get networkpolicies
```

If `kubectl get networkpolicies` shows your policy, your cluster supports it. If pods can still connect despite a deny-all policy, **your CNI doesn't enforce NetworkPolicy** (it's just decorative).

### Example Baseline Policy

See `infra/k8s/examples/networkpolicy-example.yaml` for complete examples.

**Deny-all baseline** (start locked down, then open ports):
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-all
spec:
  podSelector: {}  # Applies to all pods in namespace
  policyTypes:
  - Ingress
  - Egress
```

**Allow ingress from Ingress controller only**:
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: app-http-ingress
spec:
  podSelector:
    matchLabels:
      app: app-http
  policyTypes:
  - Ingress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx
    ports:
    - protocol: TCP
      port: 8080
```

### Cluster Requirements

| CNI | NetworkPolicy Support | Notes |
|-----|----------------------|-------|
| **Calico** | ✅ Full support | Industry standard, recommended |
| **Cilium** | ✅ Full support | eBPF-based, high performance |
| **Weave Net** | ✅ Full support | Simple setup |
| **Canal** | ✅ Full support | Flannel + Calico |
| **Flannel** | ❌ No support | Unless combined with Calico |
| **kubenet** | ❌ No support | Default on some managed K8s |

**Check your cluster's CNI**:
```bash
kubectl get pods -n kube-system | grep -E 'calico|cilium|weave|flannel'
```

### Testing Considerations

1. **Always test in staging first** (NetworkPolicy bugs cause outages)
2. **Use audit mode if available** (Cilium supports logging-only mode)
3. **Monitor DNS failures** (common mistake: blocking DNS to kube-dns)
4. **Allow health checks** (from Kubernetes API server, Prometheus, etc.)

**Common mistake**: Blocking kubelet health probes
```yaml
# Wrong: No ingress rules = health probes fail
ingress: []

# Right: Allow kubelet to reach health endpoints
ingress:
- from:
  - namespaceSelector: {}  # Allow from all namespaces (kubelet runs in kube-system)
  ports:
  - protocol: TCP
    port: 8080  # Your health check port
```

### Policy Extensions

Add policy to enforce NetworkPolicy best practices:

```rego
# In policy/k8s_networkpolicy.rego (create this file)
package main

import future.keywords.in

# Warn if production namespace has no NetworkPolicies
warn[msg] {
    input.kind == "Namespace"
    input.metadata.labels["environment"] == "production"
    not has_networkpolicy_annotation
    msg := sprintf("Production namespace %s should have NetworkPolicies configured",
                   [input.metadata.name])
}

has_networkpolicy_annotation {
    input.metadata.annotations["networkpolicies.configured"] == "true"
}

# Deny overly broad NetworkPolicies in production
deny[msg] {
    input.kind == "NetworkPolicy"
    environment == "production"
    input.spec.podSelector == {}  # Applies to all pods
    count(input.spec.ingress) == 0
    count(input.spec.egress) == 0
    msg := sprintf("NetworkPolicy %s denies all traffic - too broad for production",
                   [input.metadata.name])
}
```

### Why It's Not Default

1. **CNI dependency**: Not all clusters support NetworkPolicy enforcement
2. **Service discovery required**: Need to know all traffic flows first (break-fix cycle)
3. **Operational complexity**: Debugging connection issues is harder
4. **Environment-specific**: Dev/local doesn't need it, prod might

### How to Add to Your Deployment

1. **Verify CNI support** (see "Cluster Requirements" above)

2. **Map your service dependencies**:
   ```
   app-http needs:
   - Ingress: From ingress-nginx namespace (port 8080)
   - Egress: To PostgreSQL (port 5432)
   - Egress: To kube-dns (port 53 UDP)
   - Ingress: From Prometheus (port 8080, path /metrics)
   - Ingress: From Kubernetes API server (health probes, port 8080)
   ```

3. **Start with deny-all baseline**:
   ```bash
   cp infra/k8s/examples/networkpolicy-example.yaml infra/k8s/prod/networkpolicy.yaml
   ```

4. **Add allow rules incrementally**:
   ```yaml
   # Allow ingress from Ingress controller
   # Allow egress to database
   # Allow egress to DNS
   # etc.
   ```

5. **Test in staging first**:
   ```bash
   kubectl apply -k infra/k8s/staging

   # Test connectivity
   kubectl run test-pod --image=curlimages/curl --rm -it -- \
     curl http://app-http.my-service-staging.svc.cluster.local/health

   # Should work if policy is correct, fail if policy is too strict
   ```

6. **Monitor for blocked connections**:
   ```bash
   # Cilium example
   kubectl -n kube-system exec -it cilium-xxxxx -- cilium monitor --type drop

   # Calico example
   kubectl logs -n kube-system -l k8s-app=calico-node | grep -i drop
   ```

## Optional Module: Ingress/Gateway

### Different Approaches

Kubernetes has **two competing standards** for external traffic routing:

| Approach | Maturity | Ecosystem | Recommendation |
|----------|----------|-----------|----------------|
| **Ingress API** | Stable (v1) | Mature, widely adopted | Use for most HTTP/HTTPS services |
| **Gateway API** | Beta (v1beta1) | Growing, more flexible | Use if you need advanced routing or have newer clusters |

**Ingress API** (older, simpler):
- Single resource type (`kind: Ingress`)
- Provider-specific annotations (nginx, Traefik, etc.)
- Limited to HTTP/HTTPS routing
- Well-documented, battle-tested

**Gateway API** (newer, more powerful):
- Multiple resource types (`Gateway`, `HTTPRoute`, `GRPCRoute`, etc.)
- More expressive, less reliance on annotations
- Supports TCP/UDP, not just HTTP
- Better multi-tenancy (separate Gateway admin from app developer)

### Example nginx Ingress

See `infra/k8s/examples/ingress-example.yaml` for complete examples.

**Basic HTTP Ingress**:
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: app-http
  annotations:
    # nginx-specific annotations
    nginx.ingress.kubernetes.io/rewrite-target: /
spec:
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

**HTTPS with TLS**:
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: app-http
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
spec:
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

### TLS Considerations

**Option 1: Bring your own certificate**:
```bash
kubectl create secret tls app-http-tls \
  --cert=path/to/tls.crt \
  --key=path/to/tls.key \
  -n my-service-prod
```

**Option 2: Use cert-manager** (recommended):
```yaml
# Install cert-manager first:
# kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.0/cert-manager.yaml

# Create ClusterIssuer for Let's Encrypt
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: your-email@example.com
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: nginx
```

Then annotate your Ingress with `cert-manager.io/cluster-issuer: "letsencrypt-prod"` and cert-manager will automatically provision and renew certificates.

### Provider-Specific Annotations

Each Ingress controller uses **different annotations**:

**nginx**:
```yaml
annotations:
  nginx.ingress.kubernetes.io/rewrite-target: /
  nginx.ingress.kubernetes.io/ssl-redirect: "true"
  nginx.ingress.kubernetes.io/rate-limit: "100"
  nginx.ingress.kubernetes.io/cors-allow-origin: "*"
```

**Traefik**:
```yaml
annotations:
  traefik.ingress.kubernetes.io/router.entrypoints: websecure
  traefik.ingress.kubernetes.io/router.tls: "true"
  traefik.ingress.kubernetes.io/router.middlewares: default-ratelimit@kubernetescrd
```

**AWS ALB** (via AWS Load Balancer Controller):
```yaml
annotations:
  kubernetes.io/ingress.class: alb
  alb.ingress.kubernetes.io/scheme: internet-facing
  alb.ingress.kubernetes.io/target-type: ip
  alb.ingress.kubernetes.io/certificate-arn: arn:aws:acm:...
```

**GCP GCE**:
```yaml
annotations:
  kubernetes.io/ingress.class: gce
  kubernetes.io/ingress.global-static-ip-name: my-static-ip
  networking.gke.io/managed-certificates: my-cert
```

**Check your Ingress controller**:
```bash
kubectl get ingressclass
# Shows available IngressClass resources and which is default
```

### Why It's Not Default

1. **Provider-specific**: Each cluster has different Ingress controllers (nginx, Traefik, cloud-specific)
2. **External dependency**: Requires DNS configuration, TLS certificates
3. **Not always needed**: Many internal services don't need external access
4. **Configuration variance**: Annotations differ wildly between providers

### How to Add to Your Deployment

1. **Verify Ingress controller is installed**:
   ```bash
   kubectl get ingressclass
   kubectl get pods -n ingress-nginx  # Or your controller's namespace
   ```

2. **Choose your Ingress provider and copy example**:
   ```bash
   cp infra/k8s/examples/ingress-example.yaml infra/k8s/prod/ingress.yaml
   ```

3. **Customize for your domain and provider**:
   ```yaml
   # Update host
   spec:
     rules:
     - host: myapp.example.com  # Your actual domain

   # Update annotations for your provider
   metadata:
     annotations:
       # nginx, Traefik, ALB, etc.
   ```

4. **Configure DNS**:
   ```bash
   # Get Ingress controller's external IP
   kubectl get svc -n ingress-nginx ingress-nginx-controller

   # Create DNS A record:
   # myapp.example.com -> <EXTERNAL-IP>
   ```

5. **Set up TLS** (if using HTTPS):
   ```bash
   # Option A: Manual certificate
   kubectl create secret tls app-http-tls \
     --cert=tls.crt --key=tls.key -n my-service-prod

   # Option B: cert-manager (install first, then annotate Ingress)
   ```

6. **Reference in Kustomization**:
   ```yaml
   # infra/k8s/prod/kustomization.yaml
   resources:
     - ../dev
     - ingress.yaml
   ```

7. **Deploy and test**:
   ```bash
   kubectl apply -k infra/k8s/prod

   # Test (may take a few minutes for DNS/TLS)
   curl https://myapp.example.com/health
   ```

## Optional Module: PodDisruptionBudget

### What PDB Provides

PodDisruptionBudget (PDB) ensures **minimum availability during voluntary disruptions**:
- Node drains (for maintenance)
- Cluster upgrades
- Pod evictions

**Example**: "Keep at least 2 pods running during disruptions"

### When to Use PDB

Use PDB when:
- You're running **production with multiple replicas**
- You have **strict availability requirements** (e.g., 99.9% uptime SLA)
- Your cluster has **frequent node maintenance**

**Don't use PDB when**:
- You have **single replica** (PDB can't help)
- You're in **dev/staging** (no availability requirements)
- You use HPA with `minReplicas: 1` (PDB would block scale-down)

### Example Manifest

```yaml
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: app-http
spec:
  minAvailable: 2  # Or maxUnavailable: 1
  selector:
    matchLabels:
      app: app-http
```

See `infra/k8s/examples/` directory for more examples (not included by default, but can be added).

## Policy Extensions

### How to Add Rego Rules for Optional Modules

When you add optional modules, **extend the policy** to ensure safe configurations.

**Create new policy files** for each optional module:

```
policy/
├── k8s.rego              # Core Deployment/Service policies
├── k8s_hpa.rego          # HPA-specific policies (you create this)
├── k8s_networkpolicy.rego  # NetworkPolicy policies (you create this)
└── k8s_ingress.rego      # Ingress policies (you create this)
```

### Example: HPA Validation

**File**: `policy/k8s_hpa.rego`

```rego
package main

import future.keywords.in

# HPA policies only apply when HPA resources are present

# Require minimum replicas >= 3 in production
deny[msg] {
    input.kind == "HorizontalPodAutoscaler"
    environment == "production"
    min_replicas := to_number(input.spec.minReplicas)
    min_replicas < 3
    msg := sprintf("HPA %s must have minReplicas >= 3 in production for HA (found: %d)",
                   [input.metadata.name, min_replicas])
}

# Prevent overly aggressive CPU targets (causes scaling thrashing)
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

# Warn if maxReplicas is too low for production traffic
warn[msg] {
    input.kind == "HorizontalPodAutoscaler"
    environment == "production"
    max_replicas := to_number(input.spec.maxReplicas)
    max_replicas < 10
    msg := sprintf("HPA %s has maxReplicas: %d - ensure this handles peak production load",
                   [input.metadata.name, max_replicas])
}

# Require scale-down stabilization to prevent flapping
deny[msg] {
    input.kind == "HorizontalPodAutoscaler"
    environment == "production"
    not has_scaledown_stabilization
    msg := sprintf("HPA %s must configure scaleDown.stabilizationWindowSeconds in production",
                   [input.metadata.name])
}

has_scaledown_stabilization {
    input.spec.behavior.scaleDown.stabilizationWindowSeconds
}

# Warn if HPA targets a Deployment that also has replicas set
warn[msg] {
    input.kind == "HorizontalPodAutoscaler"
    target := input.spec.scaleTargetRef
    target.kind == "Deployment"
    deployment := data.deployments[target.name]  # Requires multi-doc validation
    deployment.spec.replicas
    msg := sprintf("HPA %s targets Deployment %s which has replicas set - HPA will override this",
                   [input.metadata.name, target.name])
}
```

### Example: NetworkPolicy Requirements

**File**: `policy/k8s_networkpolicy.rego`

```rego
package main

import future.keywords.in

# Require NetworkPolicies in production namespaces
deny[msg] {
    input.kind == "Namespace"
    input.metadata.labels["environment"] == "production"
    not has_networkpolicy_annotation
    msg := sprintf("Production namespace %s must have NetworkPolicies configured (add annotation networkpolicies.configured=true)",
                   [input.metadata.name])
}

has_networkpolicy_annotation {
    input.metadata.annotations["networkpolicies.configured"] == "true"
}

# Prevent overly broad NetworkPolicies (allow-all is pointless)
deny[msg] {
    input.kind == "NetworkPolicy"
    has_allow_all_ingress
    msg := sprintf("NetworkPolicy %s allows all ingress traffic - use Service instead",
                   [input.metadata.name])
}

has_allow_all_ingress {
    ingress_rule := input.spec.ingress[_]
    not ingress_rule.from  # No from clause = allow from anywhere
}

# Require egress rules to allow DNS (common mistake: blocking DNS)
warn[msg] {
    input.kind == "NetworkPolicy"
    has_egress_policy_type
    not allows_dns_egress
    msg := sprintf("NetworkPolicy %s blocks egress but doesn't allow DNS - pods may fail to resolve names",
                   [input.metadata.name])
}

has_egress_policy_type {
    policy_type := input.spec.policyTypes[_]
    policy_type == "Egress"
}

allows_dns_egress {
    egress_rule := input.spec.egress[_]
    port := egress_rule.ports[_]
    port.port == 53
    port.protocol == "UDP"
}

# Warn if NetworkPolicy has no effect (no pods match selector)
warn[msg] {
    input.kind == "NetworkPolicy"
    input.spec.podSelector == {}
    count(input.spec.ingress) == 0
    count(input.spec.egress) == 0
    msg := sprintf("NetworkPolicy %s has empty podSelector and no rules - has no effect",
                   [input.metadata.name])
}
```

### Testing Policy Extensions

Add test cases in `policy/testdata/`:

```yaml
# policy/testdata/k8s_hpa_invalid.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: test-hpa
  labels:
    environment: production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: test-deployment
  minReplicas: 1  # Should fail: production requires >= 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 30  # Should fail: too aggressive (< 50%)
```

```yaml
# policy/testdata/k8s_hpa_valid.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: test-hpa
  labels:
    environment: production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: test-deployment
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
```

Run policy tests:
```bash
cargo run -p xtask -- policy-test
```

## Integration Patterns

### Pattern 1: Add Gradually (Recommended)

1. Start with core resources (Deployment, Service)
2. Validate with policy: `cargo run -p xtask -- policy-test`
3. Deploy to dev: `kubectl apply -k infra/k8s/dev`
4. Add optional modules **one at a time**:
   - Add HPA, test scaling behavior
   - Add NetworkPolicy, verify connectivity
   - Add Ingress, configure DNS/TLS
5. Test each addition in staging before production

### Pattern 2: Use Feature Flags in Kustomization

```yaml
# infra/k8s/prod/kustomization.yaml
resources:
  - ../dev
  # Uncomment to enable optional modules:
  # - hpa.yaml
  # - networkpolicy.yaml
  # - ingress.yaml
```

This makes it clear what's enabled/disabled per environment.

### Pattern 3: Separate Overlays for Cluster Types

If you deploy to multiple cluster types (EKS, GKE, on-prem):

```
k8s/
├── base/              # Core resources
├── overlays/
│   ├── eks/           # AWS-specific (ALB Ingress, etc.)
│   ├── gke/           # GCP-specific (GCE Ingress, etc.)
│   └── on-prem/       # On-prem (nginx Ingress, etc.)
```

## Summary

| Module | Included? | Add When... | Cluster Requirement |
|--------|-----------|-------------|---------------------|
| **Deployment** | ✅ Core | Always | None |
| **Service** | ✅ Core | Always | None |
| **HPA** | ❌ Optional | Variable load, have metrics | metrics-server |
| **NetworkPolicy** | ❌ Optional | Multi-tenant, compliance | CNI with NetworkPolicy support |
| **Ingress** | ❌ Optional | External traffic routing | Ingress controller |
| **PDB** | ❌ Optional | High availability, multi-replica | None |
| **ServiceMonitor** | ❌ Optional | Prometheus metrics | Prometheus Operator |

**Key principle**: The template provides a **portable foundation**. You add cluster-specific enhancements as needed.

## Next Steps

1. Review examples in `infra/k8s/examples/`
2. Copy relevant examples to your environment overlays (`staging/`, `prod/`)
3. Extend policy in `policy/` to validate your additions
4. Test in staging before production
5. Document your cluster's capabilities in your fork's README

## References

- Core infrastructure: `infra/k8s/README.md`
- Policy validation: `policy/k8s.rego`
- Kustomize docs: https://kustomize.io/
- Kubernetes API reference: https://kubernetes.io/docs/reference/kubernetes-api/
