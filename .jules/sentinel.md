## 2024-05-22 - [Critical] Unauthenticated GET Access to Platform Endpoints
**Vulnerability:** The `platform_auth_guard` middleware explicitly bypassed authentication for all `GET` and `HEAD` requests, exposing sensitive configuration and governance data via `/platform/*` endpoints even when `PLATFORM_AUTH_MODE` was set to `Basic` or `Jwt`.
**Learning:** The middleware was documented as protecting "write endpoints", but the introspection endpoints it exposed contained sensitive internal state (config, env vars, graph structure). "Read-only" does not imply "Public".
**Prevention:**
1.  Remove method-based bypasses in auth middleware unless strictly necessary (e.g. OPTIONS for CORS).
2.  Test auth middleware with all methods, not just POST/PUT.
3.  Ensure "Basic" auth mode supports standard `Authorization: Basic` header so browsers can prompt for credentials.
