<!-- doclint:disable orphan-version -->
# Fork Customization Example

This example shows the **minimal changes** needed to customize this template for your own domain.

When you fork this template for your service (e.g., "Product Catalog", "Order Service", "Audit Log"), you'll add domain-specific stories, requirements, and endpoints alongside the template core capabilities you inherit.

---

## What to Customize (Minimal Path)

### 1. Service Metadata

**File:** `service_metadata.yaml` (create if missing)

```yaml
service_id: "product-catalog"
name: "Product Catalog Service"
description: "Manages product definitions, categories, and pricing"
tags: [ecommerce, catalog, core]
version: "0.1.0"
owner_team: "ecommerce-platform"
template_version: "3.3.3"
```

**Purpose:** Identifies your service in platform tooling, IDP dashboards, and `/platform/status`.

---

### 2. Spec Ledger (Add Your Domain)

**File:** `specs/spec_ledger.yaml`

**What to do:**
- Keep the template core stories (`US-TPL-001`, `US-TPL-PLT-001`) – you inherit health, version, metrics, devex flows.
- Add your domain stories using your own prefixes (e.g., `US-PROD-`, `REQ-PROD-`, `AC-PROD-`).

See `sample-spec-additions.yaml` in this directory for a minimal example.

**Key principle:** Template core = platform capabilities. Your domain = business logic.

---

### 3. BDD Feature Files

**Directory:** `specs/features/`

**What to do:**
- Keep existing template core features (`template_core.feature`, `metrics.feature`, etc.) – they verify inherited ACs.
- Add your domain feature files (e.g., `product_catalog.feature`) and tag scenarios with your AC IDs.

**Example:** See `sample-product-catalog.feature` in this directory.

---

### 4. Domain Endpoints (HTTP Handlers)

**File:** `crates/app-http/src/routes.rs` (or similar)

**What to do:**
- Keep template platform endpoints (`/health`, `/version`, `/metrics`, `/platform/*`).
- Add your domain routes under `/api/*` or your chosen namespace.

**Example pattern:**

```rust
// Keep template routes
router
    .route("/health", get(health_handler))
    .route("/version", get(version_handler))
    .route("/metrics", get(metrics_handler))
    // Add your domain routes
    .route("/api/products", post(create_product))
    .route("/api/products/:id", get(get_product))
```

---

### 5. Configuration Schema

**File:** `specs/config_schema.yaml`

**What to do:**
- Keep template core settings (`http.port`, `telemetry.otlp_endpoint`, `platform.*`).
- Add your domain-specific settings and secrets.

**Example additions:**

```yaml
settings:
  - key: catalog.max_products_per_category
    type: int
    default: 1000
    description: "Maximum products allowed per category"
  - key: catalog.cache_ttl_seconds
    type: int
    default: 300
    description: "Product cache TTL"

secrets:
  - key: catalog.pricing_api_key
    type: string
    description: "External pricing service API key"
    required: true
```

---

### 6. CI Template

**File:** `.github/workflows/kernel-ci.yaml`

A canonical GitHub Actions workflow is provided at `.github/workflows/kernel-ci.yaml` in this example directory.

**Quick Start:**

```bash
mkdir -p .github/workflows
cp examples/fork-customization/.github/workflows/kernel-ci.yaml .github/workflows/
```

**CI Tiers:**

| Tier | Environment | Commands | Purpose |
|------|-------------|----------|---------|
| Tier-1 | Nix devshell | `ci-local` | Full governance validation |
| Tier-2 | Native OS | `check` | Basic compilation and tests |

**Environment Variables:**

| Variable | Purpose |
|----------|---------|
| `CI=1` | Auto-set by GitHub Actions |
| `XTASK_NONINTERACTIVE=1` | Suppress prompts |
| `XTASK_LOW_RESOURCES=1` | For constrained runners |

See [docs/how-to/run-in-ci.md](../../docs/how-to/run-in-ci.md) for detailed configuration options.

---

### 7. README and CLAUDE.md

**Files:** `README.md`, `CLAUDE.md`

**What to do:**
- Update the service name, description, and version at the top.
- In `CLAUDE.md`, document your domain:
  - Service ID and prefixes (task, REQ, AC)
  - Domain routes and key entities
  - Business rules or constraints agents should know

**Example CLAUDE.md snippet:**

```markdown
# Product Catalog Service Development

Service ID: `product-catalog`
Task prefix: `TASK-PROD-`
Requirement prefix: `REQ-PROD-`
AC prefix: `AC-PROD-`

Domain routes:
- `POST /api/products` – create a new product
- `GET /api/products/{id}` – retrieve product details
- `GET /api/products` – list products with filters
- Inherited: `/health`, `/version`, `/metrics`, `/platform/*`

Key entities: Product, Category, PricingRule
```

---

## What NOT to Change

Keep these as-is unless you have a specific reason:

- `specs/devex_flows.yaml` – defines the template's development workflows
- `.github/workflows/ci.yml` – CI pipeline (you can extend, but keep `cargo xtask selftest`)
- `crates/xtask/` – the xtask CLI and governance tools
- `crates/spec-runtime/` – the spec ledger parser and graph
- Template core stories in `spec_ledger.yaml` – you inherit these

---

## Validation Checklist

After customization, run:

```bash
cargo xtask doctor           # Check environment
cargo xtask ac-status        # Verify spec ledger parses
cargo xtask selftest         # Full governance validation
```

**Expected state:**
- Template core ACs: **green** (inherited, already implemented)
- Your domain ACs: **red** (not implemented yet, that's OK!)

Once you implement your first domain AC and tests, run `cargo xtask selftest` to verify it goes green.

---

## Example Workflow

1. **Fork the template:**

   ```bash
   gh repo create product-catalog --template your-org/Rust-Template
   cd product-catalog
   ```

2. **Customize metadata:**
   - Edit `service_metadata.yaml`, `README.md`, `CLAUDE.md`

3. **Add your first domain story:**
   - Edit `specs/spec_ledger.yaml` (see `sample-spec-additions.yaml`)
   - Create `specs/features/product_catalog.feature` (see `sample-product-catalog.feature`)

4. **Add initial tasks:**
   - Edit `specs/tasks.yaml` to plan your first implementation

5. **Validate baseline:**

   ```bash
   cargo xtask selftest
   ```

6. **Implement your first AC:**
   - Add domain handler in `crates/app-http/src/routes.rs`
   - Implement business logic in `crates/business-core/`
   - Add tests to make the BDD scenarios pass

7. **Verify completion:**

   ```bash
   cargo xtask test-ac AC-PROD-001
   cargo xtask selftest
   ```

---

## Files in This Example

- `README.md` (this file) – overview and guidance
- `sample-spec-additions.yaml` – example domain story, REQ, and ACs
- `sample-product-catalog.feature` – example BDD scenarios
- `sample-service-metadata.yaml` – example service metadata file

Use these as templates when forking!

---

## Next Steps

- Read `docs/how-to/new-service-from-template.md` for a full walkthrough
- See `TEMPLATE-CONTRACTS.md` for what you inherit from the template
- Use `cargo xtask help-flows` to discover available workflows
- Check `docs/AGENT_GUIDE.md` for agent-native development patterns
