# Friction Log – [Project Name]

**Template Version**: v2.4.0
**Started**: YYYY-MM-DD
**Team**: [Team/Developer Name]
**Project Type**: [Greenfield / Brownfield]

---

## Purpose

This log captures pain points, surprises, and improvements discovered while using the Rust IaC Template in a real project. Use this to inform future template evolution.

---

## Setup Phase

### Initial Clone & Selftest

- [ ] Clone template (or `cargo generate`)
- [ ] Run `cargo run -p xtask -- selftest`
- [ ] Review generated structure

**Issues Encountered:**
- [ ] None / Describe below

**Observations:**
```
[Describe what worked well, what was confusing, what was missing]
```

---

## Feature Implementation

### Feature 1: [Feature Name] (AC-XXX-YYY)

**Context:**
- **Acceptance Criteria**: [Link or brief description]
- **Complexity**: [Trivial / Simple / Moderate / Complex]
- **Tools Used**: [xtask bundle, LLM, manual coding, etc.]

**Timeline:**
- Planning: [Duration]
- Implementation: [Duration]
- Testing: [Duration]
- Total: [Duration]

**Friction Points:**

1. **[Category: Setup / Coding / Testing / Docs / Other]**
   - **What happened**: [Describe the friction]
   - **Expected**: [What you expected to happen]
   - **Actual**: [What actually happened]
   - **Workaround**: [How you solved it, if applicable]
   - **Severity**: [Low / Medium / High / Blocker]

2. **[Next friction point]**
   - ...

**What Worked Well:**
- [List positive experiences, e.g., "BDD scenario generation was instant"]

---

### Feature 2: [Next Feature]

[Repeat structure above]

---

## Observability Integration

### Logs
- [ ] Added structured logging with `tracing`
- [ ] Configured `RUST_LOG` filtering
- **Issues**: [Describe any friction]

### Metrics
- [ ] Added custom metrics to `/metrics` endpoint
- [ ] Verified Prometheus scraping
- **Issues**: [Describe any friction]

### Traces
- [ ] Enabled `telemetry/otlp` feature
- [ ] Connected to OTLP collector (Jaeger/other)
- [ ] Verified distributed traces
- **Issues**: [Describe any friction]

**Observations:**
```
[Was the observability setup smooth? Missing docs? Unclear patterns?]
```

---

## Policy & Governance

### OPA/Rego Policies
- [ ] Ran `cargo run -p xtask -- policy-test`
- [ ] Added custom policies for project domain
- **Issues**: [Describe any friction]

### Spec Ledger
- [ ] Updated `specs/spec_ledger.yaml` with new ACs
- [ ] Ran `cargo run -p xtask -- ac-status`
- **Issues**: [Describe any friction]

### BDD Scenarios
- [ ] Created Gherkin scenarios for new features
- [ ] Ran `cargo run -p xtask -- bdd`
- **Issues**: [Describe any friction]

**Observations:**
```
[Was the governance flow helpful? Bureaucratic? Missing tooling?]
```

---

## LLM-Assisted Development

### Contextpack Usage
- [ ] Generated contextpack with `cargo run -p xtask -- bundle --ac AC-XXX-YYY`
- [ ] Used contextpack with LLM (Claude / other)
- [ ] LLM output quality: [Poor / Fair / Good / Excellent]

**Friction Points:**
- [e.g., "Contextpack was too large for LLM context window"]
- [e.g., "Missing domain context in bundle"]
- [e.g., "LLM hallucinated API that doesn't exist in template"]

**What Worked:**
- [e.g., "LLM correctly inferred test structure from BDD scenarios"]

---

## Adapter Integration

### Database (sqlx)
- [ ] Used `adapters-db-sqlx` as reference
- [ ] Created custom adapter for [database/service]
- **Issues**: [Describe any friction]

### gRPC (tonic)
- [ ] Used `adapters-grpc` as reference
- [ ] Created custom gRPC client/server
- **Issues**: [Describe any friction]

### Other Adapters
- [ ] [Describe custom adapter, e.g., Redis, Kafka, S3]
- **Issues**: [Describe any friction]

**Observations:**
```
[Were adapter patterns clear? Missing examples? Over-engineered?]
```

---

## Documentation Gaps

**Missing Docs:**
1. [e.g., "How to add custom xtask commands"]
2. [e.g., "Patterns for domain events"]
3. [e.g., "Kubernetes deployment in production"]

**Confusing Docs:**
1. [e.g., "docs/day-1.md assumes familiarity with X"]
2. [e.g., "OTLP guide unclear about TLS setup"]

**Excellent Docs:**
- [e.g., "docs/how-to/test-otlp-tracing.md was perfect"]

---

## Overall Assessment

### Template Strengths
1. [e.g., "Observability wired out of the box"]
2. [e.g., "Policy enforcement caught real issues early"]
3. [e.g., "Clean separation of business-core from adapters"]

### Template Weaknesses
1. [e.g., "Too much boilerplate for simple CRUD services"]
2. [e.g., "Missing guidance on database migrations"]
3. [e.g., "Unclear how to extend xtask for project-specific tasks"]

### Would You Use This Template Again?
- [ ] Yes, immediately
- [ ] Yes, with modifications
- [ ] Maybe, for specific use cases
- [ ] No

**Reasoning:**
```
[Explain your assessment]
```

---

## Recommendations for Template Evolution

### High Priority (Next Release)
1. [e.g., "Add migration tooling guide (Diesel/sqlx)"]
2. [e.g., "Simplify xtask extension API"]

### Medium Priority
1. [e.g., "Add S3 adapter example"]
2. [e.g., "Improve LLM contextpack chunking for large projects"]

### Low Priority / Nice-to-Have
1. [e.g., "Add example CI/CD pipeline for GitHub Actions"]
2. [e.g., "Create video walkthrough of Day 1 setup"]

---

## Appendix: Raw Notes

```
[Paste any raw notes, chat logs, or observations that don't fit above categories]
```

---

## Change Log

| Date       | Phase               | Notes                          |
|------------|---------------------|--------------------------------|
| YYYY-MM-DD | Setup               | Initial clone, selftest passed |
| YYYY-MM-DD | Feature 1 (AC-XXX)  | [Brief note]                   |
| YYYY-MM-DD | Feature 2 (AC-YYY)  | [Brief note]                   |

---

**End of Friction Log**
