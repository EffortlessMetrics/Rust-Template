# ADR-0006: Supply Chain Hardening via SLSA Provenance

**Status**: Accepted
**Date**: 2025-11-19
**Authors**: Steven Zimmerman
**Related ACs**: AC-TPL-006

---

## Context

Software supply chain attacks are increasing:

- **SolarWinds (2020)**: Compromised build system injected malware into trusted updates
- **Codecov (2021)**: Leaked credentials via modified bash uploader script
- **npm/PyPI typosquatting**: Malicious packages with names similar to popular libraries
- **Dependency confusion**: Attackers publish public packages matching internal names

Traditional defenses (code review, vulnerability scanning) don't address:

1. **Build-time tampering**: Artifacts differ from source (injected during CI)
2. **Artifact provenance**: No way to verify "this binary came from this commit"
3. **Dependency integrity**: Transitive dependencies aren't verified
4. **Reproducibility**: Can't independently rebuild and verify artifacts

We need:

- **Transparency**: Cryptographic proof of what was built, where, and from what source
- **Verification**: Users/auditors can validate artifacts match claimed provenance
- **Automation**: Built into CI, not manual signing processes
- **Standardization**: Industry-standard formats (SLSA, SPDX, Sigstore)

---

## Decision

We adopt **SLSA v1.0 Level 2** supply chain hardening via GitHub Artifact Attestations:

### What We Emit

For every tagged release (`v*.*.*`):

1. **Source tarball**: Reproducible archive via `git archive`
2. **SBOM (Software Bill of Materials)**: SPDX JSON format listing all dependencies
3. **Provenance attestation**: In-toto statement linking artifacts to source commit
4. **Build metadata**: GitHub Actions workflow, runner info, build environment

### How It's Generated

**Workflow**: `.github/workflows/ci-supply-chain.yml`

**Trigger**:
```yaml
on:
  push:
    tags:
      - 'v*.*.*'
```

**Build environment**: Nix devshell (matching local dev + CI)

**Steps**:
1. Checkout source at tagged commit
2. Setup Nix (DeterminateSystems/nix-installer-action@v9)
3. Build release artifacts: `nix develop -c cargo build --workspace --release`
4. Create source tarball: `git archive --format=tar.gz --prefix=rust-template-$TAG/ -o rust-template-$TAG.tar.gz $TAG`
5. Generate SBOM: `anchore/sbom-action@v0` (SPDX JSON, includes transitive deps)
6. Attest provenance: `actions/attest-build-provenance@v1` (signs with Sigstore)
7. Upload artifacts: tarball + SBOM to GitHub Releases

**Provenance includes**:
- Source repository and commit SHA
- Workflow file hash (detects workflow tampering)
- Build platform (ubuntu-latest, Nix version)
- Build steps executed
- Cryptographic signature (Sigstore/Fulcio)

### SLSA Level 2 Requirements

| Requirement | Implementation | Evidence |
|-------------|----------------|----------|
| **Build service** | GitHub Actions | Workflow logs, runner attestation |
| **Provenance generation** | `actions/attest-build-provenance@v1` | In-toto statement in GitHub Attestations API |
| **Provenance authenticated** | Sigstore (keyless signing) | Fulcio certificate, Rekor transparency log |
| **Provenance non-falsifiable** | Ephemeral OIDC token | GitHub's identity provider, no long-lived secrets |
| **Isolated build** | Separate GitHub Actions runner per job | Runner isolation enforced by GitHub |
| **Parameterless build** | Triggered only by git tags | No manual workflow_dispatch |
| **Hermetic build** | Nix devshell pins all deps | `flake.lock` ensures reproducibility |

**Why Level 2 (not Level 3)?**

- **Level 3** requires hardware-isolated builds (e.g., dedicated build servers, no shared runners)
- GitHub Actions uses shared runners (security boundary is VM isolation)
- Level 2 is sufficient for most orgs; Level 3 is for high-security contexts (e.g., crypto libraries)

### Verification

**GitHub UI**:
1. Go to repo **Security** tab → **Provenance**
2. View attestations for each release
3. Inspect SBOM, provenance metadata

**GitHub CLI**:
```bash
# Download artifact
gh release download v2.4.0 --pattern 'rust-template-*.tar.gz'

# Verify provenance
gh attestation verify rust-template-v2.4.0.tar.gz --owner EffortlessMetrics

# Output:
# ✓ Verified artifact against repository EffortlessMetrics/Rust-Template
# ✓ Built from commit abc1234 on 2025-01-19
# ✓ Workflow: .github/workflows/ci-supply-chain.yml@refs/tags/v2.4.0
```

**Manual verification (SLSA verifier)**:
```bash
# Install SLSA verifier
go install github.com/slsa-framework/slsa-verifier/v2/cli/slsa-verifier@latest

# Verify against source
slsa-verifier verify-artifact \
  rust-template-v2.4.0.tar.gz \
  --provenance-path provenance.json \
  --source-uri github.com/EffortlessMetrics/Rust-Template
```

---

## Alternatives Considered

### Alternative 1: Manual GPG Signing

**Approach**: Maintainer signs releases with personal GPG key.

**Pros**:
- Simple, widely understood
- Standard in many OSS projects

**Cons**:
- **Key management burden**: Maintainer must secure private key
- **Single point of failure**: If key compromised, all releases suspect
- **Not automated**: Requires manual signing step
- **No build provenance**: Only proves "maintainer signed this", not "built from this commit"

**Why rejected**: Doesn't address build-time tampering, doesn't scale to teams.

### Alternative 2: Cosign (Sigstore) Standalone

**Approach**: Use Cosign to sign artifacts without GitHub Attestations.

**Pros**:
- Sigstore ecosystem (keyless signing, transparency log)
- Works outside GitHub

**Cons**:
- **Manual setup**: Need to configure Cosign in workflow
- **Less integration**: GitHub Attestations are first-class in GitHub UI
- **Reinventing workflow**: `actions/attest-build-provenance` wraps Cosign best practices

**Why rejected**: GitHub Attestations provide same benefits with less complexity.

### Alternative 3: Full Reproducible Builds (SLSA Level 4)

**Approach**: Bit-for-bit reproducible builds, multiple independent rebuilds verify.

**Pros**:
- Highest assurance (can rebuild from source, compare hashes)
- Detects any tampering (even in compiler)

**Cons**:
- **Complexity**: Requires deterministic builds (Rust supports this, but setup is hard)
- **Tooling**: Need rebuild infrastructure, hash comparison system
- **Overkill for template**: Level 4 is for security-critical software (Debian, Tor)

**Why rejected**: Level 2 provides 80% of benefits with 20% of effort.

### Alternative 4: CycloneDX SBOM Only (No Provenance)

**Approach**: Generate SBOM, skip provenance attestation.

**Pros**:
- Simpler (one step instead of two)
- Satisfies vulnerability scanning use case

**Cons**:
- **No source linking**: SBOM doesn't prove "built from commit X"
- **No tamper detection**: Can't verify artifact matches source
- **Incomplete**: SBOM is dependency list, not build audit trail

**Why rejected**: Provenance is the key security property; SBOM is complementary.

---

## Consequences

### Positive

- **Verifiable provenance**: Anyone can verify artifacts match source commit
- **Tamper detection**: Altered artifacts fail verification
- **Supply chain transparency**: SBOM lists all dependencies (useful for vulnerability scanning)
- **Industry standard**: SLSA framework, SPDX format, Sigstore signing
- **No secrets management**: Keyless signing via OIDC (GitHub's identity provider)
- **Audit trail**: Rekor transparency log provides immutable history

### Negative

- **GitHub lock-in**: Attestations API is GitHub-specific (though provenance format is portable)
- **CI overhead**: Release builds take ~2min longer (SBOM generation, signing)
- **Not Level 3**: Shared runners don't meet isolated build requirement
- **SBOM maintenance**: If dependencies change, SBOM must regenerate (automated, but adds noise)

### Neutral

- **Verification burden shifts**: Users who care about provenance must run `gh attestation verify`
- **Not retroactive**: Only applies to releases after this ADR (v2.4.0+)
- **SBOM format choice**: SPDX JSON is one of many (CycloneDX is alternative)

---

## Compliance

**Automated:**

- Workflow triggers on git tags (`v*.*.*`)
- SBOM generation via `anchore/sbom-action@v0` (official GitHub action)
- Provenance via `actions/attest-build-provenance@v1` (official GitHub action)
- Uploaded to GitHub Releases (accessible via Security tab)

**Manual:**

- Release engineer tags commit: `git tag v2.4.0 && git push --tags`
- Workflow runs automatically (no manual steps)
- Verify attestation after release: `gh attestation verify <artifact>`

**Detection:**

- If workflow fails, release is blocked (no artifacts uploaded)
- If provenance verification fails, artifact is suspect (manual investigation)

**Future enforcement:**

- Block PRs that modify `.github/workflows/ci-supply-chain.yml` without ADR update
- Policy test (Rego) to validate workflow structure:
  ```rego
  # Ensure provenance step exists
  deny[msg] {
    input.jobs.sbom_provenance.steps[_].uses != "actions/attest-build-provenance@v1"
    msg := "Release workflow must include provenance attestation"
  }
  ```

---

## Implementation Notes

### Nix Integration

**Why Nix for release builds?**

- **Hermetic**: All dependencies pinned via `flake.lock`
- **Reproducible**: Same inputs → same outputs (modulo timestamps)
- **Matches dev**: Releases built with same tools as local dev
- **No drift**: CI can't diverge from dev environment over time

**Build command**:
```bash
nix develop -c cargo build --workspace --release
```

This uses the Nix devshell (not system Cargo), ensuring:
- Rust toolchain version matches `flake.nix`
- System dependencies (openssl, etc.) match `flake.nix`
- Build flags consistent with local dev

**Why not `nix build`?**

- Template is Rust-first, not Nix-first (Cargo is primary build tool)
- `nix build` requires Nix derivation (more complex than devshell)
- `nix develop -c cargo build` is simpler, still hermetic

### Artifact Scope

**What gets attested?**

- **Source tarball**: `rust-template-v2.4.0.tar.gz` (from `git archive`)
- **SBOM**: `rust-template-v2.4.0-sbom.spdx.json` (from `anchore/sbom-action`)

**What does NOT get attested (yet)?**

- **Binaries**: Not distributed (users build from source or crates.io)
- **Docker images**: Template doesn't include Docker build (see future enhancements)
- **Crate publish**: `cargo publish` to crates.io (out of scope for this ADR)

**Rationale**: Template is consumed as source (via GitHub template or git clone), not as binary.

### SBOM Contents

**anchore/sbom-action** generates SPDX JSON including:

- **Direct dependencies**: From `Cargo.toml` (all workspace crates)
- **Transitive dependencies**: From `Cargo.lock` (full dependency graph)
- **Metadata**: Licenses, versions, source URLs
- **Relationships**: Which package depends on which

**Example snippet**:
```json
{
  "spdxVersion": "SPDX-2.3",
  "packages": [
    {
      "name": "tokio",
      "versionInfo": "1.40.0",
      "licenseConcluded": "MIT",
      "externalRefs": [
        {"referenceType": "purl", "referenceLocator": "pkg:cargo/tokio@1.40.0"}
      ]
    }
  ]
}
```

**Use cases**:
- Vulnerability scanning (correlate SBOM with CVE databases)
- License compliance (audit transitive licenses)
- Dependency review (see full supply chain)

### Provenance Format

**In-toto statement** (SLSA v1.0):
```json
{
  "_type": "https://in-toto.io/Statement/v1",
  "subject": [
    {
      "name": "rust-template-v2.4.0.tar.gz",
      "digest": {
        "sha256": "abc123..."
      }
    }
  ],
  "predicateType": "https://slsa.dev/provenance/v1",
  "predicate": {
    "buildDefinition": {
      "buildType": "https://slsa.dev/github-actions/v1",
      "externalParameters": {
        "workflow": {
          "ref": "refs/tags/v2.4.0",
          "path": ".github/workflows/ci-supply-chain.yml"
        }
      },
      "resolvedDependencies": [
        {
          "uri": "git+https://github.com/EffortlessMetrics/Rust-Template@refs/tags/v2.4.0",
          "digest": {"sha1": "commit-sha"}
        }
      ]
    },
    "runDetails": {
      "builder": {
        "id": "https://github.com/actions/runner/v2.xyz"
      },
      "metadata": {
        "invocationId": "https://github.com/EffortlessMetrics/Rust-Template/actions/runs/123"
      }
    }
  }
}
```

**Signature**: Sigstore/Fulcio certificate chain, stored in Rekor transparency log.

### Permissions Required

**Workflow permissions**:
```yaml
permissions:
  contents: read          # Read source code
  id-token: write         # OIDC token for Sigstore
  attestations: write     # Write to GitHub Attestations API
```

**Why `id-token: write`?**

- Sigstore uses **keyless signing** via OIDC
- GitHub Issues ephemeral identity token (valid for workflow run)
- Fulcio CA signs certificate proving "this artifact was built by EffortlessMetrics/Rust-Template on GitHub Actions"
- No long-lived secrets, no key rotation

### Extension Points for Organizations

**If you're using this template in a production service:**

1. **Add binary attestation**: If you distribute binaries (e.g., via GitHub Releases):
   ```yaml
   - name: Attest binary
     uses: actions/attest-build-provenance@v1
     with:
       subject-path: target/release/my-service
   ```

2. **Add Docker image attestation**: If you build Docker images:
   ```yaml
   - name: Build image
     run: docker build -t my-service:$TAG .

   - name: Attest image
     uses: actions/attest-build-provenance@v1
     with:
       subject-name: ghcr.io/my-org/my-service
       subject-digest: sha256:$IMAGE_DIGEST
   ```

3. **Require verification in CI**: Block deployment unless artifacts verify:
   ```yaml
   - name: Verify artifact
     run: gh attestation verify my-service --owner my-org
   ```

4. **Integrate with policy engine**: Use Rego to enforce provenance checks:
   ```rego
   # Only allow artifacts with valid provenance
   deny[msg] {
     not input.provenance.verified
     msg := "Deployment artifact must have verified provenance"
   }
   ```

---

## Migration Path

**For existing services using this template:**

1. **Upgrade to v2.4.0**: Merge this ADR and workflow
2. **Tag next release**: `git tag v2.4.1 && git push --tags`
3. **Verify workflow**: Check GitHub Actions run succeeds
4. **Inspect attestation**: GitHub Security tab → Provenance
5. **Update deployment docs**: Add verification step to runbook

**No changes required to:**
- Build process (still `cargo build`)
- Release process (still git tagging)
- Artifact distribution (still GitHub Releases)

**Provenance is additive**: Existing workflows continue working; verification is opt-in.

---

## Future Enhancements

1. **SLSA Level 3**: Dedicated build runners (self-hosted, hardware-isolated)
   - Requires infrastructure investment
   - Needed only for high-security contexts

2. **Reproducible builds**: Bit-for-bit determinism
   - Requires controlling all build inputs (timestamps, randomness)
   - Enables independent verification by rebuilding

3. **Crate provenance**: Extend to crates.io publishes
   - `cargo publish` doesn't support attestations yet
   - Track [RFC 3691: Crate Signing](https://github.com/rust-lang/rfcs/pull/3691)

4. **Vulnerability scanning**: Correlate SBOM with CVE databases
   - GitHub Dependency Graph already does this for Cargo.lock
   - SBOM enables org-specific scanners (e.g., Snyk, Grype)

5. **Binary signing for distribution**: If template evolves to distribute binaries
   - Cosign signing for released binaries
   - Homebrew/apt package signing

---

## Notes

**Why SPDX instead of CycloneDX?**

- **SPDX**: ISO standard (ISO/IEC 5962:2021), Linux Foundation backed
- **CycloneDX**: OWASP project, more opinionated for security use cases

Both are valid; SPDX is more general-purpose, CycloneDX is security-focused. `anchore/sbom-action` supports both; we chose SPDX for broader compatibility.

**Why GitHub Attestations instead of standalone Sigstore?**

- **Integration**: First-class GitHub UI, Security tab
- **Simplicity**: `actions/attest-build-provenance@v1` wraps Sigstore best practices
- **Accessibility**: No additional tooling for basic verification (`gh attestation verify`)

Underlying technology is the same (Sigstore/Fulcio/Rekor), just different UX.

**What if GitHub shuts down Attestations API?**

- Provenance format is **portable** (SLSA in-toto statement)
- Signature is in **Rekor transparency log** (permanent, public)
- Can migrate to standalone Cosign/SLSA verifier with same artifacts

**References:**

- [SLSA Framework](https://slsa.dev/)
- [GitHub Artifact Attestations](https://docs.github.com/en/actions/security-guides/using-artifact-attestations-to-establish-provenance-for-builds)
- [Sigstore](https://www.sigstore.dev/)
- [SPDX Specification](https://spdx.dev/specifications/)
- [In-toto Attestations](https://in-toto.io/)
- [SLSA Verifier](https://github.com/slsa-framework/slsa-verifier)
