## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2026-02-25 - Unprotected UI Routes
**Vulnerability:** The Platform UI routes (`/`, `/ui/*`) were not protected by the platform authentication middleware, allowing unauthenticated access to sensitive governance data (specs, tasks, config) even when `PLATFORM_AUTH_MODE` was enabled.
**Learning:** Middleware applied to nested routers (e.g. `nest("/platform", ...)`) does not automatically apply to other routers merged at the same level (e.g. `merge(ui_router)`). Explicit middleware application is required for each router or at the top level.
**Prevention:** Verify that all routes, including UI and "read-only" views, are covered by the intended authentication middleware stack. Use integration tests that assert 401 on all endpoints when auth is enabled.
