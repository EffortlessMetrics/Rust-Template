## Investigation Report: Issue #11 - RSA Timing Vulnerability (RUSTSEC-2023-0071)

### Status
**Status:** ALREADY ADDRESSED ✅ - Properly governed and monitored
**Local gates:** `cargo audit` ✓, `cargo xtask selftest` ✓

### Evidence

| Check | Result |
|-------|--------|
| **Cargo Audit Status** | Clean (advisory properly ignored) |
| **RSA Crate Present** | Yes - `rsa v0.9.9` |
| **Dependency Chain** | `app-http` → `jsonwebtoken v10.2.0` → `rsa v0.9.9` |
| **Advisory Ignore** | Explicitly configured in `.cargo/audit.toml` |
| **Governance** | Full risk analysis in ADR-0007 |

**Governance Configuration in `.cargo/audit.toml`:**
```toml
[advisories]
ignore = [
    "RUSTSEC-2023-0071",  # rsa Marvin Attack - no fix available
]
```

**Risk Assessment (ADR-0007):**
- Severity: Medium (CVSS 5.9)
- Exploit Difficulty: High (requires timing side-channel observation)
- Mitigation: Explicit ignore with documented rationale
- Status: **NO PATCH AVAILABLE** as of latest RustSec advisory

### Impact

- **Actual Risk:** LOW for this template's use case (web services)
- **Attack Vector:** Timing side-channel requires local observation
- **Usage:** JWT signing/validation through HTTP (timing attacks mitigated by protocol layer)

### Plan

**Current state is correct.** No action needed.

**Monitoring in place:**
- `cargo audit` runs in pre-commit hook
- `cargo xtask selftest` includes audit gate
- CI Tier-1 runs full selftest

**When patch available:**
- [ ] Update `rsa` crate to patched version
- [ ] Re-run `cargo audit` to verify advisory clears
- [ ] Remove ignore entry from `.cargo/audit.toml`

### Decision / Next Action

**Recommend:** CLOSE AS MONITORING-IN-PLACE

All governance criteria satisfied:
1. ✅ Identified and tracked in audit.toml
2. ✅ Risk assessment completed (ADR-0007)
3. ✅ Conscious decision documented
4. ✅ Monitoring system active
5. ✅ Governance gates enforced

No immediate action required. The vulnerability is appropriately managed.
