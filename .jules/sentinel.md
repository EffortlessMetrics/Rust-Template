## 2024-05-23 - CORS Wildcard Subdomain Vulnerability
**Vulnerability:** The CORS configuration allowed malicious domains ending with the allowed domain (e.g., `evilexample.com`) to pass the wildcard subdomain check for `*.example.com` because it used `ends_with` without verifying a preceding dot.
**Learning:** Simple string suffix checks are insufficient for domain verification. `ends_with` does not respect domain boundary semantics.
**Prevention:** Always verify that a subdomain match is preceded by a dot `.` or use a dedicated URL parsing library that handles domain segments correctly.
