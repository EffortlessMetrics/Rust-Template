## 2026-04-28 - Hardened default Content Security Policy (CSP)
**Vulnerability:** The default CSP configuration allowed `'unsafe-inline'` for scripts and styles, and `'unsafe-eval'` for scripts, creating an XSS vulnerability vector.
**Learning:** Default configurations in security middleware must adhere to strict security postures (secure-by-default). Using permissive settings like `'unsafe-inline'` in production setups defeats the purpose of the CSP.
**Prevention:** Always verify that default security headers, especially CSP, exclude unsafe directives and enforce a strict policy unless explicitly configured otherwise for specific development or edge cases.
