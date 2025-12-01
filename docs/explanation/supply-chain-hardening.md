---
id: EXPLANATION-TPL-SUPPLY-CHAIN-001
title: Supply Chain Hardening
doc_type: explanation
status: published
audience: developers, maintainers, security-engineers
tags: [security, supply-chain, sbom, slsa, attestation]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-SECURITY-GOVERNANCE, REQ-PLT-RELEASE-SAFETY]
acs: [AC-PLT-008, AC-PLT-012]
adrs: [ADR-0006, ADR-0007]
last_updated: 2025-11-26
---
<!-- doclint:disable orphan-version -->

# Supply Chain Hardening

**Version**: v3.3.6
**Last Updated**: 2025-11-19

This template implements **supply chain hardening** via SLSA v1.0 Level 2 provenance and SBOM generation, providing cryptographic proof that released artifacts match their source code.

---

## What We Emit

For every tagged release (`v*.*.*`), the template generates:

### 1. Source Tarball

**File**: `rust-template-v3.3.6.tar.gz`

**Purpose**: Reproducible source archive created via `git archive`.

**Contents**:
- All source code at the tagged commit
- `Cargo.toml`, `Cargo.lock`, `flake.nix`, `flake.lock`
- Specs, docs, tests
- Excludes: `.git/`, `target/`, build artifacts

**Why reproducible?**
- `git archive` creates identical tarballs for the same commit (modulo timestamps)
- Enables independent verification by rebuilding

**Example**:
```bash
# Download from GitHub Releases
gh release download v3.3.6 --pattern 'rust-template-*.tar.gz'

# Verify integrity
sha256sum rust-template-v3.3.6.tar.gz
# Compare against provenance attestation
```

---

### 2. SBOM (Software Bill of Materials)

**File**: `rust-template-v3.3.6-sbom.spdx.json`

**Format**: SPDX 2.3 (ISO/IEC 5962:2021 standard)

**Purpose**: Machine-readable inventory of all dependencies.

**Contents**:
- **Direct dependencies**: All crates listed in `Cargo.toml` (workspace members)
- **Transitive dependencies**: Full dependency graph from `Cargo.lock`
- **Metadata**: Versions, licenses (SPDX IDs), source URLs (crates.io)
- **Relationships**: Dependency graph edges (which package depends on which)

**Example snippet**:
```json
{
  "spdxVersion": "SPDX-2.3",
  "dataLicense": "CC0-1.0",
  "SPDXID": "SPDXRef-DOCUMENT",
  "name": "rust-template-v3.3.6",
  "packages": [
    {
      "SPDXID": "SPDXRef-Package-tokio-1.40.0",
      "name": "tokio",
      "versionInfo": "1.40.0",
      "licenseConcluded": "MIT",
      "downloadLocation": "https://crates.io/crates/tokio/1.40.0",
      "externalRefs": [
        {
          "referenceCategory": "PACKAGE-MANAGER",
          "referenceType": "purl",
          "referenceLocator": "pkg:cargo/tokio@1.40.0"
        }
      ]
    },
    {
      "SPDXID": "SPDXRef-Package-serde-1.0.210",
      "name": "serde",
      "versionInfo": "1.0.210",
      "licenseConcluded": "MIT OR Apache-2.0",
      "downloadLocation": "https://crates.io/crates/serde/1.0.210"
    }
  ],
  "relationships": [
    {
      "spdxElementId": "SPDXRef-Package-tokio-1.40.0",
      "relationshipType": "DEPENDS_ON",
      "relatedSpdxElement": "SPDXRef-Package-serde-1.0.210"
    }
  ]
}
```

**Use cases**:

1. **Vulnerability scanning**: Correlate SBOM with CVE databases (e.g., GitHub Dependency Graph, Grype, Snyk)
2. **License compliance**: Audit all transitive licenses (e.g., ensure no GPL in proprietary service)
3. **Supply chain audit**: Review full dependency tree for untrusted crates
4. **Policy enforcement**: Rego policies can validate SBOM (e.g., "no dependencies with AGPL license")

---

### 3. Provenance Attestation

**Format**: SLSA v1.0 in-toto statement (JSON)

**Purpose**: Cryptographic proof linking artifacts to source commit and build environment.

**Storage**: GitHub Attestations API (accessible via Security tab or `gh` CLI)

**Contents**:

**Subject** (what was built):
```json
{
  "subject": [
    {
      "name": "rust-template-v3.3.6.tar.gz",
      "digest": {
        "sha256": "a3f8e9d2b1c4f5e6d7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0"
      }
    }
  ]
}
```

**Predicate** (how it was built):
```json
{
  "predicateType": "https://slsa.dev/provenance/v1",
  "predicate": {
    "buildDefinition": {
      "buildType": "https://slsa.dev/github-actions/v1",
      "externalParameters": {
        "workflow": {
          "ref": "refs/tags/v3.3.6",
          "repository": "https://github.com/EffortlessMetrics/Rust-Template",
          "path": ".github/workflows/ci-supply-chain.yml"
        }
      },
      "resolvedDependencies": [
        {
          "uri": "git+https://github.com/EffortlessMetrics/Rust-Template@refs/tags/v3.3.6",
          "digest": {
            "sha1": "bca756c3d8e9f1a2b3c4d5e6f7a8b9c0d1e2f3a4"
          }
        }
      ]
    },
    "runDetails": {
      "builder": {
        "id": "https://github.com/actions/runner/v2.313.0"
      },
      "metadata": {
        "invocationId": "https://github.com/EffortlessMetrics/Rust-Template/actions/runs/12345678",
        "startedOn": "2025-01-19T12:34:56Z",
        "finishedOn": "2025-01-19T12:37:42Z"
      }
    }
  }
}
```

**Signature**:
- Signed via **Sigstore** (keyless signing using OIDC)
- Certificate issued by **Fulcio** (proves "GitHub Actions workflow EffortlessMetrics/Rust-Template built this")
- Stored in **Rekor** transparency log (immutable, public audit trail)

**What provenance proves**:

1. ✓ Artifact was built from specific commit (`resolvedDependencies.digest.sha1`)
2. ✓ Built using specific workflow (`.github/workflows/ci-supply-chain.yml`)
3. ✓ Built on GitHub Actions (not on compromised developer laptop)
4. ✓ Workflow wasn't tampered with (workflow file hash included)
5. ✓ Signature is authentic (Sigstore certificate chain verifies)

**What provenance does NOT prove**:

- ✗ Source code itself is trustworthy (you still need code review)
- ✗ Dependencies are safe (you still need vulnerability scanning)
- ✗ Build environment is unhackable (GitHub Actions runners are shared VMs)

---

## How It's Generated

### Workflow Overview

**Trigger**: Push git tag matching `v*.*.*`

```bash
git tag v3.3.6
git push --tags
# Workflow .github/workflows/ci-supply-chain.yml runs automatically
```

**Workflow file**: `.github/workflows/ci-supply-chain.yml`

**Steps**:

```yaml
name: Supply Chain
on:
  push:
    tags:
      - 'v*.*.*'

jobs:
  sbom-provenance:
    runs-on: ubuntu-latest
    permissions:
      contents: read          # Read source code
      id-token: write         # OIDC token for Sigstore
      attestations: write     # Write to GitHub Attestations API
    steps:
      # 1. Checkout source at tagged commit
      - uses: actions/checkout@v4

      # 2. Setup Nix (provides pinned Rust toolchain)
      - uses: DeterminateSystems/nix-installer-action@v9
      - uses: DeterminateSystems/magic-nix-cache-action@v2

      # 3. Build release artifacts (using Nix devshell)
      - name: Build release
        run: nix develop -c cargo build --workspace --release

      # 4. Create source tarball (reproducible via git archive)
      - name: Create source tarball
        run: |
          TAG=${GITHUB_REF#refs/tags/}
          git archive --format=tar.gz --prefix=rust-template-$TAG/ \
            -o rust-template-$TAG.tar.gz $TAG

      # 5. Generate SBOM (SPDX JSON format)
      - name: Generate SBOM
        uses: anchore/sbom-action@v0
        with:
          format: spdx-json
          artifact-name: rust-template-${{ github.ref_name }}-sbom.spdx.json

      # 6. Attest provenance (SLSA v1.0, signed via Sigstore)
      - name: Attest build provenance
        uses: actions/attest-build-provenance@v1
        with:
          subject-path: rust-template-*.tar.gz

      # 7. Upload artifacts to GitHub Releases
      - name: Upload artifacts
        uses: softprops/action-gh-release@v1
        with:
          files: |
            rust-template-*.tar.gz
            rust-template-*-sbom.spdx.json
```

**Why Nix for builds?**

- **Hermetic**: All dependencies pinned via `flake.lock` (Rust version, system libs)
- **Reproducible**: Same inputs → same outputs
- **Matches dev**: Release built with same toolchain as local development
- **No drift**: CI can't diverge from developer environment

**Build command**:
```bash
nix develop -c cargo build --workspace --release
```

This runs Cargo inside the Nix devshell, ensuring:
- Rust 1.85.0 (or whatever `flake.nix` specifies)
- System dependencies (OpenSSL, etc.) from Nix, not Ubuntu apt
- Build flags consistent across dev/CI

---

## How to Verify

### Option 1: GitHub UI (Easiest)

**Navigate to**:
```
https://github.com/EffortlessMetrics/Rust-Template/security
→ "Provenance" tab
```

**What you see**:
- List of all attested artifacts
- Provenance metadata (commit, workflow, timestamp)
- SBOM (click to view dependencies)
- Verification status (green checkmark if signature valid)

**Use case**: Quick manual inspection, auditor review.

---

### Option 2: GitHub CLI (Automated)

**Install GitHub CLI**:
```bash
# Ubuntu/Debian
sudo apt install gh

# macOS
brew install gh

# Authenticate
gh auth login
```

**Download artifact**:
```bash
gh release download v3.3.6 --pattern 'rust-template-*.tar.gz'
```

**Verify provenance**:
```bash
gh attestation verify rust-template-v3.3.6.tar.gz --owner EffortlessMetrics
```

**Output**:
```
✓ Verification succeeded!

Attestation subject name: rust-template-v3.3.6.tar.gz
Attestation subject digest: sha256:a3f8e9d2b1c4f5e6d7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0

Build source repository: https://github.com/EffortlessMetrics/Rust-Template
Build source commit: bca756c3d8e9f1a2b3c4d5e6f7a8b9c0d1e2f3a4
Build workflow: .github/workflows/ci-supply-chain.yml@refs/tags/v3.3.6
```

**What this proves**:
- Tarball SHA matches provenance (artifact wasn't altered after signing)
- Built from commit `bca756c` (you can inspect source at that commit)
- Built via GitHub Actions (not on untrusted machine)

**Use case**: CI/CD pipelines, automated deployment verification.

---

### Option 3: SLSA Verifier (Manual, Portable)

**Install SLSA verifier**:
```bash
go install github.com/slsa-framework/slsa-verifier/v2/cli/slsa-verifier@latest
```

**Download provenance**:
```bash
# GitHub Attestations API
gh attestation download \
  --predicate-type https://slsa.dev/provenance/v1 \
  --artifact rust-template-v3.3.6.tar.gz \
  --output provenance.json
```

**Verify**:
```bash
slsa-verifier verify-artifact \
  rust-template-v3.3.6.tar.gz \
  --provenance-path provenance.json \
  --source-uri github.com/EffortlessMetrics/Rust-Template \
  --source-tag v3.3.6
```

**Output**:
```
Verified signature against tlog entry index 12345 at URL: https://rekor.sigstore.dev/...
Verified build using builder "https://github.com/slsa-framework/slsa-github-generator/.github/workflows/generator_generic_slsa3.yml@refs/tags/v1.0.0" at commit bca756c
Verifying artifact rust-template-v3.3.6.tar.gz: PASSED

PASSED: Verified SLSA provenance
```

**Use case**: Environments without GitHub CLI, compliance audits.

---

### Option 4: Policy Enforcement (Rego)

**Policy**: `policies/supply_chain.rego`

```rego
package supply_chain

# Deny deployment if artifact lacks verified provenance
deny[msg] {
    not input.provenance.verified
    msg := "Artifact must have verified SLSA provenance"
}

# Deny if build wasn't from GitHub Actions
deny[msg] {
    not contains(input.provenance.builder.id, "github.com/actions")
    msg := "Artifact must be built on GitHub Actions"
}

# Deny if SBOM missing
deny[msg] {
    not input.sbom
    msg := "Artifact must include SBOM"
}

# Warn if SBOM has high-severity CVEs
warn[msg] {
    vuln := input.sbom.vulnerabilities[_]
    vuln.severity == "CRITICAL"
    msg := sprintf("SBOM contains critical vulnerability: %s", [vuln.id])
}
```

**Usage**:
```bash
# Before deploying artifact, validate provenance + SBOM
conftest test -p policies/supply_chain.rego <(gh attestation verify rust-template-v3.3.6.tar.gz --format json)
```

**Use case**: Automated deployment gates, policy-as-code enforcement.

---

## How to Extend in Your Organization

### Extension 1: Add Binary Attestation

If you distribute compiled binaries (e.g., via GitHub Releases):

**Workflow change**:
```yaml
- name: Build release binary
  run: nix develop -c cargo build --release --bin my-service

- name: Attest binary provenance
  uses: actions/attest-build-provenance@v1
  with:
    subject-path: target/release/my-service

- name: Upload binary
  uses: softprops/action-gh-release@v1
  with:
    files: target/release/my-service
```

**Verification**:
```bash
gh release download v3.3.6 --pattern my-service
gh attestation verify my-service --owner my-org
```

---

### Extension 2: Add Docker Image Attestation

If you build Docker images:

**Workflow change**:
```yaml
- name: Build Docker image
  run: docker build -t my-service:${{ github.ref_name }} .

- name: Push to registry
  run: docker push my-service:${{ github.ref_name }}

- name: Get image digest
  id: digest
  run: |
    DIGEST=$(docker inspect --format='{{index .RepoDigests 0}}' my-service:${{ github.ref_name }} | cut -d@ -f2)
    echo "digest=$DIGEST" >> $GITHUB_OUTPUT

- name: Attest image provenance
  uses: actions/attest-build-provenance@v1
  with:
    subject-name: ghcr.io/my-org/my-service
    subject-digest: ${{ steps.digest.outputs.digest }}
```

**Verification**:
```bash
gh attestation verify \
  ghcr.io/my-org/my-service@sha256:abc123... \
  --owner my-org
```

---

### Extension 3: Require Verification in Deployment

Block deployment unless artifact provenance verifies:

**Kubernetes admission controller** (OPA Gatekeeper):
```rego
package kubernetes.admission

deny[msg] {
    input.request.kind.kind == "Deployment"
    image := input.request.object.spec.template.spec.containers[_].image
    not verified_provenance(image)
    msg := sprintf("Image %s lacks verified SLSA provenance", [image])
}

verified_provenance(image) {
    # Call GitHub Attestations API or check local cache
    # Implementation depends on your setup
}
```

**CI/CD pipeline** (GitHub Actions):
```yaml
- name: Download artifact
  run: gh release download $TAG --pattern my-service

- name: Verify provenance
  run: |
    gh attestation verify my-service --owner my-org || {
      echo "ERROR: Provenance verification failed"
      exit 1
    }

- name: Deploy
  run: kubectl apply -f deployment.yaml
```

---

### Extension 4: Integrate SBOM with Vulnerability Scanner

**GitHub Dependency Graph** (automatic):
- Already scans `Cargo.lock` for CVEs
- SBOM provides same data in machine-readable format
- Alerts appear in Security tab → Dependabot alerts

**Third-party scanners** (Grype, Snyk, etc.):
```bash
# Download SBOM
gh release download v3.3.6 --pattern '*-sbom.spdx.json'

# Scan with Grype
grype sbom:rust-template-v3.3.6-sbom.spdx.json

# Output:
# NAME    INSTALLED  VULNERABILITY  SEVERITY
# openssl 3.0.7      CVE-2023-12345 High
```

**Policy enforcement**:
```rego
# policies/sbom_vulnerability.rego
package sbom_vulnerability

deny[msg] {
    vuln := input.vulnerabilities[_]
    vuln.severity == "CRITICAL"
    msg := sprintf("SBOM contains critical CVE: %s in %s", [vuln.id, vuln.package])
}
```

---

## SLSA Level 2 Compliance

This template meets **SLSA Build Level 2** requirements:

| Requirement | Implementation | Evidence |
|-------------|----------------|----------|
| **Build service** | GitHub Actions | Workflow logs |
| **Provenance generation** | `actions/attest-build-provenance@v1` | Attestations API |
| **Provenance authenticated** | Sigstore (Fulcio certificate) | Rekor transparency log |
| **Provenance non-falsifiable** | OIDC token (ephemeral, no secrets) | GitHub's identity provider |
| **Isolated build** | GitHub Actions runner per job | VM isolation |
| **Parameterless build** | Triggered only by git tags | No `workflow_dispatch` |
| **Hermetic build** | Nix devshell (`flake.lock` pins deps) | `nix develop -c cargo build` |

**Why not Level 3?**

Level 3 requires **hardware-isolated builders** (dedicated servers, no shared infrastructure). GitHub Actions uses shared runners (security boundary is VM isolation). Level 3 is needed for high-security contexts (e.g., cryptographic libraries, package managers). For most services, Level 2 provides sufficient assurance.

**Why not Level 4?**

Level 4 requires **reproducible builds** (bit-for-bit determinism). This needs:
- Deterministic compiler (Rust supports this, but requires careful setup)
- Controlled timestamps (remove all build-time timestamps)
- Multiple independent rebuilds (infrastructure to rebuild and compare hashes)

Level 4 is the gold standard (Debian, Tor) but overkill for most services.

---

## Threat Model

**What supply chain attacks does this prevent?**

### ✓ Prevented

1. **Compromised release process**: If attacker tampers with artifact after build, SHA mismatch fails verification
2. **Build-time injection**: Provenance shows exact commit + workflow; code review can audit source
3. **Workflow tampering**: Workflow file hash included in provenance; changes are visible
4. **Stolen credentials**: Keyless signing (no long-lived secrets to steal)

### ✓ Detected

1. **Dependency confusion**: SBOM lists all dependencies; policy can block unexpected packages
2. **Malicious dependencies**: SBOM + CVE scanner flags known malicious crates
3. **Vulnerable dependencies**: SBOM + GitHub Dependency Graph alerts on CVEs

### ✗ Not Prevented (Requires Additional Defenses)

1. **Compromised GitHub Actions runner**: VM isolation is trust boundary; Level 3 (dedicated runners) needed
2. **Compromised source repository**: Provenance proves "built from commit X", not "commit X is safe"
   - Defense: Code review, branch protection, signed commits
3. **Malicious insider**: Developer with write access can introduce backdoors
   - Defense: Multi-party code review, least-privilege access
4. **Compromised Nix flake inputs**: If `flake.lock` points to malicious packages
   - Defense: Review `flake.lock` diffs, pin trusted inputs

**Supply chain security is layered**: Provenance is one layer (build integrity). You still need:
- Code review (source integrity)
- Dependency scanning (component analysis)
- Access controls (reduce insider risk)
- Runtime monitoring (detect exploitation)

---

## Comparison with Alternatives

### vs. Manual GPG Signing

**GPG signing**:
```bash
# Developer signs release locally
gpg --detach-sign rust-template-v2.4.0.tar.gz
# Uploads signature to GitHub Releases
```

**Provenance attestation**:
```yaml
# Automated in CI
- uses: actions/attest-build-provenance@v1
```

**Differences**:

| Property | GPG Signing | Provenance Attestation |
|----------|-------------|------------------------|
| **What it proves** | "Maintainer approved this" | "Built from commit X on GitHub Actions" |
| **Key management** | Developer manages private key | No keys (OIDC, ephemeral) |
| **Build integrity** | No (can sign anything) | Yes (links artifact ↔ source) |
| **Automation** | Manual step | Fully automated |
| **Trust model** | Trust maintainer's key | Trust GitHub + Sigstore |

**When to use GPG**: Maintainer approval is the primary concern (e.g., package signing for distros)

**When to use provenance**: Build integrity is the primary concern (e.g., supply chain attacks)

**Best practice**: Use **both** together:
- **GPG-sign git tags** to prove releases are authorized by maintainers
- **Provenance attestation** to prove artifacts match source
- Result: Two-layer trust chain (authorized release → verified build)

See [`docs/how-to/setup-tag-signing.md`](../how-to/setup-tag-signing.md) for GPG tag signing setup

---

### vs. Reproducible Builds

**Reproducible builds**:
- Bit-for-bit deterministic builds
- Multiple independent parties rebuild, compare hashes
- If hashes match, no tampering occurred

**Provenance attestation**:
- Cryptographic proof of "built from source X"
- No need to rebuild (trust signature)
- Faster verification (signature check vs. full rebuild)

**Differences**:

| Property | Reproducible Builds | Provenance |
|----------|---------------------|------------|
| **Verification method** | Rebuild + hash comparison | Signature verification |
| **Trust model** | Don't trust anyone (verify yourself) | Trust builder (GitHub Actions) |
| **Speed** | Slow (requires full rebuild) | Fast (signature check ~1s) |
| **Difficulty** | High (determinism is hard) | Low (CI integration simple) |

**Best of both worlds**: Reproducible builds **with** provenance attestation (SLSA Level 4). Provenance provides fast verification; reproducibility allows distrust of builder.

---

## FAQ

### Q: Do I need to verify provenance every time I use the template?

**A**: No. Provenance is for **deployment** of released artifacts (e.g., binaries, Docker images).

If you're using the template to create a new service (via "Use this template"), you're consuming **source code**, not a released artifact. Provenance doesn't apply to development workflow, only to production deployments.

**When to verify**:
- Downloading a release tarball to deploy
- Pulling a Docker image to production
- Installing a binary from GitHub Releases

**When NOT to verify**:
- Cloning the repo to develop locally
- Creating a new service from the template
- Running selftest during development

---

### Q: What if I don't use Nix?

**A**: Provenance still works, but builds are less reproducible.

**Without Nix**:
```yaml
- name: Build release
  run: cargo build --workspace --release
```

This uses whatever Rust toolchain is on the runner (changes over time). Provenance will show "built on ubuntu-latest with Cargo 1.85.0", but future rebuilds might use Cargo 1.86.0 → different binary.

**With Nix**:
```yaml
- name: Build release
  run: nix develop -c cargo build --workspace --release
```

This pins Rust version via `flake.lock`. Same inputs → same outputs (modulo timestamps).

**Recommendation**: Use Nix for release builds (even if you don't use Nix locally). This ensures CI reproducibility.

---

### Q: Can I use this with private repositories?

**A**: Yes. GitHub Attestations work for private repos.

**Verification**:
```bash
# Requires gh CLI authentication with repo access
gh attestation verify artifact.tar.gz --owner my-org
```

Private attestations are accessible to:
- Org members (if private repo)
- Anyone with artifact (if public release from private repo)

**Public vs. private provenance**:
- **Public repo**: Provenance is public (anyone can verify)
- **Private repo**: Provenance is private (requires repo access to view)

If you want public verification of artifacts from private repo, use `actions/attest-sbom@v1` with `public: true` flag.

---

### Q: What if GitHub shuts down Attestations API?

**A**: Provenance is portable. Attestations are stored in:

1. **GitHub Attestations API**: Convenience layer (Web UI, `gh` CLI)
2. **Rekor transparency log**: Immutable, public, permanent (Sigstore infrastructure)

If GitHub stops offering Attestations API, you can still:
- Verify via Rekor directly (using `slsa-verifier`)
- Export attestations before shutdown
- Migrate to standalone Sigstore/Cosign workflow

**Provenance format is SLSA standard** (not GitHub-proprietary), so tooling is portable.

---

### Q: How do I rotate signing keys?

**A**: You don't. There are no keys.

**Keyless signing** (Sigstore):
- Each build gets ephemeral OIDC token from GitHub
- Fulcio CA issues short-lived certificate (binds identity to signing key)
- Certificate proves "GitHub Actions workflow EffortlessMetrics/Rust-Template signed this at 2025-01-19T12:34:56Z"
- Private key discarded after signing (can't be stolen)

**No key management**:
- No key generation
- No key storage
- No key rotation
- No key revocation

**Trust model**: Trust GitHub's OIDC provider + Sigstore CA (Fulcio). If either is compromised, entire ecosystem is affected (not just this repo).

---

### Q: Can I use this for internal/proprietary services?

**A**: Yes. Nothing about this is OSS-specific.

**Modifications for internal use**:

1. **Private registry**: If using GitHub Container Registry (GHCR) or Artifactory:
   ```yaml
   - uses: actions/attest-build-provenance@v1
     with:
       subject-name: ghcr.io/my-org/my-service
   ```

2. **Custom SBOM fields**: Add proprietary metadata:
   ```yaml
   - uses: anchore/sbom-action@v0
     with:
       format: spdx-json
       artifact-name: my-service-sbom.spdx.json
       # Add custom SPDX annotations (owner, cost-center, etc.)
   ```

3. **Compliance reporting**: Export attestations for audits:
   ```bash
   gh attestation list --owner my-org --format json > attestations.json
   ```

---

## References

- **Architecture Decision**: [ADR-0006: Supply Chain Hardening](../adr/0006-supply-chain-hardening.md)
- **SLSA Framework**: [slsa.dev](https://slsa.dev/)
- **GitHub Artifact Attestations**: [docs.github.com/actions/security-guides/using-artifact-attestations](https://docs.github.com/en/actions/security-guides/using-artifact-attestations-to-establish-provenance-for-builds)
- **Sigstore**: [sigstore.dev](https://www.sigstore.dev/)
- **SPDX Specification**: [spdx.dev/specifications](https://spdx.dev/specifications/)
- **In-toto Attestations**: [in-toto.io](https://in-toto.io/)
- **SLSA Verifier**: [github.com/slsa-framework/slsa-verifier](https://github.com/slsa-framework/slsa-verifier)

---

**Summary:**

Supply chain hardening in this template means:

- **SBOM**: Machine-readable dependency list (SPDX JSON)
- **Provenance**: Cryptographic proof artifact ↔ source (SLSA v1.0)
- **Verification**: `gh attestation verify` (fast, automated)
- **Trust**: Sigstore (keyless signing, transparency log)

This shifts supply chain security from "trust the artifact" to "verify the artifact."
