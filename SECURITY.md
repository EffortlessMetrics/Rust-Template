# Security Policy

## Supported Versions

This project is currently pre-1.0 and under active development.

At the moment:

- The **main** branch and the **latest tagged release** (e.g. `v2.3.0`) are considered supported.
- Older tags may receive fixes on a best-effort basis but are not guaranteed.

As the project matures, we'll document a more formal support matrix here.

---

## Reporting a Vulnerability

If you believe you've found a security issue:

1. **Do not** open a public GitHub issue with details.
2. Instead, email:

   > `git@effortlesssteven.com`

   Please include:

   - A description of the vulnerability.
   - Steps to reproduce, if possible.
   - Any potential impact you've identified.

3. You can optionally include:

   - Suggested fixes or mitigations.
   - Whether you believe the issue affects only the template, or also generated services.

We will:

- Acknowledge receipt as soon as possible.
- Assess impact and plan remediation.
- Coordinate public disclosure once a fix is available (or if we determine no fix is needed).

---

## Scope

This project includes:

- A **Rust template** (multi-crate workspace).
- **xtask** tooling (development and CI automation).
- **Policy-as-code** (Rego) and spec files.
- **Nix** dev environment configuration.

All of those are in scope from a security perspective, including:

- Generated configs (e.g., K8s manifests, policy bundles).
- How the template suggests you wire authentication/authorization.
- LLM workflow docs (e.g. accidentally encouraging unsafe patterns).

What's **out of scope**:

- Services you build **from** this template (those are your responsibility).
- Third-party tools (Nixpkgs, conftest, Docker, etc.) beyond how we configure or invoke them.

---

## Recommendations for Template Users

If you're using this template in a real system:

- Review and adapt policies (`policy/*.rego`) for your regulatory and risk context.
- Review K8s manifests under `infra/k8s/` for your security baselines.
- Wire authentication/authorization according to your organization's standards.
- Keep dependencies up to date and pay attention to RustSec advisories.

If in doubt, treat this as a **starting point**, not a drop-in replacement for your organization's security review.
