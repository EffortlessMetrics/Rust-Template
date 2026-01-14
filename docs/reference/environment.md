---
id: REF-TPL-ENVIRONMENT-001
title: Environment Setup & Platform Support
doc_type: reference
status: published
audience: developers, operators
tags: [environment, setup, nix, windows, macos, linux, platform-support]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-DEVEX-CONTRACT, REQ-PLT-NIX-DEVSHELL]
acs: [AC-PLT-008, AC-PLT-012]
adrs: [ADR-0002, ADR-0003]
last_updated: 2025-11-27
---
<!-- doclint:disable orphan-version -->

# Environment Setup & Platform Support

This document specifies the **canonical development environment** and supported tiers for running this template.

---

## Philosophy

This template is **Nix-first**, not Nix-only:

- **Tier-1 (Canonical):** Linux, macOS, WSL2 with Nix devshell. Same environment as CI. Used for release validation, policy tests, and full `selftest`.
- **Tier-2 (Supported):** Native Windows, macOS/Linux without Nix. Core features work. Some tools (formatter, policy tests, clippy) may be skipped. Used for local iteration only.

**Why Nix?**
- Reproducible: Developers, CI, and production use identical toolchains.
- Declarative: No manual tool installation; one command (`nix develop`) handles everything.
- Lockable: Dependency versions are pinned in `flake.lock`, not subject to system mutation.

---

## Tier-1: Canonical Development (Linux, macOS, WSL2)

### Prerequisites

1. **Operating System:** Linux (any distro), macOS (Intel or Apple Silicon), or WSL2 on Windows
2. **Nix:** [Install Nix with flakes](https://nixos.org/download.html)
   - Recommended: [Determinate Systems installer](https://install.determinate.systems/)
   - Includes flakes enabled by default
3. **Git:** For cloning and working with the repo
4. **VS Code (optional):** Pre-configured with tasks, debug configs, and extensions

### Setup

```bash
# 1. Clone the repo
git clone https://github.com/EffortlessMetrics/Rust-Template.git my-service
cd my-service

# 2. Enter the Nix devshell (first run installs everything)
nix develop

# 3. Verify the environment
cargo xtask doctor

# 4. Run the smoke test
cargo xtask kernel-smoke
```

### What You Get in the Devshell

The `flake.nix` provides:

- **Rust toolchain:** Latest stable (from `rust-toolchain.toml`), Clippy, Rustfmt
- **Build tools:** Cargo, mold (linker), all required dependencies
- **Testing:** Cucumber/BDD harness, all test runners
- **Policy & formatting:** Conftest, OPA, Nix formatters
- **Documentation:** `cargo-doc`, Markdown linter, spell checker
- **Git hooks:** Pre-commit setup via Husky
- **Debugging:** LLDB/GDB (if supported on your OS)

### Validation

Full `selftest` is available and reliable on Tier-1:

```bash
cargo xtask selftest
# Expected: All 8 gates pass (10-20 minutes)
```

---

## Tier-2: Best-Effort Support (Native Windows, macOS/Linux without Nix)

### Overview

Core functionality works without Nix, but **some tools are unavailable or slower**:

| Tool | Status | Impact |
|------|--------|--------|
| Rust compiler | ✅ Works | Core language support |
| Cargo | ✅ Works | Dependency management |
| Clippy | ⚠️ Optional | Linter; may skip on native Windows |
| Rustfmt | ⚠️ Optional | Formatter; may skip |
| BDD tests | ✅ Works | Cucumber scenarios run |
| Policy tests | ⚠️ Limited | OPA/Conftest may fail on Windows |
| Full selftest | ⚠️ Slow | 2+ hours on native Windows; skips some gates |
| `/platform/*` APIs | ✅ Works | Service endpoints available |
| Web UI | ✅ Works | Dashboard available |

### Native Windows (Tier-2)

**Recommended:** Use **WSL2 + Nix** for Tier-1 parity. Native Windows is slower and skips some checks.

#### WSL2 + Nix (Recommended)

```bash
# In WSL2 terminal
wsl --install
# Then follow Tier-1 setup above
```

**Why WSL2?** It's Linux, so Nix works perfectly. You get Tier-1 parity with Windows GUI integration.

#### Native Windows (Not Recommended)

If you must use native Windows:

1. **Install Rust:** [rustup.rs](https://rustup.rs/)
2. **Install Cargo:** Included with rustup
3. **Optional:** Visual Studio Code for editing

```bash
# Clone and build (no Nix)
git clone <repo> && cd my-service
cargo build

# Run tests (some may skip)
cargo test
cargo xtask check    # Skips policy tests
```

**Limitations:**
- File locking: Some BDD tests may timeout
- Policy tests: Likely to fail (Conftest not available)
- Selftest: Full run may exceed 1 hour and skip gates
- Format/lint: Clippy and Rustfmt checks skipped

**Workaround:** Use `XTASK_LOW_RESOURCES=1`:

```bash
XTASK_LOW_RESOURCES=1 cargo xtask check
XTASK_LOW_RESOURCES=1 cargo xtask selftest  # Skips expensive checks
```

### macOS / Linux without Nix (Tier-2)

Install tools manually:

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup toolchain install stable

# Clippy, Rustfmt
rustup component add clippy rustfmt

# Optional: BDD harness, spellcheck, docs
# See flake.nix for exact versions and dependencies

# Then proceed as normal
cargo xtask check
cargo xtask selftest  # Should work, but slower than Nix
```

**Caveats:**
- Versions may drift from CI (toolchain updates, dependencies)
- Policy tests (Conftest) require manual installation
- No guarantee of reproducibility across machines

---

## Platform-Specific Guidance

### Linux

**Tier-1 setup:**

```bash
nix develop
cargo xtask doctor
cargo xtask selftest
```

**Expected:** All gates pass. No special considerations.

### macOS

**Tier-1 setup:**

```bash
nix develop  # May take 2-5 min on first run to build native packages
cargo xtask doctor
cargo xtask selftest
```

**Apple Silicon (M1/M2/M3):**
- Works out of the box via Nix
- No cross-compilation tricks needed

**Intel Macs:**
- Works out of the box

### Windows (WSL2)

**Recommended:** Tier-1 via WSL2.

```bash
# In Windows Terminal, open Ubuntu or Debian WSL2
wsl --list -v   # Verify WSL2 is installed

# In WSL2:
nix develop
cargo xtask doctor
cargo xtask selftest  # Tier-1 parity
```

### Windows (Native)

**Not recommended for release validation.** Use for local iteration only.

```bash
# Install Rust via rustup
rustup --version

# Build and test (may skip some checks)
cargo build
cargo test
XTASK_LOW_RESOURCES=1 cargo xtask selftest
```

**For release validation:** Run full `selftest` in CI or on WSL2/Nix.

---

## Nix Troubleshooting

### "nix command not found"

**Fix:** Install Nix

```bash
# Option 1: Determinate Systems installer (recommended)
curl --proto '=https' --tlsv1.2 -sSf https://install.determinate.systems/nix | sh -s -- install

# Option 2: Official installer
curl -L https://nixos.org/nix/install | sh
```

Then enable flakes:

```bash
mkdir -p ~/.config/nix
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
```

### "error: flake.lock is out of date"

**Fix:** Update flake.lock

```bash
nix flake update
nix develop
```

### "lazy-trees warning" in Determinate installer output

**Expected:** Harmless. Nix 2.30+ deprecated this setting. Safe to ignore.

**To silence:**

```bash
# Edit ~/.config/nix/nix.conf and remove or comment out:
# experimental-features = nix-flakes lazy-trees
```

### Devshell very slow on macOS

**Expected:** First `nix develop` may take 5+ minutes as Nix builds native packages.

**Subsequent runs:** Much faster (cached).

**Workaround:** Pre-build in background:

```bash
nix develop --max-jobs 1 &  # Build slower but use less RAM
# Continue working in another terminal
```

---

## CI Parity

CI uses **Tier-1** (Nix devshell on Linux):

```yaml
# .github/workflows/ci.yml
nix develop
cargo xtask selftest
```

To match CI locally:

```bash
# On any Tier-1 platform:
nix develop
cargo xtask selftest

# On Tier-2, you may see different results—acceptable for iteration
```

**Always validate on Tier-1 before submitting PR.**

---

## Environment Variables

### Development

| Variable | Default | Purpose |
|----------|---------|---------|
| `RUST_BACKTRACE` | unset | Set to `1` or `full` for more error details |
| `RUST_LOG` | unset | Set to `debug` for verbose logging |
| `XTASK_LOW_RESOURCES` | `0` | Set to `1` on Tier-2 to skip expensive checks |

### Platform

| Variable | Default | Purpose |
|----------|---------|---------|
| `PLATFORM_AUTH_MODE` | `none` | Set to `basic` to enable authentication |
| `PLATFORM_AUTH_TOKEN` | unset | Required if `PLATFORM_AUTH_MODE=basic` |
| `PORT` | `8080` | HTTP server port |

---

## Quick Reference

| Task | Tier-1 | Tier-2 |
|------|--------|--------|
| **Clone & setup** | `nix develop` | Manual toolchain install |
| **Dev loop** | `cargo xtask check` | `cargo xtask check` |
| **Fast tests** | `cargo xtask test-changed` | `cargo xtask test-changed` |
| **Specific AC** | `cargo xtask test-ac AC-XXX` | `cargo xtask test-ac AC-XXX` |
| **Full validation** | `cargo xtask selftest` | `XTASK_LOW_RESOURCES=1 cargo xtask selftest` |
| **Before PR** | `cargo xtask selftest` (Tier-1 required) | Run on CI or Tier-1 machine |

---

## Next Steps

- **Tier-1 users:** Proceed to [QUICKSTART.md](../QUICKSTART.md)
- **Tier-2 users:** Use for local iteration; validate on Tier-1 before merge
- **CI setup:** See [required-checks.md](required-checks.md) and `.github/workflows/`
- **Troubleshooting:** See [TROUBLESHOOTING.md](../TROUBLESHOOTING.md)
