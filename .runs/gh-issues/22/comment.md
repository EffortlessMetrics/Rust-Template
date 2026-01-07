## Investigation Report: Issue #22 - Developer Onboarding

### Status
**Status:** Well-governed, ready for usability improvements
**Local gates:** Documentation audit completed

### Evidence

**Existing onboarding resources (EXCELLENT):**
- `CLAUDE.md` (513 lines) - Full operational constitution
- `docs/how-to/first-hour.md` (290 lines) - Human-focused
- `docs/how-to/ai-first-hour.md` (243 lines) - Agent-focused
- `docs/MISSING_MANUAL.md` (680 lines) - "Things you'll wish someone told you"
- `.claude/skills/bootstrap-dev-env/SKILL.md` (230 lines) - Governed workflow

**Governance:**
- REQ-PLT-ONBOARDING with 5 ACs (AC-PLT-001, 002, 003, 018, 021)
- `cargo xtask dev-up` provides one-command bootstrap
- Friction log captures DevEx issues (FRICTION-AGENT-002 resolved)

**Gaps identified:**
1. No "START HERE" in README
2. No unified onboarding checklist
3. 6+ entry-point documents without clear hierarchy
4. No `/platform/onboarding/status` endpoint

### Plan

**Priority 1 (1-2 hours):**
- Add "START HERE" section to README
- Create `docs/ONBOARDING_CHECKLIST.md`

**Priority 2 (2-3 hours):**
- Create `/platform/onboarding/status` endpoint
- Add `/ui/onboarding` progress dashboard

### Decision / Next Action

**Recommend:** Keep open for usability improvements. Core onboarding is solid; navigation could improve.
