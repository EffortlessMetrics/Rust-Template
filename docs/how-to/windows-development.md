---
id: GUIDE-TPL-WINDOWS-DEV-001
title: Windows Development Guide
doc_type: how-to
status: published
audience: developers, windows-users
tags: [windows, wsl2, tier-2, troubleshooting, setup]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-ONBOARDING, REQ-PLT-DEVEX-CONTRACT]
acs: [AC-PLT-015, AC-PLT-016, AC-PLT-019, AC-PLT-020]
adrs: [ADR-0002, ADR-0017]
last_updated: 2025-11-26
---

# Windows Development Guide

**Comprehensive guide to developing with the Rust-as-Spec Platform Cell on Windows.**

This guide covers both WSL2 (recommended) and native Windows (Tier-2) development paths, with detailed troubleshooting, workarounds, and best practices.

---

## Quick Decision: Which Path Should I Use?

```
Are you on a team?
  → Yes: Use WSL2 + Nix (Tier 1) ✅
  → No: Continue below

Do you need canonical validation (pre-commit, PRs, releases)?
  → Yes: Use WSL2 + Nix (Tier 1) ✅
  → No: Continue below

Can you install WSL2?
  → Yes: Use WSL2 + Nix (Tier 1) ✅
  → No: Use native Windows (Tier 2) ⚠️

Want fastest iteration for solo dev?
  → Yes: Native Windows (Tier 2) acceptable ⚠️
  → No: WSL2 + Nix recommended ✅
```

**Bottom Line:**
- **Team/CI/Production:** WSL2 + Nix (Tier 1) mandatory
- **Solo rapid prototyping:** Native Windows (Tier 2) acceptable with caveats

---

## Option 1: WSL2 + Nix (Recommended)

**Why this is the recommended path:**
- Full Tier 1 validation (all selftest steps pass)
- Matches CI environment exactly
- No Windows file locking issues
- Faster builds (native Linux filesystem performance)
- Team consistency (same environment across all developers)

### Prerequisites

- Windows 10 version 2004+ or Windows 11
- Admin rights (for initial WSL2 installation)
- ~5GB free disk space

### Setup Steps

#### 1. Install WSL2

```powershell
# PowerShell as Administrator
wsl --install
wsl --set-default-version 2
```

**If WSL is already installed:**
```powershell
# Check WSL version
wsl --list --verbose

# If showing WSL 1, upgrade to WSL 2
wsl --set-version Ubuntu-22.04 2
```

**Reboot if prompted.**

#### 2. Install Ubuntu Distribution

```powershell
# Install Ubuntu 22.04 (recommended)
wsl --install -d Ubuntu-22.04

# Or list available distributions
wsl --list --online
```

**First-time setup:**
- WSL will prompt for a username and password
- This is your Linux user account (independent from Windows)
- Choose a simple username (e.g., your Windows username)

#### 3. Enter WSL2 and Install Nix

```bash
# Start WSL2
wsl

# Update Ubuntu packages (optional but recommended)
sudo apt update && sudo apt upgrade -y

# Install Nix with Determinate Systems installer
curl --proto '=https' --tlsv1.2 -sSf -L \
  https://install.determinate.systems/nix | \
  sh -s -- install --determinate

# Restart shell to load Nix
exit
wsl
```

**Verify Nix installation:**
```bash
nix --version
# Expected: nix (Nix) 2.18.x or later
```

#### 4. Clone Repository (Critical: Location Matters)

**⚠️ IMPORTANT: Clone inside WSL2 native filesystem, NOT `/mnt/c/`**

```bash
# ✅ CORRECT: WSL2 native filesystem (fast)
cd ~
git clone https://github.com/EffortlessMetrics/Rust-Template.git
cd Rust-Template

# ❌ WRONG: Windows filesystem via WSL2 (slow)
cd /mnt/c/Users/YourName/Code
git clone ...  # DON'T DO THIS
```

**Why location matters:**

| Location | Performance | Why |
|----------|-------------|-----|
| `/home/username/` | ✅ Fast (native Linux I/O) | Direct ext4 filesystem access |
| `/mnt/c/Code/...` | ❌ 10-50x slower | 9P network protocol overhead |

**Impact:**
- Native filesystem: 2-5 minute builds
- Windows filesystem via `/mnt/c/`: 10-30 minute builds

#### 5. Enter Nix Devshell and Verify

```bash
# Enter devshell (downloads packages on first run, ~5-10 minutes)
nix develop

# Verify environment
cargo xtask doctor

# Run full selftest (Tier 1 canonical validation)
cargo xtask selftest
```

**Expected output:**
```
✅ [1/7] Core checks (fmt, clippy, tests)
✅ [2/7] BDD acceptance tests
✅ [3/7] AC status mapping & ADR references
✅ [4/7] LLM context bundler
✅ [5/7] Policy tests
✅ [6/7] DevEx contract
✅ [7/7] Graph invariants

🎉 All selftest phases passed!
```

#### 6. IDE Setup (VS Code with WSL Extension)

**Install VS Code on Windows:**
```powershell
# Download from https://code.visualstudio.com/
# Or via winget
winget install Microsoft.VisualStudioCode
```

**Install WSL Extension:**
1. Open VS Code
2. Install "Remote - WSL" extension (ms-vscode-remote.remote-wsl)
3. Restart VS Code

**Open project in WSL:**
```bash
# Inside WSL2
cd ~/Rust-Template
code .
```

This opens VS Code with:
- Files accessed directly from WSL2 (fast)
- Terminal runs in WSL2 (Linux shell)
- Extensions run in WSL2 context
- Rust Analyzer works natively

**Recommended VS Code Extensions (install in WSL context):**
- rust-analyzer
- Even Better TOML
- Cucumber (Gherkin) Full Support
- EditorConfig

### WSL2 Workflow

**Daily development:**
```bash
# Start WSL2
wsl

# Navigate to project
cd ~/Rust-Template

# Enter devshell
nix develop

# Start service
cargo run -p app-http

# Run tests
cargo test

# Before commit
cargo xtask selftest
```

**Hybrid workflow (Windows + WSL2):**
```powershell
# Edit files in Windows IDE (files stored in WSL2)
# VS Code with Remote-WSL handles this transparently

# Validate in WSL2 before commit
wsl -e bash -c "cd ~/Rust-Template && nix develop -c cargo xtask selftest"
```

### WSL2 Troubleshooting

#### Issue: Builds are slow

**Cause:** Repository cloned in `/mnt/c/` (Windows filesystem)

**Fix:**
```bash
# Move to WSL2 native filesystem
cd ~
git clone https://github.com/EffortlessMetrics/Rust-Template.git
cd Rust-Template
```

#### Issue: Docker not available

**Cause:** Docker Desktop WSL2 integration not enabled

**Fix:**
1. Install Docker Desktop for Windows
2. Open Docker Desktop settings
3. General → Enable "Use the WSL 2 based engine"
4. Resources → WSL Integration → Enable for Ubuntu-22.04
5. Restart Docker Desktop

**Verify:**
```bash
# Inside WSL2
docker --version
docker ps
```

#### Issue: "nix: command not found" after install

**Cause:** Shell profile not reloaded

**Fix:**
```bash
# Source Nix profile
source ~/.nix-profile/etc/profile.d/nix.sh

# Or restart WSL
exit
wsl
```

#### Issue: WSL2 uses too much memory

**Cause:** WSL2 default memory limit (50% of total RAM)

**Fix:** Create `.wslconfig` to limit memory:
```powershell
# In Windows PowerShell
notepad $env:USERPROFILE\.wslconfig
```

Add:
```ini
[wsl2]
memory=4GB
processors=4
```

Restart WSL:
```powershell
wsl --shutdown
wsl
```

#### Issue: Can't access Windows files from WSL2

**Windows files are mounted at `/mnt/c/`:**
```bash
# Access Windows C:\Users\YourName\Documents
cd /mnt/c/Users/YourName/Documents

# Copy from Windows to WSL2
cp /mnt/c/Users/YourName/file.txt ~/
```

**Note:** Editing Windows files from WSL2 is slow. Copy to WSL2 first.

---

## Option 2: Native Windows (Tier 2)

**When to use this path:**
- Cannot install WSL2 (corporate restrictions, old Windows version)
- Solo development with fast iteration needs
- Acceptable to use WSL2 for final validation before PRs

**⚠️ Known Limitations:**
- Steps 1-6 of selftest pass reliably
- Step 7 may intermittently fail with `os error 5` (file locking)
- Does NOT match CI environment (Tier 1 uses Nix)
- Requires manual tool installation
- Not recommended for team development

### Prerequisites

- Windows 10 version 2004+ or Windows 11
- Admin rights (for tool installation)
- ~3GB free disk space

### Setup Steps

#### 1. Install Rust

**Option A: rustup-init.exe (Recommended)**
```powershell
# Download from https://rustup.rs/
# Or via winget
winget install Rustlang.Rustup

# Follow installer prompts
# Choose default options when asked

# Verify installation
rustc --version
cargo --version
```

**Expected output:**
```
rustc 1.91.x or later
cargo 1.91.x or later
```

**Option B: Manual installation**
1. Download rustup-init.exe from https://rustup.rs/
2. Run as Administrator
3. Follow prompts (default options)
4. Restart PowerShell/terminal

#### 2. Install Git for Windows

```powershell
# Via winget
winget install Git.Git

# Or download from https://git-scm.com/download/win
```

**Important:** Git for Windows includes `sh.exe`, which is required for Git hooks to work.

**Verify:**
```powershell
git --version
# Expected: git version 2.x.x or later
```

#### 3. Install conftest (Policy Testing)

```powershell
# Install cargo-binstall for faster binary installations
cargo install cargo-binstall

# Install conftest
cargo binstall conftest
```

**Verify:**
```powershell
conftest --version
# Expected: Conftest: 0.x.x
```

**If conftest installation fails:**
- Download from https://github.com/open-policy-agent/conftest/releases
- Extract to `C:\Program Files\conftest\conftest.exe`
- Add to PATH: System Properties → Environment Variables → Path

#### 4. Clone Repository

```powershell
# Clone to a path WITHOUT spaces
cd C:\Code
git clone https://github.com/EffortlessMetrics/Rust-Template.git
cd Rust-Template

# ❌ AVOID: Paths with spaces (causes issues)
# cd "C:\Users\Your Name\My Documents\Code"
```

#### 5. Verify Environment

```powershell
# Check environment
cargo run -p xtask -- doctor

# Run Tier 2 validation
cargo run -p xtask -- check
cargo test --workspace --exclude acceptance
```

**Expected output from `doctor`:**
- ✅ Rust toolchain detected
- ✅ Git detected
- ⚠️ Nix not available (expected on Tier 2)
- ✅ conftest detected (if installed)

### Windows File Locking Issue

**The Problem:**

During `cargo xtask selftest`, xtask rebuilds itself. On Windows, if `xtask.exe` is locked by another process, the rebuild fails with:

```
error: failed to remove `target\debug\xtask.exe`
Access is denied. (os error 5)
```

**This is NOT a test failure.** It's a Windows platform limitation.

**Root Cause:**

Windows does not allow deleting or replacing an executable that is currently in use. Unix systems allow this (the file is unlinked but the running process continues).

**Who locks the file:**
1. **Antivirus** (most common): Windows Defender or third-party AV scans new executables immediately
2. **File Explorer**: Thumbnail generation or Windows Search indexing
3. **IDE**: Rust Analyzer, VS Code background analysis
4. **Previous cargo process**: Not fully terminated

### Workarounds for File Locking

#### Workaround 1: Exclude target/ from Antivirus (Recommended)

**Windows Defender (built-in):**
```powershell
# PowerShell as Administrator
Add-MpPreference -ExclusionPath "C:\Code\Rust-Template\target"
```

**Verify exclusion:**
```powershell
Get-MpPreference | Select-Object -ExpandProperty ExclusionPath
```

**Third-party antivirus:**
- Open antivirus settings (e.g., Norton, McAfee, Kaspersky)
- Navigate to "Exclusions" or "Exceptions"
- Add `C:\Code\Rust-Template\target\` as an excluded directory

**Why this works:**
- Antivirus no longer scans executables as they're built
- Eliminates ~90% of file locking issues
- Safe: `target/` contains only build artifacts (not source code)

**Security tradeoff:**
- Executables in `target/` are not scanned by antivirus
- Acceptable for development (you control the source code)
- Do NOT exclude production binary directories

#### Workaround 2: Use WSL2 for Canonical Validation

**When to use:**
- Before creating a PR
- Before merging to main
- For release preparation
- When selftest MUST pass cleanly

**Workflow:**
```powershell
# Daily dev: native Windows (fast iteration)
cargo run -p app-http
cargo test

# Before commit: WSL2 (canonical validation)
wsl -e bash -c "cd ~/Rust-Template && nix develop -c cargo xtask selftest"
```

**Setup WSL2 alongside native Windows:**
1. Follow [WSL2 setup steps](#option-1-wsl2-nix-recommended) above
2. Keep both environments
3. Use native Windows for speed, WSL2 for certainty

#### Workaround 3: Retry Strategy

**Quick fix for one-off validation:**
```powershell
# Close all running cargo and xtask processes
taskkill /F /IM cargo.exe
taskkill /F /IM xtask.exe

# Wait 5 seconds for processes to fully terminate
timeout /t 5

# Retry selftest
cargo run -p xtask -- selftest
```

**When to use:**
- One-off validation when antivirus exclusion isn't possible
- Corporate policy prevents excluding directories
- WSL2 not available

**Limitations:**
- Not reliable for CI/automation
- May require multiple retries
- Doesn't fix root cause

#### Workaround 4: Skip Problematic Steps

**For fast iteration (not for final validation):**
```powershell
# Run only specific checks (bypasses selftest rebuild)
cargo run -p xtask -- check
cargo test --workspace --exclude acceptance
cargo run -p xtask -- bdd
```

**When to use:**
- Daily development loop
- Quick feedback before full selftest
- Known file locking environment

**Do NOT use:**
- Before creating PR (run full selftest in WSL2)
- For canonical validation (use Tier 1)

### Windows Testing Strategy

**Use this testing ladder on native Windows:**

```powershell
# 1. Fast checks (no rebuild issues)
cargo run -p xtask -- check

# 2. Run specific tests
cargo test -p app-http

# 3. BDD scenarios
cargo run -p xtask -- bdd

# 4. Full selftest (may hit file locking on step 7)
cargo run -p xtask -- selftest

# If step 7 fails with "os error 5":
# → This is file locking, NOT a test failure
# → Use WSL2 for canonical validation
```

### Native Windows Troubleshooting

#### Issue: "os error 5" during selftest

**Cause:** File locking (see [Windows File Locking Issue](#windows-file-locking-issue))

**Fix:** See [Workarounds for File Locking](#workarounds-for-file-locking)

#### Issue: Git hooks not running

**Cause:** Line endings converted to CRLF

**Symptom:**
```
error: cannot spawn .git/hooks/pre-commit: No such file or directory
```

**Fix:**
```bash
# In Git Bash
dos2unix .git/hooks/pre-commit

# Or reinstall hooks
cargo run -p xtask -- install-hooks
```

**Prevention:**
```powershell
# Configure Git to use LF for shell scripts
git config --global core.autocrlf input
```

#### Issue: conftest not found

**Symptom:**
```
[5/7] Policy tests ⚠️ skipped (conftest not found)
```

**Fix:**
```powershell
# Install conftest
cargo install cargo-binstall
cargo binstall conftest

# Verify
conftest --version
```

#### Issue: OpenSSL linking errors

**Symptom:**
```
error: failed to run custom build command for `openssl-sys`
```

**Cause:** Windows doesn't ship OpenSSL

**Fix (Option A): Use prebuilt OpenSSL (recommended)**
```powershell
# Install vcpkg (C++ package manager)
git clone https://github.com/microsoft/vcpkg.git C:\vcpkg
cd C:\vcpkg
.\bootstrap-vcpkg.bat

# Install OpenSSL
.\vcpkg install openssl:x64-windows-static

# Set environment variable
$env:OPENSSL_DIR = "C:\vcpkg\installed\x64-windows-static"
```

**Fix (Option B): Use rustls instead**
- Some crates offer `rustls` feature flag (pure Rust TLS)
- Check `Cargo.toml` for `default-features = false, features = ["rustls"]`

**Fix (Option C): Use WSL2**
- WSL2 has OpenSSL via Nix devshell
- No Windows-specific setup required

#### Issue: Cargo build fails with "linker 'link.exe' not found"

**Cause:** Visual Studio Build Tools not installed

**Fix:**
```powershell
# Download and install Visual Studio Build Tools
# https://visualstudio.microsoft.com/downloads/
# Or via winget
winget install Microsoft.VisualStudio.2022.BuildTools

# During installation, select:
# - "Desktop development with C++"
# - Windows 10/11 SDK
```

**Alternative:** Install full Visual Studio Community (larger download)

#### Issue: Path with spaces causes issues

**Symptom:**
```
error: could not execute process `cargo build`
The system cannot find the path specified.
```

**Cause:** Repository cloned to path with spaces (e.g., `C:\Users\Your Name\Code\`)

**Fix:**
```powershell
# Move repository to path without spaces
cd C:\
mkdir Code
cd Code
git clone https://github.com/EffortlessMetrics/Rust-Template.git
cd Rust-Template
```

---

## Git Hooks on Windows

Git hooks work identically on Windows, but there are platform-specific considerations.

### How Hooks Work on Windows

**Key Insight:** Git for Windows includes a POSIX compatibility layer (`sh.exe`, `bash.exe`).

When you run `cargo xtask install-hooks`, it generates a POSIX shell script:
```bash
#!/usr/bin/env bash
cargo xtask check
```

**On Windows:**
- Git automatically delegates hook execution to `sh.exe`
- Works from PowerShell, CMD, and Git Bash
- No `.bat` or `.cmd` version needed

### Hook Installation

```powershell
# Install pre-commit hook (works on all platforms)
cargo run -p xtask -- install-hooks
```

**Verify:**
```powershell
# Check hook exists
ls .git/hooks/pre-commit

# Check shebang
gc .git/hooks/pre-commit -Head 1
# Expected: #!/usr/bin/env bash
```

### Hook Execution Context

| Shell | How Hook Runs | Expected Behavior |
|-------|---------------|-------------------|
| **PowerShell** | Git calls `sh.exe` internally | ✅ Works automatically |
| **CMD** | Git calls `sh.exe` internally | ✅ Works automatically |
| **Git Bash** | Direct POSIX execution | ✅ Works automatically |

**You don't need to do anything special.** Git for Windows handles POSIX hooks natively.

### Hook Troubleshooting

#### Issue: Hook not running at all

**Diagnosis:**
```powershell
# Try manual execution
.\.git\hooks\pre-commit
```

**If error: "command not found" or "No such file or directory":**
```powershell
# Check if Git for Windows is installed (includes sh.exe)
where sh.exe
# Expected: C:\Program Files\Git\usr\bin\sh.exe

# If not found, install Git for Windows
winget install Git.Git
```

**If hook file doesn't exist:**
```powershell
# Reinstall hooks
cargo run -p xtask -- install-hooks
```

#### Issue: Hook fails with "cargo: command not found"

**Cause:** Cargo not in PATH when Git runs the hook

**Fix:**
```powershell
# Check cargo PATH
where cargo
# Expected: C:\Users\YourName\.cargo\bin\cargo.exe

# If not found, add to PATH:
# 1. Open System Properties → Environment Variables
# 2. User Variables → Path → Edit
# 3. Add: C:\Users\YourName\.cargo\bin
# 4. Restart PowerShell/terminal
```

#### Issue: Hook fails with CRLF line ending error

**Symptom:**
```
error: cannot spawn .git/hooks/pre-commit
```

**Cause:** Git converted LF to CRLF (Windows default)

**Fix:**
```bash
# In Git Bash
dos2unix .git/hooks/pre-commit

# Or reinstall
cargo run -p xtask -- install-hooks
```

**Prevention:**
```powershell
# Configure Git to preserve LF for shell scripts
git config --global core.autocrlf input
```

#### Issue: Hook blocks commit but I need to commit anyway

**Temporary bypass:**
```powershell
# Skip hook for one commit
git commit --no-verify -m "fix: emergency hotfix"
```

**⚠️ WARNING:** Only use `--no-verify` for emergencies. Fix the underlying issue afterward.

**Proper fix:**
1. Investigate why hook is failing
2. Run `cargo xtask check` manually
3. Fix the failures
4. Commit normally (without `--no-verify`)

---

## CI Behavior on Windows

The template's CI pipeline treats Windows differently:

### CI Matrix

```yaml
# .github/workflows/ci-template-selftest.yml
matrix:
  os: [ubuntu-latest, macos-latest, windows-latest]
  include:
    - os: ubuntu-latest
      tier: "Tier 1 (Linux + Nix)"      # BLOCKING
    - os: macos-latest
      tier: "Tier 1 (macOS + Nix)"      # BLOCKING
    - os: windows-latest
      tier: "Tier 2 (Windows native)"   # INFORMATIONAL
```

### What Runs Where

**Linux/macOS CI (Tier 1):**
```yaml
- name: Run selftest
  run: nix develop -c cargo xtask selftest
```
- All 7 selftest steps must pass
- Blocks PR merge if failing
- Matches local Tier 1 environment exactly

**Windows CI (Tier 2):**
```yaml
- name: Build xtask
  run: cargo build -p xtask

- name: Run checks
  run: cargo run -p xtask -- check

- name: Run tests
  run: cargo test --workspace --exclude acceptance --exclude xtask
```
- Does NOT run full selftest (avoids file locking)
- Does NOT block PR merge
- Informational only (catches obvious Windows-specific breakage)

### Why Windows CI is Informational

1. **File locking is non-deterministic:** Same code can pass/fail based on antivirus timing
2. **No quality signal:** File locking failure doesn't indicate code problems
3. **Tier 1 is canonical:** If Tier 1 passes, the code is correct

**If Windows CI fails:**
- Check if it's file locking (`os error 5`) → ignore
- Check if it's a real test failure → investigate
- Real failures are rare (Tier 1 catches them first)

### Branch Protection Configuration

**Required checks (blocks merge):**
- ✅ `tier1-selftest / selftest` (Linux + Nix)

**Optional checks (informational):**
- ℹ️ `template-selftest / Tier 2 (Windows native)`

**How to verify:**
1. GitHub repository → Settings → Branches
2. Branch protection rule for `main`
3. "Require status checks to pass before merging"
4. Only `tier1-selftest / selftest` is required

---

## Test Matrix: What Works Where

| Test/Command | Tier 1 (WSL2/Nix) | Tier 2 (Native Windows) |
|--------------|-------------------|-------------------------|
| `cargo build` | ✅ Always works | ✅ Always works |
| `cargo test` | ✅ All tests pass | ✅ All tests pass |
| `cargo clippy` | ✅ Matches CI | ✅ May have version drift |
| `cargo fmt` | ✅ Matches CI | ⚠️ May have version drift |
| `xtask check` | ✅ Always passes | ✅ Always passes |
| `xtask bdd` | ✅ Always passes | ✅ Always passes |
| `xtask policy-test` | ✅ Always passes | ✅ If conftest installed |
| `xtask selftest` (steps 1-6) | ✅ Always passes | ✅ Always passes |
| `xtask selftest` (step 7) | ✅ Always passes | ⚠️ May fail with `os error 5` |
| Git hooks | ✅ Always work | ✅ Always work (via Git for Windows) |
| Docker integration | ✅ Via Docker Desktop | ✅ Via Docker Desktop |

**Legend:**
- ✅ Fully supported, no issues
- ⚠️ Supported with known caveats
- ℹ️ Informational, not blocking

---

## Performance Comparison

### Build Times

**Test: Clean build of workspace (`cargo build --workspace --release`)**

| Environment | Time | Notes |
|-------------|------|-------|
| WSL2 (native FS) | 2-5 min | Fastest, Tier 1 canonical |
| Native Windows (SSD) | 3-6 min | Slightly slower, Tier 2 |
| WSL2 (`/mnt/c/`) | 10-30 min | ❌ Extremely slow, DO NOT USE |
| macOS (M1/M2) | 2-4 min | Fast, Tier 1 canonical |
| Linux (native) | 2-5 min | Fast, Tier 1 canonical |

**Key Takeaway:** WSL2 is as fast as native Linux IF you clone into WSL2 native filesystem (`/home/`).

### Selftest Times

**Test: Full selftest (`cargo xtask selftest`)**

| Environment | Time | Notes |
|-------------|------|-------|
| Tier 1 (WSL2/Nix) | 5-10 min | All 7 steps, canonical |
| Tier 2 (Native Windows) | 5-12 min | May fail on step 7 (file locking) |
| Tier 1 (Linux CI) | 6-12 min | Includes Nix setup, caching helps |

---

## When to Report a Windows-Specific Issue

### Real Issues (Please Report)

Report these to the template maintainers:

- ✅ Tests that pass on Linux/macOS but fail on Windows (logic error)
- ✅ Windows-specific compilation errors (missing dependencies)
- ✅ Path handling bugs (backslash vs forward slash)
- ✅ Docs that are incorrect or incomplete for Windows
- ✅ Hook installation that fails on Windows (not CRLF issues)

### Not Issues (Don't Report)

These are expected Windows behavior, not bugs:

- ❌ `os error 5` during selftest step 7 (file locking)
- ❌ Slower builds on Windows (inherent platform difference)
- ❌ Antivirus scanning binaries (configure exclusions)
- ❌ Nix not available on native Windows (use WSL2)
- ❌ CRLF line ending issues (configure Git: `core.autocrlf = input`)

### How to Report

**Use GitHub Issues with `[windows]` prefix:**

```markdown
Title: [windows] cargo test fails with "path not found" error

**Environment:**
- OS: Windows 11 Pro 23H2
- Rust: 1.91.0
- Tier: Native Windows (Tier 2)
- Antivirus: Windows Defender

**Steps to reproduce:**
1. Clone repository to C:\Code\Rust-Template
2. Run `cargo test -p app-http`
3. See error: ...

**Expected behavior:**
Tests should pass (they pass on WSL2/Linux)

**Actual behavior:**
Error: path not found ...

**Workaround attempted:**
- Tried WSL2: Tests pass there
- Tried excluding from antivirus: No change
```

**Include:**
1. Windows version (`winver` command)
2. Rust version (`rustc --version`)
3. Tier (native Windows or WSL2)
4. Full error message
5. Whether issue reproduces in WSL2 (if applicable)

---

## Recommended Workflows

### Solo Developer (Tier 2 acceptable)

**Daily iteration:**
```powershell
# Fast feedback loop
cargo run -p app-http
cargo test
cargo run -p xtask -- check
```

**Before commit:**
```powershell
# Option A: Native Windows (may hit file locking)
cargo run -p xtask -- selftest

# Option B: WSL2 (canonical, always passes)
wsl -e bash -c "cd ~/Rust-Template && nix develop -c cargo xtask selftest"
```

**Recommended:** Set up WSL2 alongside native Windows. Use native Windows for speed, WSL2 for certainty.

### Team Development (Tier 1 mandatory)

**Setup:**
- All team members use WSL2 + Nix
- Ensures consistency across team
- Matches CI exactly

**Workflow:**
```bash
# All work in WSL2
wsl
cd ~/Rust-Template
nix develop

# Daily dev
cargo run -p app-http
cargo test

# Before push
cargo xtask selftest
git push
```

**Result:**
- "Works on my machine" eliminated
- PRs rarely fail CI (local matches CI)
- No platform-specific debugging

### Hybrid (Native + WSL2)

**Setup:**
- Use native Windows for fast iteration
- Use WSL2 for canonical validation

**Workflow:**
```powershell
# Daily dev (native Windows)
cd C:\Code\Rust-Template
cargo run -p app-http
cargo test
cargo run -p xtask -- check

# Before PR (WSL2)
wsl
cd ~/Rust-Template
nix develop
cargo xtask selftest
exit

# If green, push from Windows
git push
```

**Benefits:**
- Fast iteration (native Windows)
- Canonical validation (WSL2)
- Best of both worlds

---

## Summary

### Quick Reference Table

| Aspect | WSL2 + Nix (Tier 1) | Native Windows (Tier 2) |
|--------|---------------------|-------------------------|
| **Setup time** | 10 min | 15 min |
| **Selftest guarantee** | All 7 steps ✅ | Steps 1-6 ✅, step 7 ⚠️ |
| **CI match** | Exact | Partial |
| **Build speed** | Fast (native FS) | Moderate |
| **File locking** | Never | Occasionally |
| **Nix available** | ✅ | ❌ |
| **Team recommended** | ✅ Yes | ❌ No |
| **Solo acceptable** | ✅ Yes | ⚠️ With caveats |
| **Git hooks** | ✅ | ✅ |
| **Docker** | ✅ | ✅ |

### Decision Matrix

**Use WSL2 + Nix (Tier 1) if:**
- ✅ You're on a team
- ✅ You need canonical validation
- ✅ You're contributing to the template
- ✅ You want to match CI exactly
- ✅ You can install WSL2

**Use native Windows (Tier 2) if:**
- ⚠️ You're doing solo rapid prototyping
- ⚠️ You can't install WSL2 (corporate restrictions)
- ⚠️ You're willing to use WSL2 for final validation
- ⚠️ You accept file locking risk

### Golden Rules

1. **For teams:** Always use Tier 1 (WSL2 + Nix)
2. **For CI/CD:** Only Tier 1 is canonical
3. **For solo dev:** Tier 2 acceptable for daily work; validate in WSL2 before PR
4. **For production releases:** Tier 1 mandatory

---

## Further Reading

- **[Platform Support Reference](../reference/platform-support.md)** - Complete platform support matrix
- **[MISSING_MANUAL.md](../MISSING_MANUAL.md)** - Operational realities, including Windows specifics
- **[ADR-0017: Tier-1 Selftest Gate](../adr/0017-tier1-selftest-gate.md)** - Why Tier 1 is canonical
- **[Development Environment Setup](../dev-environment.md)** - General setup guide
- **[CI Coverage Reference](../reference/ci-coverage.md)** - What CI validates

---

**Last Updated:** 2025-11-30 (v3.3.4)
