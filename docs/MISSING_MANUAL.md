# The Missing Manual: Operational Realities

**Audience:** Teams adopting this template  
**Purpose:** Critical context and constraints that aren't obvious from code alone

This document captures the **"things you'll wish someone told you"** before starting. Read this before your first pilot.

---

## 1. The Brownfield Adoption Path (Library Mode)

**Myth:** "This is a template, so I have to start from scratch."  
**Reality:** The governance engine is decoupled. You can add it to existing services.

### The Library Split

We separated governance logic into reusable crates:
- `spec-runtime`: Spec loaders, graph logic, validation
- `xtask` patterns: Can be added to **any** Rust workspace

### How Brownfield Adoption Works

```bash
# In your existing Rust service:
cargo add rust_iac_xtask_core --path /path/to/template/crates/spec-runtime

# Add minimal xtask binary
mkdir -p xtask
# Copy template's xtask/main.rs
# Customize for your service

# Initialize specs
cargo xtask init --mode=brownfield
```

**What this gives you:**
- AC tracking without rewriting your service
- Policy enforcement via OPA/Rego
- LLM bundling for existing code
- Gradual adoption (start with docs-check, add BDD later)

**Key Insight:** You **don't** need hexagonal architecture or Axum to get the governance. The template is **example architecture** + governance, but governance is portable.

**For Platform Teams:** This lets you roll out governed dev to legacy services without forcing migrations.

---

## 2. Scope Cuts (What We Deliberately Left Out)

These are **intentional non-goals**. If you clone this template, you still need to build:

### Database Provisioning
- **We provide:** Adapter (`adapters-db-sqlx`), migration scripts
- **We don't provide:** Terraform/OpenTofu to spin up RDS/CloudSQL
- **Assumption:** DB URL injected via K8s Secret

### Ingress / Gateway
- **We provide:** K8s `Service` (ClusterIP)
- **We don't provide:** `Ingress` resource or gateway config
- **Assumption:** Platform team has standard ingress controller

### Secrets Management
- **We provide:** Pattern (`envFrom: secretRef`)
- **We don't provide:** Vault, SOPS, SealedSecrets integration
- **Assumption:** Secrets exist before pod starts

### CI/CD Deploy Pipelines
- **We provide:** GitHub Actions for selftest
- **We don't provide:** Deployment/rollback workflows
- **Assumption:** Platform has deployment automation

**Bottom Line:** This template is **service layer**, not full platform stack.

---

## 3. The "Wall of YAML" Reality

### The Burden

Adding a feature touches:
1. `specs/spec_ledger.yaml`
2. `specs/features/*.feature`
3. `specs/devex_flows.yaml`
4. `specs/doc_index.yaml`
5. `specs/tasks.yaml`

**This is real overhead.** It can feel heavy.

### Why We Accept It

This is the **cost of the living contract**. Without it:
- Specs drift from code in weeks
- Docs become stale
- Agents hallucinate

**Tradeoff:** More structure → less drift → safe AI augmentation.

### How We Mitigate It

**Use generators, not manual edits:**

```bash
# Right
cargo xtask ac-new AC-ID "Desc" --requirement REQ-ID

# Wrong  
vim specs/spec_ledger.yaml  # ❌ Leads to errors
```

**If developers bypass generators, they will hate this template.** Make generator usage non-negotiable.

---

## 4. What "Self-Healing" Actually Means

**Marketing Claim:** "Self-healing platform cell"  
**Ground Truth:** The system **refuses to exist in an inconsistent state**

### What It Does
- Detects drift (specs ↔ code ↔ docs)
- Blocks merges when drift detected
- Surfaces violations clearly

### What It Doesn't Do
- ❌ Auto-fix broken code
- ❌ Generate missing specs
- ❌ Rewrite tests

**"Self-healing" = aggressive validation**, not magic code generation.

### UX Implication

`cargo xtask selftest` will **reject valid work** if metadata is wrong.

Example:
```
code works ✓
tests pass ✓
selftest fails ✗  # Because devex_flows.yaml not updated
```

**Cultural requirement:** Teams must accept that **governed == correct**, not **working == correct**.

---

## 5. Observability Specifics (Logs vs Traces)

We hit an OpenTelemetry constraint:

- **Traces:** Full OTLP (gRPC) to Jaeger/Tempo
- **Metrics:** Full OTLP to Prometheus
- **Logs:** Still `stdout` (not OTLP export yet)

**Impact:** You'll see traces + metrics in your observability backend, but logs still need a scraper (Fluentbit/Promtail) to aggregate.

**Why:** OTLP log export is less mature; we prioritized traces/metrics.

---

## 6. The "Pilot Discipline" (No New Features)

**Current State:** Logic freeze.

The template is in **"validate mode"**, not **"build mode"**.

### The Rule

Only change the template if:
1. Someone writes `FRICTION_LOG.md` entry
2. Entry is triaged as blocker
3. Fix doesn't violate governance model

### The Danger

If you add features before piloting, you'll:
- Break coherence we spent weeks building
- Skip validation of existing features
- Build features no one needs

**The Pilot Loop:**
```
Use Template → Hit Friction → Log It → Fix → Repeat
```

**Not:**
```
Think of Cool Feature → Build It → Hope Someone Uses It
```

---

## 7. The Agent Interface is JSON, Not CLAUDE.md

**Common Misconception:** `CLAUDE.md` is the agent interface.  
**Reality:** The **HTTP APIs are** the agent interface.

### What Agents Should Do

```bash
# Get available tasks
curl http://localhost:8080/platform/tasks | jq

# Get guidance
curl http://localhost:8080/platform/tasks/suggest-next?task=implement_ac | jq

# Check health
curl http://localhost:8080/platform/status | jq
```

**Not:**
```bash
# This is wrong
tree . > context.txt  # ❌ Unbounded, stale
cat **/*.rs > context.txt  # ❌ Overwhelming
```

### Why This Matters

- **APIs are bounded:** Max 250KB per bundle
- **APIs are current:** Generated on-demand
- **APIs are validated:** Same loaders as selftest

**Future Agents:** Will call these APIs to discover work dynamically, not parse files statically.

---

## 8. Common Pitfalls

### Pitfall: Editing Specs Without Generators

**Symptom:** `selftest` fails with schema errors  
**Cause:** Manual YAML edits introduce typos, ID collisions  
**Fix:** Delete changes, use `cargo xtask ac-new` instead

### Pitfall: Bypassing Selftest

**Symptom:** Specs diverge from code in prod  
**Cause:** Forced merge despite selftest failure  
**Fix:** Make selftest a **branch protection rule** (cannot bypass)

### Pitfall: Over-Strict Invariants

**Symptom:** Developers constantly fighting graph invariants  
**Cause:** `must_have_ac` set on too many requirements  
**Fix:** Use pilot friction log to calibrate strictness

### Pitfall: Assuming This Is "Just a Template"

**Symptom:** Team clones, deletes governance, uses as Axum starter  
**Cause:** Didn't read positioning docs  
**Fix:** This is a **platform cell**, not a boilerplate. If you don't want governance, use a different template.

---

## 9. Summary for a New Team

Before you start:

1. **Read docs:**
   - ROADMAP.md (positioning)
   - Technical Overview (architecture)
   - AGENT_GUIDE.md (if using LLMs)

2. **Set expectations:**
   - Governance is overhead (but pays off)
   - Must use CLI generators (not negotiable)
   - Selftest is supreme (cannot bypass)

3. **Environment:**
   - Nix is recommended (matches CI)
   - Manual setup possible but harder

4. **Scope:**
   - Still need infra (DB, ingress, secrets)
   - Template is service layer only

5. **Workflow:**
   - AC-first (not code-first)
   - Selftest before merge (always)
   - Friction log for issues (continuously)

6. **Agent usage:**
   - Use `/platform/*` APIs
   - Call `suggest-next` for guidance
   - Validate with `selftest`

**If you skip these basics, the template will feel needlessly complex.**

---

## 10. When to **Not** Use This Template

This template is **not** suitable if:

- ❌ You're prototyping (too heavy)
- ❌ You can't commit to AC-first workflow
- ❌ You don't have Nix and can't install it
- ❌ You need move fast without governance
- ❌ You're in a solo project (governance overhead not worth it)
- ❌ You fundamentally disagree with "specs as code"

**Use instead:**
- Simple Axum starter (prototyping)
- Zero to Production patterns (production without governance)
- Custom setup (if template philosophy doesn't fit)

**This template is for teams where governance is non-negotiable** (regulated industries, multi-team platforms, AI-assisted development at scale).

---

## Questions?

Read:
- [docs/INDEX.md](INDEX.md) - Full documentation nav
- [ROADMAP.md](ROADMAP.md) - Strategic direction
- [AGENT_GUIDE.md](AGENT_GUIDE.md) - Agent operations

Still stuck? Open an issue with `[pilot]` prefix.

---

## 11. Low-Resource Environments

If you are running on a constrained machine (e.g., small CI runner, cheap VPS, or old laptop), the default parallel builds and caching might be too heavy.

**Use Low-Resource Mode:**

```bash
XTASK_LOW_RESOURCES=1 cargo run -p xtask -- check
XTASK_LOW_RESOURCES=1 cargo run -p xtask -- selftest
```

**What this does:**
- Sets `CARGO_BUILD_JOBS=1` (serial compilation)
- Disables `sccache` (avoids cache overhead)
- Reduces memory pressure significantly

**When to use:**
- CI runners with < 4GB RAM
- Local dev on constrained hardware
- If you see "OOM" or "Killed" messages during compilation
