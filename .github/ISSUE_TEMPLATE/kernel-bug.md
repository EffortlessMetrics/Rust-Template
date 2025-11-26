---
name: Kernel Bug
about: Report broken behavior in kernel ACs
title: "[Kernel Bug] "
labels: kernel-bug
assignees: ''
---

## Which kernel AC is failing?

<!-- e.g., AC-TPL-001, AC-PLT-015, etc. -->
<!-- See docs/feature_status.md for the full list -->

## What's broken?

<!-- Describe the incorrect behavior -->

## How to reproduce

```bash
# Steps to reproduce the failure
cargo xtask selftest
# or
cargo xtask test-ac AC-XXX
```

## Expected behavior

<!-- What should happen according to the AC definition in spec_ledger.yaml? -->

## Actual behavior

<!-- What actually happens? Include error messages or test output -->

## Environment

- Template version:
- Platform: Linux | macOS | Windows
- Nix: Yes | No

## Related ACs or contracts

<!-- Are other ACs affected by this bug? -->
