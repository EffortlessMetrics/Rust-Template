# Template Foundation vs Domain Extension

## Understanding the Kernel vs Your Domain

This template provides a **governed platform cell** - a complete foundation that you extend with your domain logic. The kernel contains everything you need for operations and governance; you add your business features.

### 1. **Platform Kernel** (Keep and Extend)

These are capabilities that **every service needs**, regardless of domain. They form the operational and governance baseline:

**Core Endpoints:**
- `GET /health` - Health check for monitoring and load balancers
- `GET /version` - Build information for deployments and debugging

**Platform Introspection Endpoints:**
- `GET /platform/status` - Governance health, ledger counts, policy status
- `GET /platform/docs/index` - Documentation inventory with validation
- `GET /platform/tasks` - Task list for agent consumption
- `GET /platform/agent/hints` - Prioritized work suggestions for agents
- `GET /platform/debug/info` - Development debug information (convenience; not in OpenAPI)

**Specs:**
- **Ledger**: `US-TPL-*` with `AC-TPL-*` and `AC-PLT-*`
- **Feature**: `specs/features/template_core.feature`, `specs/features/platform.feature`
- **OpenAPI**: Documented under "Template Core" and "Platform Introspection"
- **BDD**: Step definitions in `crates/acceptance/src/steps/`

**Implementation:**
- `crates/app-http/src/routes/` - Route handlers
- `crates/business-core/src/` - Domain and governance logic
- `crates/spec-runtime/` - Spec ledger and governance kernel

**What to do:** **Keep these** and extend them as your service grows:
- Add readiness checks to `/health` (database connectivity, etc.)
- Add more build metadata to `/version` (deployment time, environment)
- Add custom platform endpoints for your domain's governance needs

---

### 2. **Your Domain Features** (Add in Forks)

When you fork this template for your service, you add your business domain on top of the kernel:

**Pattern to follow:**
1. Add AC to `specs/spec_ledger.yaml`
2. Write BDD scenario in `specs/features/your_domain.feature`
3. Implement handler in `crates/app-http/src/routes/`
4. Add domain logic in `crates/business-core/src/`
5. Add step definitions in `crates/acceptance/src/steps/`

**Example: Adding a Task Management API**

```yaml
# specs/spec_ledger.yaml
stories:
  - id: US-TASKS-001
    title: "Task Management"
    requirements:
      - id: REQ-TASKS-CRUD
        text: "Users can create and manage tasks"
        acceptance_criteria:
          - id: AC-TASKS-CREATE
            text: "POST /tasks creates a new task"
            tests: [{ type: bdd, tag: "@AC-TASKS-CREATE" }]
```

**What to do:** Use the platform kernel as your foundation and add domain endpoints following the same patterns.

---

## Why This Distinction Matters

### Platform Kernel = Stability

The kernel provides a **stable baseline** that:
- Works out of the box for any service
- Meets operational requirements (monitoring, deployments)
- Provides governance visibility (`/platform/*`)
- Doesn't need to be rewritten for your domain
- Can be extended without being replaced

### Domain Features = Your Business Value

Your domain features provide **business value** that:
- Follows the same vertical slice pattern (specs → tests → code)
- Benefits from hexagonal architecture
- Inherits the kernel's enforcement mechanisms
- Gets governance tracking for free via AC linkage

Think of it like a house:
- **Kernel** = foundation, plumbing, electrical (you keep and extend)
- **Domain** = rooms and furniture (you design for your needs)

---

## Quick Reference

### When Cloning This Template for a New Service

**Keep:**
- All template-core endpoints (`/health`, `/version`)
- All platform introspection endpoints (`/platform/*`)
- US-TPL-* and associated ACs
- Core feature files and step definitions

**Add:**
- Your domain endpoints
- Your domain stories and ACs
- Your domain features
- Your domain handlers, core logic, models

**Update:**
- `specs/service_metadata.yaml` with your service identity
- README.md with your service description
- Documentation with your domain context

---

## See Also

- [First AC Change Tutorial](../tutorials/first-ac-change.md) - How to add your first AC
- [Add HTTP Endpoint](../how-to/add-http-endpoint.md) - Step-by-step endpoint addition
- [Architecture Explanation](./architecture.md) - Understanding hexagonal design
