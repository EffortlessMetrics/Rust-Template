---
id: GUIDE-TPL-FIRST-FORK-001
title: First Fork Runbook
doc_type: how-to
status: published
audience: developers, team-leads, platform-engineers
tags: [onboarding, fork, quickstart, setup]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-ONBOARDING]
acs: [AC-PLT-001, AC-PLT-008]
adrs: []
last_updated: 2025-12-22
---

# First Fork Runbook

**Audience:** Senior dev or team lead forking this template for the first time.

**Time:** 20 minutes + ~10 minutes for CI to validate.

**Goal:** Go from "zero to a governed, production-ready service skeleton" in one command sequence.

---

## Prerequisites

- Git
- Rust (1.70+)
- Nix with flakes (recommended for Tier-1 parity; optional if you use WSL2)
- GitHub account with write access to your target org

---

## The Five Steps

### 1. Validate the template locally

```bash
git clone https://github.com/EffortlessMetrics/Rust-Template.git
cd Rust-Template

nix develop                  # Enter Nix devshell (installs all tools)
cargo xtask kernel-smoke     # Quick smoke test (~2 min)
```

**Expected:** Green. If red, troubleshoot before forking.

---

### 2. Fork the template

**Via GitHub UI:**
- Navigate to the repo
- Click **"Use this template"** → **"Create a new repository"**
- Name: `<your-service-name>`
- Visibility: **Private** (recommended for production)
- Create

**Or via CLI:**

```bash
gh repo create <your-service-name> --template EffortlessMetrics/Rust-Template --private
```

---

### 3. Decide opinionation level (5 minutes)

Before customizing, choose which template behaviors you'll **enforce** vs treat as **optional**.

**Read this first:** [`KERNEL_SNAPSHOT.md`](../KERNEL_SNAPSHOT.md) → "Kernel vs Template Defaults"

**Quick decision matrix:**

| Feature | Kernel? | Why |
|---------|---------|-----|
| **Nix devshell** | YES | Eliminates "works on my machine" |
| **BDD + AC traceability** | YES | Governance backbone |
| **AI/agent surfaces** (`/platform/*`, CLI JSON) | YES (if using agents) | Agent contracts stabilize fast |
| **Governance artifacts** (friction, questions, forks) | Optional | Keep if regulated/process-heavy; remove if lightweight |
| **Policy tests (OPA/Rego)** | Optional | Keep if compliance-critical; remove for faster CI |

**To adjust:** Edit `specs/spec_ledger.yaml` and flip `must_have_ac: true/false`. See [`change-template-opinion.md`](./change-template-opinion.md) for details.

**For most teams:** Keep kernel + AI surfaces, leave governance artifacts optional. This gives you a strong baseline and room to customize later.

---

### 4. Clone your fork and set up enforcement

```bash
git clone git@github.com:your-org/<your-service-name>.git
cd <your-service-name>

nix develop
cargo xtask kernel-smoke       # Verify inherited baseline is green
```

**Set up branch protection** (prevents bypassing selftest):

See [`setup-branch-protection.md`](./setup-branch-protection.md) for full details. TL;DR:

- **Repo Settings** → **Branches** → **Add rule**
- Branch pattern: `main`
- ✅ Require PR, require approvals (1+), require status checks
- **Required checks:** `tier1-selftest` (primary), plus any others you want
- ✅ "Do not allow bypassing"
- Save

**Set up tag signing** (authenticates releases):

See [`setup-tag-signing.md`](./setup-tag-signing.md) for full details. TL;DR:

```bash
gpg --list-secret-keys --keyid-format=long
git config --global user.signingkey <your-key-id>
git config --global tag.gpgSign true

# Upload public key to GitHub Settings → SSH and GPG keys
gpg --armor --export <your-key-id> | pbcopy
```

---

### 5. Customize and trigger CI

```bash
# Edit service metadata
# → README.md (name, description)
# → CLAUDE.md (service IDs, domain routes)
# → specs/spec_ledger.yaml (add first story + REQ + AC)

# Test locally
cargo xtask check
cargo xtask test-changed
cargo xtask selftest          # Full validation (10-20 min, Tier-1)

# Push and watch CI
git add .
git commit -m "chore: establish fork baseline"
git push origin main

# GitHub Actions runs tier1-selftest automatically
# → Watch it complete in Actions tab
```

**Expected:** CI passes. Your team now has a governed, production-ready skeleton.

---

## What You Inherit

From this moment on, your fork has:

✅ **Governance as code** – specs, BDD, AC traceability, policy tests
✅ **Enforcement gate** – CI blocks merges unless selftest passes
✅ **Introspection APIs** – `/platform/status`, `/platform/graph`, etc.
✅ **Web dashboard** – <http://localhost:8080/ui> shows governance state
✅ **LLM context** – `cargo xtask bundle` for AI-native development
✅ **Validated baseline** – All 8 governance checks certified

---

## What's Next?

### For the team

1. **Onboarding:** Share [QUICKSTART.md](../QUICKSTART.md) with developers
2. **First feature:** Follow [`docs/how-to/add-acceptance-criterion.md`](./add-acceptance-criterion.md)
3. **Reference:** Bookmark [`docs/AGENT_GUIDE.md`](../AGENT_GUIDE.md) for workflows

### For you

1. **Architecture review:** Read [`docs/explanation/template-architecture.md`](../explanation/template-architecture.md)
2. **Governance deep-dive:** Read [`docs/CONSTITUTION.md`](../CONSTITUTION.md)
3. **Release planning:** See [`docs/RELEASE_PLAYBOOK.md`](../RELEASE_PLAYBOOK.md)

---

## Troubleshooting

### Selftest fails locally

→ Run `cargo xtask doctor` for diagnostics. See [`TROUBLESHOOTING.md`](../TROUBLESHOOTING.md).

### CI fails but local passes

→ Run `nix develop --command cargo xtask selftest` to match CI environment exactly.

### Branch protection not enforcing

→ Verify "Do not allow bypassing" is checked. If you're a repo admin, admins can bypass by default—remove yourself from admin role on main.

### CI runs very slowly on Windows

→ Expected on native Windows (Tier-2). Use WSL2 for Tier-1 parity.

---

## Success Criteria

You're ready when:

- ✅ Fork created with branch protection enabled
- ✅ `cargo xtask kernel-smoke` passes locally
- ✅ CI tier1-selftest passes on main
- ✅ Team has access and knows where to start (QUICKSTART.md)

---

**You're live.** Start building.
