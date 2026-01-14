# How to Deploy to Development Environment

This guide walks through deploying the app-http service to a local or development Kubernetes cluster.

## Prerequisites

Before deploying, ensure you have the following tools installed:

- **Docker** - For building container images
  - Install: [https://docs.docker.com/get-docker/](https://docs.docker.com/get-docker/)
  - Verify: `docker --version`

- **kubectl** - Kubernetes command-line tool
  - Install: [https://kubernetes.io/docs/tasks/tools/](https://kubernetes.io/docs/tasks/tools/)
  - Verify: `kubectl version --client`

- **Kubernetes cluster** - One of:
  - [minikube](https://minikube.sigs.k8s.io/docs/start/) (local)
  - [kind](https://kind.sigs.k8s.io/) (Kubernetes in Docker)
  - [Docker Desktop](https://www.docker.com/products/docker-desktop/) (with K8s enabled)
  - Remote development cluster

## Quick Start

```bash
# 1. Build the Docker image
docker build -t app-http:latest -f crates/app-http/Dockerfile .

# 2. Load image into your cluster (if using minikube/kind)
# For minikube:
minikube image load app-http:latest

# For kind:
kind load docker-image app-http:latest

# For Docker Desktop: Skip this step (uses local Docker)

# 3. Apply Kubernetes manifests
kubectl apply -f infra/k8s/dev/

# 4. Verify deployment
kubectl get pods -l app=app-http
kubectl get service app-http
```

## Step-by-Step Guide

### 1. Build the Docker Image

Build the container image from the workspace root:

```bash
docker build -t app-http:latest -f crates/app-http/Dockerfile .
```

**Note:** If you don't have a Dockerfile yet, create one at `crates/app-http/Dockerfile`:

```dockerfile
# Multi-stage build for Rust application
FROM rust:1.89-slim as builder

WORKDIR /build
COPY . .

# Build release binary
RUN cargo build --release --bin app-http

# Runtime stage
FROM debian:bookworm-slim

# Create non-root user
RUN useradd -u 1000 -m appuser

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /build/target/release/app-http /usr/local/bin/app-http

# Set up runtime environment
USER appuser
WORKDIR /home/appuser

EXPOSE 8080

CMD ["app-http"]
```

### 2. Load Image into Your Cluster

Depending on your Kubernetes setup, you may need to load the image:

#### For minikube

```bash
minikube image load app-http:latest
```

#### For kind

```bash
kind load docker-image app-http:latest
```

#### For Docker Desktop

No action needed - Docker Desktop uses your local Docker daemon.

#### For remote clusters

Push to a container registry:

```bash
# Tag for your registry
docker tag app-http:latest <registry>/app-http:latest

# Push to registry
docker push <registry>/app-http:latest

# Update infra/k8s/dev/deployment.yaml to use registry image
```

### 3. Deploy to Kubernetes

Apply all manifests in the dev directory:

```bash
kubectl apply -f infra/k8s/dev/
```

This will create:
- A Deployment with 1 replica
- A ClusterIP Service exposing port 80

### 4. Verify Deployment

Check that the pod is running:

```bash
kubectl get pods -l app=app-http
```

Expected output:

```
NAME                        READY   STATUS    RESTARTS   AGE
app-http-xxxxxxxxxx-xxxxx   1/1     Running   0          30s
```

View logs:

```bash
kubectl logs -l app=app-http -f
```

Check service:

```bash
kubectl get service app-http
```

### 5. Access the Application

#### Option A: Port Forward (Quick Test)

```bash
kubectl port-forward service/app-http 8080:80
```

Then access at: `http://localhost:8080/health`

#### Option B: Ingress (Cluster Access)

For cluster-wide access, you'll need an Ingress controller and Ingress resource (not included in dev setup).

## Validation

Verify the deployment meets policy requirements:

```bash
# Run policy tests
cargo xtask policy-test

# Check specific K8s policies
conftest test -p policy/k8s.rego infra/k8s/dev/
```

## Updating the Deployment

After making code changes:

```bash
# Rebuild image
docker build -t app-http:latest -f crates/app-http/Dockerfile .

# Reload into cluster (if needed)
minikube image load app-http:latest  # or kind load

# Restart deployment to pick up new image
kubectl rollout restart deployment/app-http

# Watch rollout status
kubectl rollout status deployment/app-http
```

## Cleanup

Remove all resources:

```bash
kubectl delete -f infra/k8s/dev/
```

## Troubleshooting

### Pod not starting

```bash
# Check pod events
kubectl describe pod -l app=app-http

# Check logs
kubectl logs -l app=app-http
```

### Image pull errors

- Verify image exists: `docker images | grep app-http`
- For minikube/kind: Ensure image was loaded
- Check `imagePullPolicy` in deployment.yaml

### Health check failures

- Ensure your app has a `/health` endpoint
- Check liveness/readiness probe settings
- Verify port 8080 is correct for your app

### Permission errors

- The deployment runs as user 1000 (non-root)
- Ensure your app doesn't require root privileges
- Check file permissions in the container

## Next Steps

- Set up monitoring with Prometheus/Grafana
- Configure Ingress for external access
- Add ConfigMaps for environment-specific config
- Set up CI/CD pipeline for automated deployments
- Create staging and production manifests

## Related Documentation

- [Kubernetes Policies](../../policy/k8s.rego) - OPA policies enforced on deployments
- [Template Overview](../../TEMPLATE_OVERVIEW.md) - Overall project structure
- [Implementation Plan](../../IMPLEMENTATION_PLAN.md) - Roadmap and features
