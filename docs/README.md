<!-- doclint:disable orphan-version -->
# Documentation Index

Welcome to the Rust Template documentation. This template provides spec-as-code, AC-as-code, policy-as-code, and LLM-native development for Rust services.

**Quick Links:**
- 🚀 [Getting Started](#getting-started)
- 🧪 [Pilot Projects](#pilot-projects) ← Validate the template
- 📖 [Learning Path](#learning-path-new-to-template)
- 🎯 [Common Tasks](#common-tasks)
- 🗺️ [Roadmap](#roadmap)
- 📚 [Complete Reference](#complete-reference)

---

## Documentation Structure

This documentation follows the [Diátaxis](https://diataxis.fr/) framework:

| Type | Purpose | When to Use |
|------|---------|-------------|
| **Tutorials** | Learning-oriented, step-by-step | New to template, want to learn |
| **How-to Guides** | Task-oriented, solve specific problems | Know what you want to do |
| **Reference** | Information-oriented, technical specs | Need specific details |
| **Explanation** | Understanding-oriented, concepts | Want to understand why |

---

## Pilot Projects

**Want to validate the template before committing?** Create a greenfield pilot project to test real-world usage.

### Quick Start

**Option 1: GitHub "Use this template"**

```bash
# 1. In GitHub UI: click "Use this template" → create repo
# 2. Clone locally
git clone git@github.com:your-org/my-pilot-service.git
cd my-pilot-service

# 3. Enter dev environment and validate
nix develop
cargo run -p xtask -- selftest
```

**Option 2: Manual git clone**

```bash
# 1. Clone and reset git
git clone git@github.com:EffortlessMetrics/Rust-Template.git my-pilot-service
cd my-pilot-service
rm -rf .git
git init
git remote add origin git@github.com:your-org/my-pilot-service.git

# 2. Enter dev environment and validate
nix develop
cargo run -p xtask -- selftest
```

### Resources
- 📘 **[Pilot Feature Ideas](templates/PILOT_FEATURE_IDEAS.md)** - Curated features to test the template
  - Task Management API (starter)
  - E-commerce Order API (intermediate)
  - User Authentication API (advanced)
- 📋 **[Friction Log Template](templates/FRICTION_LOG.md)** - Track pain points during pilot
- 📖 **[Release Playbook](RELEASE_PLAYBOOK.md)** - Understand how friction informs template evolution

**Goal**: Understand template usability through real development, not speculation.

See [Roadmap](#roadmap) for how pilot results inform future releases.

---

## Getting Started

**Brand new?** Follow this path:

### Path 1: Quick Start (Recommended)

1. 📘 **[Quick Start Guide](QUICKSTART.md)** (15 min) **← START HERE**
   - Fast-track orientation and environment setup
   - Essential commands and validation
   - Get productive immediately

2. 📘 **[Day 1: First Change](tutorials/day-1-first-change.md)** (30 min)
   - Clone, validate, run the service
   - Add trivial AC and see it go green
   - Understand AC-first development loop

3. 📘 **[Day 7: First Real Feature](tutorials/day-7-first-real-feature.md)** (90 min)
   - Build a complete task management feature
   - Learn multi-layer architecture
   - Master validation, errors, and testing

4. 📕 **[Adoption Patterns](explanation/adoption-patterns.md)** (15 min)
   - Choose Pattern A, B, or C for your organization
   - Plan template update strategy
   - Understand trade-offs

### Path 2: Deep Dive (Alternative)

1. 📗 **[Architecture Overview](explanation/architecture.md)** (20 min read)
   - Design philosophy and rationale
   - Hexagonal architecture explained
   - Governance and policy model

2. 📘 **[Getting Started Tutorial](tutorials/getting-started.md)** (30 min)
   - Comprehensive environment setup
   - Detailed command explanations
   - All subsystems overview

3. 📕 **[First AC Change Tutorial](tutorials/first-ac-change.md)** (15 min)
   - Complete AC-first workflow
   - Spec → Test → Code → Validate

---

## Learning Path (New to Template)

### Week 1: Foundations

**Day 1: Environment & First Change**
- [ ] Complete [Day 1: First Change Tutorial](tutorials/day-1-first-change.md) (30 min)
- [ ] Run `xtask quickstart` successfully
- [ ] Make one trivial AC change end-to-end

**Day 2-4: AC-First Development**
- [ ] Read [Architecture Overview](explanation/architecture.md) (20 min)
- [ ] Complete [Getting Started Tutorial](tutorials/getting-started.md) (optional deep dive)
- [ ] Read [How to Add HTTP Endpoint](how-to/add-http-endpoint.md)

**Day 5-7: Real Feature Implementation**
- [ ] Complete [Day 7: First Real Feature Tutorial](tutorials/day-7-first-real-feature.md) (90 min)
- [ ] Read [LLM Bundles Guide](how-to/use-llm-bundles.md)
- [ ] Generate and use one LLM bundle for your feature

### Week 2: Production Readiness

**Day 1-2: Service Adaptation**
- [ ] Follow [New Service from Template](how-to/new-service-from-template.md)
- [ ] Read [Adoption Patterns](explanation/adoption-patterns.md) (choose your pattern)
- [ ] Update ownership in specs/
- [ ] Configure branch protection

**Day 3-4: CI & Governance**
- [ ] Read [Branch Protection Profiles](reference/branch-protection-profiles.md)
- [ ] Choose profile (Minimal/Standard/Strict)
- [ ] Enable required checks

**Day 5: Advanced**
- [ ] Explore policy/*.rego files
- [ ] Review TEMPLATE_API.md
- [ ] Set up pre-commit hooks
- [ ] Plan template upgrade strategy (if using Pattern B or C)

---

## Common Tasks

### Development

| Task | Guide | Time |
|------|-------|------|
| Add new HTTP endpoint | [How-to Guide](how-to/add-http-endpoint.md) | 15 min |
| Implement new AC | [Tutorial](tutorials/first-ac-change.md) | 20 min |
| Use LLM for coding | [How-to Guide](how-to/use-llm-bundles.md) | 10 min |
| Run all validations | [Reference](reference/xtask-commands.md#xtask-selftest) | 5 min |

### Setup & Configuration

| Task | Guide | Time |
|------|-------|------|
| Create service from template | [How-to Guide](how-to/new-service-from-template.md) | 10 min |
| Configure branch protection | [Reference](reference/branch-protection-profiles.md) | 15 min |
| Set up local environment | [Tutorial](tutorials/getting-started.md#step-1-clone-and-enter-environment-5-minutes) | 5 min |

### Troubleshooting

| Issue | Solution |
|-------|----------|
| `xtask check` fails | Run `cargo fmt --all` first |
| BDD tests fail | Check `specs/features/` syntax and step definitions |
| Nix issues | See [Getting Started - Troubleshooting](tutorials/getting-started.md#troubleshooting) |
| CI failures | Check [xtask Commands Reference](reference/xtask-commands.md) |
| **Any other issues** | **See [Troubleshooting Guide](TROUBLESHOOTING.md)** |

---

## Complete Reference

### Tutorials (Learning-Oriented)

📘 **[Day 1: First Change](tutorials/day-1-first-change.md)** (30 minutes) **← START HERE**
- Clone and validate template
- Add a trivial AC to the ledger
- Write Gherkin scenario
- Implement simple endpoint
- See AC go from red → green
- Complete AC-first development loop

📘 **[Day 7: First Real Feature](tutorials/day-7-first-real-feature.md)** (90 minutes)
- Implement substantial task management feature
- Multi-layer architecture (model → core → app-http)
- Proper validation and error handling
- Multiple ACs working together
- Generate LLM bundle for feature
- Production-ready patterns

📘 **[Getting Started](tutorials/getting-started.md)** (30 minutes) *[Alternative intro]*
- Clone and validate template
- Run HTTP service
- Make first change
- Understand AC-first workflow

📘 **[First AC Change](tutorials/first-ac-change.md)** (15 minutes) *[Legacy - see Day 1 instead]*
- Complete AC implementation cycle
- Spec → Test → Code → Validate
- Update AC status

### How-to Guides (Task-Oriented)

📗 **[Pre-Fork Checklist](how-to/pre-fork-checklist.md)** (10 minutes)
- Validate environment before forking
- Platform and tooling prerequisites
- Common pre-fork issues

📗 **[New Service from Template](how-to/new-service-from-template.md)** (10 minutes)
- Clone and customize
- Update ownership
- Configure CI

📗 **[Windows Development Guide](how-to/windows-development.md)** (variable)
- Platform-specific guidance for Windows
- WSL2 setup and best practices
- Native Windows tier-2 development
- Common Windows issues

📗 **[Use LLM Bundles](how-to/use-llm-bundles.md)** (variable)
- When to use LLM assistance
- Best practices
- Common workflows
- What not to do

📗 **[Add HTTP Endpoint](how-to/add-http-endpoint.md)** (15 minutes)
- Create new route
- Add handler and DTOs
- Error handling patterns
- Testing

📗 **[Deploy to Development](how-to/deploy-dev.md)** (20 minutes)
- Deploy to local Kubernetes cluster
- Environment configuration
- Prerequisite checking
- Troubleshooting deployment issues

📗 **[Branch Protection Setup](BRANCH-PROTECTION-SETUP.md)** (15 minutes)
- Configure GitHub branch protection rules
- Required checks and status checks
- Protection profiles (Minimal/Standard/Strict)

📗 **[Tag Signing Setup](how-to/setup-tag-signing.md)** (20 minutes)
- Configure GPG or SSH tag signing
- Generate and configure signing keys
- Verify signed tags

### Reference (Information-Oriented)

📚 **[xtask Commands](reference/xtask-commands.md)**
- check, bdd, bundle, quickstart, selftest
- Usage, options, examples
- Environment variables

📚 **[Branch Protection Profiles](reference/branch-protection-profiles.md)**
- Minimal, Standard, Strict
- Configuration steps
- Comparison table

📚 **[Template API](../TEMPLATE_API.md)** (root)
- Stable interfaces
- Command specifications
- Policy schemas

### Explanation (Understanding-Oriented)

📕 **[Architecture Overview](explanation/architecture.md)** (20 min read)
- Design philosophy
- Crate structure
- Hexagonal architecture
- Governance model
- Observability strategy
- Decision rationale

📕 **[Adoption Patterns](explanation/adoption-patterns.md)** (15 min read) **← IMPORTANT**
- Pattern A: Clone and Detach (single service)
- Pattern B: Template as Upstream (multiple services, get updates)
- Pattern C: Generator-Based (platform team, many services)
- Pros/cons, when to use each
- Upgrade strategies and migration paths
- Decision tree and hybrid approaches

📕 **[Template Foundation vs Examples](explanation/template-foundation-vs-examples.md)** (5 min read)
- What to keep (template core)
- What to adapt (example features)
- Why the distinction matters
- Quick reference for new services

### Design Documents (Implementation Details)

These documents capture implementation decisions, trade-offs, and rationale for internal systems. Useful for contributors and maintainers.

📐 **[AC Structured Report Design](design/ac-structured-report.md)**
- JSON schema for acceptance test results
- Analysis of Cucumber integration approaches
- Implementation plan and decisions
- Cucumber JSON vs. JUnit XML trade-offs

📐 **[.llmignore Semantics](design/llmignore-semantics.md)**
- Analysis of ignore pattern implementations
- Comparison of custom vs. gitignore-style semantics
- Recommendation and justification
- Migration strategy

📐 **[Meta-Contract Phase 1.3](meta_contract_phase1.3.md)**
- Machine-readable specifications for xtask and AC reports
- Control plane contract enforcement
- Template-core protection policies
- Implementation details and validation

📊 **[Observability Patterns](../crates/app-http/OBSERVABILITY.md)** (in app-http crate)
- Request ID correlation implementation
- Structured error handling with AC tracking
- Metrics integration guidance
- Testing and debugging patterns

📊 **[Implementation Summary 2025-11-15](implementation-summary-2025-11-15.md)**
- Complete changelog of v1.0.0 improvements
- Technical decisions and validation results
- Code statistics and test coverage
- Future roadmap

📋 **[Policy Organization](../policy/README.md)** (in policy/ directory)
- Policy structure and testing guide
- Template-core, LLM, and Kubernetes policies
- Test fixture organization
- Integration with xtask policy-test

---

## Roadmap

**Current Status**: v3.3.14 (Kernel-Ready)

The template is now a **stable kernel baseline** with full governance validation.

### What's Complete
- ✅ Complete observability stack (logs, metrics, OTLP traces)
- ✅ Hexagonal architecture with production adapters
- ✅ Governance infrastructure (policy-as-code, specs, BDD)
- ✅ LLM-native workflow with context bundling
- ✅ Release polish (docs, friction logs, reliability fixes)
- ✅ Skills and Agents governance framework
- ✅ IDP integration contracts (Backstage-ready)
- ✅ 86%+ AC coverage with full selftest validation

### Current Phase: IDP Integration & Fork Adoption
1. **Fork Creation**: Teams create services from the **v3.3.9-kernel** baseline (using **template v3.3.14**).
2. **Collect Friction**: Use friction log system to track real-world issues.
3. **Iterate**: Release patches for blockers, or v3.4.0 for new features.

Development is now driven by fork feedback and IDP integration needs.

---

## By Role

### For Developers

**Must Read:**
- [Quick Start Guide](QUICKSTART.md) **← Start here**
- [Day 1: First Change](tutorials/day-1-first-change.md)
- [Day 7: First Real Feature](tutorials/day-7-first-real-feature.md)
- [Architecture Overview](explanation/architecture.md)
- [xtask Commands](reference/xtask-commands.md)

**Frequently Used:**
- [Troubleshooting Guide](TROUBLESHOOTING.md) - When things go wrong
- [LLM Bundles](how-to/use-llm-bundles.md) - Daily workflow with AI
- [Add HTTP Endpoint](how-to/add-http-endpoint.md)
- [Template Foundation vs Examples](explanation/template-foundation-vs-examples.md)
- [Windows Development Guide](how-to/windows-development.md) - Platform-specific guidance

### For Tech Leads

**Must Read:**
- [Adoption Patterns](explanation/adoption-patterns.md) **← Critical decision**
- [Pre-Fork Checklist](how-to/pre-fork-checklist.md) - Before starting
- [Architecture Overview](explanation/architecture.md)
- [Branch Protection Profiles](reference/branch-protection-profiles.md)
- [New Service from Template](how-to/new-service-from-template.md)

**Setup & Configuration:**
- [Branch Protection Setup](BRANCH-PROTECTION-SETUP.md) - Configure GitHub protection
- [Tag Signing Setup](how-to/setup-tag-signing.md) - Configure release signing
- [Windows Development Guide](how-to/windows-development.md) - For Windows teams

**Reference:**
- [Template API](../TEMPLATE_API.md)
- All policy/*.rego files
- [Day 7: First Real Feature](tutorials/day-7-first-real-feature.md) - Share with team

### For New Hires

**Day 1:**
- [Quick Start Guide](QUICKSTART.md) - 15-minute orientation
- [Day 1: First Change](tutorials/day-1-first-change.md) - Hands-on intro
- [xtask Commands](reference/xtask-commands.md) - Skim for reference
- [Troubleshooting Guide](TROUBLESHOOTING.md) - Bookmark for later

**Week 1:**
- [Day 7: First Real Feature](tutorials/day-7-first-real-feature.md) - Build complete feature
- [Architecture Overview](explanation/architecture.md) - Read as you build
- [LLM Bundles](how-to/use-llm-bundles.md) - Use for questions

**Week 2:**
- Start making real changes on team backlog
- [Add HTTP Endpoint](how-to/add-http-endpoint.md) - Reference as needed
- [Adoption Patterns](explanation/adoption-patterns.md) - Understand team's choice

---

## Documentation Principles

### Diátaxis Framework

We follow Diátaxis to ensure documentation serves different needs:

**Tutorials** answer: *"Can you teach me to..."*
- Learning-oriented
- Practical steps
- Focused on learner
- Example: Getting Started

**How-to guides** answer: *"How do I..."*
- Task-oriented
- Series of steps
- Focused on goals
- Example: Add HTTP Endpoint

**Reference** answers: *"What is..."*
- Information-oriented
- Description/specification
- Focused on accuracy
- Example: xtask Commands

**Explanation** answers: *"Why..."*
- Understanding-oriented
- Discussion/background
- Focused on concepts
- Example: Architecture

### When to Update Docs

Update documentation when:
- ✅ Adding new xtask command → Update Reference
- ✅ Changing architecture → Update Explanation
- ✅ Adding common workflow → Add How-to
- ✅ Template version changes → Update Getting Started

### Contributing to Docs

1. **Choose document type:** Tutorial, How-to, Reference, or Explanation
2. **Follow existing structure:** See similar docs for patterns
3. **Test all code examples:** Ensure they actually work
4. **Use clear headings:** Make skimming easy
5. **Link related docs:** Help readers navigate

---

## External Resources

### Rust Ecosystem
- [Rust Book](https://doc.rust-lang.org/book/) - Learn Rust
- [Axum Docs](https://docs.rs/axum) - HTTP framework
- [Tracing Docs](https://docs.rs/tracing) - Observability

### Governance & Testing
- [Gherkin Reference](https://cucumber.io/docs/gherkin/reference/) - BDD syntax
- [OPA Docs](https://www.openpolicyagent.org/docs/) - Policy language
- [Diátaxis](https://diataxis.fr/) - Documentation framework

### Architecture
- [Hexagonal Architecture](https://alistair.cockburn.us/hexagonal-architecture/) - Ports & Adapters
- [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html) - Design principles

---

## Quick Reference Card

```bash
# Environment
nix develop                    # Enter dev shell
exit                          # Leave dev shell

# Core Commands
cargo run -p xtask -- check          # Format, lint, test
cargo run -p xtask -- bdd            # Run BDD scenarios
cargo run -p xtask -- bundle <task>  # Generate LLM context
cargo run -p xtask -- quickstart     # First-run validation
cargo run -p xtask -- selftest       # Full test suite

# Development
cargo run -p app-http                # Start HTTP service
cargo fmt --all                      # Format code
cargo clippy --all-targets           # Lint code
cargo test --workspace               # Run all tests

# Documentation
ls docs/                             # Browse docs locally
```

---

## Need Help?

**Can't find what you need?**
- Check the [Common Tasks](#common-tasks) section above
- Search docs: `grep -r "your topic" docs/`
- Review `TEMPLATE_API.md` for command details
- Check relevant `.rs` files for inline documentation

**Found an issue?**
- Documentation unclear? Open an issue
- Code example doesn't work? File a bug
- Missing guide? Suggest an addition

---

## Version

This documentation is for **Rust Template v3.3.14**.

**Recent Releases:**
- **v3.3.14 (2025-12-31)**: OpenAPI and endpoint configuration improvements
- **v3.3.13 (2025-12-27)**: Adoption Receipts and Docs Polish
  - Updated Rust toolchain to 1.89.0
  - Tool checksum validation and security advisory management
  - Security middleware and infrastructure hardening
  - Version consistency improvements

- **v3.3.8 (2025-12-09)**: Kernel Baseline
  - Full governance validation (11/11 selftest steps passing)
  - Skills and Agents governance framework
  - IDP integration contracts
  - 86%+ AC coverage

- **v3.3.0 (2025-12-01)**: Governance Framework
  - Skills governance with lint validation
  - Agents governance with safety modes
  - Enhanced platform APIs

**Next Steps:**
- 🔄 First fork dry-run validation
- 🔄 IDP adapter integration (Backstage/Port)
- 🔄 Determine v3.4.0 roadmap based on fork feedback

Further development driven by fork adoption and IDP integration.
