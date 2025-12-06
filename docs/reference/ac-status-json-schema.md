# `ac-status --json` Schema (v2.0)

This document describes the JSON schema emitted by:

```bash
cargo xtask ac-status --json
```

Use this as the contract for portals, agents, and reporting tools that consume AC status.

---

## 1. Top-level structure

```jsonc
{
  "schema_version": "2.0",
  "timestamp": "2025-12-05T12:00:00Z",
  "must_have_acs": {
    "total": 48,
    "passing": 46,
    "failing": 1,
    "unknown": 1
  },
  "optional_acs": {
    "total": 17,
    "passing": 15,
    "failing": 0,
    "unknown": 2
  },
  "coverage_percent": 93.8,
  "acs": [ /* AcJson[] */ ]
}
```

### 1.1 Fields

| Field              | Type   | Description                                                  |
| ------------------ | ------ | ------------------------------------------------------------ |
| `schema_version`   | string | Schema version. Currently `"2.0"`. Bump on breaking changes. |
| `timestamp`        | string | RFC 3339 timestamp when the report was generated.            |
| `must_have_acs`    | object | Stats for ACs with `must_have_ac=true`.                      |
| `optional_acs`     | object | Stats for ACs with `must_have_ac=false`.                     |
| `coverage_percent` | number | `(passing / total) * 100`.                                   |
| `acs`              | array  | Array of per-AC status objects.                              |

### 1.2 Stats object

Both `must_have_acs` and `optional_acs` share this shape:

```jsonc
{
  "total": 48,
  "passing": 46,
  "failing": 1,
  "unknown": 1
}
```

| Field     | Type   | Description                        |
| --------- | ------ | ---------------------------------- |
| `total`   | number | Total ACs in this classification.  |
| `passing` | number | ACs with `status == "pass"`.       |
| `failing` | number | ACs with `status == "fail"`.       |
| `unknown` | number | ACs with `status == "unknown"`.    |

---

## 2. Per-AC object

```jsonc
{
  "id": "AC-PLT-001",
  "story_id": "US-TPL-PLT-001",
  "req_id": "REQ-PLT-ONBOARDING",
  "text": "xtask doctor validates Rust, Nix, conftest",
  "status": "pass",
  "source": "coverage",
  "must_have_ac": true,
  "scenarios": ["doctor validates the environment health"],
  "tests": [
    {
      "type": "integration",
      "tag": "@AC-PLT-001",
      "file": "specs/features/xtask_devex.feature",
      "module": "xtask::commands::doctor::tests"
    }
  ],
  "tests_total": 3,
  "tests_executed": 1
}
```

### 2.1 Fields

| Field            | Type     | Description                                          |
| ---------------- | -------- | ---------------------------------------------------- |
| `id`             | string   | AC ID (e.g., `AC-PLT-001`).                          |
| `story_id`       | string   | Parent story ID.                                     |
| `req_id`         | string   | Parent requirement ID.                               |
| `text`           | string   | Human-readable AC description.                       |
| `status`         | string   | `"pass"`, `"fail"`, or `"unknown"`.                  |
| `source`         | string   | Primary result source (see below).                   |
| `must_have_ac`   | boolean  | Whether this AC participates in strict coverage.     |
| `scenarios`      | string[] | Scenario names mapped via `@AC-…` tags.              |
| `tests`          | object[] | Test mappings from ledger.                           |
| `tests_total`    | number   | Number of mapped tests (from ledger).                |
| `tests_executed` | number   | Number of mapped tests that actually ran.            |

### 2.2 Status values

| Status    | Meaning                                           |
| --------- | ------------------------------------------------- |
| `pass`    | All mapped tests passed.                          |
| `fail`    | At least one mapped test failed.                  |
| `unknown` | No test results available (no BDD/unit coverage). |

### 2.3 Source values

| Source      | Description                                                |
| ----------- | ---------------------------------------------------------- |
| `coverage`  | Result from `coverage.jsonl` (streaming BDD, preferred).   |
| `junit`     | Result from JUnit XML fallback.                            |
| `json`      | Result from Cucumber JSON fallback.                        |
| `inferred`  | No test results; status is ledger-only (`Unknown`).        |

**Preference order:** `coverage.jsonl` → `ac_report.json` → JUnit XML → inferred.

---

## 3. `must_have_ac` semantics

The `must_have_ac` classification uses **AND semantics** between requirement and AC:

```
effective_must_have = REQ.must_have_ac AND AC.must_have_ac
```

| REQ.must_have_ac | AC.must_have_ac | Effective |
| ---------------- | --------------- | --------- |
| `true` (default) | `true` (default)| `true`    |
| `true`           | `false`         | `false`   |
| `false`          | `true`          | `false`   |
| `false`          | `false`         | `false`   |

Both default to `true` if not explicitly specified in `spec_ledger.yaml`.

### 3.1 Governance enforcement

When `XTASK_STRICT_AC_COVERAGE=1` is set:

- Selftest **fails** if any `must_have_ac=true` AC has `status == "unknown"`.
- This enforces BDD coverage for kernel ACs.

Without the env var, unknown ACs are advisory (warnings only).

---

## 4. Consuming this schema

### 4.1 For agents and bots

```bash
# Get JSON report
cargo xtask ac-status --json > /tmp/ac_status.json

# Parse with jq
jq '.must_have_acs.unknown' /tmp/ac_status.json   # Count of missing coverage
jq '.acs[] | select(.must_have_ac and .status == "unknown") | .id' /tmp/ac_status.json
```

### 4.2 For dashboards

Key on `schema_version` to detect breaking changes:

```typescript
interface AcStatusReport {
  schema_version: "2.0";
  timestamp: string;
  must_have_acs: StatsBlock;
  optional_acs: StatsBlock;
  coverage_percent: number;
  acs: AcJson[];
}

interface StatsBlock {
  total: number;
  passing: number;
  failing: number;
  unknown: number;
}

interface AcJson {
  id: string;
  story_id: string;
  req_id: string;
  text: string;
  status: "pass" | "fail" | "unknown";
  source: "coverage" | "junit" | "json" | "inferred";
  must_have_ac: boolean;
  scenarios: string[];
  tests: TestMapping[];
  tests_total: number;
  tests_executed: number;
}

interface TestMapping {
  type: string;
  tag: string;
  file: string;
  module: string;
}
```

### 4.3 Version migration

**From v1 to v2:**

| v1 Field        | v2 Field         |
| --------------- | ---------------- |
| `kernel_acs`    | `must_have_acs`  |
| `template_acs`  | `optional_acs`   |
| (none)          | `schema_version` |
| (none)          | `source`         |
| (none)          | `must_have_ac`   |

v1 used ID prefix heuristics (`AC-TPL-*` vs `AC-KERN-*`). v2 uses explicit `must_have_ac` metadata.

---

## 5. See also

- [xtask-commands.md](./xtask-commands.md#xtask-ac-status) - Full ac-status command reference
- [CHANGELOG.md](../../CHANGELOG.md) - Schema version history
- `specs/spec_ledger.yaml` - Source of AC definitions and `must_have_ac` metadata
