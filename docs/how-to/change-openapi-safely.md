# How-to: Change OpenAPI Safely

This guide explains how to evolve HTTP API contracts while keeping CI green and avoiding breaking changes.

---

## Prerequisites

- Nix development environment (`nix develop`)
- Service running locally (`cargo run -p app-http`)
- Familiarity with OpenAPI 3.0 specification

---

## Step 1: Understand the Current Contract

Before making changes, examine the existing API contract:

```bash
# View the current OpenAPI spec
cat specs/openapi/openapi.yaml

# Or fetch from running service
curl http://localhost:8080/platform/openapi | jq
```

Key sections to understand:
- **paths**: All endpoint definitions
- **components/schemas**: Reusable type definitions
- **components/responses**: Standard response patterns

---

## Step 2: Plan Your Change

Classify your change:

| Change Type | Risk Level | Approach |
|-------------|------------|----------|
| **Add new endpoint** | Low | Add path, add schema, add tests |
| **Add optional field** | Low | Add to schema with `required: false` |
| **Add required field** | Medium | May break existing clients |
| **Rename field** | High | Requires deprecation cycle |
| **Remove endpoint** | High | Requires deprecation cycle |

**Golden rule**: Additions are safe. Removals and renames need careful handling.

---

## Step 3: Edit the OpenAPI Spec

Make your changes to `specs/openapi/openapi.yaml`:

```yaml
paths:
  /api/new-endpoint:       # New endpoint
    get:
      summary: "New feature endpoint"
      operationId: getNewFeature
      responses:
        '200':
          description: Success
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/NewFeatureResponse'

components:
  schemas:
    NewFeatureResponse:    # New schema
      type: object
      required:
        - id
        - data
      properties:
        id:
          type: string
        data:
          type: object
```

---

## Step 4: Validate Locally

Run the OpenAPI linter to catch issues early:

```bash
# Using Redocly (if available via Nix)
nix develop -c redocly lint specs/openapi/openapi.yaml

# Or use the xtask validation
cargo xtask openapi-lint
```

Common lint errors:
- Missing `operationId` on endpoints
- Schema references to undefined components
- Invalid HTTP status codes
- Missing required fields in schemas

---

## Step 5: Update BDD Scenarios

For any new or changed endpoints, add BDD coverage:

```gherkin
# specs/features/new-endpoint.feature
Feature: New Feature Endpoint
  @AC-NEW-001
  Scenario: New endpoint returns expected response
    When I GET /api/new-endpoint
    Then I receive 200 with JSON containing "id"
    And I receive 200 with JSON containing "data"
```

Run the BDD tests:

```bash
cargo xtask bdd
```

---

## Step 6: Implement the Handler

Add the endpoint implementation in `crates/app-http/`:

```rust
// In lib.rs - add the route
.route("/api/new-endpoint", get(new_endpoint_handler))

// Handler implementation
async fn new_endpoint_handler() -> impl IntoResponse {
    Json(NewFeatureResponse {
        id: "example".to_string(),
        data: serde_json::json!({}),
    })
}
```

---

## Step 7: Run Full Validation

```bash
cargo xtask selftest
```

This validates:
- Code compiles
- OpenAPI spec is valid
- BDD scenarios pass
- All governance checks pass

---

## Handling Breaking Changes

If you must make a breaking change:

### Option A: Versioned Endpoint

```yaml
paths:
  /api/v1/resource:  # Old endpoint (deprecated)
    deprecated: true
  /api/v2/resource:  # New endpoint
    # ... new definition
```

### Option B: Deprecation Header

Add `Deprecation` header to responses before removal:

```rust
.header("Deprecation", "true")
.header("Sunset", "2025-06-01")
```

### Option C: Feature Flag

Gate the change behind a feature flag for gradual rollout.

---

## CI Workflow Integration

The OpenAPI workflow (`.github/workflows/openapi.yml`) runs on every PR:

1. **Lint check**: Validates spec syntax
2. **Breaking change detection**: Compares against main branch
3. **Schema validation**: Ensures types are consistent

If the workflow detects a breaking change, it will:
- Add a warning comment to the PR
- Require explicit approval from a maintainer

---

## Troubleshooting

### "Schema not found" error

**Cause**: Referenced a schema that doesn't exist in components.

**Fix**: Add the schema definition to `components/schemas/`.

### "Breaking change detected"

**Cause**: Removed or renamed an existing field/endpoint.

**Fix**: Either revert the change or follow the deprecation cycle above.

### BDD tests fail with schema mismatch

**Cause**: Implementation doesn't match OpenAPI spec.

**Fix**: Ensure handler response matches the schema exactly.

---

## See Also

- **[reference/ci-workflows.md](../reference/ci-workflows.md)** - CI workflow details
- **[how-to/add-http-endpoint.md](add-http-endpoint.md)** - Adding new endpoints
- **[design/error-handling.md](../design/error-handling.md)** - Error response patterns
