# http-origin-parser - CLAUDE.md

Focused parser for HTTP/HTTPS origin strings.

## Responsibilities

- Parse origin strings into `(scheme, authority)` components
- Accept only `http://` and `https://` schemes
- Reject malformed origins deterministically

## Non-responsibilities

- Wildcard or rule matching
- Origin allowlist orchestration
- HTTP middleware orchestration
