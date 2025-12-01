<!-- doclint:disable orphan-version -->
# ADR-0007: Dependency & Security Health

**Status:** Accepted  
**Date:** 2025-11-19  
**Authors:** System Architecture Team

## Context

Modern software development, especially when accelerated by LLMs, introduces several dependency-related risks:

1. **Velocity**: Fast iteration encourages "just add a crate" without auditing
2. **Surface Area**: Template includes many dependencies (web server, database, observability, testing)
3. **Supply Chain**: Dependencies themselves have dependencies (transitive risk)
4. **Compliance**: Organizations need license policy enforcement
5. **LLM Characteristics**: AI coding assistants may suggest outdated or vulnerable crates

While we have robust testing (`selftest`) and spec validation (`ac-status`, `adr-check`), we lacked a **first-class security gate** for dependencies.

## Decision

We introduce `cargo xtask audit` as a **standard, required check** for all services built from this template:

### Tools

- **`cargo-audit`**: RustSec/OSV advisory database checks
- **`cargo-deny`**: License, ban, source, and duplication policy enforcement
- **Nix-provided**: Both tools in devshell for hermetic reproducibility

### Policy (`deny.toml`)

```toml
[advisories]
vulnerability = "deny"      # Hard fail on known CVEs
yanked = "deny"            # Fail on yanked crates
unmaintained = "warn"      # Warn on unmaintained

[licenses]
unlicensed = "deny"        # All deps must have licenses
allow = ["MIT", "Apache-2.0", "BSD-3-Clause", ...]
deny = ["GPL-3.0", ...]    # Viral licenses

[bans]
multiple-versions = "warn"  # Prefer single version per crate
```

### Workflow Integration

- **`cargo xtask audit`**: Local/CI command
- **`cargo xtask release-verify`**: Includes audit as gate
- **CI**: Separate `ci-security.yml` workflow (can be scheduled/required)

## Consequences

###  Benefits

1. **Early Detection**: Catch vulnerabilities before merge/deploy
2. **Policy as Code**: License policy is explicit, versioned, auditable
3. **LLM-Safe**: AI-suggested crates are automatically screened
4. **Organizational Hardening**: Easy to extend policy per compliance needs
5. **Supply Chain Transparency**: Pairs with existing SBOM + provenance workflows

### Trade-offs

1. **`audit` ≠ `selftest`**: Can fail independently
   - **Design**: This is intentional. A service can be functionally correct but have vulnerable deps.
   - **Impact**: PRs may need two passes (fix tests, fix audit).

2. **False Positives**: Advisories may not affect actual usage
   - **Mitigation**: `deny.toml` allows exemptions via `skip` or `ignore`.

3. **Maintenance**: Policy requires curation
   - **Responsibility**: Template maintainers update `deny.toml` as ecosystem evolves.

### Example Workflow

```bash
# Developer adds new crate
cargo add some-crate

# Before committing
cargo xtask audit
# ❌ "some-crate 1.0.0 has HIGH severity CVE-2024-XXXX"

# Fix: update to patched version
cargo update some-crate

cargo xtask audit
# ✅ All checks passed

# Later, during release
cargo xtask release-verify
# Runs: selftest + audit + docs-check
```

## Alternatives Considered

### 1. Merge `audit` into `selftest`

**Rejected**: `selftest` is "does the service work as specified." Security is orthogonal.

**Reason**: Keeps concerns separated; allows CI to schedule security checks differently (e.g., nightly for full scan, PR for blocking).

### 2. Manual Review Only

**Rejected**: Doesn't scale; LLM velocity makes this infeasible.

### 3. Use GitHub Dependabot Only

**Rejected**: Dependabot is useful but:
- Limited to GitHub ecosystem
- Doesn't enforce license policy
- Doesn't fail builds directly

**Better**: Use `audit` + Dependabot together (complementary).

## Related

- **ADR-0004**: Supply Chain Hardening (SBOM, provenance)
- **ADR-0002**: Nix-first Development (tools provisioning)
- **`deny.toml`**: Policy configuration
- **`ci-security.yml`**: CI enforcement
- **`docs/explanation/supply-chain-hardening.md`**: End-to-end security docs

## References

- [RustSec Advisory Database](https://rustsec.org/)
- [cargo-deny Documentation](https://embarkstudios.github.io/cargo-deny/)
- [OSV (Open Source Vulnerabilities)](https://osv.dev/)
