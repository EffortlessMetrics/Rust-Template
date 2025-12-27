<!-- doclint:disable orphan-version -->
<!-- External: This document references external tool versions that are not tied to template version. -->
# Development Environment

**Version**: v3.3.13
**Last Updated**: 2025-12-27

---

## TL;DR - The Golden Path

```bash
# 1. Install Nix (if you haven't already)
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install

# 2. Enter the development shell
nix develop

# 3. Validate everything works
cargo run -p xtask -- selftest
```

**That's it.** You now have the exact same environment as CI: Rust toolchain, conftest, yq, all tools pinned to the right versions.

---

## Philosophy: Nix-First, Fallback-Supported

This template is **opinionated about reproducibility**. We believe:

1. **Development environments should be declarative** - No "works on my machine" surprises
2. **Tooling should match CI exactly** - What passes locally passes in CI
3. **Onboarding should be fast** - One command, not a wiki page of brew/apt installs
4. **Dependencies should be pinned** - Same conftest version in 6 months as today

Nix gives us all of that. It's not the only way to work with this template, but it's the way we design for.

---

## Two Tiers of Support

### Tier 1: Nix Devshell (Recommended)

**Who it's for:**
- New team members onboarding to the template
- Anyone working on policies, specs, or LLM bundles
- Teams that value "git clone → nix develop → done"

**What you get:**
- ✅ Rust toolchain (exact version from `rust-toolchain.toml`)
- ✅ conftest 0.52.0 (policy testing)
- ✅ yq (YAML processing)
- ✅ opa (policy compilation)
- ✅ sqlx-cli (database migrations, if using DB)
- ✅ All tools pinned, reproducible across macOS/Linux/WSL

**How to use:**

```bash
# First time setup
nix develop

# Every subsequent session
nix develop

# Optional: Use just for common commands
just dev       # Enter devshell
just selftest  # Run full validation
just check     # Fast checks (fmt, clippy, tests)
just bdd       # BDD scenarios only
```

**Optional: Automatic Devshell with Direnv**

If you use `direnv`, you can make the Nix devshell activate automatically when you `cd` into the repo:

```bash
# Create .envrc (not committed by default)
echo "use flake" > .envrc

# Allow direnv to load it
direnv allow

# From now on, your shell drops into the Nix devshell automatically
cd /path/to/Rust-Template  # → devshell activates
cd ~                        # → devshell deactivates
```

This pattern is supported but not required. We don't commit `.envrc` to avoid surprising environments for users who don't use direnv.

If you want this pattern, install direnv first:

```bash
# On macOS
brew install direnv

# On Linux/WSL
sudo apt install direnv  # or equivalent for your distro

# Then add to your shell profile (~/.bashrc, ~/.zshrc, etc.):
eval "$(direnv hook bash)"  # or zsh, fish, etc.
```

**Validation:**

```bash
nix develop
cargo run -p xtask -- selftest
# Output: All 5 steps pass, including policy tests
```

This is the environment CI uses. If selftest passes here, it will pass in CI.

---

### Tier 2: Native Toolchain (Fallback)

**Who it's for:**
- Teams that can't use Nix (org policy, security constraints, etc.)
- Developers with strong preferences for brew/apt/asdf/mise
- Windows-native environments (though WSL + Nix works great)

**What you get:**
- ✅ Full Rust development (fmt, clippy, tests, BDD, AC mapping, bundler)
- ⚠️ Policy tests skipped if conftest not installed
- ⚠️ Manual responsibility to keep tool versions aligned with CI

**Required tools:**
- Rust (via `rustup`, matching `rust-toolchain.toml`)
- cargo (comes with Rust)
- conftest 0.52.0 (optional, but required for policy work)
- yq (optional, for manual YAML processing)

**How to install conftest (matching CI version):**

On Linux/WSL:
```bash
CONFTEST_VERSION="0.52.0"
wget "https://github.com/open-policy-agent/conftest/releases/download/v${CONFTEST_VERSION}/conftest_${CONFTEST_VERSION}_Linux_x86_64.tar.gz"
tar -xzf "conftest_${CONFTEST_VERSION}_Linux_x86_64.tar.gz"
chmod +x conftest
sudo mv conftest /usr/local/bin/
conftest --version  # Should show: Conftest: 0.52.0
```

On macOS:
```bash
brew install conftest
# Note: may not be 0.52.0 - check `conftest --version`
# For exact version match, use the wget approach above
```

**Validation:**

```bash
cargo run -p xtask -- selftest
# Output: 4 steps pass, step 5 shows:
#   ⚠ Policy tests skipped: conftest not found
# Still exits 0 (success)
```

This mode is **fully functional** for most development work. You only need conftest if:
- You're editing policies in `policy/*.rego`
- You're adding/changing policy test fixtures in `policy/testdata/`
- You want to match CI's strict validation locally

---

## CI Behavior

CI always uses the Nix environment via:

```yaml
- name: Selftest
  run: nix develop -c bash -c 'cargo run -p xtask -- selftest'
```

This ensures:
- ✅ All 5 selftest steps must pass
- ✅ All 22+ policy tests must pass
- ✅ Same tool versions as devshell

If policy tests pass locally in `nix develop` but fail in CI, that's a bug - please report it.

---

## Choosing Your Path

### Use Nix if:
- You're new to the repo (fastest onboarding)
- You're working on policies, K8s manifests, or LLM bundles
- You want "it just works" without managing tool versions
- You care about exact CI parity

### Use Native Toolchain if:
- Your org doesn't allow Nix
- You're already managing Rust via mise/asdf/rustup and are comfortable with it
- You're only doing business logic / HTTP handlers (not policy work)
- You're willing to manually install conftest when needed

---

## Troubleshooting

### "conftest not found" but I'm in `nix develop`

Check that the flake is using the right conftest:

```bash
nix develop -c conftest --version
# Should show: Conftest: 0.52.0
```

If not, try:
```bash
nix flake update
nix develop
```

### "Policy tests failed" in CI but passed locally

You're probably in Tier 2 (native toolchain) where policies were skipped. Run:

```bash
nix develop
cargo run -p xtask -- selftest
```

The policy step should now run and show you the actual failure.

### Nix is slow / taking forever

First build is slow (downloads/compiles dependencies). Subsequent runs use the Nix cache.

If it's consistently slow:
- Check your internet connection (Nix fetches from caches)
- Try: `nix develop --no-update-lock-file` to skip flake updates
- Consider setting up a local Nix cache if your team shares machines

---

## For Library Users (Not Template Users)

If you're using `rust_iac_xtask_core` as a library in your own project:

- You **don't need Nix** at all
- The library is just a Rust crate with normal Cargo deps
- Install the CLI tools (conftest, yq, etc.) however your org prefers
- `xtask selftest` will work as long as the tools are in `$PATH`

The Nix setup is specific to **this template repo**, not a requirement of the library itself.

---

## Summary

| Aspect | Tier 1 (Nix) | Tier 2 (Native) |
|--------|-------------|-----------------|
| **Onboarding** | `nix develop` | Install Rust + conftest manually |
| **Tool versions** | Pinned, automatic | Your responsibility |
| **Policy tests** | ✅ Always run | ⚠️ Skipped if no conftest |
| **CI parity** | ✅ Exact match | ⚠️ Best-effort |
| **Recommended for** | Default, policy work, new devs | Org constraints, strong preferences |

**Bottom line:** If you can use Nix, use Nix. If you can't, the template still works - you just take on more manual setup and lose strict CI parity on policies.

---

**Next steps:**
- New to the repo? → `nix develop && cargo run -p xtask -- selftest`
- Can't use Nix? → Install conftest 0.52.0, then `cargo run -p xtask -- selftest`
- Want automatic devshell activation? → Set up direnv (see Tier 1 section)
