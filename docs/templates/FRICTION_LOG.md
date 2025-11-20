# Friction Log Template

**Purpose:** Track pain points encountered during development to refine the governance model

**Instructions:**
1. Create a copy of this file as `FRICTION_LOG.md` in the repo root
2. Add entries whenever something slows you down or feels awkward
3. Use this during weekly pilot syncs to prioritize fixes

---

## Entry Template

```markdown
### [YYYY-MM-DD] {Short Title}

**Context:** What were you trying to do?

**Friction:** What got in your way?

**Impact:** How much did this slow you down?
- [ ] Blocker (couldn't proceed)
- [ ] Major (lost > 30 min)
- [ ] Minor (lost < 30 min)
- [ ] Papercut (annoying but quick)

**Proposed Fix:** What would have made this smoother?

**Related:**
- Task: {task_id}
- Flow: {flow_name}
- Command: `cargo xtask {command}`
- Spec: {file and line}
```

---

## Example Entries

### [2025-01-15] Creating AC with Long Description

**Context:** I wanted to add an AC for "Users can upload profile pictures" with a detailed description including size limits, formats, and validation rules.

**Friction:** `cargo xtask ac-new` only accepts a single quoted string. My description had quotes in it, and escaping them in the shell was painful.

**Impact:**
- [x] Minor (lost < 30 min)

**Proposed Fix:**
- Option A: `ac-new --interactive` that opens $EDITOR
- Option B: `ac-new --from-file=ac-draft.yaml`
- Option C: Better help text showing how to use heredocs

**Related:**
- Task: `implement_ac`
- Command: `cargo xtask ac-new`
- Spec: `specs/spec_ledger.yaml`

---

### [2025-01-16] Selftest Failure Message Unclear

**Context:** Running `cargo xtask selftest` after adding a new command to `devex_flows.yaml`.

**Friction:** Got an error "[7/7] Graph invariants failed" but had to dig through output to find "Command 'my-new-cmd' not found in devex.commands". The error didn't tell me which YAML section to fix.

**Impact:**
- [x] Major (lost > 30 min)

**Proposed Fix:**
- Make graph invariant errors include actionable hints:
  ```
  [COMMAND_UNREACHABLE] Command 'my-new-cmd' used in flow 'my-flow' but not defined
  → Fix: Add to devex_flows.yaml under 'commands:' section
  → See: docs/how-to/adding-commands.md
  ```

**Related:**
- Flow: `ci_parity`
- Command: `cargo xtask selftest`
- Code: `crates/spec-runtime/src/graph.rs:L165`

---

### [2025-01-18] Bundle Takes Too Long

**Context:** Running `cargo xtask bundle implement_ac` before asking LLM to implement.

**Friction:** Bundle generation took 8 seconds. When iterating quickly (implement → test → ask LLM → repeat), this adds up.

**Impact:**
- [ ] Blocker
- [ ] Major
- [x] Papercut (annoying but quick)

**Proposed Fix:**
- Cache: Only regenerate bundle if specs/tasks/docs changed
- Async: Show "Generating bundle..." progress indicator
- Parallel: Bundle could be generated in background when `ac-new` runs

**Related:**
- Task: `implement_ac`
- Command: `cargo xtask bundle`
- Code: `crates/xtask/src/commands/bundle.rs`

---

## Guidelines for Good Entries

**Do:**
- ✅ Be specific (include exact commands, error messages)
- ✅ Note what you expected vs what happened
- ✅ Suggest fixes (even if you're not sure they're right)
- ✅ Include timestamps and impact level
- ✅ Link to related files/commands/docs

**Don't:**
- ❌ Say "X is broken" without context
- ❌ Complain without proposing solutions
- ❌ Leave impact unchecked (helps prioritization)
- ❌ Dupe entries (search log first)

---

## During Pilot Syncs

Review friction log weekly:

1. **Blockers:** Must fix immediately (patch release)
2. **Major:** Prioritize for next sprint
3. **Minor:** Batch into "DX improvements" epic
4. **Papercuts:** Accumulate; fix when quick wins are needed

**Decision Framework:**
- Does it affect governance integrity? → Fix ASAP
- Does it make AC-first workflow painful? → High priority
- Does it slow every developer? → Medium priority
- Does it affect edge cases? → Low priority unless trivial fix

---

## After Pilot

Friction log becomes the input for:
- v2.5.0 improvements
- Documentation gaps
- UX refinements
- Policy/invariant tuning

Keep it updated throughout pilot. This is how we learn what "complete" actually means.
