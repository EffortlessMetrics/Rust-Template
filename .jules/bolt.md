## 2025-02-28 - Middlewares are duplicated
**Learning:** Middleware configurations in Axum applications (specifically security headers and CORS) are duplicated between `crates/http-middleware/src/` and `crates/app-http/src/middleware/`. Both need symmetric improvements. Pre-parsing header values via `CachedSecurityHeaders` prevents dynamic allocations on a hot path.
**Action:** Always replicate middleware logic symmetrically when optimizing. `HeaderValue::clone()` is cheap as it's backed by `Bytes` which increments a reference count. Pre-parse configurations inside the `.cache()` function out of the `from_fn` middleware loop.
## 2025-02-28 - CORS CachedHeaders optimizations
**Learning:** Similar to security headers, CORS string array joining `config.exposed_headers.join(", ")` within the hot path creates unnecessary dynamic string allocations per-request.
**Action:** Cached parsing via `CachedCorsHeaders` prevents per request latency allocations.
