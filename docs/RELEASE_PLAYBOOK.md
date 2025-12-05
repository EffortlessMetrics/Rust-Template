# Release Playbook
<!-- doclint:disable orphan-version -->

**Version**: 1.0.0
**Last Updated**: 2025-11-17
**Scope**: Rust IaC Template (adaptable to other governed systems)

---

## Purpose

This playbook documents the systematic approach for shipping governed releases. It captures the process used successfully for v2.0.x → v2.3.0 and provides a reusable pattern for future releases.

**Key Principles:**
- **Incremental**: Small, focused releases
- **Governed**: Specs → Code → Tests → Policies → Docs
- **Safe**: Multiple validation gates before release
- **Traceable**: Clear commit history, tags, CHANGELOG

---

## Release Phases

### Phase 1: Planning

**Deliverable**: `docs/vX.Y.Z-plan.md`

**Steps:**

1. **Create Release Plan Document**
   ```bash
   # Create plan from template
   cp docs/templates/RELEASE_PLAN.md docs/vX.Y.Z-plan.md
   ```

2. **Define Scope**
   - **Theme**: Single sentence describing the release focus
   - **Epics**: 1-3 major features/improvements (prefer 1 for focused releases)
   - **Guardrails**: Explicit list of what will NOT change (critical for stability)

3. **Acceptance Criteria Mapping**
   - Create new AC IDs in `specs/spec_ledger.yaml`
   - Link to relevant existing specs (API, domain model, policy)
   - Define clear exit criteria

4. **Decision Log**
   - Document any architectural decisions upfront (e.g., library version choices)
   - Include rationale and alternatives considered
   - Update as new decisions emerge during implementation

**Example** (from v2.3.0):
```markdown
## Scope
**Theme**: OTLP Tracing + Telemetry Production Readiness

**Epic**: Implement OTLP export for distributed tracing

**Guardrails** (Locked):
- ❌ No changes to business-core, model, rust_iac_xtask_core APIs
- ❌ No new xtask commands
- ✅ Only: OTLP implementation + minimal docs/config

## Decision Log
**Decision 1**: Stick with OpenTelemetry 0.31.0 (current workspace version)
- Rationale: No dependency churn, better long-term maintenance
- Rejected: Pinning to 0.24.x (technical debt)
```

**Gate**: Plan approved (self or team review)

---

### Phase 2: Implementation Roadmap

**Deliverable**: TODO list or tracker (can be in plan doc or separate issue)

**Steps:**

1. **Break Down Epics into Tasks**
   - Use TODO lists (e.g., TodoWrite tool in Claude Code)
   - Or create GitHub issues with `vX.Y.Z` milestone
   - Each task should be testable/verifiable

2. **Identify Dependencies**
   - Library version research
   - API compatibility checks
   - External documentation review

3. **Estimate Scope**
   - Rough time estimate (not commitments, just sanity check)
   - Flag high-risk/complex items early

**Example** (from v2.3.0):
```
1. Research OpenTelemetry 0.31.x API patterns
2. Implement try_init_otlp() in telemetry crate
3. Add otlp feature flag to Cargo.toml
4. Test OTLP with/without feature flag
5. Create OTLP testing guide (docs/how-to/test-otlp-tracing.md)
6. Update CHANGELOG.md
7. Tag and push release
```

**Gate**: Roadmap is clear and bounded

---

### Phase 3: Code Implementation

**Steps:**

1. **Create Feature Branch** (optional for solo work)
   ```bash
   git checkout -b feat/vX.Y.Z-epic-name
   ```

2. **Implement Code**
   - Follow guardrails from Phase 1
   - Add `#[cfg(feature = "...")]` for optional features
   - Write inline docs as you go

3. **Write Tests**
   - Unit tests for new functions
   - Integration tests for new adapters
   - BDD scenarios for new user-facing behavior

4. **Maintain TODO List**
   - Mark tasks as in_progress → completed
   - Add new tasks if discovered during implementation

**Example Code Pattern** (feature-gated):
```rust
#[cfg(feature = "otlp")]
fn try_init_otlp(service_name: &str, endpoint: &str) -> Result<(), Box<dyn Error>> {
    // Implementation
}

pub fn init_tracing(service_name: &str) {
    #[cfg(feature = "otlp")]
    {
        if let Ok(endpoint) = std::env::var("OTLP_ENDPOINT") {
            match try_init_otlp(service_name, &endpoint) {
                Ok(()) => return,
                Err(e) => eprintln!("OTLP init failed: {e}, falling back"),
            }
        }
    }

    // Default console tracing
    tracing_subscriber::fmt().init();
}
```

**Gate**: Code compiles, passes local smoke tests

---

### Phase 4: Validation Gates

**All gates must pass before release.**

#### Gate 1: Format Check
```bash
cargo fmt --all -- --check
```
**Fix**: `cargo fmt --all`

#### Gate 2: Clippy (Linter)
```bash
cargo clippy --all-targets --all-features -- -D warnings
```
**Fix**: Address clippy warnings (prefer fixing over `#[allow(...)]`)

#### Gate 3: Unit Tests
```bash
cargo test --workspace
```
**Fix**: Debug failing tests, add missing test cases

#### Gate 4: Selftest (Comprehensive)
```bash
cargo run -p xtask -- selftest
```
**Includes**:
- Format, clippy, tests (as above)
- BDD scenarios (`cargo run -p xtask -- bdd`)
- AC mapping (`cargo run -p xtask -- ac-status`)
- Policy tests (`cargo run -p xtask -- policy-test`, if `conftest` available)
- Bundler smoke test

**Fix**: Address any failures systematically

#### Gate 5: Feature Flag Matrix (if applicable)
```bash
# Default build (no features)
cargo build -p <crate>

# With optional features
cargo build -p <crate> --features <feature-name>

# All features
cargo build --all-features
```

**Example** (from v2.3.0):
```bash
cargo build -p telemetry                    # Console-only
cargo build -p telemetry --features otlp    # With OTLP
```

**Gate**: All validation passes cleanly

---

### Phase 5: Documentation

**Deliverables**: Updated docs, CHANGELOG entry

**Steps:**

1. **Create How-To Guides** (if new features)
   - `docs/how-to/test-<feature>.md`
   - Include copy-paste commands
   - Add troubleshooting section

2. **Update Crate READMEs** (if applicable)
   - `crates/<crate>/README.md`
   - Explain new APIs, feature flags, environment variables

3. **Update CHANGELOG.md**
   - Create new `## [X.Y.Z] - YYYY-MM-DD` section
   - Structure:
     - **Added**: New features, APIs
     - **Changed**: Breaking changes (avoid if possible)
     - **Fixed**: Bug fixes
     - **Technical Details**: File changes, dependencies
     - **Design Decisions**: Rationale for choices
     - **Validation**: What tests passed

**CHANGELOG Template**:
```markdown
## [X.Y.Z] - YYYY-MM-DD

### Added

**Feature Name**
- Description of what was added
- Key capabilities
- Environment variables / feature flags

**Documentation**
- `docs/how-to/...` - Brief description
- `crates/<crate>/README.md` - Brief description

### Technical Details

**Files Modified:**
- `path/to/file.rs` - What changed

**New Files:**
- `path/to/new-file.md` - Purpose

**Validation:**
- ✅ `cargo build ...` - Status
- ✅ `cargo clippy ...` - Status
- ✅ `cargo test ...` - Status

**Dependencies:**
- Library X.Y.Z - Why chosen

### Design Decisions

**Decision Name:**
- Chose: X
- Rationale: Why
- Rejected: Y (reason)

### Notes

- Important user-facing information
- How to enable new features
- Migration guidance (if applicable)
```

**Gate**: Docs are complete and accurate

---

### Phase 6: Release Tagging

**Deliverable**: Git tag `vX.Y.Z`, pushed to origin

**Steps:**

1. **Final Git Status Check**
   ```bash
   git status
   # Should be clean or only show intended changes
   ```

2. **Stage All Changes**
   ```bash
   git add <files>
   git status  # Verify staged files
   ```

3. **Create Commit**
   ```bash
   git commit -m "$(cat <<'EOF'
   feat(<scope>): <brief summary>

   <Detailed description>

   Key changes:
   - Change 1
   - Change 2

   Validation:
   - cargo clippy (clean)
   - cargo test (passing)
   - selftest (green)
   EOF
   )"
   ```

4. **Create Annotated Tag**
   ```bash
   git tag -a vX.Y.Z -m "$(cat <<'EOF'
   Project Name vX.Y.Z: <Release Theme>

   <Brief summary of what shipped>

   See CHANGELOG.md for full details.
   EOF
   )"
   ```

5. **Verify Tag**
   ```bash
   git tag -l | tail -5  # Should show new tag
   git show vX.Y.Z       # Should show tag message + commit
   ```

6. **Push to Origin**
   ```bash
   git push origin main --tags
   ```

**Example** (from v2.3.0):
```bash
git commit -m "feat(telemetry): add OTLP tracing support via feature flag

Implements OpenTelemetry 0.31.x OTLP export with graceful fallback.

Key changes:
- Added telemetry/otlp feature flag
- Implemented try_init_otlp() with gRPC transport
- Created docs/how-to/test-otlp-tracing.md
- Updated CHANGELOG.md with v2.3.0 entry

Validation:
- cargo clippy --all-features (clean)
- cargo test --workspace (passing)
- selftest (core checks green)"

git tag -a v2.3.0 -m "Rust IaC Template v2.3.0: OTLP Tracing

OTLP tracing support via feature flag with comprehensive testing guide.

See CHANGELOG.md for full details."

git push origin main --tags
```

**Gate**: Tag exists on remote, CI passes (if applicable)

---

### Phase 7: Post-Release

**Steps:**

1. **Verify Release Artifacts**
   - GitHub tag visible: `https://github.com/<org>/<repo>/releases/tag/vX.Y.Z`
   - CHANGELOG updated on main branch
   - Documentation updated

2. **Update Project Board** (if using)
   - Close `vX.Y.Z` milestone
   - Move any deferred tasks to next milestone

3. **Create Next Version Plan** (only if needed)
   - Don't rush into vX.Y+1.0
   - Wait for real usage feedback
   - Consider pilot projects first

4. **Document Lessons Learned**
   - What went well?
   - What was painful?
   - Update this playbook if process improved

**Example Post-Release Actions**:
```markdown
## v2.3.0 Retrospective

**What Worked:**
- Research phase (context7 MCP) found exact API patterns quickly
- Feature flag kept default builds clean
- Graceful fallback prevented production risk

**What Could Improve:**
- Initial clippy warnings (let-chain) - should have run clippy earlier
- Lifetime issues with .to_string() - better awareness of 'static requirements

**Next Steps:**
- Run greenfield pilot before planning v2.4.0
- Maintain friction log to inform future improvements
```

---

## Checklists

### Pre-Release Kernel Checklist

Quick sanity checks before any kernel release:

- [ ] `cargo xtask selftest` passes
- [ ] `cargo xtask docs-check` passes
- [ ] `cargo xtask contracts-check` passes (docs match selftest steps + AC counts)
- [ ] `cargo xtask ui-contract-check` passes (UI HTML matches specs/ui_contract.yaml)
- [ ] `cargo xtask idp-check` passes (OpenAPI + TS consumer + config)

### Pre-Release Checklist

- [ ] Plan document created (`docs/vX.Y.Z-plan.md`)
- [ ] Scope clearly defined (theme + epics + guardrails)
- [ ] TODO list or tracker created
- [ ] Code implementation complete
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] `cargo run -p xtask -- selftest` passes
- [ ] `cargo run -p xtask -- docs-check` passes
- [ ] `cargo run -p xtask -- contracts-check` passes (docs match selftest + ledger)
- [ ] `cargo run -p xtask -- idp-check` passes (OpenAPI + TS consumer + config)
- [ ] Feature flag matrix tested (if applicable)
- [ ] How-to guides created (if new features)
- [ ] Crate READMEs updated (if applicable)
- [ ] CHANGELOG.md updated with vX.Y.Z entry
- [ ] Git status clean (only intended changes)
- [ ] Commit message follows format
- [ ] Tag created with annotated message

### Post-Release Checklist

- [ ] Tag pushed to origin
- [ ] CI passes (if applicable)
- [ ] GitHub release visible
- [ ] CHANGELOG visible on main branch
- [ ] Milestone closed (if using)
- [ ] Retrospective notes captured
- [ ] Friction log template ready for pilots

---

## Governance Alignment

This playbook enforces:

1. **Traceability**: Every release links to specs (AC IDs), code (commits), tests (BDD), and docs (CHANGELOG)
2. **Quality**: Multiple validation gates prevent regressions
3. **Transparency**: Decision log captures rationale for future reference
4. **Stability**: Guardrails prevent scope creep and breaking changes

**Spec Integration**:
- Plan documents reference AC IDs from `specs/spec_ledger.yaml`
- BDD scenarios map to AC IDs
- `xtask ac-status` shows coverage before release

**Policy Integration**:
- `xtask policy-test` runs OPA/Rego checks
- Prevents releasing code that violates policies (K8s, privacy, etc.)

---

## Adaptations for Other Projects

This playbook is designed for the Rust IaC Template but adapts to:

### Smaller Projects
- **Simplify**: Skip plan docs for trivial releases
- **Keep**: Validation gates (fmt, clippy, test, CHANGELOG)
- **Keep**: Git tagging for traceability

### Larger Teams
- **Add**: PR reviews before merge
- **Add**: Release manager role
- **Add**: Staging environment validation
- **Keep**: All validation gates

### Non-Rust Projects
- **Replace**: `cargo` commands with language-specific tooling (e.g., `pytest`, `npm test`)
- **Keep**: Phase structure (Plan → Code → Validate → Document → Tag)
- **Keep**: CHANGELOG discipline

---

## References

- **Template Releases**: [CHANGELOG.md](../CHANGELOG.md)
- **Example Plan**: [docs/v2.3.0-plan.md](v2.3.0-plan.md)
- **Selftest Docs**: [docs/SELFTEST.md](SELFTEST.md)
- **AC Ledger**: [specs/spec_ledger.yaml](../specs/spec_ledger.yaml)

---

## Version History

| Version | Date       | Changes                          |
|---------|------------|----------------------------------|
| 1.0.0   | 2025-11-17 | Initial playbook extraction      |

---

**End of Release Playbook**
