# ADR-0022: Platform Metadata & Test Isolation

**Status**: ACCEPTED
**Date**: 2025-11-27
**Related**: ADR-0003 (Spec-as-Source-of-Truth), ADR-0017 (Selftest Summary), ADR-0021 (Agents Governance)
**Scope**: `/platform/status` endpoint, `specs/service_metadata.yaml`, `service-init` and metadata-related acceptance tests

---

## 1. Context

The platform template exposes service metadata through:

- `specs/service_metadata.yaml` (authoritative metadata file)
- `/platform/status` HTTP endpoint (runtime view of platform status, including metadata)
- `service-init` DevEx command (`cargo xtask service-init`), which updates:
  - `specs/service_metadata.yaml`
  - `README.md` branding and description

Acceptance tests cover:

- **AC-PLT-021** – `service-init` behaviour:
  - Updates service metadata & README branding
  - Validates service ID format (kebab-case)
  - Is idempotent (second run is safe)
- **AC-TPL-METADATA-COMPLETE** – metadata exposure:
  - `/platform/status` returns complete metadata and pointers to key docs/runbooks

The app-http server currently:

- Loads `service_metadata.yaml` **once at startup**.
- Caches metadata in application state for the lifetime of the server.
- Does *not* reload metadata on each request or on demand.

The acceptance harness:

- Creates an isolated temp workspace for most scenarios.
- But for `service-init` we deliberately run against the **real workspace root** because:
  - `service-init` is a real, workspace-mutating command.
  - It must see the actual `README.md`, `specs/`, and workspace layout.
- To keep tests hermetic, we:
  - Backup `specs/service_metadata.yaml` and `README.md` to `.bak` files.
  - Restore them after each `service-init` scenario.

This creates a tension:

- `service-init` legitimately mutates metadata.
- `/platform/status` is bound to whatever metadata was loaded at app startup.
- Tests that assume live metadata reactivity would require more sophisticated app lifecycle control than we currently have.

---

## 2. Decision

We will:

1. **Keep app-http's metadata as "load-once at startup"**
   `/platform/status` continues to read metadata from an in-memory state initialized from `specs/service_metadata.yaml` at server startup. We do not add runtime reloads or hot-reload endpoints for now.

2. **Treat `service-init` as a real workspace mutation, tested directly via files**
   For AC-PLT-021, we consider it sufficient to:

   - Validate that `service-init` updates:
     - `specs/service_metadata.yaml` (service_id, display_name, description)
     - `README.md` branding and description
   - Validate idempotency by running the command twice and asserting no failures.
   - Restore files to their original state after each scenario by using explicit backups rather than git stash.

3. **Keep AC-TPL-METADATA-COMPLETE focused on static metadata completeness**
   AC-TPL-METADATA-COMPLETE continues to:

   - Start app-http once with the default metadata.
   - Hit `/platform/status`.
   - Assert that the response contains metadata fields and doc references (e.g., `docs/runbooks/platform-kernel.md`, `docs/AGENT_GUIDE.md`).

   It does *not* assert dynamic reflection of post-`service-init` changes.

4. **Enforce test hygiene via backup/restore rather than git stash**
   For `service-init` scenarios:

   - We back up `specs/service_metadata.yaml` and `README.md` into `.bak` files under the repo root.
   - We restore from those backups at the end of each scenario.
   - We no longer rely on `git stash push/pop`, avoiding cross-test interference and stash-stack confusion.

---

## 3. Consequences

### 3.1 Behaviour

- `service-init` remains a **real** workspace command:
  - It is exercised against the actual repo root.
  - It mutates real files, which are then restored by the test harness.
- `/platform/status` remains **simple and cache-based**:
  - No risk of partial reloads or race conditions.
  - No extra complexity in the runtime path.

### 3.2 Testing

- AC-PLT-021 scenarios:
  - Assert directly on `specs/service_metadata.yaml` and `README.md`.
  - Use backup/restore + `test_repo_path` to ensure reads/writes hit the right location.
  - Are safe to run repeatedly without permanently mutating the repo.

- AC-TPL-METADATA-COMPLETE:
  - Assumes metadata is "whatever existed at app startup".
  - Is independent of `service-init` scenarios when the cleanup is correct.

- We accept that **"platform status reflects service identity after service-init"** is currently interpreted as:

  > "After service-init, the source-of-truth metadata file reflects the new service identity"

  not:

  > "/platform/status immediately reflects the new identity without restart".

  This is intentional and documented.

### 3.3 Tooling & DX

- `cargo xtask check` and `cargo xtask precommit`:
  - Can safely run `service-init` scenarios without leaving the repo dirty.
  - No longer depend on git stash for cleanup.
- Developers:
  - Can run acceptance locally without worrying about broken metadata or README state.
  - Can trust that service-init acceptance is exercising the real template, not a synthetic mirror.

---

## 4. Alternatives Considered

### 4.1 Add dynamic metadata reload to app-http (Not chosen for now)

We considered:

- Adding a `reload_metadata()` hook to app-http, and either:
  - A test-only helper (e.g., `World::reload_metadata()`), or
  - An admin/test-only endpoint (e.g., `/platform/status/reload`).

Downsides:

- Increases complexity of the runtime code for a test concern.
- Introduces race and state concerns:
  - When exactly is reload safe?
  - What about in-flight requests?
- Requires more nuanced test orchestration and app lifecycle management.

Given current requirements, this was judged unnecessary.

### 4.2 Per-scenario app instances

We could create a new app-http instance per scenario instead of sharing one across scenarios. This would:

- Ensure `/platform/status` always sees the latest metadata.
- Allow truly dynamic reflection of `service-init` changes.

Downsides:

- Slower test startup (higher per-scenario overhead).
- More complex test harness (per-scenario app init/teardown).
- Still doesn't remove the need for robust file cleanup.

We opted instead to keep a shared app instance and isolate file mutations.

---

## 5. Future Work

If in the future we need **truly dynamic metadata reflection**, we can:

1. Introduce a `PlatformMetadata` shared state (e.g., `Arc<RwLock<...>>`) in app-http.
2. Add a `reload_metadata_from_disk()` function that:
   - Reads `specs/service_metadata.yaml`.
   - Validates it.
   - Swaps it into the shared state.
3. Add:
   - A test-only hook in the acceptance harness, or
   - An internal admin endpoint guarded by environment or build flags.

At that point we can:

- Revert the AC-PLT-021 "platform status reflects service identity" scenario to hit `/platform/status` and assert on the live JSON instead of the file.

Until then, we explicitly accept the "cached at startup" semantics and test accordingly.

---

## 6. Summary

We:

- Keep `/platform/status` simple and cache-based.
- Treat `specs/service_metadata.yaml` as the canonical source for `service-init` tests.
- Exercise `service-init` against the real workspace root, with robust backup/restore.
- Avoid coupling runtime complexity to test-only concerns.

This keeps the template stable, the tests realistic, and the governance story clear without overcomplicating app-http.
