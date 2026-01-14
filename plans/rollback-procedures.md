# Rollback Procedures

**Purpose**: This guide provides detailed rollback procedures for all implemented fixes to ensure teams can safely revert changes if issues arise during implementation or deployment.

## Overview

This guide covers rollback strategies for security, build infrastructure, code quality, and documentation fixes. Each rollback procedure includes specific commands, verification steps, and criteria for determining when rollback is appropriate.

## Security Rollback Procedures

### 1. CORS Middleware Rollback

**Scenario**: CORS middleware causing issues with cross-origin requests

**Rollback Commands**:

```bash
# Revert CORS middleware to previous state
git checkout HEAD~1 -- crates/app-http/src/middleware/cors.rs

# Remove security headers middleware if it was added with CORS
git checkout HEAD~1 -- crates/app-http/src/middleware/security_headers.rs

# Revert JWT validation changes if related to CORS
git checkout HEAD~1 -- crates/app-http/src/security.rs

# Restore previous configuration
git checkout HEAD~1 -- config/local.yaml.template
```

**Verification Commands**:

```bash
# Verify CORS is reverted to previous state
curl -I http://localhost:8080/health | grep -v "Access-Control"

# Test that CORS issues are resolved
curl -H "Origin: https://problematic-domain.com" -X OPTIONS http://localhost:8080/health

# Verify security headers are still present
curl -I http://localhost:8080/health | grep -E "(X-Frame-Options|Content-Security-Policy)"

# Test JWT functionality still works
cargo test -p app-http jwt_validation
```

**Rollback Success Criteria**:
- [ ] CORS behavior matches previous working state
- [ ] No new security issues introduced
- [ ] Existing functionality preserved
- [ ] Tests pass with rollback code

### 2. Security Headers Rollback

**Scenario**: Security headers causing application failures

**Rollback Commands**:

```bash
# Revert security headers middleware
git checkout HEAD~1 -- crates/app-http/src/middleware/security_headers.rs

# Revert related changes if needed
git checkout HEAD~1 -- crates/app-http/src/lib.rs

# Restore previous CSP configuration
git checkout HEAD~1 -- crates/app-http/src/middleware/cors.rs
```

**Verification Commands**:

```bash
# Verify security headers are reverted
curl -I http://localhost:8080/health | grep -v -E "(X-Frame-Options|Content-Security-Policy|Strict-Transport-Security)"

# Test application functionality
cargo test -p app-http
```

### 3. JWT Validation Rollback

**Scenario**: JWT validation breaking authentication

**Rollback Commands**:

```bash
# Revert JWT validation changes
git checkout HEAD~1 -- crates/app-http/src/security.rs

# Restore previous token handling
git checkout HEAD~1 -- crates/app-http/src/middleware/platform_auth.rs

# Restore previous configuration
git checkout HEAD~1 -- config/local.yaml.template
```

**Verification Commands**:

```bash
# Verify JWT validation is restored
cargo test -p app-http jwt_validation

# Test authentication flow end-to-end
curl -X POST http://localhost:8080/platform/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"test"}' | jq -r '.token' \
  http://localhost:8080/platform/status
```

### 4. Secrets Management Rollback

**Scenario**: Configuration template causing deployment issues

**Rollback Commands**:

```bash
# Revert to previous configuration template
git checkout HEAD~1 -- config/local.yaml.template

# Verify configuration is restored
cp config/local.yaml.template config/local.yaml
# Test with previous secrets
export JWT_SECRET="old-secret" cargo run -p app-http
```

## Build Infrastructure Rollback Procedures

### 1. Tool Checksums Rollback

**Scenario**: Tool checksum verification failing in CI/CD

**Rollback Commands**:

```bash
# Restore previous tool checksums
git checkout HEAD~1 -- scripts/tools.sha256

# Verify checksums are restored
sha256sum -c scripts/tools.sha256

# Test bootstrap process
ENFORCE_CHECKSUMS=1 ./bootstrap-tools.sh
```

### 2. Rust Version Alignment Rollback

**Scenario**: Version misalignment breaking builds

**Rollback Commands**:

```bash
# Revert Rust version changes
git checkout HEAD~1 -- rust-toolchain.toml Cargo.toml

# Verify version alignment is restored
grep -r "rust-version" Cargo.toml rust-toolchain.toml | diff

# Test build with restored versions
cargo build --workspace
```

### 3. MSRV Compliance Rollback

**Scenario**: MSRV changes causing build failures

**Rollback Commands**:

```bash
# Revert MSRV compliance changes
git checkout HEAD~1 -- $(find crates -name Cargo.toml -exec grep -l "rust-version" {} \;)

# Verify MSRV compliance is restored
cargo check --workspace --all-targets --all-features
```

### 4. Security Advisory Rollback

**Scenario**: New deny.toml rules blocking deployment

**Rollback Commands**:

```bash
# Restore previous deny.toml
git checkout HEAD~1 -- deny.toml

# Verify security advisory configuration
cargo deny check

# Test security advisory functionality
cargo audit --version-lock
```

## Code Quality Rollback Procedures

### 1. Error Type Rollback

**Scenario**: Optimized error types causing serialization issues

**Rollback Commands**:

```bash
# Revert error type optimization
git checkout HEAD~1 -- crates/app-http/src/errors.rs

# Verify error types are restored
cargo expand --dry-run | grep -A 20 "struct.*Error" | wc -c

# Test error handling functionality
cargo test -p app-http errors
```

### 2. Panic! Removal Rollback

**Scenario**: Panic! reintroduction causing instability

**Rollback Commands**:

```bash
# Find files with panic! usage
grep -rl "panic!" crates/ --include="*.rs" | cut -d: -f1

# Revert all panic! changes
git checkout HEAD~1 -- $(grep -rl "panic!" crates/ --include="*.rs" | cut -d: -f1)

# Verify panic! removal
grep -r "panic!" crates/ --include="*.rs" | wc -l

# Test application stability
cargo test --workspace
```

### 3. TaskStatus Enum Rollback

**Scenario**: TaskStatus changes breaking existing integrations

**Rollback Commands**:

```bash
# Revert TaskStatus enum changes
git checkout HEAD~1 -- crates/gov-model/src/lib.rs

# Verify TaskStatus functionality
cargo test -p gov-model

# Test backward compatibility
cargo test -p gov-model -- --ignored static
```

## Documentation Rollback Procedures

### 1. Platform API Contract Rollback

**Scenario**: API contract changes breaking client integrations

**Rollback Commands**:

```bash
# Revert platform API contract changes
git checkout HEAD~1 -- docs/reference/platform_api_contract.md

# Verify API contract is stable
curl http://localhost:8080/platform/status | jq '.'
```

### 2. Agent Skills Reference Rollback

**Scenario**: Skills documentation changes breaking agent workflows

**Rollback Commands**:

```bash
# Revert agent skills documentation
git checkout HEAD~1 -- docs/AGENT_SKILLS.md

# Verify skills documentation is stable
find .claude/skills -name SKILL.md | wc -l
```

### 3. Database Integration Rollback

**Scenario**: Database integration guide causing deployment failures

**Rollback Commands**:

```bash
# Revert database integration changes
git checkout HEAD~1 -- docs/how-to/add-database.md

# Verify database integration is removed
test ! -f docs/how-to/add-database.md

# Test without database integration
cargo test -p adapters-db-sqlx
```

## Rollback Decision Framework

### When to Rollback

**Immediate Rollback Required**:
- Security vulnerabilities in production
- Authentication/authorization failures
- Build process completely broken
- Application crashes or instability
- Data corruption or loss

**Rollback Considerations**:
- Impact on existing deployments
- Data migration requirements
- Client compatibility impact
- Rollback complexity vs. fix time
- Team availability to execute rollback

### Rollback Execution Steps

1. **Assess Impact**: Evaluate affected systems and users
2. **Communicate**: Notify stakeholders about rollback
3. **Execute Rollback**: Use appropriate rollback commands
4. **Verify**: Confirm rollback success with verification commands
5. **Monitor**: Watch for additional issues post-rollback
6. **Document**: Record rollback reason and outcomes

### Rollback Verification

```bash
# Verify rollback success
# Run comprehensive health checks
cargo xtask selftest

# Verify specific component functionality
# Component-specific tests based on what was rolled back

# Check system stability
# Monitor application logs for errors
```

## Emergency Rollback Procedures

### Critical Issue Response

For critical security vulnerabilities or production outages:

1. **Immediate Isolation**: Take affected systems offline if necessary
2. **Hotfix Deployment**: Deploy minimal fix to critical issue
3. **Communication**: Notify all stakeholders immediately
4. **Post-mortem**: Document root cause and prevention measures

### Rollback Testing

```bash
# Test rollback procedures in staging environment
# Execute rollback in staging before production
# Verify staging system stability
# Test with realistic load
```

## Rollback Success Criteria

### Successful Rollback Metrics

- [ ] System stability restored (no crashes, no errors)
- [ ] Security issues resolved (vulnerabilities patched)
- [ ] Build process working (reproducible builds)
- [ ] Code quality stable (tests passing, no regressions)
- [ ] Documentation accurate (references correct, examples working)
- [ ] User impact minimized (downtime < 5 minutes)
- [ ] Rollback fully documented (reasons, steps, outcomes)

This comprehensive rollback guide ensures teams can safely revert any changes while maintaining system stability and data integrity.
