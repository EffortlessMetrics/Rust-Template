# Friction Log

This log captures friction points discovered during development - process, tooling, and developer experience issues that create unnecessary pain or inefficiency.

**Purpose:** Track and resolve workflow friction to continuously improve the development experience.

**Format:** Friction entries are stored as structured YAML files in the `friction/` directory for machine-readable governance integration. This markdown file serves as a human-readable index and summary.

**Schema:** See `specs/friction_schema.yaml` for the complete schema definition.

---

## When to Use the Friction Log

Use the **friction log** for:
- Process or tooling problems
- Developer experience pain points
- Workflow inefficiencies
- CI/CD issues
- Flaky tests or intermittent failures
- Poor error messages or unclear diagnostics

**Not friction:** Feature work (use GitHub issues), architectural decisions (use ADRs), or ambiguous specs (use questions).

See `specs/friction_schema.yaml` for complete guidance on when to use friction log vs ADR vs issue vs question.

---

## View Friction Entries

**CLI:**
```bash
# List all friction entries
cargo xtask friction-list

# Filter by status
cargo xtask friction-list --status open

# Filter by severity
cargo xtask friction-list --severity high
```

**HTTP API:**
```bash
# Get friction counts and recent entries
curl http://localhost:8080/platform/status

# Get all friction entries
curl http://localhost:8080/platform/friction
```

---

## Create New Friction Entry

**CLI:**
```bash
cargo xtask friction-new \
  --id FRICTION-XYZ-001 \
  --category devex \
  --severity medium \
  --summary "Brief description of friction"
```

**Manual:**
Create a YAML file in `friction/` following the schema in `specs/friction_schema.yaml`.

---

## Active Friction Entries

### Open

- **FRICTION-CI-001** (2025-12-27) - GitHub Actions intentionally disabled during v3.3.13 release prep
  - **Category:** ci
  - **Severity:** medium
  - **Workaround:** Use local selftest receipts as temporary branch protection
  - **Details:** [friction/FRICTION-CI-001.yaml](friction/FRICTION-CI-001.yaml)

### Resolved

- **FRICTION-ENV-001** (2025-12-01, resolved 2025-12-26) - Nix devshell sccache/libz.so.1 breaks xtask wrapper
  - **Category:** tooling
  - **Severity:** medium
  - **Resolution:** Fixed by adding Nix-managed sccache to devshell and enforcing PATH order
  - **Details:** [friction/FRICTION-ENV-001.yaml](friction/FRICTION-ENV-001.yaml)

- **FRICTION-AGENT-001** (2025-11-20) - UI/API inconsistency - tasks not shown in UI/hints when tasks_state.yaml missing
  - **Category:** api
  - **Severity:** high
  - **Status:** Resolved
  - **Details:** [friction/FRICTION-AGENT-001.yaml](friction/FRICTION-AGENT-001.yaml)

- **FRICTION-AGENT-002** (2025-11-20) - Port discovery requires manual lsof lookup
  - **Category:** devex
  - **Severity:** low
  - **Status:** Resolved
  - **Details:** [friction/FRICTION-AGENT-002.yaml](friction/FRICTION-AGENT-002.yaml)

---

## Metrics

Run `cargo xtask status` or visit `/platform/status` to see:
- Total friction entries
- Open vs resolved counts
- Breakdown by severity and category
- Recent friction entries
- Average time to resolution

---

## Process

1. **Discover:** Encounter friction during development, testing, or agent operations
2. **Capture:** Create friction entry immediately with full context
3. **Triage:** Assess severity and prioritize based on impact
4. **Resolve:** Fix the underlying tool, process, or documentation issue
5. **Verify:** Confirm the fix eliminates the friction in practice
6. **Close:** Update entry status to "resolved" with resolution details

Resolved entries remain in the `friction/` directory for historical reference and pattern analysis.
