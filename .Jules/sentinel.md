## 2026-04-29 - Insecure Default CSP
**Vulnerability:** The default Content Security Policy (CSP) in `SecurityHeadersConfig::default()` included `'unsafe-inline'` and `'unsafe-eval'`.
**Learning:** Default configurations often unintentionally fallback to permissive development settings if environment checks or configuration parsing fails, which can silently bypass intended production strictness.
**Prevention:** Always hardcode default configurations (like CSP) to the strictest production-ready settings. Permissive development settings must require explicit opt-in (e.g., via `is_development` environment checks), rather than being the fallback default.
