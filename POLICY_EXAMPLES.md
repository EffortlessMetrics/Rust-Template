# K8s Secrets Policy - Examples

## How the Policy Catches Violations

### Example 1: Literal DATABASE_URL (DENIED)

**Manifest:**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app
spec:
  template:
    spec:
      containers:
      - name: app
        env:
        - name: DATABASE_URL
          value: "postgresql://user:password@db.example.com/dbname"
```

**Policy Result:**
```
FAIL - SECURITY: Deployment my-app container app has literal value for sensitive env var 'DATABASE_URL'. Must use secretRef or secretKeyRef from a Secret resource.
```

**Why it fails:** The policy detects that `DATABASE_URL` is a sensitive pattern and has a literal `value` instead of using a Secret reference.

---

### Example 2: ConfigMap for Password (DENIED)

**Manifest:**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app
spec:
  template:
    spec:
      containers:
      - name: app
        env:
        - name: REDIS_PASSWORD
          valueFrom:
            configMapKeyRef:
              name: app-config
              key: REDIS_PASSWORD
```

**Policy Result:**
```
FAIL - SECURITY: Deployment my-app container app uses configMapKeyRef for sensitive env var 'REDIS_PASSWORD'. Must use secretKeyRef from a Secret resource instead.
```

**Why it fails:** ConfigMaps are for non-sensitive data. Passwords must use Secrets.

---

### Example 3: Literal API Keys (DENIED)

**Manifest:**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app
spec:
  template:
    spec:
      containers:
      - name: app
        env:
        - name: STRIPE_API_KEY
          value: "EXAMPLE_KEY_DO_NOT_USE_12345678"
        - name: JWT_SECRET
          value: "super-secret-jwt-key-12345"
```

**Policy Result:**
```
FAIL - SECURITY: Deployment my-app container app has literal value for sensitive env var 'STRIPE_API_KEY'. Must use secretRef or secretKeyRef from a Secret resource.
FAIL - SECURITY: Deployment my-app container app has literal value for sensitive env var 'JWT_SECRET'. Must use secretRef or secretKeyRef from a Secret resource.
```

**Why it fails:** Both `STRIPE_API_KEY` and `JWT_SECRET` match sensitive patterns and have literal values.

---

### Example 4: Correct Pattern (PASSES)

**Manifest:**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app
  labels:
    app: my-app
    version: v1
    component: api
spec:
  template:
    spec:
      securityContext:
        runAsNonRoot: true
      containers:
      - name: app
        # Load non-sensitive config from ConfigMap
        envFrom:
        - configMapRef:
            name: app-config
        # Load sensitive config from Secret
        - secretRef:
            name: app-secrets
        resources:
          limits:
            cpu: "500m"
            memory: "256Mi"
          requests:
            cpu: "100m"
            memory: "128Mi"
```

**ConfigMap (app-config):**
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config
data:
  ENVIRONMENT: "production"
  LOG_LEVEL: "info"
  PORT: "8080"
```

**Secret (app-secrets):**
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: app-secrets
type: Opaque
stringData:
  DATABASE_URL: "postgresql://user:password@db/dbname"
  API_KEY: "EXAMPLE_KEY_DO_NOT_USE_prod"
  JWT_SECRET: "production-jwt-secret"
```

**Policy Result:**
```
PASS - All checks passed
```

**Why it passes:**
- Uses `envFrom` pattern for clean config management
- Non-sensitive values in ConfigMap
- Sensitive values in Secret
- No literal credentials in the Deployment manifest

---

## Sensitive Pattern Detection

The policy recognizes these patterns as sensitive:

| Pattern | Example Matches | Exception Examples |
|---------|----------------|-------------------|
| `PASSWORD` | `DB_PASSWORD`, `REDIS_PASSWORD`, `PASSWORD_HASH` | - |
| `TOKEN` | `API_TOKEN`, `REFRESH_TOKEN`, `TOKEN_SECRET` | - |
| `SECRET` | `JWT_SECRET`, `CLIENT_SECRET`, `SECRET_KEY` | - |
| `KEY` | `API_KEY`, `PRIVATE_KEY`, `ENCRYPTION_KEY` | `SORT_KEY`, `PARTITION_KEY` |
| `DATABASE_URL` | `DATABASE_URL`, `DB_URL`, `POSTGRES_DATABASE_URL` | - |
| `API_KEY` | `STRIPE_API_KEY`, `API_KEY`, `SENDGRID_API_KEY` | - |
| `PRIVATE_KEY` | `RSA_PRIVATE_KEY`, `SSH_PRIVATE_KEY` | - |
| `CREDENTIALS` | `AWS_CREDENTIALS`, `GCP_CREDENTIALS` | - |
| `AUTH` | `AUTH_TOKEN`, `OAUTH_SECRET`, `BASIC_AUTH` | - |

---

## Best Practices

### ✅ DO

1. **Use envFrom for all configuration:**
   ```yaml
   envFrom:
   - configMapRef:
       name: app-config
   - secretRef:
       name: app-secrets
   ```

2. **Separate sensitive from non-sensitive:**
   - ConfigMap: `LOG_LEVEL`, `PORT`, `ENVIRONMENT`, feature flags
   - Secret: `DATABASE_URL`, `API_KEY`, `PASSWORD`, tokens

3. **Document secret injection:**
   ```yaml
   # secret.yaml
   stringData:
     # PRODUCTION: Inject via CI/CD - never commit real values
     DATABASE_URL: "placeholder"
   ```

4. **Use secret management tools:**
   - Vault
   - Sealed Secrets
   - External Secrets Operator
   - AWS Secrets Manager / GCP Secret Manager

### ❌ DON'T

1. **Don't use literal values for credentials:**
   ```yaml
   # WRONG
   env:
   - name: DATABASE_URL
     value: "postgresql://..."
   ```

2. **Don't store secrets in ConfigMaps:**
   ```yaml
   # WRONG
   env:
   - name: API_KEY
     valueFrom:
       configMapKeyRef:
         name: config
         key: API_KEY
   ```

3. **Don't commit real secrets to git:**
   ```yaml
   # WRONG - in git
   stringData:
     DATABASE_URL: "postgresql://realuser:realpass@prod-db/dbname"
   ```

---

## Testing Your Manifests

### Run Policy Tests
```bash
# Test all policies
cargo run -p xtask -- policy-test

# Or with conftest directly
conftest test -p policy/k8s.rego infra/k8s/dev/deployment.yaml
```

### Expected Output
```
Kubernetes Policy:
  ✓ k8s_secrets_valid.json (correctly passed)
  ✓ k8s_secrets_literal_database_url.json (correctly failed)
  ✓ k8s_secrets_literal_api_key.json (correctly failed)
  ✓ k8s_secrets_configmap_for_secret.json (correctly failed)

✓ All 4 policy tests passed!
```

---

## Integration with LLM Code Generation

When an LLM generates or modifies Kubernetes manifests, this policy ensures safety:

### Scenario 1: LLM adds database configuration
**LLM attempt:**
```yaml
env:
- name: DATABASE_URL
  value: "postgresql://user:password@localhost/db"
```

**Policy response:**
```
DENIED: has literal value for sensitive env var 'DATABASE_URL'
```

**LLM learns:** Use `envFrom.secretRef` instead

### Scenario 2: LLM migrates to K8s
**LLM sees:** Environment variable `STRIPE_API_KEY=EXAMPLE_KEY_...`

**LLM might try:**
```yaml
env:
- name: STRIPE_API_KEY
  value: "EXAMPLE_KEY_..."
```

**Policy response:**
```
DENIED: has literal value for sensitive env var 'STRIPE_API_KEY'
```

**LLM corrects to:**
```yaml
envFrom:
- secretRef:
    name: payment-secrets
```

**Result:** Credentials never enter version control

---

## Summary

The K8s secrets policy:
- ✅ Prevents credential leakage in git
- ✅ Enforces separation of sensitive/non-sensitive config
- ✅ Provides clear error messages
- ✅ Enables safe LLM-assisted development
- ✅ Follows K8s security best practices
