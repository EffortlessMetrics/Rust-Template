---
id: DESIGN-TPL-GOV-HOOKS-001
title: Governance Hooks
author: governance-system
doc_type: design_doc
date: 2025-11-22
status: draft
stories: [US-TPL-PLT-001]
requirements: [REQ-TPL-GOV-HOOKS]
tags: [platform, devex, structural]
acs: [AC-TPL-HOOKS-INSTALL]
adrs: [ADR-0005]
---

# Governance Hooks

## Problem

Developers can commit code that fails `cargo xtask check` (fmt, clippy, tests), discovering failures only in CI. This wastes time and breaks the fast feedback loop.

## Solution

Provide `cargo xtask install-hooks` command that installs a Git pre-commit hook running `cargo xtask check` before allowing commits. The hook will use the Nix environment when available to ensure consistent tool versions.

## Implementation Approach

**Command**: `crates/xtask/src/commands/install_hooks.rs`

**Logic**:
1. Create `.git/hooks/pre-commit` with execute permissions
2. Hook script runs `cargo xtask check` (or `nix develop -c cargo xtask check` if Nix detected)
3. If check fails, abort commit and show error
4. If check passes, allow commit to proceed

**Hook script**:

```bash
#!/bin/sh
if command -v nix &> /dev/null && [ -f flake.nix ]; then
  nix develop -c cargo xtask check
else
  cargo xtask check
fi
```

**Integration**: Add to `cargo xtask dev-up` so new developers get hooks automatically during onboarding.

**Benefits**: Prevents broken commits, enforces governance before code leaves developer machine, reduces CI failures.
