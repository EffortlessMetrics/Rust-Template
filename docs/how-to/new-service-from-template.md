# How To: Create a New Service from Template

This guide walks you through creating a new service from the Rust Template in **under 10 minutes**.

---

## Prerequisites

- GitHub account with permissions to create repositories
- Git installed locally
- Nix installed (for devShell)

---

## Step 1: Create Repository from Template (2 min)

### On GitHub

1. Navigate to the template repository
2. Click "Use this template" button (top right)
3. Choose "Create a new repository"
4. Fill in:
   - **Repository name:** `your-service-name`
   - **Description:** Brief service description
   - **Visibility:** Private (recommended) or Public
5. Click "Create repository"

### Clone Locally

```bash
git clone git@github.com:your-org/your-service-name.git
cd your-service-name
```

---

## Step 2: Update Ownership & Metadata (3 min)

### Update CODEOWNERS

Edit `.github/CODEOWNERS`:

```
# Replace with your team
* @your-org/your-team
```

### Update Cargo Workspace

Edit `Cargo.toml`:

```toml
[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Your Team <team@company.com>"]
repository = "https://github.com/your-org/your-service-name"
```

### Update Flags Ownership

Edit `flags/registry.yaml`:

```yaml
flags:
  - key: refunds_v2
    owner: your-team-name  # Change this
    default: false
    expires_at: 2026-06-30
```

### Update Privacy Ownership

Edit `specs/privacy.yaml`:

```yaml
fields:
  - path: user.email
    classification: PII
    owner: your-team-name  # Change this
    retention: "365d"
    purpose: "User authentication"
```

---

## Step 3: Verify Setup Works (3 min)

### Enter Nix Shell

```bash
nix develop
```

**Expected:** Drops you into a shell with all tools available.

If Nix isn't available:
```bash
# macOS
sh <(curl -L https://nixos.org/nix/install)

# Linux
sh <(curl -L https://nixos.org/nix/install) --daemon
```

### Run All Checks

```bash
cargo run -p xtask -- check
```

**Expected output:**
```
Running format check...
Running clippy...
Running tests...
✓ All checks passed
```

If this fails, review the error messages. Common issues:
- Rust version mismatch
- Missing dependencies (Nix should handle)

### Run Acceptance Tests

```bash
cargo run -p xtask -- bdd
```

**Expected output:**
```
Feature: Refunds
  Scenario: Create a refund
   ✔  Given an order "ORD-1" totalling 5000 cents
   ✔  When I POST /refunds with { "orderId": "ORD-1", "amountCents": 5000 }
   ✔  Then I receive 201 with a "refundId"
[Summary]
1 feature
1 scenario (1 passed)
3 steps (3 passed)
✓ Acceptance tests passed
JUnit output: target/junit/acceptance.xml
```

### Generate AC Status

```bash
cargo run -p xtask -- ac-status
```

**Expected output:**
```
✓ Generated /home/user/your-service-name/docs/feature_status.md
✓ All ACs passed
```

### Create LLM Bundle

```bash
cargo run -p xtask -- bundle implement_ac
```

**Expected output:**
```
Building context bundle: implement_ac
  Max size: 250000 bytes
  Description: Context for implementing an AC: ledger, specs, features, and core code
  Files included: 4
  Bundle size: 986 bytes

Bundle written to: .llm/bundle/implement_ac.md
✓ Bundle generated: .llm/bundle/implement_ac.md
```

**If all four commands succeed, your setup is correct!**

---

## Step 4: Configure Branch Protection (2 min)

### Choose Your Profile

- **Minimal:** Prototypes, early-stage (recommended for new services)
- **Standard:** Production services with stable contracts
- **Strict:** Mission-critical services

See: [Branch Protection Profiles](../reference/branch-protection-profiles.md)

### Apply Minimal Profile (Recommended Start)

1. GitHub → Settings → Branches
2. Click "Add rule"
3. Branch name pattern: `main`
4. Check:
   - ✅ "Require status checks to pass before merging"
   - ✅ "Require branches to be up to date before merging"
5. Search and select these required checks:
   - `Lints`
   - `Nix Flake Check`
   - `MSRV`
6. Click "Create" or "Save changes"

### Other Settings

Also enable:
- ✅ "Require a pull request before merging"
- ✅ "Require approvals" (1)
- ✅ "Dismiss stale pull request approvals when new commits are pushed"

---

## Step 5: Commit Initial Customization

```bash
git add -A
git commit -m "Customize template for your-service-name

- Update CODEOWNERS to @your-org/your-team
- Update Cargo.toml metadata
- Update ownership in flags and privacy specs
- Verified: xtask check, bdd, ac_status, and bundler all pass
"
git push origin main
```

---

## Step 6: Verify CI Works on GitHub

1. Create a test branch:
   ```bash
   git checkout -b test/ci-verification
   echo "# Test" >> README.md
   git add README.md
   git commit -m "Test CI"
   git push origin test/ci-verification
   ```

2. Open a Pull Request on GitHub

3. Watch CI checks run (should take 5-10 minutes)

4. Verify these checks appear and pass:
   - ✅ Lints
   - ✅ Nix Flake Check
   - ✅ MSRV
   - ✅ Template Self-Test
   - (plus others in advisory mode)

5. Merge or close the PR

---

## What You Have Now

✅ A working Rust service template with:
- Comprehensive CI (22 workflows)
- AC-driven development (ledger + BDD)
- Policy enforcement (Rego)
- LLM context bundling
- Single `xtask` CLI for all operations

✅ Branch protection configured (Minimal profile)

✅ Verified working: local commands + CI

---

## Next Steps

### Customize for Your Domain

1. **Update the example:**
   - Replace `refund.feature` with your domain (e.g., `user-registration.feature`)
   - Update ACs in `specs/spec_ledger.yaml`
   - Implement your domain logic in `crates/core`

2. **Add OpenAPI spec:**
   - Edit `specs/openapi/openapi.yaml`
   - Define your API endpoints
   - CI will catch breaking changes automatically

3. **Add first real AC:**
   - Follow: [First AC Change Tutorial](../tutorials/first-ac-change.md)

### Upgrade Profile When Ready

- **After 2-4 weeks:** Upgrade to **Standard** profile
- **When mission-critical:** Consider **Strict** profile

See: [Branch Protection Profiles](../reference/branch-protection-profiles.md)

### Integrate with Your Infra

- **Secrets:** Add to GitHub Settings → Secrets (for Schema Registry, DB, etc.)
- **Deployment:** Wire up your CD pipeline to trigger on `main` pushes
- **Monitoring:** Integrate observability tools

---

## Common Issues

### "Nix command not found"

**Fix:** Install Nix:
```bash
# macOS
sh <(curl -L https://nixos.org/nix/install)

# Linux
sh <(curl -L https://nixos.org/nix/install) --daemon
```

### "xtask not found"

**Fix:** Run from workspace root (where `Cargo.toml` is):
```bash
cargo run -p xtask -- check
```

### "conftest not found"

**Fix:** Ensure you're in `nix develop` shell:
```bash
nix develop
conftest --version
```

### CI fails but local works

**Fix:**
- Check if you need secrets configured in GitHub
- Verify workflows in `.github/workflows/` match your expectations
- Look at specific failing step in GitHub Actions logs

---

## Help & Support

- **Template API:** See [TEMPLATE_API.md](../../TEMPLATE_API.md) for all stable interfaces
- **AC Development:** See [First AC Change Tutorial](../tutorials/first-ac-change.md)
- **Branch Protection:** See [Branch Protection Profiles](../reference/branch-protection-profiles.md)
- **Issues:** Open an issue in the template repository
