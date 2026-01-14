---
id: HOWTO-SECURITY-CONFIG-001
title: Security Configuration
doc_type: how_to
status: published
audience: developers, operators, platform-engineers
tags: [security, auth, cors, jwt, headers, configuration]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-PLATFORM-APIS]
acs: []
adrs: []
last_updated: 2025-12-26
---

# Security Configuration

> **One-page guide:** Configure authentication, CORS, and security headers for your platform cell.

---

## Authentication Modes

The platform supports three authentication modes for write operations on `/platform/*` endpoints.

| Mode | Env Var | Required Credentials | Behavior |
|------|---------|---------------------|----------|
| `open` / `none` | `PLATFORM_AUTH_MODE=open` (or `none`) | None | All requests allowed (default) |
| `basic` | `PLATFORM_AUTH_MODE=basic` | `PLATFORM_AUTH_TOKEN` | Static token validation |
| `jwt` | `PLATFORM_AUTH_MODE=jwt` | `PLATFORM_JWT_SECRET` | JWT signature + claims validation |

### Auth Environment Variables

```bash
# Auth mode (required for protected endpoints)
export PLATFORM_AUTH_MODE=jwt     # Options: open, none, basic, jwt

# For basic mode
export PLATFORM_AUTH_TOKEN=your-secure-token-here

# For JWT mode
export PLATFORM_JWT_SECRET=your-jwt-signing-secret

# Generate secure values:
openssl rand -base64 32   # For JWT secret
openssl rand -base64 64   # For basic token
```

### Fail-Closed Behavior

**Invalid auth mode = startup panic.** The platform refuses to start if `PLATFORM_AUTH_MODE` is set to an unrecognized value. This prevents silent fallback to open mode.

```text
FATAL: Invalid platform auth configuration: Invalid auth mode 'typo'.
Valid options: basic, jwt, none, open
```

**Missing credentials = writes rejected.** If auth is enabled but credentials are missing:
- Startup succeeds with a warning log
- All write requests return `401 Unauthorized`
- Read requests (GET/HEAD/OPTIONS) are always allowed

### JWT Claims

JWT tokens must include:

```json
{
  "sub": "user-or-service-id",
  "iss": "your-issuer-name",
  "iat": 1735200000,
  "exp": 1735203600
}
```

Validation includes:
- 60-second clock skew leeway
- Required: `exp`, `sub`, `iss`, `iat`
- `iat` must not be >5 minutes in the future

---

## CORS Configuration

CORS is **enabled by default** with localhost origins for development.

### CORS Environment Variables

```bash
# Enable/disable
export CORS_ENABLED=true

# Allowed origins (comma-separated)
export CORS_ALLOWED_ORIGINS="https://app.example.com,https://admin.example.com"

# Allowed methods
export CORS_ALLOWED_METHODS="GET,POST,PUT,DELETE,OPTIONS"

# Allowed headers
export CORS_ALLOWED_HEADERS="authorization,content-type,x-request-id,x-platform-token"

# Credentials (cookies, auth headers)
export CORS_ALLOW_CREDENTIALS=false

# Preflight cache (seconds)
export CORS_MAX_AGE=86400
```

### Defaults (Development)

```yaml
allowed_origins: ["http://localhost:3000", "http://localhost:8080"]
allowed_methods: ["GET", "POST", "PUT", "DELETE", "OPTIONS", "PATCH"]
allowed_headers: ["authorization", "content-type", "x-request-id", "x-platform-token", "accept", "origin"]
allow_credentials: false
max_age: 86400
```

### Production Override

```bash
# Restrict to specific domains
export CORS_ALLOWED_ORIGINS="https://app.example.com"
export CORS_ALLOW_CREDENTIALS=false
```

### Wildcard Patterns

Supports subdomain wildcards:

```bash
export CORS_ALLOWED_ORIGINS="https://*.example.com"
# Matches: https://app.example.com, https://api.example.com
```

---

## Security Headers

Security headers are **enabled by default** with production-safe values.

### Key Headers

| Header | Default | Purpose |
|--------|---------|---------|
| `X-Frame-Options` | `DENY` | Prevent clickjacking |
| `X-Content-Type-Options` | `nosniff` | Prevent MIME sniffing |
| `Content-Security-Policy` | Strict | XSS protection |
| `Strict-Transport-Security` | 1 year | HTTPS enforcement |
| `Referrer-Policy` | `strict-origin-when-cross-origin` | Control referrer leakage |

### Headers Environment Variables

```bash
# Enable/disable all headers
export SECURITY_HEADERS_ENABLED=true

# Override specific headers
export CSP_HEADER="default-src 'self'; script-src 'self'"
export X_FRAME_OPTIONS="SAMEORIGIN"   # Use SAMEORIGIN to allow same-origin iframes
export STRICT_TRANSPORT_SECURITY="max-age=31536000; includeSubDomains"
```

### Development vs Production

| Setting | Development (`ENV=development`) | Production |
|---------|--------------------------------|------------|
| CSP | Allows `'unsafe-inline'`, `'unsafe-eval'`, localhost | Strict, no unsafe directives |
| HSTS | Disabled | Enabled (1 year) |

### Known Gotchas

1. **UI Embedding:** Default `X-Frame-Options: DENY` blocks iframes. Use `SAMEORIGIN` or remove if embedding is needed.

2. **CSP + CDN:** If using external CDNs, add them to CSP:

   ```bash
   export CSP_HEADER="default-src 'self'; script-src 'self' https://cdn.example.com"
   ```

3. **HSTS + HTTP:** HSTS only works over HTTPS. Disabled automatically in development.

4. **CORS + Credentials:** If `CORS_ALLOW_CREDENTIALS=true`, you cannot use wildcard origins (`*`).

---

## Verify in Staging

Run these commands to validate your security configuration:

### 1. Check Auth Mode

```bash
# Should return governance status with auth_mode
curl -s http://localhost:8080/platform/status | jq '.config.auth // .auth // empty'
# Expected: { "mode": "jwt", "token_present": true }
```

### 2. Test Write Protection

```bash
# Without auth (should fail with 401 if auth enabled)
curl -X POST http://localhost:8080/platform/tasks/TASK-001/status \
  -H "Content-Type: application/json" \
  -d '{"status": "InProgress"}'

# With auth (should succeed)
curl -X POST http://localhost:8080/platform/tasks/TASK-001/status \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -d '{"status": "InProgress"}'
```

### 3. Check Security Headers

```bash
curl -I http://localhost:8080/health 2>/dev/null | grep -E "^(X-Frame|X-Content|Content-Security|Strict-Transport)"
# Expected:
# X-Frame-Options: DENY
# X-Content-Type-Options: nosniff
# Content-Security-Policy: default-src 'self'...
# Strict-Transport-Security: max-age=31536000...
```

### 4. Test CORS Preflight

```bash
curl -X OPTIONS http://localhost:8080/api/echo \
  -H "Origin: https://app.example.com" \
  -H "Access-Control-Request-Method: POST" \
  -H "Access-Control-Request-Headers: authorization,content-type" \
  -I 2>/dev/null | grep -E "^Access-Control"
# Expected (if origin allowed):
# Access-Control-Allow-Origin: https://app.example.com
# Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS, PATCH
```

### 5. Verify Fail-Closed

```bash
# This should cause startup failure (not silent fallback)
PLATFORM_AUTH_MODE=invalid cargo run -p app-http
# Expected: panic with "Invalid auth mode 'invalid'"
```

---

## Quick Reference

### Minimal Production Config

```bash
export PLATFORM_AUTH_MODE=jwt
export PLATFORM_JWT_SECRET=$(openssl rand -base64 32)
export CORS_ALLOWED_ORIGINS="https://app.example.com"
export ENV=production
```

### Minimal Development Config

```bash
export PLATFORM_AUTH_MODE=open
export CORS_ENABLED=true
export SECURITY_HEADERS_ENABLED=true
export ENV=development
```

---

## See Also

- [SECURITY.md](../SECURITY.md) — Comprehensive security reference
- [TROUBLESHOOTING.md](../TROUBLESHOOTING.md) — Common issues and fixes
- [platform_api_contract.md](../reference/platform_api_contract.md) — API contract details
