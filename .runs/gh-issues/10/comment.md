## Investigation Report: Issue #10 - Path Traversal Protection

### Status
**Status:** CONFIRMED CRITICAL - Path traversal vulnerabilities identified
**Local gates:** Code audit, pattern analysis

### Evidence

**1. Primary Vulnerability: Path Traversal in Configuration (CRITICAL)**
- `crates/rust_iac_config/src/config.rs:98-101, 244, 258, 273`
- Multiple unsafe `.join()` operations without path boundary validation
- Line 244: `workspace_root.join(&self.manifests_path)` - user-supplied path joined without checks

**2. Secondary Vulnerability: Missing Symlink Detection (HIGH)**
- `crates/gov-http/src/forks.rs:147` - `fs::read_to_string(path)` without symlink check
- No `is_symlink()` verification before reading

**3. Validation Module Also Vulnerable (HIGH)**
- `crates/rust_iac_config/src/validation.rs:19, 34`
- Required directories/files joined without validation

**Attack Scenarios:**
```yaml
# attacker-config.yaml
environments:
  - name: "dev"
    manifests_path: "../../../etc"  # Path traversal
```

```bash
# Symlink attack in forks/
ln -s /etc/passwd forks/FORK-MALICIOUS-001.yaml
```

### Impact

**Security Risks:**
- Arbitrary file disclosure via path traversal
- Secrets exposure (API keys, private certificates)
- Complete workspace escape

**Blast Radius:**
- All path-based configuration fields vulnerable
- `/platform/forks` and `/platform/friction` endpoints
- Configuration loading across entire codebase

### Plan

**Minimal fix:**

1. **Add path validation function:**
```rust
fn validate_path_within_workspace(base: &Path, user_path: &Path) -> Result<PathBuf, ConfigError> {
    let canonical = base.join(user_path).canonicalize()?;
    if !canonical.starts_with(base.canonicalize()?) {
        return Err(ConfigError::PathTraversalAttempt);
    }
    Ok(canonical)
}
```

2. **Add symlink detection:**
```rust
if path.symlink_metadata()?.is_symlink() {
    return Err(PlatformError::SymlinkNotAllowed);
}
```

3. **Update all `.join()` calls** to use validated path function

4. **Add error variants:**
- `ConfigError::PathTraversalAttempt`
- `PlatformError::SymlinkNotAllowed`

**Test plan:**
```bash
# Test path traversal rejection
cargo test -p rust_iac_config path_traversal
# Test symlink rejection
cargo test -p gov-http symlink_rejection
```

### Decision / Next Action

**Recommend:** P0 CRITICAL - Fix immediately. This allows arbitrary file read access and must be addressed before any production deployment.
