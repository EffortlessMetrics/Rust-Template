<!-- doclint:disable orphan-version -->
---
id: GUIDE-TPL-NIX-ENV-ISSUES
title: Nix Environment Issues & Solutions
doc_type: guide
status: published
audience: developers
tags: [nix, environment, troubleshooting, devshell, libz]
stories: [US-TPL-DEVEX-001]
requirements: [REQ-PLT-DEVEX-CONTRACT, REQ-PLT-ENV-DIAGNOSTICS]
acs: [AC-PLT-015, AC-PLT-016]
adrs: [ADR-0005]
last_updated: 2025-12-02
---

# Nix Environment Issues & Solutions

This guide explains two common Nix configuration issues that can prevent builds and tests from working correctly, how they arise, and how to fix them.

---

## Quick Reference

| Issue | Symptom | Root Cause | Fix |
|-------|---------|-----------|-----|
| **libz.so.1 missing** | `error while loading shared libraries: libz.so.1: cannot open shared object file` | Nix devshell lacks `pkgs.zlib` + LD_LIBRARY_PATH setup | Add `pkgs.zlib` to flake.nix buildInputs + export LD_LIBRARY_PATH in shellHook |
| **lazy-trees warning** | `warning: unknown setting 'lazy-trees'` | Nix config has deprecated setting unsupported by current Nix version | Either upgrade to Determinate Nix 3.5+ or remove setting from nix.conf |

---

## Issue 1: libz.so.1 Missing in Nix Devshell

### Symptom

When running `cargo xtask check` or any rustc invocation inside `nix develop`, you see:

```
error while loading shared libraries: libz.so.1: cannot open shared object file: No such file or directory
```

This error occurs even though `zlib1g` is installed in your Linux distro (Ubuntu/Debian).

### Root Cause

The Nix devshell does not include `pkgs.zlib` in its package list and does not set `LD_LIBRARY_PATH` to point to the Nix zlib store location. When rustc (or sccache, which wraps rustc) tries to load dynamic libraries at runtime, it cannot find `libz.so.1` in the expected search paths.

**This is NOT a "WSL2 limitation" or a Windows/Microsoft bug.** It's a Nix environment configuration issue that can affect any system (Linux, macOS, WSL2) if the devshell is misconfigured.

### Solution

Update `flake.nix` to include `pkgs.zlib` in both the package list and buildInputs, and export `LD_LIBRARY_PATH`:

**Before:**
```nix
devShells = forAllSystems ({ pkgs, rust, ... }: {
  default = pkgs.mkShell {
    packages = [
      rust
      pkgs.just
      # ... other packages ...
    ];
    shellHook = ''
      export PATH="$PWD/.tools/bin:$PATH"
      echo "DevShell ready — try: just checks"
    '';
  };
});
```

**After:**
```nix
devShells = forAllSystems ({ pkgs, rust, ... }: {
  default = pkgs.mkShell {
    packages = [
      rust
      pkgs.just
      pkgs.zlib  # Required for rustc/sccache dynamic linking
      # ... other packages ...
    ];
    buildInputs = [ pkgs.zlib ];  # Also in buildInputs for linker visibility
    shellHook = ''
      export PATH="$PWD/.tools/bin:$PATH"
      export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [ pkgs.zlib ]}:$LD_LIBRARY_PATH"
      echo "DevShell ready — try: just checks"
    '';
  };
});
```

### Verification

```bash
nix flake update
nix develop
cargo xtask check      # Should now pass
```

### Why This Works

- `pkgs.zlib` in `packages` makes the zlib binary available in PATH
- `pkgs.zlib` in `buildInputs` makes it available to the Nix build environment
- `export LD_LIBRARY_PATH` ensures that dynamically linked binaries (rustc, sccache) can find `libz.so.1` at runtime

The dynamic linker searches `LD_LIBRARY_PATH` at runtime; without this explicit export, even though zlib is in the devshell, binaries can't find it.

---

## Issue 2: "unknown setting 'lazy-trees'" Warning

### Symptom

When running any Nix command (including entering `nix develop`), you see:

```
warning: unknown setting 'lazy-trees'
```

The warning appears but does **not** prevent the command from succeeding.

### Root Cause

`lazy-trees` is a performance feature introduced by Determinate Systems' Nix fork (Determinate Nix 3.5+). If you have `lazy-trees = true` in your Nix config (typically in `/etc/nix/nix.conf` if installed via Determinate's installer), but you're running:

1. **Upstream Nix** (not Determinate Nix), or
2. **Determinate Nix < 3.5.x**

...then Nix will warn that it doesn't recognize the setting.

**This is NOT a build blocker.** The warning is cosmetic and has no impact on functionality.

### Solution

**Option 1 (If using Determinate Nix):** Upgrade to version 3.5.x or later

```bash
nix --version
# Expected: Determinate Nix 3.5.x or later

# If you need to upgrade:
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

**Option 2 (If using upstream Nix or prefer not to upgrade):** Remove the setting from nix.conf

```bash
# Check where the setting is
grep lazy-trees /etc/nix/nix.conf /home/*/.config/nix/nix.conf 2>/dev/null

# Remove it (may require sudo if in /etc/nix/)
sudo nano /etc/nix/nix.conf
# Delete the line: lazy-trees = true
# Save and exit

# Or, if it's in your user config:
nano ~/.config/nix/nix.conf
# Delete the line and save
```

### Verification

```bash
nix --version         # Check current Nix version
nix develop -c true   # Enter devshell; warning should not appear
```

### Why This Happens

Determinate Systems' installer automatically adds `lazy-trees = true` to the managed Nix config. If you later use a different Nix version or tool, that version may not support the setting yet. The Nix architecture requires all config options to be known at parse time, so unknown options produce warnings.

**Future:** As Determinate Nix becomes more widely used and upstream Nix adopts the feature, this warning should disappear.

---

## Related Troubleshooting

If you have related issues, see:

- **General environment diagnostics**: `cargo xtask doctor`
- **Full troubleshooting guide**: `docs/TROUBLESHOOTING.md`
- **Complete flake.nix reference**: `flake.nix` (this repository)

---

## Environment Validation

To verify your Nix environment is correctly set up:

```bash
# 1. Check Nix version
nix --version

# 2. Check if zlib is available in devshell
nix develop -c pkg-config --modversion zlib

# 3. Check LD_LIBRARY_PATH is set
nix develop -c bash -c 'echo $LD_LIBRARY_PATH | grep -q zlib && echo "zlib in LD_LIBRARY_PATH" || echo "zlib NOT in LD_LIBRARY_PATH"'

# 4. Run doctor command
cargo xtask doctor

# 5. Run full check
cargo xtask check
```

If all commands above succeed, your Nix environment is correctly configured.

---

## For CI/CD

In CI environments (GitHub Actions, GitLab CI, etc.), ensure:

1. **Nix is installed** (recommended: Determinate Nix via their GitHub Action installer)
2. **flake.nix includes pkgs.zlib** (as shown in Solution above)
3. **CI runs inside `nix develop`** or uses `nix develop -c <command>` wrapper

Example GitHub Actions step:

```yaml
- name: Run checks
  run: |
    nix develop -c cargo xtask check
    nix develop -c cargo xtask selftest
```

---

## Questions & Feedback

If you encounter issues not covered here:

1. Run `cargo xtask doctor` and share the output
2. Check `docs/TROUBLESHOOTING.md` for common issues
3. File a GitHub issue linking the error message and your environment details

