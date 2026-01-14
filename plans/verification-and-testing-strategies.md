# Verification Procedures and Testing Strategies

**Purpose**: This guide provides comprehensive verification procedures and testing strategies for all implemented fixes to ensure the Rust template meets release readiness criteria.

## Overview

This guide covers verification procedures and testing strategies for security, build infrastructure, code quality, and documentation fixes. Each category has specific verification commands, testing approaches, and success criteria to ensure implementation quality and reliability.

## Security Verification Procedures

### 1. CORS Middleware Verification

```bash
# Test CORS configuration
curl -v -H "Origin: https://example.com" \
  -H "Access-Control-Request-Method: POST" \
  -H "Access-Control-Request-Headers: authorization" \
  -X OPTIONS http://localhost:8080/health

# Verify CORS headers in response
curl -I http://localhost:8080/health | grep -E "(Access-Control-Allow-Origin|Access-Control-Allow-Methods)"

# Test preflight request handling
curl -X OPTIONS -H "Origin: https://test-domain.com" http://localhost:8080/health

# Test credentials handling
curl -H "Origin: https://trusted-domain.com" \
  -H "Authorization: Bearer $TOKEN" \
  -v -X POST http://localhost:8080/platform/tasks

# Verify CORS wildcard support
curl -H "Origin: https://api.example.com" http://localhost:8080/health
```

### 2. Security Headers Verification

```bash
# Test all security headers are present
curl -I http://localhost:8080/health | grep -E "(X-Frame-Options|Content-Security-Policy|Strict-Transport-Security)"

# Test CSP in development vs production
ENV=development cargo run -p app-http
ENV=production cargo run -p app-http

# Verify HSTS is disabled in development
curl -I http://localhost:8080/health | grep -v "Strict-Transport-Security"

# Test permissions policy
curl -I http://localhost:8080/health | grep -E "Permissions-Policy"
```

### 3. JWT Validation Verification

```bash
# Test JWT token creation and validation
cargo test -p app-http jwt_validation

# Test token expiration handling
# Create expired token and verify rejection
cargo test -p app-http jwt_validation expired_token

# Test JWT leeway for clock skew
# Test with tokens created 30 seconds in future/past
cargo test -p app-http jwt_validation clock_skew

# Test claim validation
# Test tokens with invalid claims
cargo test -p app-http jwt_validation invalid_claims
```

### 4. Secrets Management Verification

```bash
# Verify configuration template works
cp config/local.yaml.template config/local.yaml
# Edit with environment variables
export JWT_SECRET="test-secret" ADMIN_PASSWORD="admin123"
cargo run -p app-http

# Verify no hardcoded secrets in source
grep -r "secret\|password\|token" crates/ --include="*.rs"

# Test environment variable override
JWT_SECRET="override-secret" cargo run -p app-http
```

## Build Infrastructure Verification Procedures

### 1. Tool Checksums Verification

```bash
# Verify all tool checksums are present
sha256sum -c scripts/tools.sha256

# Test checksum verification during bootstrap
ENFORCE_CHECKSUMS=1 ./bootstrap-tools.sh

# Test with corrupted tool binary
echo "corrupted data" > .tools/bin/oasdiff
ENFORCE_CHECKSUMS=1 ./bootstrap-tools.sh

# Verify tool download and caching
rm -rf .tools/bin
./bootstrap-tools.sh
# Check if tools were cached properly
```

### 2. Rust Version Alignment Verification

```bash
# Verify rust-toolchain.toml and Cargo.toml alignment
grep -r "rust-version" Cargo.toml rust-toolchain.toml | diff

# Check MSRV consistency across workspace
cargo check --workspace --all-targets --all-features

# Test with different Rust versions
rustup default 1.88.0 && cargo build --workspace
rustup default 1.89.0 && cargo build --workspace
```

### 3. MSRV Compliance Verification

```bash
# Validate MSRV across all crates
cargo check --workspace --all-targets --all-features

# Test MSRV with minimum supported version
rustup default 1.75.0 && cargo build --workspace

# Verify workspace inheritance working
find crates -name Cargo.toml -exec grep -q "rust-version.*workspace"
```

### 4. Security Advisory Management Verification

```bash
# Test deny.toml configuration
cargo deny check

# Test advisory database updates
cargo audit --version-lock

# Verify ignored advisories are documented
grep -A 5 -B 5 "RUSTSEC-" deny.toml
```

## Code Quality Verification Procedures

### 1. Clippy Warnings Verification

```bash
# Run comprehensive clippy check
cargo clippy --workspace --all-targets -- -D warnings

# Verify specific warning fixes
cargo clippy --workspace --all-targets -- -W clippy::all

# Check for new warnings introduced
cargo clippy --workspace --all-targets 2>&1 | grep -E "warning"
```

### 2. Error Type Optimization Verification

```bash
# Verify error type sizes
cargo expand --dry-run | grep -A 20 "struct.*Error" | wc -c

# Test error handling performance
cargo bench -p app-http error_handling

# Verify memory usage improvements
cargo test --release -- -p app-http errors
```

### 3. Panic! Removal Verification

```bash
# Verify panic! removal
grep -r "panic!" crates/ --include="*.rs" | wc -l

# Test error handling improvements
cargo test -p app-http errors
```

### 4. TaskStatus Enum Verification

```bash
# Test TaskStatus functionality
cargo test -p gov-model

# Test backward compatibility
cargo test -p gov-model -- --ignored static

# Test status transitions
cargo test -p gov-model transitions
```

## Documentation Verification Procedures

### 1. Platform API Contract Verification

```bash
# Test platform API endpoints
curl http://localhost:8080/platform/status | jq '.'
curl http://localhost:8080/platform/docs/index | jq '.'
curl http://localhost:8080/platform/agent/hints | jq '.'
```

### 2. Agent Skills Reference Verification

```bash
# Test agent skills linting
cargo xtask skills-lint

# Verify skills documentation completeness
find .claude/skills -name SKILL.md | wc -l

# Test skills examples work
# Test skills mentioned in documentation actually exist and function
```

### 3. Database Integration Verification

```bash
# Test add-database.md exists
test -f docs/how-to/add-database.md

# Test database integration examples
cargo test -p adapters-db-sqlx database_integration

# Verify database configuration examples work
# Test with different database configurations
```

## Integration Testing Strategies

### End-to-End Security Testing

```bash
# Complete security workflow test
curl -X POST http://localhost:8080/platform/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"test"}' | jq -r '.token' \
  http://localhost:8080/platform/status

# Test CORS with authentication
curl -H "Origin: https://trusted-domain.com" \
  -H "Authorization: Bearer $TOKEN" \
  -v -X POST http://localhost:8080/platform/tasks

# Test security headers under load
# Use siege or similar load testing tool
siege -c 10 -t 30s http://localhost:8080/health
```

### Build Infrastructure Integration Testing

```bash
# Test reproducible builds
docker build -t rust-template .
docker run --rm rust-template cargo build --workspace

# Test cross-platform builds
cargo build --workspace --target x86_64-unknown-linux-gnu
cargo build --workspace --target x86_64-apple-darwin
cargo build --workspace --target x86_64-pc-windows-msvc

# Test CI/CD pipeline integration
# Simulate CI pipeline execution
# Test tool checksum verification in CI environment
```

### Code Quality Integration Testing

```bash
# Test error handling under load
# Use load testing tools to verify error handling performance

# Test TaskStatus API compatibility
# Verify existing integrations continue to work with new enum

# Test backward compatibility
# Ensure existing task status strings still parse correctly
cargo test -p gov-model -- --ignored static
```

### Documentation Integration Testing

```bash
# Test database integration with new documentation
# Follow add-database.md guide with new documentation in place

# Test complete documentation workflow
# 1. Create documentation
# 2. Update references
# 3. Validate documentation
# 4. Test examples
```

## Performance Testing Strategy

### Security Performance Testing

```bash
# Middleware overhead measurement
ab -n 1000 -c 10 http://localhost:8080/health

# JWT validation performance
time cargo test -p app-http jwt_validation

# TLS handshake overhead with security headers
# Measure impact of security headers on connection establishment
```

### Build Performance Testing

```bash
# Build time optimization
time cargo build --workspace --release

# Incremental build performance
cargo build --workspace && cargo build --workspace

# Dependency resolution performance
time cargo generate-lockfile
time cargo metadata --format-version 1
```

### Code Quality Performance Testing

```bash
# Error handling performance
cargo bench -p app-http error_handling

# Memory usage validation
cargo test --release -- -p app-http errors
# Use memory profiling tools
valgrind --tool=massif cargo test -p app-http
```

## Success Criteria

### Overall Success Metrics

- ✅ All security components verified and working correctly
- ✅ All build infrastructure components validated and consistent
- ✅ All code quality improvements implemented and measured
- ✅ All documentation components created and validated
- ✅ Integration tests passing across all components
- ✅ Performance benchmarks meeting or exceeding targets

### Verification Checklists

#### Security Verification

- [ ] CORS middleware configured and working: `curl -I http://localhost:8080/health | grep -E "Access-Control"`
- [ ] Security headers present in responses: `curl -I http://localhost:8080/health | grep -E "(X-Frame-Options|Content-Security-Policy)"`
- [ ] JWT validation working correctly: `cargo test -p app-http jwt_validation`
- [ ] Secrets management functional: `test -f config/local.yaml && cargo run -p app-http`

#### Build Infrastructure Verification

- [ ] Tool checksums verified: `sha256sum -c scripts/tools.sha256`
- [ ] Rust versions aligned: `grep -r "rust-version" Cargo.toml rust-toolchain.toml | diff`
- [ ] MSRV compliance achieved: `cargo check --workspace --all-targets --all-features`
- [ ] Security advisories functional: `cargo deny check && cargo deny --workspace`

#### Code Quality Verification

- [ ] Clippy warnings eliminated: `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] Error types optimized: `cargo expand --dry-run | grep -A 20 "struct.*Error" | wc -c`
- [ ] Panic! usage removed: `grep -r "panic!" crates/ --include="*.rs" | wc -l`
- [ ] TaskStatus functionality working: `cargo test -p gov-model`

#### Documentation Verification

- [ ] Platform API contract functional: `curl http://localhost:8080/platform/status | jq '.'`
- [ ] Agent skills reference complete: `cargo xtask skills-lint`
- [ ] Database integration documented: `test -f docs/how-to/add-database.md`

## Rollback Verification Procedures

### Security Rollback Verification

```bash
# Verify security rollback restores previous state
git checkout HEAD~1 -- crates/app-http/src/middleware/cors.rs
curl -I http://localhost:8080/health | grep -v "Access-Control"

# Test JWT rollback
git checkout HEAD~1 -- crates/app-http/src/security.rs
cargo test -p app-http jwt_validation
```

### Build Infrastructure Rollback Verification

```bash
# Verify build rollback restores functionality
git checkout HEAD~1 -- scripts/tools.sha256
cargo build --workspace

# Test MSRV rollback
git checkout HEAD~1 -- Cargo.toml rust-toolchain.toml
cargo check --workspace
```

### Code Quality Rollback Verification

```bash
# Verify error type rollback
git checkout HEAD~1 -- crates/app-http/src/errors.rs
cargo test -p app-http errors

# Verify TaskStatus rollback
git checkout HEAD~1 -- crates/gov-model/src/lib.rs
cargo test -p gov-model
```

This comprehensive verification and testing strategy ensures all implemented fixes work correctly and can be safely deployed to production.
