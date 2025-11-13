# Template Foundation vs Examples

## Understanding What to Keep and What to Adapt

This template includes two types of functionality:

### 1. **Template Foundation** (Keep and Extend)

These are capabilities that **every service needs**, regardless of domain. They form the operational baseline that you build upon:

**Endpoints:**
- `GET /health` - Health check for monitoring and load balancers
- `GET /version` - Build information for deployments and debugging

**Specs:**
- **Ledger**: `US-TPL-001` with `AC-TPL-001` and `AC-TPL-002`
- **Feature**: `FT-TPL-CORE.yaml` - Template core operations
- **OpenAPI**: `/health` and `/version` documented under "Template Core"
- **BDD**: `specs/features/template_core.feature`

**Implementation:**
- `crates/app-http/src/lib.rs` - Core handlers marked as "Template Core"
- Step definitions in `crates/acceptance/src/steps/template_core.rs`

**What to do:** **Keep these** and extend them as your service grows. For example:
- Add readiness checks to `/health` (database connectivity, etc.)
- Add more build metadata to `/version` (deployment time, environment)
- Add new operational endpoints like `/metrics` or `/ready`

---

### 2. **Example Features** (Adapt or Replace)

These demonstrate **how to add domain logic** to the template. They're realistic examples, but specific to a sample domain (refunds):

**Endpoints:**
- `POST /refunds` - Create a refund (example business operation)

**Specs:**
- **Ledger**: `US-42` with `REQ-411` and `AC-123`
- **Feature**: `FT-123.yaml` - Refund creation (marked as "Example")
- **OpenAPI**: `/refunds` documented under "Example Domain Endpoint"
- **BDD**: `specs/features/refund.feature`

**Implementation:**
- `crates/app-http/src/lib.rs` - Refund handler marked as "Example Domain"
- `crates/core/src/lib.rs` - `refund_ok()` domain logic
- `crates/model/src/lib.rs` - `Refund` entity
- Step definitions in `crates/acceptance/src/steps/refunds.rs`

**What to do:** **Adapt these** to your service's actual domain:
1. Study how the refund feature is structured end-to-end
2. Identify the pattern: spec → AC → BDD → handler → domain
3. Replace "refund" with your domain (e.g., "invoice", "user", "order")
4. Follow the same layering and testing approach
5. Delete the refund example once you've built your first real feature

---

## Why This Distinction Matters

### Template Foundation = Stability

The core endpoints provide a **stable baseline** that:
- Works out of the box for any service
- Meets operational requirements (monitoring, deployments)
- Doesn't need to be rewritten for your domain
- Can be extended without being replaced

### Examples = Learning Path

The refund example provides a **learning path** that:
- Shows the complete vertical slice (specs → tests → code)
- Demonstrates hexagonal architecture in practice
- Proves the template's enforcement mechanisms work
- Gives you a concrete pattern to replicate

Think of it like a house:
- **Foundation** = plumbing, electrical, framing (you keep and extend)
- **Example** = furniture arrangement (you adapt to your lifestyle)

---

## Quick Reference

### When Cloning This Template for a New Service

**Keep:**
- ✅ All template-core endpoints (`/health`, `/version`)
- ✅ US-TPL-001 and associated ACs
- ✅ `template_core.feature` and step definitions
- ✅ Core handlers and DTOs in `app-http`

**Adapt:**
- 🔄 Refund endpoints → Your domain endpoints
- 🔄 US-42, AC-123 → Your stories and ACs
- 🔄 `refund.feature` → Your domain features
- 🔄 Domain handlers, core logic, models

**Delete (once confident):**
- ❌ Refund-specific code (after you've built your first real feature)
- ❌ Example specs (after you've replaced them with real ones)

---

## See Also

- [First AC Change Tutorial](../tutorials/first-ac-change.md) - How to add your first real AC
- [Add HTTP Endpoint](../how-to/add-http-endpoint.md) - Step-by-step endpoint addition
- [Architecture Explanation](./architecture.md) - Understanding hexagonal design
