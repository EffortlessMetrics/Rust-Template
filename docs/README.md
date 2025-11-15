# Documentation Index

Welcome to the Rust Template documentation. This template provides spec-as-code, AC-as-code, policy-as-code, and LLM-native development for Rust services.

**Quick Links:**
- 🚀 [Getting Started](#getting-started)
- 📖 [Learning Path](#learning-path-new-to-template)
- 🎯 [Common Tasks](#common-tasks)
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

## Getting Started

**Brand new?** Start here:

1. 📘 **[Getting Started Tutorial](tutorials/getting-started.md)** (30 min)
   - Clone, validate, run the service
   - Make your first change
   - Understand core concepts

2. 📗 **[Architecture Overview](explanation/architecture.md)** (20 min read)
   - Design philosophy
   - Hexagonal architecture
   - Why these choices

3. 📕 **[First AC Change Tutorial](tutorials/first-ac-change.md)** (15 min)
   - Complete AC-first workflow
   - Spec → Test → Code → Validate

---

## Learning Path (New to Template)

### Week 1: Foundations

**Day 1-2: Environment & Basics**
- [ ] Complete [Getting Started Tutorial](tutorials/getting-started.md)
- [ ] Read [Architecture Overview](explanation/architecture.md)
- [ ] Run `xtask quickstart` successfully

**Day 3-4: AC-First Development**
- [ ] Complete [First AC Change Tutorial](tutorials/first-ac-change.md)
- [ ] Read [How to Add HTTP Endpoint](how-to/add-http-endpoint.md)
- [ ] Make one small change end-to-end

**Day 5: Tooling & Workflow**
- [ ] Read [xtask Commands Reference](reference/xtask-commands.md)
- [ ] Read [LLM Bundles Guide](how-to/use-llm-bundles.md)
- [ ] Generate and use one LLM bundle

### Week 2: Production Readiness

**Day 1-2: Service Adaptation**
- [ ] Follow [New Service from Template](how-to/new-service-from-template.md)
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

---

## Complete Reference

### Tutorials (Learning-Oriented)

📘 **[Getting Started](tutorials/getting-started.md)** (30 minutes)
- Clone and validate template
- Run HTTP service
- Make first change
- Understand AC-first workflow

📘 **[First AC Change](tutorials/first-ac-change.md)** (15 minutes)
- Complete AC implementation cycle
- Spec → Test → Code → Validate
- Update AC status

### How-to Guides (Task-Oriented)

📗 **[New Service from Template](how-to/new-service-from-template.md)** (10 minutes)
- Clone and customize
- Update ownership
- Configure CI

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

---

## By Role

### For Developers

**Must Read:**
- [Getting Started](tutorials/getting-started.md)
- [Architecture Overview](explanation/architecture.md)
- [Template Foundation vs Examples](explanation/template-foundation-vs-examples.md)
- [xtask Commands](reference/xtask-commands.md)

**Frequently Used:**
- [Add HTTP Endpoint](how-to/add-http-endpoint.md)
- [First AC Change](tutorials/first-ac-change.md)
- [LLM Bundles](how-to/use-llm-bundles.md)

### For Tech Leads

**Must Read:**
- [Architecture Overview](explanation/architecture.md)
- [Branch Protection Profiles](reference/branch-protection-profiles.md)
- [New Service from Template](how-to/new-service-from-template.md)

**Reference:**
- [Template API](../TEMPLATE_API.md)
- All policy/*.rego files

### For New Hires

**Week 1:**
- [Getting Started](tutorials/getting-started.md)
- [Architecture Overview](explanation/architecture.md) (skim)
- [xtask Commands](reference/xtask-commands.md) (skim)

**Week 2:**
- [First AC Change](tutorials/first-ac-change.md)
- [Add HTTP Endpoint](how-to/add-http-endpoint.md)
- Start making real changes

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

This documentation is for **Rust Template v1.0.0**.

**Changelog:**
- v1.0.0 (2025-11-13): Initial stable release
  - Complete Diátaxis documentation
  - Runtime architecture (app-http)
  - Rust-native tooling (xtask)
  - AC-first + Policy-as-code + LLM-native

**Next version (v1.1.0) planned:**
- Rust-native AC status (xtask ac-status)
- Database adapter example
- More how-to guides
