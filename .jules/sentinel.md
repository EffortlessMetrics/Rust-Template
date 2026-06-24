## 2024-05-24 - Missing Vary: Origin header in CORS middleware
**Vulnerability:** The CORS middleware dynamically reflected the `Origin` header into `Access-Control-Allow-Origin` but failed to append `Vary: Origin`. This is a risk for intermediate cache poisoning where a response intended for one allowed origin might be cached and served to a different unauthorized origin.
**Learning:** `axum` and `hyper` `HeaderMap::insert` removes all previous values for a key. When appending to multi-value headers like `Vary`, `HeaderMap::append` must be used. Both preflight and actual requests must have `Vary: Origin` set when dynamically reflecting the `Origin`.
**Prevention:** Always verify that dynamically reflected CORS origin responses include `Vary: Origin` via `append`.
