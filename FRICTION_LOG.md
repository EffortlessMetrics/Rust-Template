# Friction Log

**Purpose:** Track pain points encountered during development to refine the governance model

**Status:** Template - Copy to repo root as `FRICTION_LOG.md` and maintain actively

---

## Active Friction Items

### [2025-11-20] Pre-Commit Hook Calling Wrong Command

**Context:** Installed git hook via `cargo xtask install-hooks`

**Friction:** Hook was calling `cargo xtask check` which doesn't exist inside `nix develop` (no cargo alias)

**Impact:**
- [x] Blocker (couldn't commit)

**Root Cause:** Hook assumed `cargo xtask` alias exists, but xtask is a workspace crate, not a cargo plugin

**Fix Applied:** Changed hook to call `cargo run -p xtask -- check` which works everywhere

**Lesson:** Always use `cargo run -p xtask --` in automated contexts (hooks, CI, skills)

**Related:**
- Command: `cargo xtask install-hooks`
- File: `crates/xtask/src/commands/install_hooks.rs`

---

## Guidelines for New Entries

```markdown
### [YYYY-MM-DD] {Short Title}

**Context:** What were you trying to do?

**Friction:** What got in your way?

**Impact:**
- [ ] Blocker (couldn't proceed)
- [ ] Major (lost > 30 min)
- [ ] Minor (lost < 30 min)
- [ ] Papercut (annoying but quick)

**Root Cause:** Why did this happen?

**Proposed Fix:** What would make this smoother?

**Related:**
- Task: {task_id}
- Flow: {flow_name}
- Command: `cargo xtask {command}`
- File: {file and line}
```

---

## Triage Process (Weekly)

1. **Blockers** → Immediate patch (v2.5.1, v2.5.2, etc.)
2. **Major** → Batch into next minor (v2.5.x)
3. **Minor** → Accumulate for v2.6.0
4. **Papercuts** → Fix when quick wins needed

**Decision Framework:**
- Does it reduce cognitive load? → Consider
- Does it increase spec/code drift risk? →Reject
- Does it make agents smarter? → Prioritize
- Does it require new state/config? → Scrutinize

---

## Resolved Items

*(Move entries here after they've been addressed)*

None yet - just starting pilot!
