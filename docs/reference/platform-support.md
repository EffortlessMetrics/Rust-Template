---
id: GUIDE-TPL-PLATFORM-SUPPORT-001
title: Platform Support Reference
doc_type: reference
status: published
audience: developers, platform-teams
tags: [platform, windows, linux, macos, wsl2, nix, troubleshooting]
stories: [US-TPL-PLT-001]
requirements:
  - REQ-PLT-DEVEX-CONTRACT
  - REQ-PLT-ONBOARDING
acs: [AC-PLT-015, AC-PLT-016, AC-PLT-019, AC-PLT-020]
adrs: [ADR-0002, ADR-0005, ADR-0017]
last_updated: 2025-11-25
---
<!-- doclint:disable orphan-version -->

# Platform Support Reference

**Complete guide to platform support, validation guarantees, and troubleshooting.**

This document consolidates all platform-specific guidance for developing with the Rust-as-Spec Platform Cell across different operating systems and environments.

---

## Quick Reference

### Platform Selector Table

| Platform | Tier | Selftest Guarantee | Setup Time | CI Match | Recommended For |
|----------|------|-------------------|------------|----------|-----------------|
| **Linux + Nix** | Tier 1 | All 7 steps always pass | 5 min | ✅ Exact | Teams, CI |
| **macOS + Nix** | Tier 1 | All 7 steps always pass | 5 min | ✅ Exact | Teams, CI |
| **WSL2 + Nix** | Tier 1 | All 7 steps always pass | 10 min | ✅ Exact | Windows teams |
| **Windows Native** | Tier 2 | Steps 1-6 pass; 7-8 may fail with "os error 5" | 15 min | ⚠️ Partial | Solo Windows dev |

**Bottom Line:** Use Tier 1 (Nix on any OS or WSL2) for canonical validation. Use Tier 2 (native Windows) for fast iteration with known caveats.

---

## Platform Support Overview

The template supports two validation tiers with different guarantees:

### Tier 1: Nix Devshell (Recommended)

**Platforms:**
- Linux (Ubuntu, Debian, Fedora, NixOS, etc.)
- macOS (Intel and Apple Silicon)
- WSL2 on Windows 10/11

**Validation Guarantee:**

All 7 selftest steps **always pass**:

```
[1/7] Core checks (fmt, clippy, tests)      ✅
[2/7] BDD acceptance tests                  ✅
[3/7] AC status mapping & ADR references    ✅
[4/7] LLM context bundler                   ✅
[5/7] Policy tests                          ✅
[6/7] DevEx contract                        ✅
[7/7] Graph invariants                      ✅
```

**Why this is canonical:**
- Environment exactly matches CI (deterministic builds)
- No local toolchain drift
- Reproducible across team members
- Hermetic isolation (no system dependency conflicts)

**When to use:**
- Team development (consistency across developers)
- Pre-commit validation
- Production CI/CD pipelines
- When contributing to the template itself

### Tier 2: Native Toolchain

**Platforms:**
- Windows 10/11 with native Rust toolchain

**Validation Guarantee:**

Steps 1-6 **always pass**; steps 7-8 may intermittently fail with `os error 5`:

```
[1/7] Core checks (fmt, clippy, tests)      ✅
[2/7] BDD acceptance tests                  ✅
[3/7] AC status mapping & ADR references    ✅
[4/7] LLM context bundler                   ✅
[5/7] Policy tests                          ✅
[6/7] DevEx contract                        ✅
[7/7] Graph invariants                      ⚠️ May fail with file locking
```

**Why file locking happens:**

Windows has stricter file locking than Unix. During `cargo rebuild`, the executable (`xtask.exe`) may be locked by:
- Antivirus real-time scanning (most common)
- File explorer thumbnail generation
- IDE background analysis (Rust Analyzer, VS Code)
- Previous cargo process not fully terminated

**This is not a behavioral failure**-it's a platform limitation in how Windows handles in-use executables.

**Fmt gate policy:** `cargo fmt --all -- --check` is enforced only on Tier 1 (Nix devshell/CI). Tier 2 runs and any session with `XTASK_LOW_RESOURCES=1` skip the fmt gate and print a warning to avoid rustfmt recursion; use a Tier 1 shell for final formatting.

**Policy tests:** `cargo xtask policy-test` (conftest/OPA) runs in Tier 1 (`tier1-selftest` in CI). Tier 2/Windows may skip policy tests locally; use WSL/Nix for canonical policy validation before merging.

**When to use:**
- Solo Windows development
- Fast iteration (daily dev work)
- When WSL2 is not available
- Non-critical changes

**Not recommended for:**
- Final validation before PR
- Team CI/CD
- Release preparation

---

## Tier 1: Nix Devshell

### Linux with Nix

**Supported Distributions:**
- Ubuntu 20.04+ (LTS recommended)
- Debian 11+
- Fedora 36+
- Arch Linux
- NixOS
- Any Linux with systemd and Nix package manager

**Setup:**

```bash
# 1. Install Nix (one-time)
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | \
  sh -s -- install --determinate

# 2. Clone repository
git clone https://github.com/EffortlessMetrics/Rust-Template.git
cd Rust-Template

# 3. Enter devshell
nix develop

# 4. Verify
cargo xtask doctor
cargo xtask selftest
```

**What Nix provides:**
- Rust 1.91+ (pinned version)
- `conftest` (OPA policy testing)
- `cargo-binstall` (fast binary installation)
- All required build tools (gcc, pkg-config, etc.)

**Expected behavior:**
- `cargo xtask selftest` always passes
- Matches CI environment exactly
- No "works on my machine" issues

**Troubleshooting:**

| Issue | Solution |
|-------|----------|
| `nix: command not found` | Restart shell or source `~/.nix-profile/etc/profile.d/nix.sh` |
| Nix daemon not running | `sudo systemctl start nix-daemon` (systemd) |
| Permission denied on `/nix` | Run Nix installer with sudo or check `/nix` ownership |

### macOS with Nix

**Supported Versions:**
- macOS 12 (Monterey) or later
- Both Intel (x86_64) and Apple Silicon (aarch64) supported

**Setup:**

```bash
# 1. Install Nix (one-time)
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | \
  sh -s -- install --determinate

# 2. Clone repository
git clone https://github.com/EffortlessMetrics/Rust-Template.git
cd Rust-Template

# 3. Enter devshell
nix develop

# 4. Verify
cargo xtask doctor
cargo xtask selftest
```

**macOS-Specific Notes:**

- **XCode Command Line Tools**: May be required. Install with:

  ```bash
  xcode-select --install
  ```

- **Apple Silicon**: Nix automatically selects `aarch64-darwin` binaries. Everything works natively.

- **Rosetta**: Not needed. All Nix packages are native ARM64.

**Expected behavior:**
- Identical to Linux
- All selftest steps pass
- Matches CI environment exactly

**Troubleshooting:**

| Issue | Solution |
|-------|----------|
| "xcrun: error" | Install XCode CLI tools: `xcode-select --install` |
| Nix installer hangs | Check firewall/VPN settings; try again without VPN |
| `direnv` integration issues | Run `direnv allow .` after entering directory |

### WSL2 on Windows

**Why WSL2 for Windows teams:**
- Full Tier 1 validation guarantee
- No file locking issues
- Matches CI exactly
- Native Unix I/O performance (faster than native Windows builds)

**Prerequisites:**
- Windows 10 version 2004+ or Windows 11
- WSL2 enabled (not WSL1)

**Setup:**

```powershell
# 1. Install WSL2 (PowerShell as Admin)
wsl --install
wsl --set-default-version 2

# 2. Install Ubuntu (recommended)
wsl --install -d Ubuntu-22.04

# 3. Inside WSL2 shell
wsl

# 4. Install Nix
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | \
  sh -s -- install --determinate

# 5. Clone repository (inside WSL2, not /mnt/c/)
cd ~
git clone https://github.com/EffortlessMetrics/Rust-Template.git
cd Rust-Template

# 6. Enter devshell
nix develop

# 7. Verify
cargo xtask doctor
cargo xtask selftest
```

**Critical: Clone location matters**

| Location | Performance | Recommendation |
|----------|-------------|----------------|
| `/home/username/` | ✅ Fast (native Linux FS) | **Use this** |
| `/mnt/c/Code/...` | ❌ Slow (Windows FS via 9P) | Avoid |

**Why:** WSL2 file access to `/mnt/c/` goes through a network protocol layer. Native WSL2 filesystem (`/home/`) is 10-50x faster.

**Expected behavior:**
- Identical to native Linux
- All selftest steps pass
- No file locking issues
- Matches CI exactly

**Troubleshooting:**

| Issue | Solution |
|-------|----------|
| WSL2 not installed | Run `wsl --install` in PowerShell as Admin |
| Still using WSL1 | `wsl --set-version Ubuntu-22.04 2` |
| Slow builds from `/mnt/c/` | Move repo to `/home/username/` (WSL2 native FS) |
| Docker not available | Install Docker Desktop with WSL2 backend enabled |

---

## Tier 2: Native Windows

**Supported:**
- Windows 10 (version 2004+)
- Windows 11

**Not recommended, but supported with caveats.**

### Setup (Manual)

```powershell
# 1. Install Rust
# Download from https://rustup.rs/
rustup-init.exe
rustup install 1.91
rustup default 1.91

# 2. Install conftest
cargo install cargo-binstall
cargo binstall conftest

# 3. Install Git for Windows (if not already)
# Download from https://git-scm.com/download/win

# 4. Clone repository
git clone https://github.com/EffortlessMetrics/Rust-Template.git
cd Rust-Template

# 5. Verify
cargo xtask doctor
```

**Expected behavior:**
- Steps 1-6 of selftest pass reliably
- Step 7-8 may fail intermittently with:

  ```
  error: failed to remove `target\debug\xtask.exe`: Access is denied. (os error 5)
  ```

**This is normal on native Windows.** It's not a test failure—it's Windows file locking.

### File Locking on Windows

**The Problem:**

During `cargo xtask selftest`, xtask rebuilds itself. On Windows, if `xtask.exe` is locked by another process, the rebuild fails with `os error 5`.

**Root Cause:**

Windows does not allow deleting/replacing an in-use executable. Unix systems allow this (file is unlinked but process continues).

**Who locks the file:**
1. **Antivirus** (most common): Windows Defender or third-party AV scans new executables immediately
2. **File Explorer**: Thumbnails or indexing
3. **IDE**: Rust Analyzer, VS Code analysis
4. **Previous cargo process**: Not fully terminated

### Windows Workarounds

#### Option 1: Exclude `target/` from Antivirus (Recommended)

**Windows Defender:**

```powershell
# PowerShell as Administrator
Add-MpPreference -ExclusionPath "C:\Code\Rust\Rust-Template\target"
```

**Third-party antivirus:**
- Open antivirus settings
- Navigate to "Exclusions" or "Exceptions"
- Add `C:\Code\Rust\Rust-Template\target\` as an excluded path

**Why this works:**
- Antivirus no longer scans binaries as they're built
- Eliminates most file locking issues
- Safe: `target/` contains only build artifacts (not source code)

**Tradeoff:**
- Slightly reduces security (executables in `target/` not scanned)
- Acceptable for development (not production binaries)

#### Option 2: Use WSL2 for Canonical Validation

**When to use:**
- Before creating a PR
- Before merging to main
- For release preparation
- When selftest must pass cleanly

**Workflow:**

```powershell
# Daily dev: native Windows (fast iteration)
cargo run -p app-http

# Before commit: WSL2 (canonical validation)
wsl
cd ~/Rust-Template
nix develop
cargo xtask selftest
```

**Why this works:**
- WSL2 is Tier 1 (no file locking)
- Gives you canonical validation when needed
- Lets you iterate fast on Windows day-to-day

#### Option 3: Retry Strategy

**Quick fix:**

```powershell
# Close all running cargo/xtask processes
taskkill /F /IM cargo.exe
taskkill /F /IM xtask.exe

# Wait 5 seconds
timeout /t 5

# Retry
cargo xtask selftest
```

**When to use:**
- One-off validation
- When you can't exclude from antivirus (corporate policy)
- When WSL2 not available

**Limitation:**
- Not reliable for CI/automation
- May require multiple retries

### When to Do What

| Scenario | Recommended Path |
|----------|------------------|
| **First setup (team)** | WSL2 + Nix (Tier 1) |
| **Daily dev on Windows** | Native Windows + antivirus exclusion (Tier 2) |
| **Before PR** | WSL2 final validation (Tier 1) |
| **CI/CD** | GitHub Actions with Nix (Tier 1) |
| **Solo prototype** | Native Windows (acceptable for fast iteration) |
| **Production release** | Linux + Nix (Tier 1) |

---

## Git Hooks (Cross-Platform)

### Unified POSIX Hooks (v3.3.1)

Git hooks are **POSIX shell scripts** on all platforms, including Windows.

**How this works:**
- `cargo xtask install-hooks` generates a `#!/usr/bin/env bash` hook
- On Windows, **Git for Windows runs hooks via its bundled `sh.exe`**
- No need for `.bat` or `.cmd` files

**Installation:**

```bash
# Works on Linux, macOS, Windows
cargo xtask install-hooks
```

**Expected behavior:**

```bash
# Git automatically runs the hook on commit
git commit -m "feat: add feature"
# → Hook runs: cargo xtask check
# → If check passes, commit proceeds
# → If check fails, commit is blocked
```

**Platform-specific execution:**

| Platform | Hook Runner | Expected Behavior |
|----------|-------------|-------------------|
| Linux | `/bin/bash` | ✅ Direct execution |
| macOS | `/bin/bash` | ✅ Direct execution |
| Git Bash (Windows) | `sh.exe` | ✅ Via Git for Windows sh |
| PowerShell (Windows) | Git calls `sh.exe` | ✅ Git internally uses sh |
| CMD (Windows) | Git calls `sh.exe` | ✅ Git internally uses sh |

**Why this works universally:**
- Git for Windows includes a POSIX compatibility layer (`sh.exe`, `bash.exe`)
- Git internally delegates hook execution to `sh.exe` even when called from CMD/PowerShell
- No platform-specific hooks needed

### Troubleshooting Git Hooks

**Issue: Hook not executing**

```bash
# Verify hook exists
ls .git/hooks/pre-commit

# Verify shebang
head -1 .git/hooks/pre-commit
# Should show: #!/usr/bin/env bash
```

**Fix:**

```bash
# Reinstall
cargo xtask install-hooks
```

**Issue: "cannot spawn .git/hooks/pre-commit" on Windows**

**Cause:** Line endings converted to CRLF (Windows Git auto-conversion).

**Fix:**

```bash
# In Git Bash or PowerShell
dos2unix .git/hooks/pre-commit

# Or reinstall
cargo xtask install-hooks
```

**Issue: Hook fails with "cargo: command not found" on Windows**

**Cause:** Cargo not in PATH when Git runs the hook.

**Fix:**

```bash
# Verify cargo in PATH
where cargo

# If not found, add to PATH:
# Settings → System → Environment Variables → User Variables → Path
# Add: C:\Users\<username>\.cargo\bin
```

**Workaround: Skip hook temporarily**

```bash
# If hook is broken, bypass it once
git commit --no-verify -m "fix: message"

# Then fix the hook
cargo xtask install-hooks
```

---

## Platform Comparison Matrix

### Feature Comparison

| Feature | Linux+Nix | macOS+Nix | WSL2+Nix | Windows Native |
|---------|-----------|-----------|----------|----------------|
| **Selftest guarantee** | All 7 steps ✅ | All 7 steps ✅ | All 7 steps ✅ | Steps 1-6 ✅, 7-8 ⚠️ |
| **CI match** | Exact | Exact | Exact | Partial |
| **Setup time** | 5 min | 5 min | 10 min | 15 min |
| **Build speed** | Fast | Fast | Fast | Moderate |
| **File locking issues** | Never | Never | Never | Occasionally |
| **Nix devshell** | ✅ | ✅ | ✅ | ❌ |
| **Hot reload** | ✅ | ✅ | ✅ | ✅ |
| **BDD tests** | ✅ | ✅ | ✅ | ✅ |
| **Policy tests** | ✅ | ✅ | ✅ | ✅ (conftest manual install) |
| **Git hooks** | ✅ | ✅ | ✅ | ✅ (via Git for Windows) |
| **Docker integration** | Native | Via VM | Native (Docker Desktop) | Via VM |
| **Recommended for teams** | ✅ | ✅ | ✅ | ❌ |
| **Recommended for solo dev** | ✅ | ✅ | ✅ | ⚠️ Acceptable |

### Decision Tree

```
Start
  ├─ Team environment?
  │   └─ Yes → Use Tier 1 (Nix on any OS or WSL2) ✅
  │
  ├─ Windows-only constraint?
  │   ├─ Can use WSL2? → Yes → WSL2 + Nix ✅
  │   └─ Can't use WSL2 → Native Windows (antivirus excluded) ⚠️
  │
  ├─ CI/CD?
  │   └─ Always Tier 1 (Linux + Nix) ✅
  │
  └─ Fast iteration, solo dev?
      ├─ Linux/macOS → Nix ✅
      └─ Windows → Native (acceptable for iteration) ⚠️
```

**Golden Rule:**
- **For canonical validation:** Always use Tier 1
- **For fast iteration:** Tier 2 acceptable (Windows only)
- **For teams/CI:** Only Tier 1

---

## Special Environments

### Low-Resource Mode

**When to use:**
- CI runners with < 4GB RAM
- Local dev on constrained hardware (old laptops, VPS)
- If you see "OOM" (out of memory) or "Killed" messages during builds

**How to enable:**

```bash
# Set environment variable
export XTASK_LOW_RESOURCES=1

# Run commands as normal
cargo xtask check
cargo xtask selftest
```

**What it does:**
- Sets `CARGO_BUILD_JOBS=1` (serial compilation, not parallel)
- Disables `sccache` (avoids cache overhead)
- Reduces memory usage by ~60%

**Tradeoff:**
- **Slower builds** (serial vs parallel)
- **Lower memory usage** (fits in 2GB)

**When NOT to use:**
- Modern development machines (>= 8GB RAM)
- When build speed is critical

### CI Behavior

**GitHub Actions:**

```yaml
# Tier 1: Linux + Nix (canonical)
- name: Selftest
  run: |
    nix develop -c cargo xtask selftest

# Tier 1: macOS + Nix (canonical)
- name: Selftest
  run: |
    nix develop -c cargo xtask selftest

# Tier 2: Windows Native (known caveats)
# Excluded from required checks due to file locking
```

**Key Points:**
- CI always uses Tier 1 (Nix) for Linux and macOS
- Windows CI validation is **informational** (not blocking)
- If Tier 1 CI passes, release is valid

---

## Troubleshooting by Platform

### Linux Issues

**Issue: Nix installer fails**

```bash
# Cause: SELinux blocking /nix creation
# Fix: Temporarily disable SELinux
sudo setenforce 0
curl ... | sh -s -- install --determinate
sudo setenforce 1
```

**Issue: "error: linking with `cc` failed"**

```bash
# Cause: Missing build tools
# Fix: Enter Nix shell (has all tools)
nix develop

# Or install manually (not recommended)
sudo apt install build-essential pkg-config libssl-dev
```

**Issue: Permission denied on `/nix`**

```bash
# Cause: Nix directory ownership wrong
# Fix: Reinstall Nix or fix ownership
sudo chown -R $(whoami) /nix
```

### macOS Issues

**Issue: XCode Command Line Tools required**

```bash
# Fix: Install tools
xcode-select --install
```

**Issue: "library not found for -liconv"**

```bash
# Cause: Missing libiconv (rare on modern macOS)
# Fix: Enter Nix shell (provides libiconv)
nix develop
```

**Issue: Nix installer hangs**

```bash
# Cause: VPN or firewall blocking downloads
# Fix: Disable VPN temporarily, retry install
```

### WSL2 Issues

**Issue: Builds very slow**

```bash
# Cause: Repository in /mnt/c/ (Windows filesystem)
# Fix: Move to WSL2 native filesystem
cd ~
git clone ...
```

**Issue: Docker not available in WSL2**

```bash
# Cause: Docker Desktop WSL2 backend not enabled
# Fix:
# 1. Open Docker Desktop settings
# 2. General → "Use WSL 2 based engine" ✅
# 3. Resources → WSL Integration → Enable for your distro
```

**Issue: "nix: command not found" after install**

```bash
# Cause: Shell profile not reloaded
# Fix: Restart shell or source profile
source ~/.nix-profile/etc/profile.d/nix.sh
```

### Windows Native Issues

**Issue: "os error 5" during selftest**

**Cause:** File locking (see [Windows Workarounds](#windows-workarounds))

**Fix (best):**

```powershell
Add-MpPreference -ExclusionPath "C:\Code\Rust\Rust-Template\target"
```

**Fix (alternative):**

```bash
# Use WSL2 for final validation
wsl
cd ~/Rust-Template
nix develop
cargo xtask selftest
```

**Issue: Git hooks not running**

**Cause:** Line endings (CRLF vs LF)

**Fix:**

```bash
# In Git Bash
dos2unix .git/hooks/pre-commit

# Or reinstall
cargo xtask install-hooks
```

**Issue: conftest not found**

```powershell
# Fix: Install conftest
cargo install cargo-binstall
cargo binstall conftest
```

**Issue: "failed to run custom build command for openssl-sys"**

**Cause:** Missing OpenSSL (Windows doesn't ship it)

**Fix (manual, not recommended):**

```powershell
# Download and install OpenSSL from:
# https://slproweb.com/products/Win32OpenSSL.html

# Or: Use WSL2 (has OpenSSL via Nix)
```

---

## Migration Paths

### Native Windows → WSL2

**When to migrate:**
- Hitting repeated file locking errors
- Team standardizing on Tier 1
- Need canonical CI match

**Migration steps:**

```powershell
# 1. Install WSL2 (PowerShell as Admin)
wsl --install
wsl --set-default-version 2

# 2. Wait for reboot (if prompted)

# 3. Start WSL2
wsl

# 4. Inside WSL2 shell (now Linux)
# Install Nix
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | \
  sh -s -- install --determinate

# 5. Clone repo (into WSL2, not /mnt/c/)
cd ~
git clone https://github.com/EffortlessMetrics/Rust-Template.git
cd Rust-Template

# 6. Enter devshell
nix develop

# 7. Verify
cargo xtask doctor
cargo xtask selftest
```

**What you gain:**
- Tier 1 validation (all selftest steps pass)
- No file locking issues
- Faster builds (native Linux FS)
- Matches CI exactly

**What you lose:**
- Must use WSL2 shell (not native PowerShell/CMD)
- IDE integration requires WSL2 extension (VS Code, IntelliJ support this)

**Recommended workflow after migration:**

```bash
# Option 1: Work entirely in WSL2
wsl
cd ~/Rust-Template
code .   # Opens VS Code with WSL2 extension

# Option 2: Hybrid (edit on Windows, validate in WSL2)
# Edit in native Windows IDE
# Before commit:
wsl -e bash -c "cd ~/Rust-Template && nix develop -c cargo xtask selftest"
```

### Manual Setup → Nix

**When to migrate:**
- Toolchain drift from team/CI
- Manual dependency management burden
- Want reproducible environment

**Migration steps (any platform):**

```bash
# 1. Install Nix
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | \
  sh -s -- install --determinate

# 2. In your existing repo
cd /path/to/Rust-Template

# 3. Enter devshell
nix develop

# 4. Verify (Nix now provides all tools)
cargo xtask doctor
cargo xtask selftest
```

**What you gain:**
- Declarative environment (defined in `flake.nix`)
- Matches CI exactly
- No manual tool installation
- Version pinning (no surprise updates)

**What you lose:**
- Disk space (~500MB for Nix store)
- Slightly slower first build (Nix downloads packages)

### Tier 2 → Tier 1 (Windows Team Migration)

**Recommended path:**

```
Current: Team using native Windows (Tier 2)
  ↓
Step 1: Pilot WSL2 with 1-2 developers
  ↓
Step 2: Validate workflow (IDE, hooks, Docker)
  ↓
Step 3: Team migration (1-2 weeks)
  ↓
Result: Full team on Tier 1 (WSL2 + Nix)
```

**Pilot checklist:**
- [ ] WSL2 installed and working
- [ ] Nix devshell functional
- [ ] VS Code WSL extension setup
- [ ] Git hooks work
- [ ] Docker available
- [ ] All selftest steps pass
- [ ] Daily workflow comfortable

**Team rollout:**
- Week 1: Pilot + document issues
- Week 2: Team training session
- Week 3: Pair programming for stragglers
- Week 4: Deprecate Tier 2 instructions

**Rollback plan:**
- Keep native Windows docs for 1 month
- If blocker found, team stays on Tier 2 temporarily
- Fix blocker, retry migration

---

## Summary

### Platform Tiers

| Tier | Guarantee | Use For |
|------|-----------|---------|
| **Tier 1** (Nix) | All selftest steps always pass | Teams, CI, production, canonical validation |
| **Tier 2** (Windows Native) | Steps 1-6 pass; 7-8 may fail with file locking | Solo dev, fast iteration (non-critical) |

### Golden Rules

1. **For teams:** Always use Tier 1 (Nix on any OS or WSL2)
2. **For CI/CD:** Only Tier 1 (Linux + Nix)
3. **For solo Windows dev:** Tier 2 acceptable for daily work; validate in WSL2 before PR
4. **For canonical validation:** Tier 1 is ground truth

### Quick Decision

**"What should I use?"**

```
Do you need canonical validation (team, CI, release)?
  → Yes: Tier 1 (Nix or WSL2) ✅
  → No: Tier 2 acceptable (Windows only) ⚠️

Are you on Windows?
  → Can use WSL2: WSL2 + Nix ✅
  → Can't use WSL2: Native Windows + antivirus exclusion ⚠️

Are you on Linux/macOS?
  → Always use Nix ✅
```

### Further Reading

- **Setup Guides:**
  - [Setup Without Nix](../how-to/setup-without-nix.md) (deprecated, Tier 2)
  - [Development Environment](../how-to/dev-environment.md) (general guidance)

- **Troubleshooting:**
  - [MISSING_MANUAL.md](../MISSING_MANUAL.md) (operational realities)
  - [AGENT_GUIDE.md](../AGENT_GUIDE.md) (platform-specific agent notes)

- **Reference:**
  - [xtask Commands](xtask-commands.md) (all CLI commands)
  - [CI Coverage](ci-coverage.md) (what CI validates)

---

**Last Updated:** 2025-11-25 (v3.3.1)
