# Reference: Required Checks

This document describes the CI checks that should be required for branch protection on `main`.

> **Source of Truth**: [`REQUIRED_CHECKS.md`](../../REQUIRED_CHECKS.md) at the repository root.

---

## Check Categories

### Contracts / Interfaces

These checks validate that API contracts are consistent and valid:

| Check | Purpose | When Required |
|-------|---------|---------------|
| `OpenAPI` | Validates OpenAPI spec syntax and breaking changes | If `specs/openapi/` exists |
| `Proto` | Validates Protocol Buffer definitions | If `specs/proto/` exists |
| `EventSchemas` | Validates event schema definitions | If event schemas are used |
| `DB` | Validates database migrations | If migrations exist |
| `Privacy` | Validates privacy policy compliance | If `specs/privacy.yaml` is active |

### Behavior / Tests

These checks validate that code behaves correctly:

| Check | Purpose | When Required |
|-------|---------|---------------|
| `Lints` | Runs `cargo fmt`, `cargo clippy`, unit tests | Always |
| `ACs` | Runs BDD acceptance tests | Always (for governed repos) |
| `MSRV` | Tests against Minimum Supported Rust Version | Always |

### Policy / Governance

These checks validate governance compliance:

| Check | Purpose | When Required |
|-------|---------|---------------|
| `Features` | Validates feature flag configuration | If feature flags are used |
| `Flags` | Validates flag declarations | If feature flags are used |
| `PolicyVerify` | Runs OPA/Rego policy tests | Always (for governed repos) |
| `Nix Flake Check` | Validates Nix flake configuration | If using Nix |
| `Security` | Runs security scanning (deps, secrets) | Recommended always |
| `ScopeGuard` | Validates change scope boundaries | Optional, for strict governance |
| `Docs` | Validates documentation governance | If docs are external contract |

### Advisory Jobs (Usually Not Required)

These jobs provide information but shouldn't block merges:

| Check | Purpose | Notes |
|-------|---------|-------|
| `Coverage` | Generates coverage reports | Tag-only; don't require on PRs |
| `FlagsWarn` | Checks for stale feature flags | Weekly scheduled job |
| `Maintenance – Pin Actions` | Updates pinned action versions | Scheduled maintenance |
| `Release` | SBOM generation and signing | Tag-only |

---

## Branch Protection Profiles

Choose a profile based on your governance needs:

### Profile: Minimal

For internal services with low regulatory requirements.

**Required checks:**
- `Lints`
- `MSRV`
- `Nix Flake Check`

**Optional checks:**
- `Coverage` (tag-only)
- Contract checks where specs exist
- `Security`

### Profile: Standard

For most production services.

**Required checks:**
- All from Minimal, plus:
- Contract checks where specs exist (`OpenAPI`, `Proto`, `DB`, `EventSchemas`, `Privacy`)
- `ACs`, `Gherkin`, `Features`, `Flags`, `PolicyVerify`
- `Security` (at least deps + secrets)

**Optional checks:**
- `Coverage` (tag-only)
- `Docs`

### Profile: Strict

For services with compliance requirements or external contracts.

**Required checks:**
- All from Standard, plus:
- `ScopeGuard` (if enabled)
- Full `Security` (CodeQL + deps + secrets)
- `Docs`

**Optional checks:**
- `Coverage` (tag-only; enforce on release tags, not PRs)

---

## Configuring Branch Protection

To enable these checks in GitHub:

1. Go to **Settings > Branches > Branch protection rules**
2. Add rule for `main` branch
3. Enable **Require status checks to pass before merging**
4. Search and add the required checks for your profile

Or use the setup script:

```bash
.github/scripts/setup-branch-protection.sh
```

---

## See Also

- **[branch-protection-profiles.md](branch-protection-profiles.md)** - Detailed profile configuration
- **[ci-workflows.md](ci-workflows.md)** - CI workflow reference
- **[how-to/setup-branch-protection.md](../how-to/setup-branch-protection.md)** - Setup guide
