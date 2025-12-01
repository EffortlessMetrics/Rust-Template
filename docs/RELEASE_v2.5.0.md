<!-- doclint:disable orphan-version -->
<!-- Historical: This document describes a completed release and version references are intentionally preserved. -->

# v2.5.0 Release Tag Instructions

## Status
✅ Kernel Complete  
✅ Pilot Validated (Platform bootstrapped itself)  
✅ Documentation Updated  
✅ CHANGELOG.md Published

## Tag the Release

```bash
# Verify clean state
cargo xtask selftest
# Should pass: ✅ All 7 steps

# Create annotated tag
git add -A
git commit -m "Release v2.5.0: Agent-Ready Platform Cell (Kernel Frozen)"
git tag -a v2.5.0 -m "Release v2.5.0: Agent-Ready Platform Cell

- Phase 4 complete (graph invariants, suggest-next, policy status, Web UI)
- Pilot complete (docker-compose, install-hooks, .claude/skills/*)
- Platform used itself to build final features
- Kernel frozen: next phase is real-world service pilot

All 7 selftest steps pass. 22/22 policy tests pass.
Graph invariants enforced. Agent interface complete.

See CHANGELOG.md for full details."

# Push
git push origin main
git push origin v2.5.0
```

## Verification

After pushing:

1. **Check GitHub Actions CI**
   - All workflows should pass
   - Release artifacts should generate

2. **Verify Tag**
   ```bash
   git tag -l -n9 v2.5.0
   ```

3. **Announce Freeze**
   - Update GitHub repo description
   - Pin issues about pilot phase
   - Create `FRICTION_LOG.md` template at repo root

## Next Steps

**DO NOT:**
- Add new xtask commands
- Modify spec schemas
- Add graph invariants
- Expand platform APIs
- Build new UI features

**DO:**
- Build real service on top
- Log friction in `FRICTION_LOG.md`
- Fix bugs found via usage
- Improve error messages
- Enhance documentation clarity

**Pilot Template:**
```bash
# Option A: Clone as new service
git clone https://github.com/EffortlessMetrics/Rust-Template.git task-api
cd task-api
./scripts/init-service.sh task-api "Task Management API"

# Option B: In-mono repo experimentation
# Add crates/task-api-* to existing workspace
```

## Freeze Boundaries

**Locked (no changes without FRICTION_LOG.md entry):**
- `specs/*.yaml` schema structure
- `crates/spec-runtime/src/graph.rs` invariants
- `crates/xtask/src/commands/*` (existing commands)
- `/platform/*` API contracts
- `/ui` pages and routes
- `.claude/skills/*` workflow definitions

**Open (continuous improvement):**
- Documentation (`docs/*`, `README.md`)
- Error messages and help text
- Examples and tutorials
- Bug fixes (non-breaking)
- Performance optimizations

---

**Release Complete. Kernel Frozen. Begin Pilot Phase.**
