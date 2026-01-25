## 2024-05-23 - Authentication Bypass on Read Operations
**Vulnerability:** The `platform_auth` middleware explicitly bypassed authentication for GET and HEAD requests, exposing sensitive introspection endpoints (`/platform/idp/snapshot`, `/platform/status`) to unauthenticated access even when `PLATFORM_AUTH_MODE` was configured.
**Learning:** The code comments and implementation diverged from the security requirements. Developers likely assumed "read-only" meant "public", but introspection data can be sensitive.
**Prevention:** Ensure authentication middleware is "deny by default" and only explicitly allow safe methods (like OPTIONS for CORS). Verify that "read-only" does not equate to "public" for internal platform data.
