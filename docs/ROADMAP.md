# Rust Template Roadmap

**Last Updated**: 2025-11-17
**Current Version**: v2.3.0
**Status**: Pilot Phase

---

## Current Status: Production-Ready Template

The Rust IaC Template has reached a natural completion point after three focused releases:

| Release | Theme | Status |
|---------|-------|--------|
| **v2.1.0** | Metrics foundation | ✅ Shipped (2025-11-17) |
| **v2.2.0** | Adapter integration + LLM ergonomics | ✅ Shipped (2025-11-17) |
| **v2.3.0** | OTLP tracing + pilot infrastructure | ✅ Shipped (2025-11-17) |

**What's Complete:**

- ✅ Complete observability stack (logs, metrics, OTLP traces)
- ✅ Hexagonal architecture with production adapters (DB, gRPC)
- ✅ Governance infrastructure (policy-as-code, specs, BDD)
- ✅ LLM-assisted workflow (contextpack, bundles, AC mapping)
- ✅ Release playbook and pilot evaluation tooling
- ✅ Comprehensive documentation (Diátaxis framework)

**Strategic Position:**
The template is "production-ready" as a starting point. Further evolution should be **demand-driven**, informed by real-world pilot projects rather than speculative features.

---

## Near-Term: Pilot Validation Phase

**Goal**: Validate template usability through real greenfield development.

### Pilot Project Workflow

```bash
# 1. Create pilot project
./scripts/create-pilot.sh <project-name> ~/projects/

# 2. Day 0: Verify setup
cargo run -p xtask -- selftest

# 3. Day 1+: Implement features
#    - Add AC to specs/spec_ledger.yaml
#    - Add BDD scenario
#    - Run: cargo run -p xtask -- bundle implement_ac
#    - Feed to LLM, apply changes
#    - Validate: cargo run -p xtask -- selftest

# 4. Record friction
#    - Every rough edge → FRICTION_LOG.md
#    - Missing docs, confusing errors, unclear behavior

# 5. After 1-2 weeks: Analyze friction
#    - Review FRICTION_LOG.md
#    - Classify: 🔴 Blockers / 🟡 Annoyances / 🟢 Nice-to-have
#    - Decide: v2.3.1 patch, v2.4.0 features, or template-as-is
```

### Pilot Infrastructure Available

**Tools:**

- `scripts/create-pilot.sh` - Automated pilot setup
- `docs/templates/FRICTION_LOG.md` - Structured friction capture
- `docs/templates/PILOT_FEATURE_IDEAS.md` - Curated feature examples
- `docs/RELEASE_PLAYBOOK.md` - Release process guidance

**Recommended Pilot Features** (from PILOT_FEATURE_IDEAS.md):

1. Create Task (30 min) - Basic HTTP + model creation
2. List Tasks (20 min) - Repository pattern
3. Update Task Status (60 min) - State transitions + validation
4. Task Metrics with Tracing (90 min) - Observability integration

**Exit Criteria:**

- Pilot implements 3-5 real features
- FRICTION_LOG.md has 5-10 meaningful entries
- Decision made: patch, new release, or status quo

---

## Decision Point: Post-Pilot (After Pilot Completes)

Based on friction log analysis, one of three paths:

### Path A: v2.3.1 Patch (If Blockers Found)

**Trigger**: Friction log contains 🔴 blocker-level issues

**Examples of blockers:**

- Selftest crashes on valid projects
- Critical docs missing (e.g., how to add database migrations)
- LLM workflow doesn't work end-to-end
- Policy tests fail on template defaults

**Scope**: Bug fixes and critical doc updates only

**Timeline**: 1-2 days

**Release**: Patch version (v2.3.1)

---

### Path B: v2.4.0 Features (If Quality-of-Life Improvements Needed)

**Trigger**: Friction log contains 🟡 annoyances worth addressing

**Potential features (examples, not committed):**

- Improved `xtask` output formatting
- Additional how-to guides based on pilot learnings
- Database migration tooling
- Enhanced error messages from selftest
- Additional policy examples

**Scope**: Non-breaking enhancements driven by pilot experience

**Timeline**: 1-2 weeks

**Release**: Minor version (v2.4.0)

**Planning Process:**

1. Extract top 3-5 annoyances from friction log
2. Create v2.4.0-plan.md using docs/templates/RELEASE_PLAN.md
3. Follow Release Playbook phases
4. Ship when scope complete

---

### Path C: Template "Good Enough" (If Minimal Friction)

**Trigger**: Friction log shows template is usable as-is

**Actions:**

- Declare template production-ready for general use
- Focus on adoption, not new features
- Address issues reactively as users report them
- Maintain stability over novelty

**What "good enough" means:**

- All core workflows function without blockers
- Documentation covers common use cases
- Pilot project completes without major pain points
- Friction points are minor or subjective

**Future work**: Demand-driven, not roadmap-driven

---

## Long-Term: Demand-Driven Evolution

After the pilot phase and any immediate follow-up (v2.3.1 or v2.4.0), the template should evolve based on:

1. **User reports** - Issues filed by teams using the template
2. **Adoption patterns** - Common customizations extracted as template improvements
3. **Ecosystem changes** - Rust/OpenTelemetry/infra tooling updates
4. **Strategic needs** - EffortlessMetrics platform requirements

### Potential Future Areas (Examples, Not Commitments)

These are not planned features—they're examples of what *might* happen if pilot projects demand them.

**Observability (examples):**

- OTLP log export (today: traces via OTLP, logs to console)
- Custom trace attributes (user_id, tenant_id)
- Distributed tracing examples across adapters
- Structured logging improvements

**Adapters:**

- Message queue adapter (RabbitMQ, Kafka)
- Cache adapter (Redis, Memcached)
- Additional database adapters (DynamoDB, MongoDB)

**Developer Experience:**

- `xtask lint` combining fmt + clippy in one command
- Improved bundle generation (per-AC context)
- IDE integrations beyond VSCode tasks
- Docker build automation in `xtask deploy`

**Governance:**

- Additional policy examples (RBAC, cost limits)
- Policy testing in CI with detailed reports
- Spec ledger tooling (dependency graphs, coverage)

**Infrastructure:**

- Helm chart generation
- Terraform/Pulumi integration
- Multi-region deployment patterns

**LLM Workflow:**

- Pre-commit hooks for bundle regeneration
- AC-to-code diff tracking
- LLM prompt templates for common tasks

---

## Principles for Future Work

### 1. Demand-Driven, Not Speculation

- Features should solve real problems reported by users
- Avoid "wouldn't it be cool if..." additions
- Friction logs and issue reports drive priorities

### 2. Stability Over Novelty

- Template users value predictability
- Breaking changes require strong justification
- Maintain backward compatibility by default

### 3. Bounded Scope

- Small, focused releases (like v2.1.0, v2.2.0, v2.3.0)
- Avoid sprawling feature lists
- Ship when scope is complete, not on deadlines

### 4. Documentation First

- New features ship with how-to guides
- Update tutorials when workflows change
- Keep Release Playbook and templates up-to-date

### 5. Governance Hygiene

- All features have ACs and BDD scenarios
- Policy tests cover new config/infra
- Selftest validates new commands

---

## Version History & Context

### v2.0.0 → v2.3.0: Observability Arc

**Three-release progression** to close observability gaps:

1. **v2.1.0 (Metrics)**: Prometheus `/metrics` endpoint, policy enforcement
2. **v2.2.0 (Adapters + LLM)**: DB/gRPC integration tests, contextpack improvements
3. **v2.3.0 (Tracing + Pilots)**: OTLP export, pilot infrastructure

**Result**: Complete logs/metrics/traces stack + governance + LLM workflow

### Pre-v2.0.0: Foundation

- v1.0.0: Initial stable release (xtask, BDD, policies, docs)
- v1.1.0: Template contracts, K8s manifests, deploy command
- v2.0.0: Workspace stabilization, hexagonal architecture

**Lessons Learned:**

- Small, focused releases ship faster and cleaner
- Post-release reflection (Phase 7) prevents feature creep
- Governance artifacts (playbooks, templates) are as valuable as code

---

## How to Influence the Roadmap

**If you're using the template:**

1. **File issues** for bugs, missing docs, confusing behavior
2. **Share friction logs** from your pilot projects (attach `FRICTION_LOG.md` excerpts)
3. **Propose features** with specific use cases (not abstractions)
4. **Contribute PRs** following the Release Playbook process

**What helps:**

- Concrete examples: "I couldn't figure out how to X"
- Context: "For my Y use case, I needed Z"
- Friction logs: "After 2 days, these 3 things were painful"

**What doesn't help:**

- Feature requests without use cases
- "It would be cool if..." without demand signal
- Spec-first PRs without real-world validation

---

## Current Focus: Pilot Phase

**Next milestone**: Complete a greenfield pilot project using v2.3.0

**Success criteria:**

- Pilot implements 3-5 real features
- Friction log has actionable data
- Decision made on next release (if any)

**Timeline**: ~1-2 weeks of dev time per pilot (guideline, not a hard deadline)

**Then**: Review this roadmap based on pilot learnings

---

## FAQ

### Q: When is v2.4.0 planned?

**A**: Not planned yet. Waiting for pilot friction analysis to determine if it's needed.

### Q: Will there be breaking changes in v3.0.0?

**A**: Not planned. Template is stable at v2.x. Breaking changes would require strong justification.

### Q: How do I request a feature?

**A**: File an issue with your use case. Explain the problem you're solving, not the solution you want.

### Q: Is the template "done"?

**A**: The template is "production-ready" but not "done." It will evolve based on real-world usage, not a predetermined roadmap.

### Q: Can I fork and customize?

**A**: Yes! See [Adoption Patterns](explanation/adoption-patterns.md) for guidance on forking, upstream tracking, and platform customization.

---

## References

- [Release Playbook](RELEASE_PLAYBOOK.md) - How releases are planned and executed
- [v2.3.0 Plan](v2.3.0-plan.md) - Most recent release details
- [Pilot Feature Ideas](templates/PILOT_FEATURE_IDEAS.md) - Suggested pilot features
- [Friction Log Template](templates/FRICTION_LOG.md) - How to capture pilot friction

---

**Last reviewed**: 2025-11-17
**Next review**: After pilot project completes (estimated 1-2 weeks)
