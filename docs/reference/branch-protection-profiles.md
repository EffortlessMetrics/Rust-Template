# Branch Protection Profiles

This document defines three standard profiles for GitHub branch protection. Choose the profile that matches your service maturity and risk tolerance.

---

## Profile Comparison

| Check Category | Minimal | Standard | Strict |
|----------------|---------|----------|--------|
| **Core Checks** |
| Lints (fmt, clippy, tests) | ✅ Required | ✅ Required | ✅ Required |
| Nix Flake Check | ✅ Required | ✅ Required | ✅ Required |
| MSRV | ✅ Required | ✅ Required | ✅ Required |
| **Contract Checks** |
| OpenAPI | 🟡 Advisory | ✅ Required | ✅ Required |
| Proto | 🟡 Advisory | ✅ Required | ✅ Required |
| DB Migrations | 🟡 Advisory | ✅ Required | ✅ Required |
| Events | 🟡 Advisory | ✅ Required | ✅ Required |
| **Governance** |
| ACs | 🟡 Advisory | ✅ Required | ✅ Required |
| Gherkin | 🟡 Advisory | ✅ Required | ✅ Required |
| Features | 🟡 Advisory | ✅ Required | ✅ Required |
| Flags | 🟡 Advisory | ✅ Required | ✅ Required |
| Privacy | 🟡 Advisory | ✅ Required | ✅ Required |
| **Quality** |
| Coverage | 🟡 Advisory | 🟡 Advisory | ✅ Required |
| Security | 🟡 Advisory | ✅ Required | ✅ Required |
| **Documentation** |
| Docs Build | 🟡 Advisory | 🟡 Advisory | ✅ Required |
| Scope Guard | ❌ Disabled | 🟡 Advisory | ✅ Required |

Legend:
- ✅ Required: Must pass before merge
- 🟡 Advisory: Runs but doesn't block
- ❌ Disabled: Doesn't run

---

## Minimal Profile

**Use for:** Prototypes, spikes, early-stage services

**Philosophy:** Enforce code quality basics, allow rapid iteration on contracts and governance.

### Required Checks

Select these in GitHub → Settings → Branches → Branch Protection:

- `Lints`
- `Nix Flake Check`
- `MSRV`

### Advisory Checks

These run but don't block merges:
- All contract checks (OpenAPI, Proto, DB, Events)
- All governance checks (ACs, Gherkin, Features, Flags, Privacy)
- Coverage
- Security

### When to Use

- First 2-4 weeks of a new service
- Proof-of-concept work
- When contracts are still in flux

### When to Graduate

Move to **Standard** when:
- Service has real users (even internal)
- Contracts are semi-stable
- You want to prevent accidental breakage

---

## Standard Profile

**Use for:** Production services, stable APIs

**Philosophy:** Enforce contracts and governance, advisory on coverage.

### Required Checks

- `Lints`
- `Nix Flake Check`
- `MSRV`
- `OpenAPI`
- `Proto`
- `DB`
- `Events`
- `ACs`
- `Gherkin`
- `Features`
- `Flags`
- `Privacy`
- `Security`

### Advisory Checks

- `Coverage`
- `Docs`
- `Scope Guard`

### When to Use

- Any service with external API consumers
- Services in production
- When you want strong governance without perfection

### When to Graduate

Move to **Strict** when:
- Service is mission-critical
- You want maximum safety
- Team is mature enough to handle strict requirements

---

## Strict Profile

**Use for:** Mission-critical services, public APIs

**Philosophy:** Enforce everything.

### Required Checks

All checks are required:
- `Lints`
- `Nix Flake Check`
- `MSRV`
- `OpenAPI`
- `Proto`
- `DB`
- `Events`
- `ACs`
- `Gherkin`
- `Features`
- `Flags`
- `Privacy`
- `Security`
- `Coverage`
- `Docs`
- `Scope Guard`

### When to Use

- Public-facing APIs
- Payment/billing services
- Services with strict compliance requirements
- When downtime is very expensive

---

## How to Apply a Profile

### Step 1: Go to Branch Protection Settings

1. Navigate to your repository on GitHub
2. Settings → Branches
3. Click "Add rule" or edit existing rule for `main`

### Step 2: Enable Status Checks

1. ✅ Check "Require status checks to pass before merging"
2. ✅ Check "Require branches to be up to date before merging"

### Step 3: Select Required Checks

Click in the "Search for status checks" box and select checks based on your profile.

**For Minimal:**
- Lints
- Nix Flake Check
- MSRV

**For Standard:**
- Everything from Minimal, plus:
- OpenAPI
- Proto
- DB
- Events
- ACs
- Gherkin
- Features
- Flags
- Privacy
- Security

**For Strict:**
- All checks (search and select all visible check names)

### Step 4: Other Recommended Settings

- ✅ "Require a pull request before merging"
- ✅ "Require approvals" (1 for most teams, 2 for strict)
- ✅ "Dismiss stale pull request approvals when new commits are pushed"
- ❌ "Allow force pushes" (keep disabled)
- ❌ "Allow deletions" (keep disabled)

---

## Upgrading Between Profiles

### Minimal → Standard

1. Ensure all contract specs exist and are valid:
   - `specs/openapi/openapi.yaml`
   - `specs/proto/` (if using gRPC)
   - `specs/db/` (if using database)
   - `specs/events/` (if using events)
2. Ensure ledger and features are in good shape
3. Run full CI locally to verify all checks pass
4. Update branch protection to include new required checks
5. Announce to team with 1-2 day notice

### Standard → Strict

1. Get coverage above floor (default 60%)
2. Ensure docs build successfully
3. Ensure ScopeGuard passes (if enabled)
4. Run full CI locally
5. Update branch protection
6. Announce to team with 1-week notice (strict is a big change)

---

## FAQ

**Q: Can I have different profiles for different branches?**

A: Yes. Create separate branch protection rules for `main`, `staging`, `develop`, etc.

**Q: What if a required check is flaky?**

A: Disable it temporarily in branch protection, fix the flakiness, then re-enable. Never leave flaky checks required long-term.

**Q: Can I override a failed check?**

A: Admins can, but shouldn't. The point of required checks is to enforce quality. If a check is wrong, fix the check or the code, don't bypass it.

**Q: How do I test a profile change?**

A: Create a test branch with the new rules, open a PR, see what fails. Adjust before applying to `main`.
