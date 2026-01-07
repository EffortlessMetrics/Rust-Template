## Investigation Report: Adoption Phases 1-4 (#64-67)

### Phase 1 (#64) - Fork Usage Baseline
**Goal:** Test kernel invariants in real service environment

**Status:** OPEN (planned)

**Key Activities:**
- Fork template using `v3.3.9-kernel` tag into new service repository
- Validate baseline with `nix develop && cargo xtask doctor && cargo xtask selftest`
- Wire service identity via `specs/service_metadata.yaml`
- Add domain stories/REQs/ACs to `specs/spec_ledger.yaml`
- Capture friction in fork's `FRICTION_LOG.md` and surface via `/platform/friction` API

**Success Criteria:**
- Selftest green on every fork PR
- Friction systematically logged and categorized
- Service builds and runs with template governance intact

**Execution Scope:** Happens in external fork repos, not in kernel repo itself

---

### Phase 2 (#65) - IDP Tile Integration
**Goal:** Surface governance and docs health in developer portal

**Status:** OPEN (planned; depends on Phase 1)

**Key Activities:**
- Build **Governance Health tile** using `/platform/status` endpoint (AC pass/fail, policy status, selftest results)
- Build **Docs Health tile** using `/platform/docs/index` endpoint (doc types, coverage, staleness)
- (Optional) Add **Task/Hints tile** using `/platform/agent/hints` endpoint for prioritized work visibility
- Reference `docs/explanation/json-contracts.md` for JSON schema contracts
- Validate tile data against Phase 1 fork services

**Success Criteria:**
- Template-based services visible in IDP with health metrics
- Teams can see governance drift in real-time
- Documentation health surfaced without manual audits

**Reference:** `examples/backstage-plugin/` provides starter implementation patterns

**Data Sources:**
| Tile | Endpoint | Key Fields |
|------|----------|------------|
| Governance Health | `/platform/status` | `ac_coverage`, `policy_status`, `auth_mode` |
| Docs Health | `/platform/docs/index` | `doc_count`, `types`, `coverage` |
| Agent Hints | `/platform/agent/hints` | `tasks`, `priority`, `context` |

---

### Phase 3 (#66) - Governed Agent Pilot
**Goal:** Validate kernel is truly agent-friendly

**Status:** OPEN (planned; depends on Phase 1)

**Key Activities:**
- Deploy Claude Code agents to 2-3 fork repos from Phase 1
- Use Skills: `bootstrap-dev-env`, `governed-feature-dev`, `governed-maintenance`
- Agent workflow loop:
  1. Query `/platform/agent/hints` for prioritized tasks
  2. Generate context bundle via `cargo xtask bundle <task_name>`
  3. Edit code/tests/docs within bundle scope
  4. Validate with `cargo xtask test-ac <AC_ID>`
  5. Gate on `cargo xtask selftest` before PR
- Require AC/REQ/Doc invariants green (`docs-check` + `selftest`)
- Capture agent friction separately from human developer friction

**Success Criteria:**
- Agents productive in 2-3 real service repos without human intervention
- Agent-generated PRs pass selftest on first attempt >80% of the time
- Clear friction log distinguishing agent vs. human developer pain points

**Expected Output:** AI First-Hour Receipt with workflow completion status, selftest pass rate, and friction items

---

### Phase 4 (#67) - Kernel vNext (Demand-Driven)
**Goal:** Batch real feedback from Phases 1-3 into next kernel version

**Status:** OPEN (planned; depends on Phases 1-3)

**Key Activities:**
- Review friction logs from all three prior phases (fork usage, IDP integration, agent pilots)
- Categorize feedback into:
  - **Kernel fixes:** gaps in `spec_ledger.yaml`, broken contracts, missing flows
  - **Soft → Hard promotions:** checks validated in real usage, ready to gate
  - **JSON contract refinements:** IDP/agent usage reveals schema gaps
  - **Out-of-scope:** fork-specific needs (document, don't implement)
- Promote validated soft checks to hard gates
- Refine JSON contracts based on IDP and agent integration patterns
- Implement versioning engine refactor if `release-prepare` friction is systematic
- Add new patterns discovered in forks

**Success Criteria:**
- v3.4.0 released with changes **driven by friction**, not speculation
- All promoted hard gates have evidence from ≥2 fork repos
- JSON contracts validated by real IDP/agent consumers
- Kernel changelog clearly attributes improvements to fork feedback

**Feedback Classification:**
| Category | Action | Example |
|----------|--------|---------|
| Kernel fix | Fix in kernel | Broken AC mapping |
| Soft → Hard | Promote gate | `docs-check` strictness |
| Contract refinement | Update schema | Add field to `/platform/status` |
| Out-of-scope | Document, don't fix | Domain-specific validation |

---

### Overall Adoption Roadmap Status

**Adoption Program Structure:**
1. **Phase 1** validates kernel in real service environments (fork baseline)
2. **Phase 2** surfaces kernel health in IDP tiles (developer visibility)
3. **Phase 3** validates agent-friendliness with Claude Code pilots (automation)
4. **Phase 4** batches real feedback into kernel improvements (demand-driven evolution)

**Current State:** All four phases are OPEN and sequentially planned
- Phase 1 is the critical path entry point (happens in external repos)
- Phase 2 depends on Phase 1 outputs (friction + domain ACs)
- Phase 3 depends on Phase 1 (populated fork repos with real work)
- Phase 4 depends on Phases 1-3 (friction logs + feedback)

**Key Contracts:**
- Fork repos use `v3.3.9-kernel` tag as baseline
- Each phase produces evidence (friction logs, receipts, PR pass rates)
- Phase 4 decision to upgrade to v3.4.0 requires evidence from ≥2 Phase 1 forks
- IDP tiles and agent workflows validate that governance contracts work in real usage

---

### Decision / Next Action

**Recommended Next Steps:**

1. **Initiate Phase 1 (Fork Usage Baseline)**
   - Select or create 2-3 real service forks using `v3.3.9-kernel` tag
   - Run baseline validation (`doctor`, `selftest`)
   - Establish friction logging discipline
   - Expected timeline: 2-4 weeks per fork

2. **Prepare Phase 2 (IDP Integration)**
   - Identify target IDP platform (e.g., Backstage instance)
   - Reference `examples/backstage-plugin/` implementation patterns
   - Begin tile prototype against kernel `/platform/status` endpoint
   - Can prototype in parallel with Phase 1 fork work

3. **Stage Phase 3 (Agent Pilots)**
   - Ensure Phase 1 forks have populated domain ACs and tasks
   - Deploy Claude Code agents with `bootstrap-dev-env` skill first
   - Track agent productivity metrics and friction separately
   - Target: agents productive in >80% of work items

4. **Track Phase 4 Trigger**
   - Establish friction log review cadence (weekly/bi-weekly)
   - Define promotion criteria for soft→hard gate migrations
   - Document each kernel fix with fork evidence trail
   - Plan v3.4.0 release once Phase 1-3 friction is consolidated

**Success Indicators:**
- Phase 1: 2+ fork repos with selftest green, friction logs populated
- Phase 2: IDP tiles display real governance data from Phase 1 forks
- Phase 3: Agents complete 80%+ of tasks without human intervention
- Phase 4: v3.4.0 changelog lists friction-driven improvements with fork references

