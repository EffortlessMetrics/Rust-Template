---
id: DESIGN-TPL-TS-CONFIG-GOVERNANCE-001
title: TypeScript Configuration Governance
author: platform-team
doc_type: design_doc
date: 2025-12-04
status: published
stories: [US-TPL-PLT-001]
requirements:
  - REQ-TPL-TS-CONFIG-GOVERNANCE
tags: [platform, devex, typescript, governance]
acs:
  - AC-TPL-TS-CONFIG-VALIDATION
adrs: [ADR-0005]
---

# TypeScript Configuration Governance

## Problem

TypeScript configurations in the repository (particularly in `examples/backstage-plugin/`) can silently use deprecated settings that cause subtle issues:

- `moduleResolution: "node"` (deprecated in favor of `NodeNext` or `Bundler`)
- `ignoreDeprecations` flags that mask underlying problems
- Inconsistent settings across different tsconfig files

These issues surface as confusing errors for consumers who fork or integrate with the template.

## Decision

We enforce **modern, enforceable TypeScript standards** at the template level:

1. **No deprecated moduleResolution**: Must use `NodeNext` or `Bundler`, not `node` or `node10`
2. **No ignoreDeprecations**: Address deprecations rather than suppressing them
3. **CI enforcement**: The `ts-config-lints` job validates all tsconfig.json files

## Scope

This is a **template-level** (non-kernel) concern:

- The Rust kernel itself has no TypeScript
- Example consumers (Backstage plugin) demonstrate best practices
- Forks may adjust TypeScript settings for their needs, but start from a clean baseline

## Enforcement

```bash
# Validate all TypeScript configurations
./scripts/validate-ts-config.sh

# Or via xtask (includes OpenAPI lint)
cargo xtask idp-check
```

The validation script:

1. Scans for all `tsconfig*.json` files in the repository
2. Checks for deprecated `moduleResolution` values (`node`, `node10`)
3. Checks for `ignoreDeprecations` flags
4. Exits non-zero if violations found

## Invariants

These rules are tracked in `docs/INDEX.md` under the invariants section:

| Invariant | Enforcement |
|-----------|-------------|
| No deprecated moduleResolution | `scripts/validate-ts-config.sh` |
| No ignoreDeprecations flags | `scripts/validate-ts-config.sh` |
| TypeScript builds clean | `pnpm run build` in CI |

## Implementation Notes

- `scripts/validate-ts-config.sh` performs the actual validation
- `cargo xtask idp-check` runs this as part of IDP surface validation
- CI job `ts-config-lints` runs on PRs touching TypeScript files
- The Backstage plugin's `tsconfig.json` uses `moduleResolution: "NodeNext"`

## Related Documents

- `docs/INDEX.md` - Invariants index tracking enforcement
- `examples/backstage-plugin/tsconfig.json` - Reference configuration
- `scripts/validate-ts-config.sh` - Validation script
