---
id: HOWTO-FORK-BUILD-001
title: Fork and Build Your First Feature
doc_type: how-to
status: published
audience: [developers, teams]
tags: [adoption, getting-started, onboarding, fork, feature-development]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-ONBOARDING, REQ-PLT-DEVEX-CONTRACT]
acs: [AC-PLT-ENV-CHECK, AC-PLT-009, AC-PLT-015]
adrs: [ADR-0002, ADR-0005, ADR-0017]
related_docs:
  - docs/how-to/FIRST_FORK.md
  - docs/how-to/first-hour.md
  - docs/tutorials/day-1-first-change.md
  - docs/tutorials/day-7-first-real-feature.md
created: 2025-12-08
updated: 2025-12-22
---
<!-- doclint:disable orphan-version -->

# Fork and Build Your First Feature

This is the **golden path** from forking the template to shipping your first governed feature. It consolidates guidance from multiple docs into a single narrative flow.

**Time:** 2-3 hours | **Result:** Working feature with specs, BDD, tests, and docs

---

## Quick Navigation

| Day | Goal | Time |
|-----|------|------|
| [Day 0](#day-0-fork-and-clone) | Fork, clone, enter Nix | 15 min |
| [Day 1](#day-1-bootstrap-and-validate) | Bootstrap, selftest, explore | 45 min |
| [Day 2](#day-2-build-your-feature) | Add AC, BDD, steps, handler | 60-90 min |
| [Day 3](#day-3-complete-and-commit) | Validate, document, commit | 30 min |

---

## Prerequisites

- **Nix** with flakes enabled ([install guide](https://nix.dev))
- **Git** 2.30+
- **GitHub account** (for forking)

---

## Day 0: Fork and Clone

### 1. Fork the Repository

On GitHub: Click **Fork** → Choose your org → Optionally rename

### 2. Clone and Enter Nix

```bash
git clone https://github.com/YOUR_USERNAME/YOUR_REPO.git
cd YOUR_REPO
nix develop
```

First run downloads dependencies (~5-10 min). You'll see a new shell prompt.

### 3. Verify Environment

```bash
cargo xtask doctor
```

All checks should pass (✓).

---

## Day 1: Bootstrap and Validate

### 4. One-Command Setup

```bash
cargo xtask dev-up
```

This runs: doctor → install-hooks → smoke-tests → ac-status → help-flows

### 5. Run Selftest

```bash
cargo xtask selftest
```

All 11 gates should pass. This validates the template baseline.

### 6. Start the Service

```bash
cargo run -p app-http &
curl http://localhost:8080/health
# {"status":"healthy",...}
```

### 7. Explore Platform APIs

```bash
curl http://localhost:8080/platform/status | jq    # Governance health
curl http://localhost:8080/platform/graph | jq     # Full AC graph
curl http://localhost:8080/platform/agent/hints | jq  # Next work suggestions
```

### 8. Study the MYSERV Example

MYSERV is the canonical teaching example showing AC → BDD → Steps → Handler:

```bash
# See MYSERV ACs
cargo xtask ac-status | grep MYSERV

# Run MYSERV tests
cargo xtask bdd --tags @myserv

# Read the teaching guide
cat forks/example-myservice/README.md
```

**Key files to study:**
- `specs/spec_ledger.yaml` (search for `US-MYSERV-001`)
- `specs/features/myserv_todos.feature`
- `crates/acceptance/src/steps/myserv.rs`
- `crates/app-http/src/todos.rs`

---

## Day 2: Build Your Feature

Follow the **AC-first workflow**: Define AC → Write BDD → Implement Steps → Write Handler

### 9. Define Your AC

Add to `specs/spec_ledger.yaml`:

```yaml
stories:
  - id: US-YOURSERVICE-001
    title: "Your Feature"
    requirements:
      - id: REQ-YOURSERVICE-ENDPOINT
        title: "Your Endpoint"
        acceptance_criteria:
          - id: AC-YOURSERVICE-001
            text: "GET /your-endpoint returns expected data"
            tests:
              - type: bdd
                tag: "@AC-YOURSERVICE-001"
                file: "specs/features/your_feature.feature"
```

Or use the CLI:

```bash
cargo xtask ac-new AC-YOURSERVICE-001 \
  "Your endpoint returns expected data" \
  --story US-YOURSERVICE-001 \
  --requirement REQ-YOURSERVICE-ENDPOINT
```

### 10. Write BDD Scenarios

Create `specs/features/your_feature.feature`:

```gherkin
Feature: Your Feature

@AC-YOURSERVICE-001
Scenario: Happy path returns data
  Given the system is ready
  When I send a GET request to "/your-endpoint"
  Then the response status should be 200
  And the response should contain expected data

@AC-YOURSERVICE-001
Scenario: Error case returns 404
  When I send a GET request to "/your-endpoint/nonexistent"
  Then the response status should be 404
```

### 11. Implement Step Definitions

Create `crates/acceptance/src/steps/your_feature.rs`:

```rust
use cucumber::{given, when, then};
use crate::world::World;

#[given("the system is ready")]
async fn given_system_ready(world: &mut World) {
    // Setup test state
}

#[then("the response should contain expected data")]
async fn then_expected_data(world: &mut World) {
    let response = world.last_response.as_ref().unwrap();
    assert!(response.body.get("data").is_some());
}
```

**Tip:** Reuse existing steps from `governance_tasks.rs`:
- `when_get_request` / `when_post_request`
- `then_status_code`

### 12. Write the Handler

Create `crates/app-http/src/your_feature.rs`:

```rust
use axum::{Json, extract::State, http::StatusCode};
use crate::{state::AppState, errors::AppError};

/// GET /your-endpoint
/// Implements AC-YOURSERVICE-001
#[tracing::instrument(skip(state))]
pub async fn your_handler(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "data": "your response"
    })))
}
```

Register in `crates/app-http/src/lib.rs`:

```rust
.route("/your-endpoint", get(your_feature::your_handler))
```

### 13. Validate with the Ladder

```bash
cargo xtask check                       # fmt, clippy, unit tests
cargo xtask bdd --tags @AC-YOURSERVICE-001  # Your BDD scenarios
cargo xtask ac-tests AC-YOURSERVICE-001     # See linked tests
cargo xtask ac-status                   # AC health
cargo xtask selftest                    # Full governance gate
```

---

## Day 3: Complete and Commit

### 14. Update Documentation

Add API docs in `docs/api/your_feature.md` and link in spec_ledger:

```yaml
- id: AC-YOURSERVICE-001
  doc_refs:
    - docs/api/your_feature.md
```

### 15. Pre-commit Validation

```bash
cargo xtask precommit
```

This auto-fixes formatting and validates governance.

### 16. Commit with Traceability

```bash
git add .
git commit -m "feat(your-feature): Add your endpoint (AC-YOURSERVICE-001)

- Define AC in spec_ledger.yaml
- Add BDD scenarios in your_feature.feature
- Implement step definitions
- Write handler with proper error handling
- Selftest passes all 11 gates
"
```

### 17. Push and Verify CI

```bash
git push origin main
```

CI runs Tier-1 selftest. Verify all checks pass on GitHub Actions.

---

## What You've Learned

1. **Specs drive everything**: Story → REQ → AC → Test → Code → Docs
2. **Validation ladder**: Fast checks locally, full selftest before PR
3. **Platform APIs**: Use introspection instead of grepping
4. **MYSERV pattern**: Copy it, modify it, then build your own

---

## Next Steps

- **Deep dive**: [FIRST_FORK.md](FIRST_FORK.md) - Detailed fork setup
- **Exploration**: [first-hour.md](first-hour.md) - Hands-on template tour
- **Tutorial**: [day-7-first-real-feature.md](../tutorials/day-7-first-real-feature.md) - Full task management feature
- **Reference**: [AGENT_GUIDE.md](../AGENT_GUIDE.md) - Platform APIs and agent workflows
- **Teaching example**: [forks/example-myservice/README.md](../../forks/example-myservice/README.md) - MYSERV deep dive

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| `nix: command not found` | Re-run `nix develop` in new terminal |
| BDD tests fail with "connection refused" | Start service: `cargo run -p app-http &` |
| Selftest fails on gate X | See [Selftest Gate Reference](../TROUBLESHOOTING.md#selftest-gate-reference) |
| Pre-commit blocks commit | Run `cargo xtask precommit` to see errors |

For comprehensive troubleshooting, see [TROUBLESHOOTING.md](../TROUBLESHOOTING.md).

---

**Last Updated:** 2025-12-27 (v3.3.13)
