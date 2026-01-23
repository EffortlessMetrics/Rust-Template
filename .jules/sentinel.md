## 2024-05-23 - Explicit GET Auth Bypass
**Vulnerability:** The `platform_auth` middleware explicitly bypassed authentication for `GET` and `HEAD` requests, exposing sensitive internal endpoints (like `/platform/status` and `/platform/idp/snapshot`) to unauthenticated access.
**Learning:** The developers likely intended to only protect "write" operations (mutations) but failed to consider that "read" operations on internal platforms also expose sensitive data.
**Prevention:** Authentication middleware should deny by default. Only explicitly public endpoints or harmless methods (like OPTIONS for CORS) should be bypassed. "Read-only" does not mean "Public".
