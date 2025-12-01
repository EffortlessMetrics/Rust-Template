---
id: GUIDE-TPL-TAG-SIGNING-001
title: Setup GPG Tag Signing
doc_type: how-to
status: published
audience: developers, maintainers, release-engineers
tags: [security, gpg, release, supply-chain, tag-signing]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-RELEASE-SAFETY]
acs: [AC-PLT-011, AC-PLT-012, AC-PLT-013]
adrs: [ADR-0005, ADR-0006]
last_updated: 2025-11-26
---
<!-- doclint:disable orphan-version -->

# How-to: Setup GPG Tag Signing

**Time:** 15-30 minutes (first-time GPG setup)
**Prerequisites:** Git installed, GitHub account

This guide shows you how to configure GPG signing for git tags to ensure release authenticity and supply chain security.

---

## Why Tag Signing Matters

For the Rust-as-Spec template, signed tags are a critical component of supply chain security:

1. **Release authenticity** - Proves tags were created by authorized maintainers, not attackers
2. **Supply chain integrity** - Verifies the release pipeline starts from a legitimate tag
3. **Non-repudiation** - Creates cryptographic proof of who created each release
4. **Trust chain** - Integrates with GitHub's verified badge system for public visibility

Without signed tags:
- Anyone with write access can create release tags
- Compromised accounts can inject malicious releases
- No cryptographic proof linking releases to maintainers
- Supply chain attestations (SLSA provenance) start from unverified tags

With signed tags, you establish a trust root for the entire release pipeline: signed tag → CI build → SLSA attestation → SBOM.

---

## Overview: The GPG + Git + GitHub Flow

1. **Generate GPG key** (or use existing)
2. **Configure Git** to sign tags with your key
3. **Add GPG key to GitHub** for verified badge display
4. **Test signing** locally
5. **Integrate with release workflow** (`release-prepare`, `release-bundle`)

---

## Step 1: Check for Existing GPG Keys

Before generating a new key, check if you already have one:

```bash
gpg --list-secret-keys --keyid-format LONG
```

**If you see output like this, you already have keys:**

```
/home/user/.gnupg/secring.gpg
-----------------------------
sec   rsa4096/ABCD1234EFGH5678 2024-01-15 [SC]
uid                 [ultimate] Your Name <your.email@example.com>
ssb   rsa4096/1234ABCD5678EFGH 2024-01-15 [E]
```

- The key ID is `ABCD1234EFGH5678` (the part after `rsa4096/`)
- If you have a key, skip to **Step 3: Configure Git to Use Your Key**

**If you see no output or `gpg: directory '/home/user/.gnupg' created`:**

- You need to generate a new key (proceed to Step 2)

---

## Step 2: Generate a New GPG Key

### 2.1 Start Key Generation

```bash
gpg --full-generate-key
```

### 2.2 Choose Key Type

```
Please select what kind of key you want:
   (1) RSA and RSA (default)
   (2) DSA and Elgamal
   (3) DSA (sign only)
   (4) RSA (sign only)
Your selection? 1
```

**Recommendation**: Select `1` (RSA and RSA) for maximum compatibility.

### 2.3 Choose Key Size

```
RSA keys may be between 1024 and 4096 bits long.
What keysize do you want? (3072) 4096
```

**Recommendation**: Enter `4096` for strong security (GitHub supports up to 4096-bit keys).

### 2.4 Choose Expiration

```
Please specify how long the key should be valid.
         0 = key does not expire
      <n>  = key expires in n days
      <n>w = key expires in n weeks
      <n>m = key expires in n months
      <n>y = key expires in n years
Key is valid for? (0) 2y
```

**Recommendation**: Enter `2y` (2 years). Key expiration is a security best practice.

- Keys should expire to limit impact of compromise
- You can extend expiration before it expires (no need to generate new key)
- Non-expiring keys (`0`) are acceptable for personal projects

### 2.5 Enter User Information

```
Real name: Your Full Name
Email address: your.email@example.com
Comment: (optional, can leave blank or use "Release Signing")
```

**Important**: Use the **same email address** as your GitHub account, or an email you've added to GitHub. GitHub uses this to link the signature to your account.

### 2.6 Set Passphrase

You'll be prompted for a passphrase. **Choose a strong passphrase.**

- This protects your private key if someone accesses your machine
- You'll enter it when signing tags
- Store it securely (password manager recommended)

### 2.7 Generate Entropy (if prompted)

GPG may ask you to generate random activity:

```
We need to generate a lot of random bytes. It is a good idea to perform
some other action (type on the keyboard, move the mouse, utilize the
disks) during the prime generation; this gives the random number
generator a better chance to gain enough entropy.
```

Move your mouse, type some text, or run some commands until generation completes.

### 2.8 Verify Key Creation

```bash
gpg --list-secret-keys --keyid-format LONG
```

You should see your new key:

```
/home/user/.gnupg/secring.gpg
-----------------------------
sec   rsa4096/ABCD1234EFGH5678 2024-01-15 [SC] [expires: 2026-01-15]
      ABCDEF1234567890ABCDEF1234567890ABCDEF12
uid                 [ultimate] Your Full Name <your.email@example.com>
ssb   rsa4096/1234ABCD5678EFGH 2024-01-15 [E] [expires: 2026-01-15]
```

**Note the key ID**: `ABCD1234EFGH5678` (you'll need this in the next step).

---

## Step 3: Configure Git to Use Your Key

### 3.1 Tell Git Your Key ID

Replace `ABCD1234EFGH5678` with your actual key ID:

```bash
git config --global user.signingkey ABCD1234EFGH5678
```

**Verify configuration:**

```bash
git config --global user.signingkey
# Should output: ABCD1234EFGH5678
```

### 3.2 Configure Git to Sign Tags by Default (Recommended)

```bash
git config --global tag.gpgSign true
```

This makes `git tag` sign by default (you can override with `--no-sign` if needed).

**Alternative (manual signing)**: Skip this step and always use `git tag -s` to sign explicitly.

### 3.3 Configure Git to Use GPG (not GPG2)

Some systems have both `gpg` and `gpg2`. Ensure Git uses the correct one:

```bash
# Check which GPG binary you're using
which gpg
# Output: /usr/bin/gpg (or /opt/homebrew/bin/gpg on macOS)

# Configure Git to use it
git config --global gpg.program $(which gpg)
```

**On Windows (Git Bash or WSL):**

```bash
# Git Bash
git config --global gpg.program "C:/Program Files/Git/usr/bin/gpg.exe"

# WSL
git config --global gpg.program /usr/bin/gpg
```

---

## Step 4: Add GPG Key to GitHub

GitHub needs your **public key** to verify signatures and display the "Verified" badge.

### 4.1 Export Your Public Key

```bash
gpg --armor --export ABCD1234EFGH5678
```

This outputs your public key in ASCII format:

```
-----BEGIN PGP PUBLIC KEY BLOCK-----

mQINBGXxAbcBEAC... (many lines of text)
...
-----END PGP PUBLIC KEY BLOCK-----
```

**Copy the entire output** (including `-----BEGIN PGP PUBLIC KEY BLOCK-----` and `-----END PGP PUBLIC KEY BLOCK-----`).

### 4.2 Add to GitHub

1. Go to GitHub: https://github.com/settings/keys
2. Click **New GPG key**
3. Paste your public key into the text box
4. Click **Add GPG key**

**Verify addition:**

Your key should now appear in the list with your email address and key ID.

---

## Step 5: Test Tag Signing Locally

### 5.1 Create a Signed Test Tag

```bash
cd /path/to/your/repo
git tag -s test-signed-tag -m "Test GPG signing"
```

**If prompted for passphrase:** Enter your GPG passphrase.

**Expected output:**

```
# (Passphrase prompt, then success)
```

### 5.2 Verify the Signature

```bash
git tag -v test-signed-tag
```

**Expected output:**

```
object 1234abc567def890...
type commit
tag test-signed-tag
tagger Your Name <your.email@example.com> 1706198400 -0500

Test GPG signing
gpg: Signature made Tue Jan 15 14:20:00 2024 EST
gpg:                using RSA key ABCDEF1234567890ABCDEF1234567890ABCDEF12
gpg: Good signature from "Your Name <your.email@example.com>" [ultimate]
```

**Key phrase:** `Good signature` means signing works correctly.

### 5.3 Clean Up Test Tag

```bash
git tag -d test-signed-tag
```

---

## Step 6: Integrate with Release Workflow

### 6.1 Update Your Release Commands

When using `release-prepare`, the workflow creates a commit but not a tag. You create the tag afterward:

**Current workflow (from `governed-release` skill):**

```bash
# 1. Prepare release (bumps version, updates CHANGELOG)
cargo xtask release-prepare 3.4.0

# 2. Validate
cargo xtask selftest

# 3. Generate SBOM
cargo xtask sbom-local

# 4. Create SIGNED tag (this is where GPG signing happens)
git tag -s v3.4.0 -m "Release 3.4.0: Tag Signing Documentation"

# 5. Verify signature locally
git tag -v v3.4.0

# 6. Push tag to remote
git push origin v3.4.0
```

**If you configured `tag.gpgSign=true` in Step 3.2**, you can omit `-s`:

```bash
git tag -a v3.4.0 -m "Release 3.4.0: Tag Signing Documentation"
# Automatically signed due to tag.gpgSign=true
```

### 6.2 Verify on GitHub After Pushing

1. Go to your repository on GitHub
2. Navigate to **Tags** (usually `https://github.com/owner/repo/tags`)
3. Click on your tag (e.g., `v3.4.0`)
4. Look for the **Verified** badge next to the tag

**Example:**

```
v3.4.0  ✓ Verified
```

The badge means:
- GitHub successfully verified your GPG signature
- The tag was created by your GitHub account
- The signature matches the commit and tag message

---

## Step 7: Document Signing Policy (Recommended)

### 7.1 Update CONTRIBUTING.md

Add a section on tag signing requirements:

```markdown
## Release Signing Policy

All release tags MUST be GPG-signed by authorized maintainers.

**Authorized signing keys:**
- Alice (alice@example.com): GPG key `ABCD1234EFGH5678`
- Bob (bob@example.com): GPG key `1234ABCD5678EFGH`

**Verification:**

\`\`\`bash
# Fetch tags
git fetch --tags

# Verify a release tag
git tag -v v3.4.0

# Expected: "Good signature from [maintainer name]"
\`\`\`

**Setup instructions:** See `docs/how-to/setup-tag-signing.md`
```

### 7.2 Update GitHub Branch Protection (Future)

GitHub does not yet support enforcing GPG-signed tags via branch protection rules. However, you can:

1. **Document the policy** in CONTRIBUTING.md (as above)
2. **Audit manually** before publishing releases
3. **Use GitHub's API** to check signatures in CI

**Example CI check (future enhancement):**

```yaml
# .github/workflows/verify-tag-signature.yml
name: Verify Tag Signature
on:
  push:
    tags:
      - 'v*.*.*'
jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Verify tag signature
        run: |
          git tag -v ${GITHUB_REF#refs/tags/}
```

---

## Troubleshooting

### Problem: "gpg: signing failed: Inappropriate ioctl for device"

**Cause:** GPG cannot prompt for passphrase (common in CI, SSH sessions, or minimal terminals).

**Solution:**

```bash
export GPG_TTY=$(tty)
```

Add to `~/.bashrc` or `~/.zshrc` to make permanent:

```bash
echo 'export GPG_TTY=$(tty)' >> ~/.bashrc
source ~/.bashrc
```

### Problem: "gpg: signing failed: No secret key"

**Cause:** Git is configured to use a key ID that doesn't exist or is inaccessible.

**Solution:**

1. Check configured key:
   ```bash
   git config --global user.signingkey
   ```

2. List available keys:
   ```bash
   gpg --list-secret-keys --keyid-format LONG
   ```

3. If key is missing, generate a new one (Step 2) or import existing key:
   ```bash
   gpg --import /path/to/private-key.asc
   ```

4. Update Git config with correct key ID:
   ```bash
   git config --global user.signingkey CORRECT_KEY_ID
   ```

### Problem: "error: cannot run gpg: No such file or directory"

**Cause:** Git cannot find the GPG binary.

**Solution:**

1. Install GPG:
   ```bash
   # Ubuntu/Debian
   sudo apt-get install gnupg

   # macOS
   brew install gnupg

   # Windows (Git Bash)
   # GPG comes with Git for Windows
   ```

2. Tell Git where GPG is:
   ```bash
   git config --global gpg.program $(which gpg)
   ```

### Problem: "gpg: skipped 'ABCD1234': No secret key"

**Cause:** The key ID is too short or incorrect.

**Solution:**

Use the **long format** key ID (16 characters):

```bash
# Wrong (short format)
git config --global user.signingkey ABCD1234

# Correct (long format)
git config --global user.signingkey ABCD1234EFGH5678
```

Get long format key ID:

```bash
gpg --list-secret-keys --keyid-format LONG
# Look for "sec   rsa4096/ABCD1234EFGH5678"
#                        ^^^^^^^^^^^^^^^^
#                        This is the long key ID
```

### Problem: GitHub shows "Unverified" even though signature is valid

**Cause:** The email in your GPG key doesn't match any email on your GitHub account.

**Solution:**

**Option 1: Add the email to GitHub**

1. Go to https://github.com/settings/emails
2. Add the email from your GPG key
3. Verify the email

**Option 2: Add a new UID to your GPG key**

```bash
# Edit your key
gpg --edit-key ABCD1234EFGH5678

# Add new email
gpg> adduid
# Follow prompts to add GitHub email

# Trust the new UID
gpg> uid 2
gpg> trust
# Select "5 = I trust ultimately"

# Save
gpg> save
```

Re-export and re-upload to GitHub (Step 4).

### Problem: "error: tag 'vX.Y.Z' already exists"

**Cause:** Tag was created unsigned, now you want to sign it.

**Solution:**

Delete and recreate:

```bash
# Delete local tag
git tag -d v3.4.0

# Delete remote tag (if already pushed)
git push origin :refs/tags/v3.4.0

# Recreate as signed
git tag -s v3.4.0 -m "Release 3.4.0"

# Push signed tag
git push origin v3.4.0
```

**⚠️ Warning:** Only delete remote tags if they haven't been released yet. Deleting published release tags can break downstream users.

---

## Key Rotation and Expiration

### Extending Key Expiration

If your key is expiring soon:

```bash
# Edit key
gpg --edit-key ABCD1234EFGH5678

# Extend expiration
gpg> expire
# Follow prompts to set new expiration (e.g., "2y" for 2 more years)

# Save
gpg> save

# Re-export and re-upload to GitHub (Step 4.1-4.2)
```

### Rotating to a New Key

If you need to replace your key (compromise, lost passphrase, etc.):

1. **Generate new key** (Step 2)
2. **Configure Git** with new key ID (Step 3.1)
3. **Add new key to GitHub** (Step 4)
4. **Revoke old key** (if compromised):
   ```bash
   gpg --edit-key OLD_KEY_ID
   gpg> revkey
   gpg> save

   # Export revocation certificate
   gpg --output revoke.asc --armor --gen-revoke OLD_KEY_ID

   # Publish revocation (optional: upload to keyserver)
   gpg --send-keys OLD_KEY_ID
   ```

5. **Document key change** in CONTRIBUTING.md

---

## CI/CD Integration (Advanced)

For automated release pipelines, you may need to sign tags in CI. This requires securely storing your GPG private key.

**⚠️ Security Warning:** Storing private keys in CI is risky. Only do this if:
- CI environment is trusted (e.g., GitHub-hosted runners, not public)
- Repository is private or tightly controlled
- You use a dedicated signing key (not your personal key)

### Option 1: GitHub Actions with GPG Key Secret

```yaml
# .github/workflows/release.yml
name: Release
on:
  push:
    branches: [ main ]

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Import GPG key
        run: |
          echo "${{ secrets.GPG_PRIVATE_KEY }}" | gpg --import
          echo "${{ secrets.GPG_PASSPHRASE }}" | gpg --batch --yes --passphrase-fd 0 --pinentry-mode loopback

      - name: Configure Git
        run: |
          git config user.name "Release Bot"
          git config user.email "release@example.com"
          git config user.signingkey ${{ secrets.GPG_KEY_ID }}
          git config commit.gpgSign true
          git config tag.gpgSign true

      - name: Create signed tag
        run: |
          git tag -s v${{ github.run_number }} -m "Automated release"
          git push origin v${{ github.run_number }}
```

**Setup:**

1. Export private key:
   ```bash
   gpg --armor --export-secret-keys ABCD1234EFGH5678 > private-key.asc
   ```

2. Add to GitHub Secrets:
   - `GPG_PRIVATE_KEY`: Contents of `private-key.asc`
   - `GPG_PASSPHRASE`: Your GPG passphrase
   - `GPG_KEY_ID`: Your key ID (`ABCD1234EFGH5678`)

3. **Secure the file:** Delete `private-key.asc` immediately after uploading to GitHub Secrets.

### Option 2: Use a Dedicated CI Signing Key

**Recommended approach:**

1. Generate a separate GPG key for CI (e.g., "Release Bot <ci@example.com>")
2. Use a strong passphrase (store in GitHub Secrets)
3. Document in CONTRIBUTING.md that CI-signed tags are also valid
4. If CI key is compromised, revoke it without affecting personal keys

---

## Security Best Practices

1. **Use strong passphrases** - Protect private keys with long, random passphrases
2. **Set key expiration** - 1-2 years is recommended (extendable before expiry)
3. **Back up keys** - Export and store private keys in a secure location (password manager, encrypted USB)
   ```bash
   gpg --armor --export-secret-keys ABCD1234EFGH5678 > backup-private-key.asc
   # Store in password manager or encrypted storage
   ```
4. **Revoke if compromised** - Immediately revoke and rotate keys if machine is compromised
5. **Separate keys for CI** - Don't use personal keys in CI; use dedicated signing keys
6. **Document authorized signers** - List authorized GPG keys in CONTRIBUTING.md
7. **Audit signatures** - Periodically verify release tags have valid signatures

---

## Integration with Template Governance

### Adding Tag Signature Verification to Selftest (Future)

Currently, `cargo xtask selftest` does not verify tag signatures. This could be added as a gate:

**Proposed gate:**

```rust
// crates/xtask/src/commands/selftest.rs

fn gate_verify_tag_signatures() -> Result<()> {
    println!("Verifying release tag signatures...");

    let output = Command::new("git")
        .args(&["tag", "--list", "v*"])
        .output()?;

    let tags = String::from_utf8_lossy(&output.stdout);
    for tag in tags.lines() {
        let verify = Command::new("git")
            .args(&["tag", "-v", tag])
            .output()?;

        if !verify.status.success() {
            eprintln!("❌ Tag {} is not signed or signature is invalid", tag);
            return Err(anyhow!("Tag signature verification failed"));
        }
    }

    println!("✅ All release tags have valid signatures");
    Ok(())
}
```

**To implement:**

1. Add to `cargo xtask selftest` as optional gate
2. Document in spec_ledger.yaml as an AC
3. Run in CI (requires GPG keys in CI environment)

### Adding to ROADMAP Completion Tracking

Once tag signing is documented and tested:

1. **Update ROADMAP.md** section 3.1:
   ```markdown
   | **Tag signing not enforced** | Release tags can be created without verification | Medium | See `docs/how-to/setup-tag-signing.md` |
   ```

2. **Consider moving to "Completed" section** if enforcement is added to CI.

---

## Quick Reference

### Daily Commands

```bash
# Sign a release tag
git tag -s v3.4.0 -m "Release 3.4.0"

# Verify a tag signature
git tag -v v3.4.0

# Verify all release tags
git tag -l "v*" | xargs -I {} git tag -v {}

# List your GPG keys
gpg --list-secret-keys --keyid-format LONG

# Check Git signing config
git config user.signingkey
git config tag.gpgSign
```

### Emergency: Key Compromised

```bash
# 1. Revoke immediately
gpg --edit-key COMPROMISED_KEY_ID
gpg> revkey
gpg> save

# 2. Generate new key
gpg --full-generate-key

# 3. Update Git config
git config --global user.signingkey NEW_KEY_ID

# 4. Add new key to GitHub
gpg --armor --export NEW_KEY_ID
# Paste at github.com/settings/keys

# 5. Notify team
# Document in CONTRIBUTING.md or security advisory
```

---

## Summary

By following this guide, you've established:

1. ✅ **GPG keypair** for cryptographic signing
2. ✅ **Git configuration** to sign tags automatically or manually
3. ✅ **GitHub integration** for "Verified" badges on releases
4. ✅ **Testing workflow** to verify signatures locally
5. ✅ **Release integration** with `cargo xtask release-prepare` workflow

**Next steps:**

- **Document signing policy** in CONTRIBUTING.md (see Step 7.1)
- **Enforce in CI** (optional, see Step 7.2 and CI/CD Integration)
- **Add to selftest** (optional, see Integration with Template Governance)
- **Train team** on signing workflow

**Result:** Your release tags are now cryptographically signed, providing proof of authenticity and completing the supply chain security story: signed tag → SLSA attestation → SBOM → verified artifacts.

---

## References

- **Git tag signing**: https://git-scm.com/book/en/v2/Git-Tools-Signing-Your-Work
- **GitHub GPG verification**: https://docs.github.com/en/authentication/managing-commit-signature-verification
- **GPG documentation**: https://gnupg.org/documentation/
- **SLSA provenance** (upstream context): `docs/explanation/supply-chain-hardening.md`
- **Release workflow**: `.claude/skills/governed-release/SKILL.md`
- **SemVer versioning**: https://semver.org/
