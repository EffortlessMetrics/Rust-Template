## Investigation Report: Issue #68 - Security Hardening Sprint

### Status
**Status:** triaged / fix-ready
**Local gates:** `cargo audit`, grep for unsafe patterns, header examination, JWT validation analysis

### Evidence

#### What Exists Today (SOLID):

**Security Headers:**
- ✅ Content Security Policy (CSP) with strict mode for production
- ✅ X-Frame-Options: DENY (clickjacking prevention)
- ✅ X-Content-Type-Options: nosniff
- ✅ X-XSS-Protection: 1; mode=block
- ✅ HSTS: max-age=31536000; includeSubDomains; preload
- ✅ Referrer-Policy: strict-origin-when-cross-origin
- ✅ Permissions-Policy: comprehensive feature restrictions
- ✅ Cross-Origin-Embedder-Policy: require-corp
- ✅ Cross-Origin-Opener-Policy: same-origin
- ✅ Cross-Origin-Resource-Policy: same-origin

**CORS Protection:**
- ✅ Configurable allowed origins (defaults: localhost:3000, localhost:8080)
- ✅ Origin validation with wildcard support
- ✅ Credentials handling (default: false for security)

**Authentication:**
- ✅ Platform auth with three modes: Open, Basic, JWT
- ✅ JWT validation with expiration checks (leeway: 60s)
- ✅ Constant-time token comparison (timing attack resistant)
- ✅ JWT claims validation: exp, iat, nbf, iss, sub
- ✅ Fail-closed auth configuration

**Input Validation:**
- ✅ JSON rejection handling with proper error codes
- ✅ Title length validation (max 256 chars for todos)
- ✅ Duplicate ID detection (409 Conflict)
- ✅ Structured error responses with AC IDs

---

#### What's Missing or At-Risk:

**1. Request Size Limits** ⚠️
- **Missing:** No explicit DefaultBodyLimit middleware configured
- **Risk:** Unlimited request body sizes could enable DoS attacks
- **Severity:** MEDIUM (Axum default is 2MB, but not explicitly documented)

**2. Request Timeouts** ⚠️
- **Missing:** No explicit request timeout configuration
- **Risk:** Slow clients or slow loris attacks could exhaust connections
- **Severity:** MEDIUM

**3. Rate Limiting** ⚠️
- **Missing:** No rate limiting middleware
- **Risk:** API brute force attacks, credential stuffing, resource exhaustion
- **Severity:** HIGH for production

**4. Panics in HTTP Handlers** ⚠️
- **Found:** `.expect()` calls in `crates/app-http/src/platform.rs:703` (serialization)
- **Impact:** Resource overload could crash serialization
- **Severity:** MEDIUM

### Impact

**Who is at Risk:**
- Public API consumers (if exposed without reverse proxy rate limiting)
- Operators deploying without understanding auth modes
- Internal services under sustained load without rate limits

**Threat Vectors:**
- **DoS via unlimited request sizes:** Medium risk
- **Brute force attacks:** High risk without rate limiting
- **Slow loris:** Medium risk without timeouts
- **JWT token reuse:** Low risk (expiration enforced)
- **CORS bypass:** Low risk (configuration strict by default)
- **Injection attacks:** Low risk (prepared statements, input validation)

### Plan

#### Minimal Fix (v3.3.14):

1. **Add Request Size Limits**
   - File: `crates/app-http/src/lib.rs`
   - Add: `DefaultBodyLimit::max(2 * 1024 * 1024)` layer

2. **Replace panics with error handling**
   - File: `crates/app-http/src/platform.rs:703`
   - Change: `.expect()` → `.map_err(|e| AppError::internal_error(...))?`

3. **Document Security Headers Config**
   - File: `docs/SECURITY.md` (new)
   - Content: Explain each header, production vs development settings

#### Follow-ups (v3.4.0):

1. **Rate Limiting Middleware** - Use `tower_http::rate_limit` or `governor` crate
2. **Request Timeouts** - Use `tower::timeout` middleware (30-60s)
3. **Connection Limits** - Axum connection config
4. **Security ADR** - Document security decisions

### Security Checklist (for ACs)

- [ ] Request body size limit middleware configured (2 MB default)
- [ ] All `.expect()` calls in HTTP handlers replaced with proper error handling
- [ ] Security headers configured for both development and production
- [ ] CORS configuration prevents unrestricted origin access
- [ ] JWT token expiration and signature validation enforced
- [ ] Constant-time comparison used for token matching
- [ ] Input validation tests cover boundary conditions
- [ ] All security headers verified in integration tests
- [ ] Request ID propagation confirmed in error responses

### Decision / Next Action

**Recommend:** Keep issue open with follow-up tasks
- **Immediate (v3.3.14):** Add request size limits, fix panics, document security
- **Short-term (v3.4.0):** Rate limiting, timeouts, comprehensive security testing

**Security Posture Summary:**
- **Foundation:** SOLID (headers, CORS, JWT validation all correct)
- **Gaps:** OPERATIONAL (rate limiting, timeouts, request limits)
- **Risk Level:** MEDIUM (API DoS vulnerability without rate limiting)
