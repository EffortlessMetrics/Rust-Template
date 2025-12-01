---
id: GUIDE-TPL-TROUBLESHOOT-001
title: Troubleshooting Guide
doc_type: guide
status: published
audience: developers
tags: [troubleshooting, debugging, errors, faq]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-ONBOARDING, REQ-PLT-DEVEX-CONTRACT]
acs: [AC-PLT-015, AC-PLT-016, AC-PLT-019, AC-PLT-020]
adrs: [ADR-0002, ADR-0005, ADR-0017]
last_updated: 2025-11-26
---

# Troubleshooting Guide

**Quick reference for diagnosing and fixing common issues in the Rust-as-Spec Platform Cell.**

This guide uses a FAQ format organized by problem domain. Use your browser's search (Ctrl+F / Cmd+F) to quickly find specific error messages.

---

## Table of Contents

- [Environment Setup Issues](#environment-setup-issues)
- [Build Failures](#build-failures)
- [Test Failures](#test-failures)
- [CI/CD Failures](#cicd-failures)
- [Pre-commit Hook Issues](#pre-commit-hook-issues)
- [Windows-Specific Issues](#windows-specific-issues)
- [Policy Test Failures](#policy-test-failures)
- [BDD/Acceptance Test Issues](#bddacceptance-test-issues)
- [xtask Command Issues](#xtask-command-issues)
- [Docker and Container Issues](#docker-and-container-issues)
- [Performance Issues](#performance-issues)

---

## Environment Setup Issues

### Q: "nix: command not found" after installation

**Symptom:**
```bash
$ nix develop
bash: nix: command not found
```

**Cause:** Nix profile not loaded in current shell

**Diagnostic:**
```bash
# Check if Nix is installed
ls ~/.nix-profile/

# Check if profile is sourced
echo $NIX_PROFILES
```

**Fix:**
```bash
# Source Nix profile manually
source ~/.nix-profile/etc/profile.d/nix.sh

# Or restart shell
exit
# Open new terminal

# For permanent fix, ensure your shell sources Nix
# The Determinate Systems installer should have done this automatically
```

**Prevention:**
```bash
# Verify shell profile includes Nix
grep nix ~/.bashrc  # or ~/.zshrc
# Should see: source ~/.nix-profile/etc/profile.d/nix.sh
```

---

### Q: "conftest not found" but I'm in nix develop

**Symptom:**
```bash
$ cargo xtask policy-test
Error: conftest not available
```

**Cause:** Nix flake may need updating

**Diagnostic:**
```bash
# Verify conftest is available in devshell
nix develop -c conftest --version
# Should show: Conftest: 0.52.0 or later
```

**Fix:**
```bash
# Update flake lock
nix flake update

# Re-enter devshell
exit
nix develop

# Verify
conftest --version
```

**Alternative (Tier 2 - Native toolchain):**
```bash
# Install conftest via cargo-binstall
cargo install cargo-binstall
cargo binstall conftest

# Verify
conftest --version
```

---

### Q: "cargo xtask doctor" reports issues

**Symptom:**
```bash
$ cargo xtask doctor
⚠️ Some checks failed
```

**Diagnostic:**
```bash
# Run doctor with verbose output
cargo xtask doctor

# Check individual components
rustc --version
cargo --version
git --version
docker --version
conftest --version
```

**Common issues and fixes:**

| Issue | Cause | Fix |
|-------|-------|-----|
| Rust version mismatch | Using wrong toolchain | `rustup default stable` or enter `nix develop` |
| conftest not found | Not in PATH or not installed | Install via `cargo binstall conftest` or use `nix develop` |
| Docker not available | Docker not running | Start Docker Desktop |
| Git not configured | No user.name/email | `git config --global user.name "Your Name"` |

---

### Q: Nix flake check fails

**Symptom:**
```bash
$ nix flake check
error: builder for '/nix/store/...' failed
```

**Diagnostic:**
```bash
# Check flake validity
nix flake metadata

# Check individual outputs
nix build .#packages.x86_64-linux.default --show-trace
```

**Fix:**
```bash
# Update flake inputs
nix flake update

# Clear Nix cache if corrupted
nix-collect-garbage -d

# Retry
nix develop
```

---

### Q: "warning: unknown setting 'lazy-trees'" appears in Nix output

**Symptom:**
```bash
$ nix develop
warning: unknown setting 'lazy-trees'
```

**Cause:** The `lazy-trees` setting was an experimental Nix feature that has been removed in Nix 2.30+. It's configured in `/etc/nix/nix.conf` by Determinate Nix installer but is no longer recognized.

**Impact:** Cosmetic only - does not affect functionality. The warning is safe to ignore.

**Diagnostic:**
```bash
# Check Nix version
nix --version
# Nix 2.30+ will show this warning

# Verify setting location
grep lazy-trees /etc/nix/nix.conf
# Shows: lazy-trees = true
```

**Fix (System-wide - requires sudo):**
```bash
# Override in custom config
echo "# Override deprecated lazy-trees setting from managed config" | sudo tee -a /etc/nix/nix.custom.conf
echo "lazy-trees = false" | sudo tee -a /etc/nix/nix.custom.conf

# Verify warning is gone
nix develop
```

**Note:** If you don't have sudo access or prefer not to modify system config:
- The warning is harmless and can be ignored
- It will not affect builds, tests, or any functionality
- It's listed as a known cosmetic issue in the project's ROADMAP.md

**Prevention:** When Determinate Nix installer is updated, this setting should be removed from their managed configuration.

---

### Q: Nix devshell: rustc/libz.so.1 / sccache error

**Symptom:**
```bash
$ nix develop -c cargo xtask ac-status
error while loading shared libraries: libz.so.1: cannot open shared object file
```

Or xtask commands that shell out to `cargo` may hang or fail before tests run, leaving `target/junit/acceptance.xml` empty.

**Cause:** `sccache` (the RUSTC_WRAPPER) is unable to find `libz.so.1` when invoked via `nix develop -c`. This is a Nix environment composition issue where certain libraries aren't correctly propagated to subprocesses.

**Impact:** JUnit report generation fails silently, causing `ac-status` and `docs/feature_status.md` to show stale or incorrect data.

**Workaround (Option 1):** Run xtask as if already inside Nix shell
```bash
# Inside nix develop, or with environment simulated:
IN_NIX_SHELL=1 RUSTC_WRAPPER="" ./target/release/xtask ac-status
```

**Workaround (Option 2):** Disable sccache for the session
```bash
RUSTC_WRAPPER="" cargo xtask ac-status
```

**Workaround (Option 3):** Use low-resource mode
```bash
XTASK_LOW_RESOURCES=1 cargo xtask ac-status
```

**Note:** This is an environment-level issue; it does not reflect kernel correctness but may need coordination with your Nix/tooling setup. See `FRICTION_LOG.md` for tracking.

---

## Build Failures

### Q: "failed to remove xtask.exe" (os error 5) on Windows

**Symptom:**
```
error: failed to remove file 'target\debug\xtask.exe'
Caused by: Access is denied. (os error 5)
```

**Cause:** Windows file locking (antivirus, file explorer, or running process)

**This is NOT a test failure.** It's a Windows platform limitation.

**Diagnostic:**
```powershell
# Check what's using the file
tasklist | findstr "xtask cargo"

# Check antivirus activity
Get-MpComputerStatus | Select-Object RealTimeProtectionEnabled
```

**Fix (Recommended):** Exclude target/ from antivirus
```powershell
# Windows Defender (run as Administrator)
Add-MpPreference -ExclusionPath "C:\Code\Rust-Template\target"

# Verify
Get-MpPreference | Select-Object -ExpandProperty ExclusionPath
```

**Fix (Alternative):** Use WSL2 for canonical validation
```bash
# In WSL2
wsl
cd ~/Rust-Template
nix develop
cargo xtask selftest
```

**Fix (Quick workaround):**
```powershell
# Kill processes and retry
taskkill /F /IM cargo.exe
taskkill /F /IM xtask.exe
timeout /t 5
cargo xtask selftest
```

**See also:** [Windows-Specific Issues](#windows-specific-issues)

---

### Q: "linker 'link.exe' not found" on Windows

**Symptom:**
```
error: linker `link.exe` not found
```

**Cause:** Visual Studio Build Tools not installed

**Fix:**
```powershell
# Install via winget
winget install Microsoft.VisualStudio.2022.BuildTools

# During installation, select:
# - "Desktop development with C++"
# - Windows 10/11 SDK

# Restart terminal after installation
```

**Alternative:** Install full Visual Studio Community
```powershell
winget install Microsoft.VisualStudio.2022.Community
```

---

### Q: OpenSSL linking errors on Windows

**Symptom:**
```
error: failed to run custom build command for `openssl-sys`
```

**Cause:** Windows doesn't ship OpenSSL by default

**Fix (Option 1):** Use vcpkg (recommended)
```powershell
# Install vcpkg
git clone https://github.com/microsoft/vcpkg.git C:\vcpkg
cd C:\vcpkg
.\bootstrap-vcpkg.bat

# Install OpenSSL
.\vcpkg install openssl:x64-windows-static

# Set environment variable
[System.Environment]::SetEnvironmentVariable(
    "OPENSSL_DIR",
    "C:\vcpkg\installed\x64-windows-static",
    "User"
)

# Restart terminal
```

**Fix (Option 2):** Use WSL2
```bash
# WSL2 has OpenSSL via Nix devshell
wsl
cd ~/Rust-Template
nix develop
cargo build
```

---

### Q: Build is extremely slow on WSL2

**Symptom:** Builds take 10-30 minutes instead of 2-5 minutes

**Cause:** Repository cloned in `/mnt/c/` (Windows filesystem)

**Diagnostic:**
```bash
# Check where repo is located
pwd
# If output shows /mnt/c/..., you're on Windows filesystem (SLOW)
```

**Fix:** Move to WSL2 native filesystem
```bash
# Clone to WSL2 native filesystem
cd ~
git clone https://github.com/EffortlessMetrics/Rust-Template.git
cd Rust-Template

# Verify location
pwd
# Should show: /home/username/Rust-Template (FAST)
```

**Performance comparison:**
- Native WSL2 filesystem (`/home/`): 2-5 minute builds ✅
- Windows filesystem via WSL2 (`/mnt/c/`): 10-30 minute builds ❌

---

### Q: "out of memory" during compilation

**Symptom:**
```
error: linking with `cc` failed: signal: 9 (SIGKILL)
```

**Cause:** Insufficient RAM or swap space

**Fix (Quick):** Reduce parallel jobs
```bash
# Set environment variable
export CARGO_BUILD_JOBS=1

# Or use low-resource mode
XTASK_LOW_RESOURCES=1 cargo xtask check
```

**Fix (Permanent):** Configure Cargo
```bash
# Edit ~/.cargo/config.toml
[build]
jobs = 2  # Reduce from default (num CPUs)
```

**WSL2 specific:** Limit WSL2 memory
```powershell
# In Windows, create/edit C:\Users\YourName\.wslconfig
notepad $env:USERPROFILE\.wslconfig
```

Add:
```ini
[wsl2]
memory=4GB
processors=2
```

Restart WSL:
```powershell
wsl --shutdown
wsl
```

---

### Q: "path with spaces" errors

**Symptom:**
```
error: could not execute process
The system cannot find the path specified.
```

**Cause:** Repository cloned to path with spaces (e.g., `C:\Users\Your Name\Code\`)

**Fix:** Move to path without spaces
```powershell
# ❌ BAD: C:\Users\Your Name\Documents\Code\
# ✅ GOOD: C:\Code\

# Move repository
cd C:\
mkdir Code
cd Code
git clone https://github.com/EffortlessMetrics/Rust-Template.git
```

---

## Test Failures

### Q: Tests pass locally but fail in CI

**Symptom:** `cargo test` passes on your machine but fails in GitHub Actions

**Diagnostic:**
```bash
# Check which environment you're using
echo $NIX_PROFILES  # Should show Nix paths if in Tier 1

# Run tests exactly as CI does
nix develop -c cargo test --workspace

# Compare Rust versions
rustc --version  # Local
# Check .github/workflows/*.yml for CI Rust version
```

**Common causes:**

| Issue | Cause | Fix |
|-------|-------|-----|
| Tier mismatch | Local Tier 2, CI Tier 1 | Use `nix develop` locally |
| Rust version drift | Old local toolchain | `rustup update` or use Nix |
| Platform differences | Testing on Windows, CI uses Linux | Test in WSL2 |
| Timing issues | Tests have race conditions | Fix test isolation |

**Fix:** Match CI environment
```bash
# Use Tier 1 environment (matches CI)
nix develop
cargo test --workspace
```

---

### Q: Specific test fails with "connection refused"

**Symptom:**
```
Error: Connection refused (os error 111)
test tests::api_integration ... FAILED
```

**Cause:** Test expects service to be running, or port already in use

**Diagnostic:**
```bash
# Check if service is running
curl http://localhost:8080/health

# Check if port is in use
lsof -i :8080  # macOS/Linux
netstat -ano | findstr :8080  # Windows
```

**Fix (Option 1):** Stop conflicting service
```bash
# Find process using port
lsof -i :8080  # Note the PID
kill <PID>

# Or on Windows
netstat -ano | findstr :8080  # Note the PID
taskkill /PID <PID> /F
```

**Fix (Option 2):** Use dynamic port in tests
```rust
// In test code, use port 0 for dynamic allocation
let listener = TcpListener::bind("127.0.0.1:0")?;
let port = listener.local_addr()?.port();
```

---

### Q: BDD tests fail with "step not found"

**Symptom:**
```
Undefined step: Given the service is running
Feature: Health endpoint
  Scenario: Check health
    Given the service is running  # undefined
```

**Cause:** Step definition missing or not imported

**Diagnostic:**
```bash
# Search for step definition
grep -r "the service is running" crates/acceptance/src/

# Check if module is imported in main.rs
cat crates/acceptance/tests/acceptance.rs
```

**Fix:** Add step definition
```rust
// In crates/acceptance/src/steps/service.rs
use cucumber::given;

#[given(expr = "the service is running")]
async fn service_is_running(world: &mut MyWorld) {
    world.start_service().await;
}

// Ensure module is imported
// In crates/acceptance/tests/acceptance.rs
mod steps {
    pub mod service;  // Add this
}
```

---

### Q: Test isolation failures (tests pass individually, fail together)

**Symptom:**
```bash
$ cargo test test_a test_b  # Both fail
$ cargo test test_a  # Passes
$ cargo test test_b  # Passes
```

**Cause:** Shared state between tests (global variables, filesystem, ports)

**Diagnostic:**
```bash
# Run with single thread to see if order-dependent
cargo test -- --test-threads=1
```

**Fix:** Isolate tests
```rust
// Use unique identifiers per test
#[test]
fn test_a() {
    let temp_dir = tempfile::tempdir().unwrap();
    // Use temp_dir for test-specific files
}

// Use random ports
#[test]
fn test_b() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
}
```

---

## CI/CD Failures

### Q: CI fails with "required checks have not passed"

**Symptom:** PR cannot be merged, shows red X

**Diagnostic:**
```bash
# Check CI workflow logs in GitHub Actions tab
# Look for which check failed:
# - tier1-selftest / selftest
# - policy-test
# - ci-lints
# - ci-security
```

**Common failures:**

| Check | Failure | Fix |
|-------|---------|-----|
| **tier1-selftest** | selftest failed | Run `nix develop -c cargo xtask selftest` locally |
| **policy-test** | Rego policy violation | Run `cargo xtask policy-test` locally |
| **ci-lints** | clippy or fmt issues | Run `cargo clippy --fix` and `cargo fmt` |
| **ci-security** | Audit findings | Run `cargo audit` and address CVEs |

**Fix:** Reproduce locally
```bash
# Match CI environment
nix develop

# Run the failing check
cargo xtask selftest  # For tier1-selftest
cargo xtask policy-test  # For policy-test
cargo clippy --all-targets  # For ci-lints
cargo audit  # For ci-security
```

---

### Q: CI timeout (job exceeded maximum time)

**Symptom:**
```
Error: The operation was canceled.
The job running on runner GitHub Actions X has exceeded the maximum execution time of 360 minutes.
```

**Cause:** Infinite loop, deadlock, or missing cache

**Diagnostic:**
```bash
# Check workflow logs for where it hung
# Look for last completed step

# Check if cache was restored
# Logs should show: "Rust cache restored"
```

**Fix:** If reproducible locally
```bash
# Add timeout to specific tests
#[test]
#[timeout(Duration::from_secs(30))]
fn my_test() { ... }

# Or run with timeout
timeout 30s cargo test
```

**Fix:** If only in CI
```yaml
# In .github/workflows/*.yml
# Increase timeout or split into parallel jobs
timeout-minutes: 45  # Increase from default 30
```

---

### Q: "No space left on device" in CI

**Symptom:**
```
error: No space left on device (os error 28)
```

**Cause:** GitHub Actions runner disk full (target/ directory, caches)

**Fix (In workflow):**
```yaml
# Add cleanup step before build
- name: Free disk space
  run: |
    sudo rm -rf /usr/share/dotnet
    sudo rm -rf /opt/ghc
    df -h  # Show available space
```

**Fix (For local):**
```bash
# Clean Cargo build artifacts
cargo clean

# Clean Nix cache
nix-collect-garbage -d

# Clean Docker
docker system prune -af
```

---

### Q: CI fails but "All checks have passed" shown

**Symptom:** PR shows green checkmark but can't merge

**Cause:** Branch protection rule references check name that doesn't exist or isn't required

**Diagnostic:**
```bash
# Check required status checks
# GitHub repo → Settings → Branches → Branch protection rule for `main`
# Look at "Require status checks to pass before merging"
```

**Fix:**
1. Go to Settings → Branches → Edit rule for `main`
2. "Require status checks to pass before merging" → Search for:
   - `tier1-selftest / selftest`
   - `policy-test`
3. Ensure they're checked
4. Save

**Verify:**
```bash
# Make a test PR
# Check that required checks appear in PR status section
```

---

## Pre-commit Hook Issues

### Q: Pre-commit hook not running

**Symptom:** Commit succeeds even when code is unformatted

**Diagnostic:**
```bash
# Check if hook exists
ls -la .git/hooks/pre-commit

# Check if hook is executable
ls -l .git/hooks/pre-commit  # Should show -rwxr-xr-x

# Try manual execution
.git/hooks/pre-commit
```

**Fix:**
```bash
# Reinstall hooks
cargo xtask install-hooks

# Make executable (if needed)
chmod +x .git/hooks/pre-commit

# Test manually
.git/hooks/pre-commit
```

---

### Q: Pre-commit hook fails with "cargo: command not found"

**Symptom:**
```bash
$ git commit -m "test"
.git/hooks/pre-commit: line 3: cargo: command not found
```

**Cause:** Cargo not in PATH when Git runs hook

**Diagnostic:**
```bash
# Check cargo PATH
which cargo
echo $PATH | grep cargo
```

**Fix (Linux/macOS):**
```bash
# Ensure cargo is in PATH for all shells
# Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/.cargo/bin:$PATH"

# Reload shell
source ~/.bashrc  # or ~/.zshrc
```

**Fix (Windows):**
```powershell
# Add to PATH environment variable
# 1. System Properties → Environment Variables
# 2. User Variables → Path → Edit
# 3. Add: C:\Users\YourName\.cargo\bin
# 4. Restart terminal
```

**Verify:**
```bash
# Close and reopen terminal
which cargo  # Should show path
git commit -m "test"  # Should run hook
```

---

### Q: Pre-commit hook blocks commit but tests pass

**Symptom:**
```bash
$ cargo test
test result: ok. 42 passed; 0 failed

$ git commit -m "feat: new feature"
error: tests failed
```

**Cause:** Hook runs different checks than `cargo test`

**Diagnostic:**
```bash
# Check what the hook does
cat .git/hooks/pre-commit

# Typically runs:
# - cargo fmt --check
# - cargo clippy
# - cargo test
```

**Fix:** Run same checks manually
```bash
# Match hook checks
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test

# Or use xtask
cargo xtask check
```

---

### Q: How to skip pre-commit hook (emergency)

**Symptom:** Need to commit urgently but hook fails

**Fix (Use sparingly):**
```bash
# Skip hook for one commit
git commit --no-verify -m "fix: emergency hotfix"

# ⚠️ WARNING: Only for emergencies
# Fix the underlying issue afterward
```

**Proper workflow:**
```bash
# 1. Fix the failures
cargo xtask check

# 2. Commit normally
git commit -m "fix: proper commit"
```

---

## Windows-Specific Issues

### Q: Git hooks fail on Windows

**Symptom:**
```
error: cannot spawn .git/hooks/pre-commit: No such file or directory
```

**Cause:** Line endings converted to CRLF or Git for Windows not installed

**Diagnostic:**
```powershell
# Check line endings
file .git/hooks/pre-commit
# Should show: "POSIX shell script, ASCII text executable"

# Check for Git for Windows sh.exe
where sh.exe
# Expected: C:\Program Files\Git\usr\bin\sh.exe
```

**Fix (Line endings):**
```bash
# In Git Bash
dos2unix .git/hooks/pre-commit

# Or reinstall hooks
cargo xtask install-hooks
```

**Fix (Missing Git for Windows):**
```powershell
winget install Git.Git
# Restart terminal
```

**Prevention:**
```powershell
# Configure Git to preserve LF for shell scripts
git config --global core.autocrlf input
```

---

### Q: WSL2 Docker integration not working

**Symptom:**
```bash
$ docker ps
Cannot connect to the Docker daemon at unix:///var/run/docker.sock
```

**Cause:** Docker Desktop WSL2 integration not enabled

**Fix:**
1. Install Docker Desktop for Windows
2. Open Docker Desktop settings
3. General → Enable "Use the WSL 2 based engine"
4. Resources → WSL Integration → Enable for Ubuntu distribution
5. Click "Apply & Restart"

**Verify:**
```bash
# In WSL2
docker --version
docker ps
```

---

### Q: "Access denied" on Windows file operations

**Symptom:**
```
error: Access denied. (os error 5)
```

**Cause:** File in use by another process or insufficient permissions

**Diagnostic:**
```powershell
# Check what's using the file
tasklist | findstr "cargo rust"

# Check if antivirus is scanning
Get-Process | Where-Object {$_.Description -like "*Defender*"}
```

**Fix:**
```powershell
# Close VS Code and other IDEs
# Stop rust-analyzer

# Kill relevant processes
taskkill /F /IM rust-analyzer.exe
taskkill /F /IM cargo.exe

# Exclude from antivirus (see "Build Failures" section)
```

---

**See comprehensive Windows guide:** [docs/how-to/windows-development.md](/home/steven/code/Rust/Rust-Template/docs/how-to/windows-development.md)

---

## Policy Test Failures

### Q: "Policy test failed" but I didn't touch policies

**Symptom:**
```bash
$ cargo xtask policy-test
Ledger Policy:
  ✗ ledger_valid.json (expected pass, got fail)
FAIL - AC-TPL-001 has no tests
```

**Cause:** Spec ledger changed but tests not updated

**Diagnostic:**
```bash
# Check which AC is missing tests
cargo xtask ac-coverage | grep "❌"

# Check spec ledger
cat specs/spec_ledger.yaml | grep "AC-TPL-001"
```

**Fix:** Add tests to AC
```yaml
# In specs/spec_ledger.yaml
acceptance_criteria:
  - id: AC-TPL-001
    title: "..."
    tests:
      - type: bdd
        tag: "@AC-TPL-001"  # Add this
```

---

### Q: "conftest not found" during policy tests

**Symptom:**
```bash
$ cargo xtask policy-test
Error: conftest not available
```

**Fix (Tier 1):**
```bash
nix develop
cargo xtask policy-test
```

**Fix (Tier 2):**
```bash
cargo install cargo-binstall
cargo binstall conftest
cargo xtask policy-test
```

---

### Q: Policy test fails with "undefined variable"

**Symptom:**
```
Error: undefined variable in policy: input.ledger.stories
```

**Cause:** Policy expects structure that doesn't exist in test fixture

**Diagnostic:**
```bash
# Check policy file
cat policy/ledger.rego

# Check test fixture
cat policy/testdata/ledger_valid.json
```

**Fix:** Update test fixture to match policy expectations
```json
// In policy/testdata/ledger_valid.json
{
  "ledger": {
    "stories": [ ... ]  // Add missing structure
  }
}
```

---

## BDD/Acceptance Test Issues

### Q: BDD tests fail with "scenario undefined"

**Symptom:**
```
Feature: Platform Health
  Scenario: Health endpoint returns 200
    Given the service is running  # undefined
```

**Fix:** Implement step definition
```rust
// In crates/acceptance/src/steps/service.rs
use cucumber::given;

#[given("the service is running")]
async fn service_is_running(world: &mut MyWorld) {
    world.start_service().await;
}
```

---

### Q: "No scenarios found" for AC tag

**Symptom:**
```bash
$ cargo xtask test-ac AC-PLT-001
[WARN] No BDD scenarios found for AC-PLT-001
```

**Cause:** Feature file missing `@AC-PLT-001` tag

**Diagnostic:**
```bash
# Search for AC tag
grep -r "@AC-PLT-001" specs/features/

# If not found, need to add scenario
```

**Fix:** Add scenario with tag
```gherkin
# In specs/features/health.feature
@AC-PLT-001
Scenario: Health endpoint returns 200
  Given the service is running
  When I request GET /health
  Then I should receive a 200 status code
```

---

### Q: BDD scenario passes but AC coverage shows ❌

**Symptom:**
```bash
$ cargo xtask test-ac AC-PLT-001
[PASS] All tests passed

$ cargo xtask ac-coverage | grep AC-PLT-001
❌ AC-PLT-001: Health endpoint exists
```

**Cause:** AC in spec_ledger.yaml doesn't list the test

**Fix:** Add test metadata to AC
```yaml
# In specs/spec_ledger.yaml
acceptance_criteria:
  - id: AC-PLT-001
    title: "Health endpoint exists"
    tests:
      - type: bdd
        tag: "@AC-PLT-001"  # Add this
```

---

## xtask Command Issues

### Q: "AC not found" but it exists in spec_ledger.yaml

**Symptom:**
```bash
$ cargo xtask test-ac AC-PLT-001
Error: AC not found: AC-PLT-001
```

**Cause:** Typo in AC ID or spec_ledger.yaml not parsing correctly

**Diagnostic:**
```bash
# Check exact AC ID in spec
grep -A 5 "AC-PLT-001" specs/spec_ledger.yaml

# Try reloading spec
cargo run -p xtask -- ac-coverage
```

**Fix:** Ensure AC ID matches exactly (case-sensitive)
```yaml
# ❌ Wrong
acceptance_criteria:
  - id: ac-plt-001  # lowercase

# ✅ Correct
acceptance_criteria:
  - id: AC-PLT-001  # matches convention
```

---

### Q: "task-create" fails with "requirement not found"

**Symptom:**
```bash
$ cargo xtask task-create --id TASK-001 --req REQ-MISSING
Error: Requirement REQ-MISSING not found in spec_ledger.yaml
```

**Cause:** Requirement doesn't exist or is misspelled

**Diagnostic:**
```bash
# List all requirements
grep "^  - id: REQ-" specs/spec_ledger.yaml
```

**Fix:** Use existing requirement or create it
```yaml
# In specs/spec_ledger.yaml
requirements:
  - id: REQ-PLT-HEALTH
    title: "Platform health monitoring"
```

---

### Q: "cargo xtask" itself fails to compile

**Symptom:**
```bash
$ cargo xtask check
error: could not compile `xtask`
```

**Diagnostic:**
```bash
# Try building xtask directly
cargo build -p xtask

# Check error message for specifics
```

**Common causes:**

| Error | Cause | Fix |
|-------|-------|-----|
| Missing dependency | Cargo.toml issue | Verify dependencies |
| Type mismatch | API changed | Update xtask code |
| Macro error | proc-macro issue | Update proc-macro crates |

**Fix:** Build xtask manually
```bash
# Clean build
cargo clean -p xtask
cargo build -p xtask

# If still fails, check git status
git status
# Ensure no uncommitted breaking changes
```

---

## Docker and Container Issues

### Q: "Docker daemon not running"

**Symptom:**
```bash
$ docker ps
Cannot connect to the Docker daemon. Is the docker daemon running?
```

**Fix (Linux/macOS):**
```bash
# Start Docker service
sudo systemctl start docker  # Linux systemd
open -a Docker  # macOS

# Verify
docker ps
```

**Fix (Windows):**
```powershell
# Start Docker Desktop
# Or via WSL2:
wsl -d docker-desktop
```

---

### Q: "Port already in use" when starting service

**Symptom:**
```
Error: Address already in use (os error 48)
```

**Diagnostic:**
```bash
# Find process using port
lsof -i :8080  # macOS/Linux
netstat -ano | findstr :8080  # Windows

# Note the PID
```

**Fix:**
```bash
# Kill the process
kill <PID>  # Linux/macOS
taskkill /PID <PID> /F  # Windows

# Or use a different port
SERVICE_PORT=8081 cargo run -p app-http
```

---

### Q: Container fails with "permission denied"

**Symptom:**
```
Error: permission denied while trying to connect to Docker daemon
```

**Fix (Linux):**
```bash
# Add user to docker group
sudo usermod -aG docker $USER

# Log out and back in
# Or run:
newgrp docker

# Verify
docker ps
```

---

## Performance Issues

### Q: Builds are very slow

**Symptom:** `cargo build` takes 10+ minutes

**Diagnostic:**
```bash
# Check if on slow filesystem (WSL2 + /mnt/c)
pwd
# If shows /mnt/c/..., see "Build Failures" section

# Check disk I/O
iostat -x 1  # Linux
# Look for high %util

# Check available RAM
free -h  # Linux
Get-ComputerInfo | Select-Object CsTotalPhysicalMemory  # Windows
```

**Fix (Parallel jobs):**
```bash
# Reduce parallel jobs if out of memory
export CARGO_BUILD_JOBS=2
cargo build
```

**Fix (Cache):**
```bash
# Enable sccache
export RUSTC_WRAPPER=sccache
cargo build

# Verify cache is working
sccache -s
```

**Fix (WSL2 location):** See "Build Failures" → "Build is extremely slow on WSL2"

---

### Q: Tests take too long

**Symptom:** `cargo test` takes 20+ minutes

**Fix:** Use selective testing
```bash
# Test only changed files
cargo xtask test-changed

# Test specific AC
cargo xtask test-ac AC-PLT-001

# Skip slow integration tests
cargo test --lib
```

**See:** [docs/SELECTIVE_TESTING.md](/home/steven/code/Rust/Rust-Template/docs/SELECTIVE_TESTING.md)

---

### Q: IDE (VS Code) is slow or unresponsive

**Symptom:** Rust Analyzer takes minutes to respond

**Fix:**
```bash
# Limit rust-analyzer targets
# In .vscode/settings.json
{
  "rust-analyzer.checkOnSave.allTargets": false,
  "rust-analyzer.cargo.features": []
}

# Exclude target/ from indexing
# In .vscode/settings.json
{
  "files.watcherExclude": {
    "**/target/**": true
  }
}

# Restart rust-analyzer
# VS Code Command Palette → "Rust Analyzer: Restart Server"
```

---

## Getting More Help

### Diagnostic Commands Cheat Sheet

```bash
# Environment validation
cargo xtask doctor
nix flake check

# Run selftest (Tier 1 canonical)
nix develop -c cargo xtask selftest

# Check specific subsystems
cargo xtask check           # Fast checks
cargo xtask test-changed    # Selective tests
cargo xtask ac-coverage     # AC test coverage
cargo xtask policy-test     # Policy validation
cargo xtask bdd             # Full BDD suite

# System info
rustc --version
cargo --version
nix --version
docker --version
conftest --version

# Platform status
curl http://localhost:8080/platform/status
cargo xtask status
```

---

### When to Report an Issue

**Report these to template maintainers:**

- ✅ Tests pass on one platform, fail on another (logic error)
- ✅ Compilation errors that shouldn't occur
- ✅ Documentation errors or missing information
- ✅ xtask commands that fail unexpectedly
- ✅ Policy tests that incorrectly reject valid code

**Don't report these (expected behavior):**

- ❌ `os error 5` on Windows (file locking, see workarounds)
- ❌ Slow builds on Windows (inherent platform difference)
- ❌ CRLF issues (configure Git: `core.autocrlf = input`)
- ❌ Nix not available on native Windows (use WSL2)

**How to report:**
```markdown
Title: [category] Brief description

**Environment:**
- OS: Ubuntu 22.04 / Windows 11 / macOS 14
- Tier: Tier 1 (Nix) / Tier 2 (Native)
- Rust: 1.91.0
- Nix: 2.18.1 (if applicable)

**Steps to reproduce:**
1. Clone repository
2. Run `cargo xtask check`
3. See error: ...

**Expected:** Tests pass
**Actual:** Error: ...

**Diagnostic output:**
```
cargo xtask doctor
cargo xtask ac-coverage
```

**Workaround attempted:**
- Tried X: No change
- Tried Y: Still fails
```

---

## Related Documentation

- **[AGENT_GUIDE.md](/home/steven/code/Rust/Rust-Template/docs/AGENT_GUIDE.md)** - Complete developer workflow
- **[MISSING_MANUAL.md](/home/steven/code/Rust/Rust-Template/docs/MISSING_MANUAL.md)** - Platform support tiers and operational realities
- **[docs/how-to/windows-development.md](/home/steven/code/Rust/Rust-Template/docs/how-to/windows-development.md)** - Comprehensive Windows guide
- **[docs/SELECTIVE_TESTING.md](/home/steven/code/Rust/Rust-Template/docs/SELECTIVE_TESTING.md)** - Faster test iteration
- **[docs/dev-environment.md](/home/steven/code/Rust/Rust-Template/docs/dev-environment.md)** - Environment setup
- **[CONTRIBUTING.md](/home/steven/code/Rust/Rust-Template/CONTRIBUTING.md)** - Contribution guidelines
- **[.github/workflows/README.md](/home/steven/code/Rust/Rust-Template/.github/workflows/README.md)** - CI/CD documentation

---

**Last Updated:** 2025-12-01 (v3.3.4)
