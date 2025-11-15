# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-11-15

### Added

**Core Template Features:**
- Template foundation with health and version endpoints (`AC-TPL-001`, `AC-TPL-002`)
- Refund example feature to demonstrate AC-first patterns (`AC-REFUND-001`)
- Complete hexagonal architecture with adapters, core, and model crates
- Axum HTTP runtime with telemetry integration
- Cucumber BDD acceptance testing with JSON and JUnit XML output
- Comprehensive Diátaxis documentation framework

**xtask Rust-Native Tooling:**
- `xtask check` - Fast feedback loop (fmt, clippy, tests)
- `xtask bdd` - Run Cucumber acceptance tests
- `xtask ac-status` - Generate AC status report from JSON/JUnit
  - Primary: Structured JSON from Cucumber with full metadata
  - Fallback: JUnit XML + feature file parsing (legacy)
- `xtask policy-test` - Run Rego policy tests via conftest
- `xtask bundle` - Generate LLM context bundles with `.llmignore` support
- `xtask quickstart` - First-run validation
- `xtask selftest` - Comprehensive validation suite (all of the above)

**Policy-as-Code (Rego):**
- Ledger coverage policy - Ensures all ACs have tests
- Features policy - Validates feature-to-AC mapping
- Flags policy - Feature flag governance
- Privacy policy - PII handling validation

**Observability Patterns:**
- Request ID correlation middleware (custom implementation)
  - Reads `X-Request-ID` header or generates UUID
  - Automatically adds to tracing spans
  - Returns in response for client tracking
- Enhanced `AppError` with structured logging
  - 12 machine-readable error codes
  - AC ID and Feature ID tracking
  - Structured context for debugging (logged, not exposed)
  - JSON error responses
- Metrics integration stubs with examples
- Full instrumentation examples in handlers

**LLM Integration:**
- Context bundler with gitignore-style `.llmignore` semantics
- Task-based bundle generation for focused context
- Environment variable configuration (`AC_REPORT_JSON`, etc.)
- Example contextpack.yaml with implement_ac task

**Documentation:**
- Complete Diátaxis framework (Tutorials, How-tos, Reference, Explanation)
- Getting Started tutorial
- First AC Change tutorial
- Architecture overview and design rationale
- Template Foundation vs Examples guide
- xtask Commands reference
- Branch Protection Profiles reference
- Design documents for key implementation decisions
- OBSERVABILITY.md for runtime patterns

**CI/CD:**
- GitHub Actions workflows for:
  - Template self-test (xtask selftest)
  - Policy verification
  - Lints, MSRV, Coverage
  - Nix flake check
- Branch protection profile recommendations (Minimal/Standard/Strict)

### Changed

- Migrated all core workflows from bash scripts to Rust-native `xtask` commands
- Replaced custom .llmignore matching with full gitignore semantics (using `ignore` crate)
- Improved AC status mapping to use structured JSON instead of fragile text parsing
- Updated all documentation to canonize xtask as primary interface
- Converted legacy bash scripts to thin wrappers (backward compatibility)

### Deprecated

- `scripts/test-policies.sh` - Use `cargo run -p xtask -- policy-test`
- `scripts/test-all.sh` - Use `cargo run -p xtask -- selftest`
- JUnit XML + feature file parsing in ac-status (fallback only, may be removed in v2.0)

### Technical Details

**Code Statistics:**
- ~1,500 lines of new code and features
- ~1,200 lines of new documentation
- 14 new unit tests (100% coverage of new code)
- 26 files created or modified
- 3 comprehensive design documents

**Validation:**
- All `xtask selftest` checks passing:
  - ✅ Format, clippy, unit tests
  - ✅ 3/3 BDD scenarios, 7/7 steps
  - ✅ All ACs correctly mapped
  - ✅ LLM bundler generating correct output
  - ✅ Policy tests (with conftest)

**Dependencies Added:**
- `ignore = "0.4"` - Gitignore-style pattern matching
- `cucumber` with `output-json` feature - Structured test reports
- `quick-xml` - JUnit XML parsing (fallback)
- `uuid` with `v4` - Request ID generation

### Breaking Changes

**NONE** - v1.0.0 is the initial stable release. All changes are backward compatible with the template's pre-release state.

### Migration Guide

If migrating from pre-1.0 template usage:

1. **Replace bash script calls with xtask:**
   ```bash
   # Old
   bash scripts/test-policies.sh
   bash scripts/test-all.sh

   # New
   cargo run -p xtask -- policy-test
   cargo run -p xtask -- selftest
   ```

2. **Update CI workflows:**
   ```yaml
   # Old
   run: nix develop -c bash scripts/test-policies.sh

   # New
   run: nix develop -c cargo run -p xtask -- policy-test
   ```

3. **Enable JSON AC reports (recommended):**
   - Acceptance tests now automatically generate `target/ac_report.json`
   - `xtask ac-status` uses this by default
   - JUnit fallback still works but is legacy

4. **Update .llmignore if using advanced patterns:**
   - Full gitignore syntax now supported (`*.log`, `**/*.tmp`, etc.)
   - Simple patterns (like `target/`) work unchanged
   - See `docs/how-to/use-llm-bundles.md` for full syntax

### Security

- No security vulnerabilities introduced
- All dependencies vetted and up-to-date
- No unsafe code in new implementations

### For Contributors

- Read `docs/implementation-summary-2025-11-15.md` for full technical details
- Review `docs/design/*.md` for implementation rationale
- Check `crates/app-http/OBSERVABILITY.md` for runtime patterns
- Follow `TEMPLATE_API.md` for stable interfaces

---

## [Unreleased]

### Planned for v1.1.0

- Remove JUnit fallback path (JSON will be required)
- Add `.llmignore-local` support for user-specific ignores
- Full OpenTelemetry metrics implementation (replace stubs)
- Database adapter example
- More how-to guides

---

[1.0.0]: https://github.com/your-org/rust-template/releases/tag/v1.0.0
