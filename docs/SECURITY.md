# Security Configuration Guide

This document provides comprehensive guidance on configuring security features in the Rust template to protect against common web vulnerabilities.

## Overview

The Rust template includes several built-in security features:

- **CORS (Cross-Origin Resource Sharing)** protection
- **Security Headers** (CSP, XSS protection, clickjacking prevention)
- **JWT Authentication** with proper validation and leeway
- **Secret Management** with environment variable support

## CORS Configuration

### Default Configuration

By default, CORS is enabled with secure defaults:

```yaml
cors.enabled: true
cors.allowed_origins:
  - "http://localhost:3000"
  - "http://localhost:8080"
cors.allowed_methods:
  - "GET"
  - "POST"
  - "PUT"
  - "DELETE"
  - "OPTIONS"
  - "PATCH"
cors.allowed_headers:
  - "authorization"
  - "content-type"
  - "x-request-id"
  - "x-platform-token"
  - "accept"
  - "origin"
cors.allow_credentials: false
cors.max_age: 86400
```

### Environment Variables

You can configure CORS using environment variables:

```bash
# Enable/disable CORS
export CORS_ENABLED=true

# Allowed origins (comma-separated)
export CORS_ALLOWED_ORIGINS="https://example.com,https://api.example.com"

# Allowed methods (comma-separated)
export CORS_ALLOWED_METHODS="GET,POST,PUT,DELETE,OPTIONS"

# Allowed headers (comma-separated)
export CORS_ALLOWED_HEADERS="authorization,content-type,x-request-id"

# Allow credentials
export CORS_ALLOW_CREDENTIALS=false

# Max age for preflight requests (seconds)
export CORS_MAX_AGE=86400
```

### Production Recommendations

For production environments:

1. **Restrict origins to specific domains:**

   ```yaml
   cors.allowed_origins:
     - "https://yourdomain.com"
     - "https://app.yourdomain.com"
   ```

2. **Disable credentials unless absolutely needed:**

   ```yaml
   cors.allow_credentials: false
   ```

3. **Use specific methods and headers:**

   ```yaml
   cors.allowed_methods: ["GET", "POST", "PUT", "DELETE"]
   cors.allowed_headers: ["authorization", "content-type"]
   ```

## Security Headers

### Default Configuration

The template includes comprehensive security headers by default:

```yaml
security_headers.enabled: true
security_headers.content_security_policy: "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self'; frame-ancestors 'none';"
security_headers.x_frame_options: "DENY"
security_headers.x_content_type_options: "nosniff"
security_headers.x_xss_protection: "1; mode=block"
security_headers.referrer_policy: "strict-origin-when-cross-origin"
security_headers.permissions_policy: "geolocation=(), microphone=(), camera=(), payment=(), usb=(), magnetometer=(), gyroscope=(), accelerometer=(), autoplay=(), encrypted-media=(), fullscreen=(), picture-in-picture=()"
security_headers.cross_origin_embedder_policy: "require-corp"
security_headers.cross_origin_opener_policy: "same-origin"
security_headers.cross_origin_resource_policy: "same-origin"
```

### Environment Variables

Configure security headers via environment variables:

```bash
# Enable/disable security headers
export SECURITY_HEADERS_ENABLED=true

# Custom CSP policy
export CSP_HEADER="default-src 'self'; script-src 'self' https://cdn.example.com"

# Custom frame options
export X_FRAME_OPTIONS="SAMEORIGIN"

# Custom content type options
export X_CONTENT_TYPE_OPTIONS="nosniff"

# Custom XSS protection
export X_XSS_PROTECTION="1; mode=block"

# Custom referrer policy
export REFERRER_POLICY="strict-origin-when-cross-origin"

# Custom permissions policy
export PERMISSIONS_POLICY="geolocation=(), microphone=(), camera=()"

# Cross-origin policies
export CROSS_ORIGIN_EMBEDDER_POLICY="require-corp"
export CROSS_ORIGIN_OPENER_POLICY="same-origin"
export CROSS_ORIGIN_RESOURCE_POLICY="same-origin"
```

### Content Security Policy (CSP)

The CSP is automatically adjusted based on environment:

- **Development**: More permissive with `unsafe-inline` and `unsafe-eval` for local development
- **Production**: Stricter CSP without unsafe directives

#### Custom CSP Examples

**Strict CSP for APIs:**

```yaml
security_headers.content_security_policy: "default-src 'none'; script-src 'self'; connect-src 'self'"
```

**CSP for Web Applications:**

```yaml
security_headers.content_security_policy: "default-src 'self'; script-src 'self' 'unsafe-inline' https://cdn.trusted.com; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; img-src 'self' data: https:; font-src 'self' https://fonts.gstatic.com"
```

### HSTS Configuration

HTTP Strict Transport Security (HSTS) is automatically configured:

- **Development**: Disabled (doesn't work with HTTP)
- **Production**: Enabled with 1-year max age, include subdomains, and preload

```bash
# Custom HSTS configuration
export STRICT_TRANSPORT_SECURITY="max-age=31536000; includeSubDomains; preload"
```

## JWT Authentication

### Configuration

JWT authentication supports both basic tokens and JWTs:

```yaml
platform.auth_mode: "jwt"  # or "basic", "open"
secrets:
  platform.auth_token: "your-strong-token"
  platform.jwt_secret: "your-jwt-secret-key"
```

### Environment Variables

```bash
# Auth mode
export PLATFORM_AUTH_MODE=jwt

# Basic token
export PLATFORM_AUTH_TOKEN=your-strong-token

# JWT secret
export PLATFORM_JWT_SECRET=your-jwt-secret-key
```

### JWT Validation Improvements

The template includes enhanced JWT validation:

1. **60-second leeway** for clock skew tolerance
2. **Enhanced claim validation** (issuer, subject, issued-at-time)
3. **NBF (Not Before) claim support**
4. **Algorithm validation** (HS256 only)

### Generating Secure Secrets

Use OpenSSL to generate secure secrets:

```bash
# Generate JWT secret (32 bytes, base64 encoded)
openssl rand -base64 32

# Generate auth token
openssl rand -base64 64
```

### JWT Claims Structure

```json
{
  "sub": "user123",
  "exp": 1640995200,
  "iat": 1640991600,
  "iss": "your-app-name"
}
```

## Secret Management

### Best Practices

1. **Never commit real secrets** to version control
2. **Use environment variables** in production
3. **Rotate secrets regularly**
4. **Use different secrets** for different environments
5. **Store secrets securely** (vault, AWS Secrets Manager, etc.)

### Configuration Template

Use the provided template for secure configuration:

```bash
# Copy the template
cp config/local.yaml.template config/local.yaml

# Edit with your values
vim config/local.yaml
```

### Environment-Specific Configuration

**Development:**

```yaml
env: dev
secrets:
  db.url: "postgres://dev_user:dev_pass@localhost:5432/dev_db"
  platform.jwt_secret: "dev-jwt-secret-not-for-production"
```

**Production:**

```yaml
env: production
secrets:
  db.url: "${DATABASE_URL}"  # From environment
  platform.jwt_secret: "${PLATFORM_JWT_SECRET}"  # From environment
```

## Testing Security Features

### Running Security Tests

```bash
# Run all security-related tests
cargo test security_middleware

# Run JWT validation tests
cargo test jwt_validation

# Run CORS tests
cargo test cors

# Run security headers tests
cargo test security_headers
```

### Manual Testing

**CORS Testing:**

```bash
# Test preflight request
curl -X OPTIONS http://localhost:8080/api/echo \
  -H "Origin: http://localhost:3000" \
  -H "Access-Control-Request-Method: POST" \
  -H "Access-Control-Request-Headers: authorization,content-type"

# Test actual request
curl -X POST http://localhost:8080/api/echo \
  -H "Origin: http://localhost:3000" \
  -H "Content-Type: application/json" \
  -d '{"message": "test"}'
```

**Security Headers Testing:**

```bash
# Check security headers
curl -I http://localhost:8080/health

# Expected headers:
# X-Frame-Options: DENY
# X-Content-Type-Options: nosniff
# X-XSS-Protection: 1; mode=block
# Content-Security-Policy: default-src 'self'...
```

**JWT Testing:**

```bash
# Generate test token
export JWT_SECRET="test-secret"
export TOKEN=$(echo '{"sub":"test","exp":'$(($(date +%s)+3600))',"iat":'$(date +%s)',"iss":"test"}' | base64 | tr -d '\n')

# Test with JWT
curl -X POST http://localhost:8080/platform/protected \
  -H "Authorization: Bearer $TOKEN"
```

## Security Checklist

### Before Deployment

- [ ] Replace all placeholder secrets with real values
- [ ] Configure appropriate CORS origins for production
- [ ] Review and tighten CSP policy for production
- [ ] Enable HSTS with appropriate settings
- [ ] Set appropriate referrer policy
- [ ] Configure permissions policy based on actual needs
- [ ] Test security headers with tools like securityheaders.com
- [ ] Run all security tests
- [ ] Verify JWT validation with clock skew scenarios

### Ongoing Security

- [ ] Regularly rotate JWT secrets
- [ ] Monitor for security header bypasses
- [ ] Keep dependencies updated
- [ ] Review CORS configuration regularly
- [ ] Audit authentication logs
- [ ] Test for common vulnerabilities

## Security Tools and Resources

### Recommended Tools

- **OWASP ZAP**: Web application security testing
- **Burp Suite**: Web application penetration testing
- **securityheaders.com**: Security headers analysis
- **csp-evaluator.withgoogle.com**: CSP policy analysis
- **jwt.io**: JWT debugging and validation

### Security Resources

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [MDN Web Security](https://developer.mozilla.org/en-US/docs/Web/Security)
- [CSP Spec](https://www.w3.org/TR/CSP3/)
- [CORS Spec](https://fetch.spec.whatwg.org/#cors-protocol)

## Troubleshooting

### Common Issues

**CORS Issues:**
- Check that the origin is in the allowed list
- Verify preflight requests include required headers
- Ensure credentials are properly configured if needed

**Security Header Issues:**
- Some headers may conflict with certain CDN configurations
- CSP may need adjustment for third-party resources
- HSTS requires HTTPS in production

**JWT Issues:**
- Clock skew between servers can cause validation failures
- Ensure secrets are properly escaped in environment variables
- Check token expiration and issued-at times

### Debug Mode

Enable debug logging to troubleshoot security issues:

```bash
export RUST_LOG=debug
export RUST_LOG_STYLE=always
```

This will show detailed information about:
- CORS header processing
- Security header application
- JWT validation steps
- Authentication decisions
