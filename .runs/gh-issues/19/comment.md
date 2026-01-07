## Investigation Report: Issue #19 - Input Validation & Size Limits

### Status
**Status:** CRITICAL for security - Multiple validation gaps identified
**Local gates:** Code audit, security analysis

### Evidence

**1. ID Format Validation - MISSING**
- `gov-http/src/forks.rs:189` - Fork IDs not validated before file I/O
- `gov-http/src/friction.rs:189` - Friction IDs not validated
- All `Path<String>` parameters bypass validation

**Risk:** `GET /platform/forks/../../../../etc/passwd`

**2. No Body Size Limits - MISSING**
- `app-http/src/lib.rs:270-298` - No `DefaultBodyLimit` middleware
- Axum defaults to ~2GB
- JSON extraction accepts arbitrarily large payloads

**3. Unrestricted Directory Iteration - MISSING LIMITS**
- `gov-http/src/forks.rs:98-142` - No file count limits
- `gov-http/src/friction.rs:113-147` - No pagination

**Risk:** Memory exhaustion with thousands of files

**4. String Field Validation - PARTIAL**
- ✅ `todos.rs:179-192` - Title length check (256 chars)
- ✅ Empty field checks in todos
- ✗ No length limits on fork/friction fields
- ✗ No special character sanitization

**5. Unrestricted YAML Deserialization - MISSING LIMITS**
- `app-http/src/platform.rs:537,668,607` - `serde_yaml::from_str()` without limits
- `spec-runtime/src/config.rs:111-121` - No depth/size limits

**Risk:** Billion laughs attack, stack overflow

### Impact

**Security Risks:**
- Path traversal (HIGH)
- DoS via large requests (CRITICAL)
- Memory exhaustion (CRITICAL)
- Data injection (MEDIUM)

### Plan

**Phase 1: ID Format Validation**
```rust
fn validate_id(id: &str, pattern: &str) -> Result<(), ValidationError> {
    let re = Regex::new(pattern)?;
    if !re.is_match(id) {
        return Err(ValidationError::InvalidIdFormat);
    }
    Ok(())
}
```

**Phase 2: Body Size Limits**
```rust
// In router setup
.layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB
```

**Phase 3: Collection Size Limits**
- Add `MAX_FILES_IN_DIR = 1000`
- Implement pagination with `offset` and `limit`

**Phase 4: YAML Limits**
- Create `deserialize_yaml_with_limits()` function
- MAX_YAML_DEPTH = 16, MAX_YAML_SIZE = 1MB

**Test plan:**
```bash
# Test validation rejection
cargo test -p gov-http validation
# Test size limits
curl -X POST --data-binary @large_file.json http://localhost:8080/todos
```

### Decision / Next Action

**Recommend:** P1 HIGH PRIORITY - Multiple security gaps. Related to #68 (security hardening). Implement Phases 1-2 in v3.3.14, Phases 3-4 in v3.4.0.
