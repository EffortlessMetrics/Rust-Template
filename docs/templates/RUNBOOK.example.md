# Runbook: [Service Name]

**Version**: [Service version, e.g., 0.1.0]
**Last Updated**: [YYYY-MM-DD]
**Owner**: [Team or individual, e.g., team-platform]

This runbook provides operational guidance for running, monitoring, and troubleshooting this service in production.

---

## Service Overview

### Purpose

Brief description of what this service does and why it exists.

Example:
> This service manages user accounts, authentication, and authorization. It provides a REST API for user CRUD operations and handles session management for the platform.

### Architecture

- **Pattern**: Hexagonal (ports-and-adapters)
- **Language**: Rust
- **Framework**: Axum (HTTP), Tower (middleware)
- **Deployment**: Kubernetes
- **Dependencies**: PostgreSQL, Redis (optional), Auth0 (optional)

### Key Endpoints

| Endpoint | Purpose | Expected Response Time |
|----------|---------|------------------------|
| `GET /health` | Liveness check | <10ms |
| `GET /ready` | Readiness check (checks DB, deps) | <50ms |
| `GET /version` | Build info (version, git SHA) | <10ms |
| `GET /metrics` | Prometheus metrics | <100ms |
| `POST /users` | Create user account | <200ms |

---

## Health and Readiness

### Health Check (`/health`)

**What it validates:**

- Service process is alive
- No panic/crash loop

**Expected response:**

```json
{
  "status": "ok"
}
```

**If unhealthy:**

- Check pod logs for panics or OOM
- Check if container is restarting frequently
- Verify CPU/memory limits are not exceeded

### Readiness Check (`/ready`)

**What it validates:**

- Database connection is healthy
- External dependencies are reachable
- Service is ready to handle traffic

**Expected response:**

```json
{
  "status": "ready",
  "checks": {
    "database": "ok",
    "cache": "ok"
  }
}
```

**If not ready:**

1. Check database connectivity: `kubectl exec -it <pod> -- psql -h $DB_HOST -U $DB_USER -c "SELECT 1;"`
2. Check network policies: ensure service can reach DB/Redis
3. Check secrets: verify DB credentials are correct

---

## Metrics and Monitoring

### Key Metrics

The service exposes Prometheus metrics at `GET /metrics`. Key metrics to monitor:

#### Request Metrics

| Metric | Type | Description | Alert Threshold |
|--------|------|-------------|-----------------|
| `http_requests_total` | Counter | Total HTTP requests by method, path, status | N/A |
| `http_request_duration_seconds` | Histogram | Request latency distribution | p99 > 500ms |
| `http_requests_in_flight` | Gauge | Current concurrent requests | > 100 |

#### Application Metrics

| Metric | Type | Description | Alert Threshold |
|--------|------|-------------|-----------------|
| `users_created_total` | Counter | Total user accounts created | N/A |
| `auth_failures_total` | Counter | Failed authentication attempts | Rate > 10/min |
| `db_connection_pool_size` | Gauge | Current DB pool size | > 90% of max |

#### System Metrics

| Metric | Type | Description | Alert Threshold |
|--------|------|-------------|-----------------|
| `process_cpu_seconds_total` | Counter | CPU time consumed | N/A |
| `process_resident_memory_bytes` | Gauge | Memory usage | > 450MB (if limit is 512MB) |

### Recommended Alerts

**Critical:**

- `rate(http_requests_total{status=~"5.."}[5m]) > 0.05` → 5xx error rate > 5%
- `up{job="example-api"} == 0` → Service is down
- `rate(db_connection_failures_total[1m]) > 0` → DB connection failures

**Warning:**

- `histogram_quantile(0.99, http_request_duration_seconds_bucket) > 0.5` → p99 latency > 500ms
- `http_requests_in_flight > 100` → High concurrency, may need scaling
- `rate(auth_failures_total[5m]) > 10` → Spike in auth failures (possible attack)

---

## Logs

### Log Format

- **Format**: JSON (structured logging)
- **Level**: `info` (configurable via `RUST_LOG` env var)
- **Output**: stdout (captured by Kubernetes)

### Log Fields

Each log entry includes:

```json
{
  "timestamp": "2025-01-18T12:34:56.789Z",
  "level": "INFO",
  "target": "app_http::routes::users",
  "message": "User created",
  "request_id": "req-abc123",
  "user_id": "usr-xyz789",
  "span": {
    "name": "POST /users"
  }
}
```

### Useful Log Queries

**Find all errors in the last hour:**

```bash
kubectl logs -l app=example-api --since=1h | jq 'select(.level == "ERROR")'
```

**Trace a specific request:**

```bash
kubectl logs -l app=example-api | jq 'select(.request_id == "req-abc123")'
```

**Find slow requests (>1s):**

```bash
kubectl logs -l app=example-api | jq 'select(.duration_ms > 1000)'
```

### Privacy Considerations

**What we DON'T log** (per `privacy_compliance.rego`):

- User emails (only user IDs)
- Passwords or tokens
- Credit card numbers or SSNs
- Any PII (Personally Identifiable Information)

If you see PII in logs, file a security incident immediately.

---

## Deployment

### Kubernetes

**Namespace**: `default` (or `prod`, `staging`)
**Deployment**: `example-api`
**Service**: `example-api-service`

**Scale up/down:**

```bash
kubectl scale deployment example-api --replicas=4
```

**View pods:**

```bash
kubectl get pods -l app=example-api
```

**Check resource usage:**

```bash
kubectl top pods -l app=example-api
```

### Configuration

**Environment Variables:**

| Variable | Required | Default | Purpose |
|----------|----------|---------|---------|
| `PORT` | No | `8080` | HTTP listen port |
| `DATABASE_URL` | Yes | N/A | PostgreSQL connection string |
| `RUST_LOG` | No | `info` | Log level |
| `OTLP_ENDPOINT` | No | N/A | OpenTelemetry collector endpoint |

**Secrets:**

- `db-credentials`: Database username/password
- `api-keys`: External service API keys (if applicable)

**Verify configuration:**

```bash
kubectl describe deployment example-api
kubectl get configmap example-api-config -o yaml
kubectl get secret db-credentials -o yaml
```

---

## Troubleshooting

### Service is down / pods crashing

1. **Check pod status:**

   ```bash
   kubectl get pods -l app=example-api
   ```

   Look for `CrashLoopBackOff`, `Error`, `OOMKilled`.

2. **Check logs:**

   ```bash
   kubectl logs -l app=example-api --tail=100
   ```

   Look for panics, errors on startup.

3. **Check events:**

   ```bash
   kubectl get events --sort-by='.lastTimestamp' | grep example-api
   ```

4. **Common causes:**

   - **OOM**: Memory limit too low → increase `resources.limits.memory`
   - **Panic on startup**: Missing config/secrets → check env vars
   - **DB connection failure**: Wrong credentials or network policy → verify `DATABASE_URL`

### High latency (p99 > 500ms)

1. **Check metrics:**

   ```promql
   histogram_quantile(0.99, http_request_duration_seconds_bucket{job="example-api"})
   ```

2. **Check DB query performance:**

   - Review slow query logs in PostgreSQL
   - Check DB connection pool size (may be saturated)

3. **Check CPU/memory:**

   ```bash
   kubectl top pods -l app=example-api
   ```

   If CPU is maxed out, consider scaling horizontally.

4. **Check downstream dependencies:**

   - Slow external API calls
   - Network latency to DB or cache

### 5xx Errors

1. **Check error logs:**

   ```bash
   kubectl logs -l app=example-api | jq 'select(.level == "ERROR")'
   ```

2. **Common causes:**

   - **DB connection lost**: Check `ready` endpoint, verify DB is up
   - **Panic in handler**: Check for stack traces in logs
   - **Resource exhaustion**: OOM, file descriptor limit

3. **Mitigation:**

   - Restart pods: `kubectl rollout restart deployment example-api`
   - Scale up if load-related: `kubectl scale deployment example-api --replicas=6`

### Policy Test Failures in CI

If CI fails with policy violations:

1. **Check which policy failed:**

   ```
   FAIL - policy/k8s_standards.rego - container must run as non-root
   ```

2. **Fix the violation:**

   - Update `infra/k8s/deployment.yaml` to comply
   - Example: add `securityContext.runAsNonRoot: true`

3. **Test locally:**

   ```bash
   nix develop -c conftest test -p policy/ infra/k8s/deployment.yaml
   ```

4. **Re-run selftest:**

   ```bash
   cargo run -p xtask -- selftest
   ```

---

## Rollback

If a deployment introduces a regression:

```bash
# View deployment history
kubectl rollout history deployment example-api

# Rollback to previous version
kubectl rollout undo deployment example-api

# Rollback to specific revision
kubectl rollout undo deployment example-api --to-revision=3
```

**Verify rollback:**

```bash
kubectl rollout status deployment example-api
kubectl logs -l app=example-api --tail=50
```

---

## Runbook Maintenance

**This runbook should be updated when:**

- New endpoints are added
- Metrics or alerts change
- Deployment configuration changes
- New failure modes are discovered

**Last reviewed**: [YYYY-MM-DD]
**Next review**: [YYYY-MM-DD + 6 months]

**Owner**: [Team or individual responsible for keeping this current]

---

## References

- [Service README](../README.md)
- [OpenAPI Spec](../specs/openapi/openapi.yaml)
- [ADRs](../docs/adr/)
- [Spec Ledger](../specs/spec_ledger.yaml)
- [Monitoring Dashboard](https://grafana.example.com/d/example-api) (if applicable)
