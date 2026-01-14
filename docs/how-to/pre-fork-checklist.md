---
id: GUIDE-TPL-PRE-FORK-001
title: Pre-Fork Checklist
doc_type: how-to
status: published
audience: developers, platform-engineers, team-leads
tags: [onboarding, setup, security, governance, fork-preparation]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-DOC-TEMPLATES]
acs: [AC-PLT-008, AC-PLT-011, AC-PLT-012]
adrs: [ADR-0005, ADR-0006]
last_updated: 2025-11-26
---
<!-- doclint:disable orphan-version -->

# Pre-Fork Checklist

**Time:** 30-45 minutes (first-time setup)
**Prerequisites:** Git, Rust, Nix (recommended), GitHub account

This checklist ensures your local environment and GitHub configuration are ready before forking the Rust-as-Spec template for production use.

---

## Why Validate Before Forking?

Forking is the easy part. The critical work is:
1. **Verifying your environment works** - Catching toolchain issues before you customize
2. **Setting up enforcement** - Branch protection and tag signing prevent governance bypass
3. **Establishing trust** - Signed tags create a cryptographic chain for releases

Complete these steps before forking so your team inherits a production-ready foundation.

---

## Phase 1: Environment Validation

### 1.1 Basic Toolchain

```bash
# Check Rust version (1.70+ required)
- [ ] rustc --version
- [ ] cargo --version

# Check Git
- [ ] git --version

# Optional but recommended: Nix for reproducible builds
- [ ] nix --version
```

### 1.2 Template Smoke Test

Clone the template locally first:

```bash
git clone https://github.com/your-org/Rust-Template.git
cd Rust-Template
```

Run the validation ladder:

```bash
# Tier-1 (full selftest, matches CI exactly)
- [ ] nix develop
- [ ] cargo xtask doctor
- [ ] cargo xtask selftest

# Verify all 8 governance checks pass:
#   1. Spec validation (ledger, schema, flows, tasks)
#   2. Code compilation (all crates)
#   3. Unit + integration tests
#   4. BDD scenarios (AC coverage)
#   5. Documentation consistency
#   6. Security scan (cargo audit, cargo deny)
#   7. Policy tests (OPA/Rego)
#   8. Graph completeness (REQ/AC/test/doc linkage)
```

**Expected:** All checks green. If any fail, troubleshoot before forking.

**Tier-2 (Native Windows):** If using native Windows without WSL2/Nix:

```powershell
- [ ] cargo xtask doctor
- [ ] cargo xtask check
- [ ] cargo xtask test-changed
```

Note: Full `selftest` may intermittently fail on Windows due to file locking (see `docs/MISSING_MANUAL.md`). Use WSL2 for production validation.

### 1.3 Service Startup

```bash
- [ ] cargo run -p app-http
- [ ] curl http://localhost:8080/health
- [ ] curl http://localhost:8080/version
- [ ] Visit http://localhost:8080/ui (governance dashboard)
```

**Expected:** Service starts, endpoints respond, UI loads.

### 1.4 Docker (Optional)

If your team uses Docker for deployment:

```bash
- [ ] docker --version
- [ ] docker build -t template-test .
- [ ] docker run -p 8080:8080 template-test
```

**Expected:** Container builds and runs successfully.

---

## Phase 2: GitHub Security Configuration

Complete these **before** forking to establish trust from day one.

### 2.1 GPG Tag Signing Setup

Signed tags are critical for release authenticity and supply chain security.

**Follow:** [Setup Tag Signing Guide](./setup-tag-signing.md)

```bash
# Verify GPG is configured
- [ ] gpg --list-secret-keys --keyid-format=long
- [ ] git config --global user.signingkey <your-key-id>
- [ ] git config --global tag.gpgSign true

# Test local signing
- [ ] git tag -s v0.1.0-test -m "Test signed tag"
- [ ] git tag -v v0.1.0-test

# Upload public key to GitHub
- [ ] gpg --armor --export <your-key-id> | pbcopy  # (or xclip on Linux)
- [ ] GitHub Settings → SSH and GPG keys → New GPG key → Paste
```

**Why this matters:** Without signed tags, anyone with write access can inject malicious releases. Signed tags establish a cryptographic trust root for your entire release pipeline.

### 2.2 GitHub CLI Setup (Optional but Recommended)

```bash
- [ ] gh --version
- [ ] gh auth login
- [ ] gh auth status
```

This enables programmatic branch protection setup.

---

## Phase 3: Fork and Configure

### 3.1 Create Your Fork

**Via GitHub UI:**

```bash
- [ ] Navigate to template repo
- [ ] Click "Use this template" → "Create a new repository"
- [ ] Name: <your-service-name>
- [ ] Visibility: Private (recommended for production services)
- [ ] Create repository
```

**Or via CLI:**

```bash
- [ ] gh repo create <your-service-name> --template your-org/Rust-Template --private
- [ ] cd <your-service-name>
```

### 3.2 Branch Protection Rules

Enforce governance checks at the platform level.

**Follow:** [Setup Branch Protection Guide](./setup-branch-protection.md)

**Via GitHub UI:**

```bash
- [ ] Repo Settings → Branches → Add rule
- [ ] Branch name pattern: main
- [ ] ✅ Require a pull request before merging
- [ ] ✅ Require approvals: 1 (adjust for team size)
- [ ] ✅ Dismiss stale PR approvals when new commits are pushed
- [ ] ✅ Require status checks to pass before merging
- [ ] ✅ Require branches to be up to date before merging
- [ ] Select required checks:
      - [ ] tier1-selftest (comprehensive governance)
      - [ ] ci-security (cargo audit, cargo deny)
      - [ ] ci-docs (documentation consistency)
      - [ ] ci-policy-verify (OPA policy checks)
- [ ] ✅ Do not allow bypassing the above settings
- [ ] ✅ Restrict who can push to matching branches (maintainers only)
- [ ] ✅ Require signed commits (optional but recommended)
- [ ] Save changes
```

**Or via GitHub CLI:**

```bash
- [ ] gh api repos/:owner/:repo/branches/main/protection -X PUT --input branch-protection.json
```

(See `setup-branch-protection.md` for JSON template)

**Why this matters:** Without branch protection, developers can push directly to main and bypass selftest, breaking governance.

### 3.3 Tag Protection (GitHub Enterprise)

If using GitHub Enterprise:

```bash
- [ ] Repo Settings → Tags → Add rule
- [ ] Tag name pattern: v*
- [ ] ✅ Require signed tags
- [ ] ✅ Restrict tag creation to maintainers
```

**Why this matters:** Prevents unauthorized release tags from being created.

---

## Phase 4: Baseline Validation in Fork

### 4.1 Clone Your Fork

```bash
- [ ] git clone git@github.com:your-org/<your-service-name>.git
- [ ] cd <your-service-name>
```

### 4.2 Verify Inherited Baseline

Before customizing, confirm the kernel is green:

```bash
- [ ] nix develop
- [ ] cargo xtask kernel-smoke

# Or full selftest
- [ ] cargo xtask selftest
```

**Expected:** All inherited template ACs pass (health, version, errors, metrics, platform endpoints).

### 4.3 Trigger CI

```bash
- [ ] git commit --allow-empty -m "test: trigger CI"
- [ ] git push origin main
- [ ] Watch CI run in GitHub Actions
- [ ] Verify tier1-selftest passes
```

**Expected:** CI completes successfully. This confirms GitHub + CI integration works.

---

## Phase 5: Choose Your Opinionation Level

Before customizing, decide which template behaviors your fork will **enforce as kernel** vs treat as **optional defaults**.

**If you do nothing:** You inherit the template's opinionated defaults—AI-native surfaces on, governance artifacts wired, traceability via refs. That's a strong, tested baseline for most services.

### 5.1 Understand Kernel vs Template ACs

The template distinguishes two AC categories:

| Category | `must_have_ac` | What It Means |
|----------|---------------|---------------|
| **Kernel** | `true` | Non-negotiable. Selftest fails if missing or broken. |
| **Template Default** | `false` | Enabled and green here, but forks can demote or remove. |

**Read first:** [`docs/KERNEL_SNAPSHOT.md`](../KERNEL_SNAPSHOT.md) → "Kernel vs Template Defaults" table

### 5.2 Common Opinionation Choices

For each category below, decide if you want it **enforced** (keep/promote to kernel) or **optional** (leave as template default or remove):

**AI/IDP Integration:** *(Recommended: keep kernel for AI-native services)*

```yaml
# Keep these kernel if you want AI agents and IDPs to have stable contracts:
- AC-TPL-CLI-JSON-CORE         # version --json, ac-status --json
- AC-TPL-PLATFORM-GOVERNANCE-APIS  # /platform/status includes forks, friction
```

**Governance Artifacts:** *(Recommended: promote to kernel for regulated/process-heavy environments)*

```yaml
# Promote these to kernel if your team relies on them:
- AC-TPL-GOV-FRICTION          # friction-new/list CLI + /platform/friction API
- AC-TPL-QUESTIONS-LOGGED      # question-new/list CLI + /platform/questions API
- AC-TPL-GOV-FORKS             # fork-register/list CLI + /platform/forks API
```

**Traceability:** *(Recommended: keep kernel for audit trails and governance clarity)*

```yaml
# Keep kernel if you want governance artifacts linked to REQ/AC IDs:
- AC-TPL-ARTIFACTS-HAVE-REFS   # refs field on questions/friction
```

### 5.3 Apply Your Decisions

**To promote a template default to kernel** (make it mandatory):

```bash
# Edit specs/spec_ledger.yaml
# Change: must_have_ac: false
# To:     must_have_ac: true

# Then verify selftest still passes:
cargo xtask selftest
```

**To demote a kernel AC** (make it optional or remove):

```bash
# Edit specs/spec_ledger.yaml
# Change: must_have_ac: true
# To:     must_have_ac: false

# Or remove the AC entirely if you don't want the behavior

# Document why in an ADR:
cargo xtask adr-new "demote-friction-to-optional"
```

**Detailed guide:** [`docs/how-to/change-template-opinion.md`](./change-template-opinion.md)

### 5.4 Record Your Baseline

After making opinionation decisions:

```bash
# Capture your fork's kernel contract
- [ ] cargo xtask selftest          # Must pass
- [ ] cargo xtask ac-status         # Review what's enforced
- [ ] git commit -m "chore: establish fork kernel baseline"
```

This commit becomes your fork's "what we enforce" reference point.

---

## Phase 6: Customization Readiness

### 6.1 Update Service Metadata

Edit these files to reflect your service:

```bash
- [ ] README.md (service name, description)
- [ ] CLAUDE.md (service ID, task/REQ/AC prefixes, domain routes)
- [ ] service_metadata.yaml (if present)
- [ ] specs/spec_ledger.yaml (add first user story)
```

### 6.2 Seed Your First AC

Add your domain's first requirement and AC:

```bash
- [ ] Edit specs/spec_ledger.yaml (add story + REQ + AC)
- [ ] Create specs/features/<your-feature>.feature
- [ ] Add task to specs/tasks.yaml
- [ ] cargo xtask ac-status (verify AC appears)
```

### 6.3 Validate Customization

```bash
- [ ] cargo xtask check
- [ ] cargo xtask test-changed
- [ ] cargo xtask selftest
```

**Expected:** Template ACs still pass. Your new AC may be red (not implemented yet) - that's OK.

### 6.4 Create First PR

Test branch protection:

```bash
- [ ] git checkout -b feat/first-customization
- [ ] git add .
- [ ] git commit -m "feat: customize service metadata"
- [ ] git push origin feat/first-customization
- [ ] gh pr create --title "Customize service metadata" --body "Initial fork customization"
- [ ] Verify CI runs tier1-selftest
- [ ] Verify branch protection prevents direct merge
- [ ] Approve and merge PR
```

**Expected:** Branch protection enforces review + CI checks. This confirms enforcement is working.

---

## Phase 7: Team Onboarding Prep

### 7.1 Documentation Review

```bash
- [ ] Read docs/QUICKSTART.md
- [ ] Read docs/AGENT_GUIDE.md
- [ ] Read docs/MISSING_MANUAL.md
- [ ] Read docs/how-to/new-service-from-template.md
```

### 7.2 Share Fork Context

Prepare for team handoff:

```bash
- [ ] Create team Slack/Discord channel
- [ ] Share fork repo URL
- [ ] Document custom service prefix (TASK-XXX, REQ-XXX, AC-XXX)
- [ ] Link to setup guides (branch protection, tag signing)
- [ ] Schedule kickoff session (optional)
```

### 7.3 Verify Platform APIs

```bash
- [ ] cargo run -p app-http
- [ ] curl http://localhost:8080/platform/status
- [ ] curl http://localhost:8080/platform/graph
- [ ] curl http://localhost:8080/platform/docs/index
- [ ] curl http://localhost:8080/platform/tasks
- [ ] curl http://localhost:8080/platform/agent/hints
```

**Expected:** All platform endpoints return structured JSON. These are the APIs agents and developers use for discovery.

---

## Success Criteria

You're ready to start development when:

- ✅ Template baseline passes selftest locally
- ✅ GPG tag signing configured and tested
- ✅ Fork created with branch protection enabled
- ✅ CI runs tier1-selftest successfully
- ✅ First customization PR merged via governed process
- ✅ Team has access to repo and documentation

---

## Troubleshooting

### Selftest Fails Locally

**Symptom:** `cargo xtask selftest` fails with governance errors.

**Fixes:**
1. Run `cargo xtask doctor` for diagnostics
2. Check `docs/MISSING_MANUAL.md` → Platform Support
3. Use `nix develop` for reproducible environment
4. On Windows: Use WSL2 or validate with `check` + `test-changed` instead

### CI Fails but Local Passes

**Symptom:** Local selftest green, CI red.

**Fixes:**
1. Check CI uses same Nix environment (`.github/workflows/ci.yml`)
2. Run `nix develop --command cargo xtask selftest` locally to match CI exactly
3. Check for platform-specific dependencies (Docker, file paths)

### Branch Protection Not Enforcing

**Symptom:** Able to push directly to main.

**Fixes:**
1. Verify you're not a repo admin (admins can bypass by default)
2. In branch protection settings: ✅ "Do not allow bypassing the above settings"
3. Check "Restrict who can push" includes your user

### GPG Signing Fails

**Symptom:** `git tag -s` fails with "gpg failed to sign the data".

**Fixes:**
1. Check `gpg --list-secret-keys` shows your key
2. Export key ID: `git config --global user.signingkey <key-id>`
3. Test GPG directly: `echo "test" | gpg --clearsign`
4. See [Setup Tag Signing Guide](./setup-tag-signing.md) for full troubleshooting

---

## Next Steps

After completing this checklist:

1. **Start feature development:** Follow [docs/how-to/add-http-endpoint.md](./add-http-endpoint.md) or AC-first workflows
2. **Review governance flows:** Run `cargo xtask help-flows` to discover available workflows
3. **Plan first release:** See [docs/how-to/new-service-from-template.md](./new-service-from-template.md) Step 7
4. **Provide feedback:** Use [docs/how-to/report-fork-feedback.md](./report-fork-feedback.md) to share your experience

---

## Reference Links

- [Quick Start Guide](../QUICKSTART.md)
- [Agent Guide](../AGENT_GUIDE.md)
- [Setup Branch Protection](./setup-branch-protection.md)
- [Setup Tag Signing](./setup-tag-signing.md)
- [New Service from Template](./new-service-from-template.md)
- [Missing Manual](../MISSING_MANUAL.md)
- [Kernel Snapshot](../KERNEL_SNAPSHOT.md)

---

**You're ready to fork!** This template is designed to be production-ready from day one. Complete this checklist and you'll have a governed, secure foundation for your team.
