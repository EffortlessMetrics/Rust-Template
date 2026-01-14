---
id: API-FORKS-001
title: Platform Forks API
doc_type: reference
status: published
audience: developers, idp-operators, integration-developers
tags: [api, forks, platform]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-PLATFORM-APIS]
acs: [AC-PLT-015]
adrs: [ADR-0005]
last_updated: 2025-12-27
---

# Platform Forks API

The Forks API provides endpoints for tracking template forks and customizations. Forks represent known deployments/customizations of the template, tracking their kernel versions, status, and relationships.

## Endpoints

### GET /platform/forks

Returns all registered template forks/branches.

#### Request Example

```bash
curl http://localhost:8080/platform/forks
```

#### Response

```json
{
  "forks": [
    {
      "id": "FORK-EXAMPLE-001",
      "name": "Example Service Fork",
      "domain": "rust-sdk",
      "status": "active",
      "kernel_version": "v3.3.3"
    }
  ],
  "total": 1
}
```

#### Response Schema

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `forks` | array | Yes | Array of fork summary objects |
| `forks[].id` | string | Yes | Fork identifier (e.g., "FORK-EXAMPLE-001") |
| `forks[].name` | string | Yes | Fork name |
| `forks[].domain` | string | Yes | Domain or service area |
| `forks[].status` | string | Yes | Fork status |
| `forks[].kernel_version` | string | Yes | Kernel version this fork is based on |
| `total` | integer | Yes | Total number of forks |

---

### GET /platform/forks/{name}

Returns detailed information about a specific fork. The `{name}` parameter can be either the full fork ID or just the identifier portion.

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|--------|-----------|-------------|
| `name` | string | Yes | Fork ID or name (e.g., "FORK-EXAMPLE-001" or "EXAMPLE-001") |

#### Request Examples

```bash
# Get by full ID
curl http://localhost:8080/platform/forks/FORK-EXAMPLE-001

# Get by identifier (without FORK- prefix)
curl http://localhost:8080/platform/forks/EXAMPLE-001

# Get by name (case-insensitive)
curl http://localhost:8080/platform/forks/Example%20Service%20Fork
```

#### Response

```json
{
  "id": "FORK-EXAMPLE-001",
  "name": "Example Service Fork",
  "domain": "rust-sdk",
  "kernel_version": "v3.3.3",
  "status": "active",
  "url": "https://github.com/example/fork",
  "maintainer": {
    "name": "Example Maintainer",
    "contact": "maintainer@example.com"
  },
  "forked_at": "2025-11-26",
  "last_synced": "2025-12-10",
  "features": [
    "custom-auth",
    "extended-logging"
  ],
  "pain_points": [
    "Missing feature X",
    "Complex configuration"
  ],
  "notes": "Initial fork for example service",
  "related_items": {
    "issues": ["ISSUE-001"],
    "adrs": ["ADR-0019"],
    "friction": ["FRICTION-001"]
  }
}
```

#### Response Schema

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `id` | string | Yes | Fork identifier |
| `name` | string | Yes | Fork name |
| `domain` | string | Yes | Domain or service area |
| `kernel_version` | string | Yes | Kernel version this fork is based on |
| `status` | string | Yes | Fork status |
| `url` | string | No | Repository URL |
| `maintainer` | object | No | Maintainer information |
| `maintainer.name` | string | Yes | Maintainer name |
| `maintainer.contact` | string | No | Contact information |
| `forked_at` | string | No | ISO 8601 date when fork was created |
| `last_synced` | string | No | ISO 8601 date of last kernel sync |
| `features` | array | Yes | List of custom features (may be empty) |
| `pain_points` | array | Yes | Known pain points (may be empty) |
| `notes` | string | No | Additional notes |
| `related_items` | object | No | Related governance items |
| `related_items.issues` | array | Yes | Related issue IDs (may be empty) |
| `related_items.adrs` | array | Yes | Related ADR IDs (may be empty) |
| `related_items.friction` | array | Yes | Related friction IDs (may be empty) |

---

## Fork Status Values

| Status | Description |
|--------|-------------|
| `active` | Fork is actively maintained |
| `archived` | Fork is no longer maintained |

---

## File Naming Convention

Fork files are stored in `forks/` directory with the following naming pattern:

```
forks/
├── README.yaml
├── fork_registry.yaml
├── fork_schema.yaml
└── FORK-{DOMAIN}-{NUMBER}.yaml
```

Example: `FORK-RUST-SDK-001.yaml`

---

## Fork Registry

The `forks/fork_registry.yaml` file maintains the master list of all registered forks:

```yaml
forks:
  - id: FORK-EXAMPLE-001
  - id: FORK-EXAMPLE-002
```

---

## Related Endpoints

- [`/platform/issues`](./issues.md) - Unified issues including forks
- [`/platform/status`](./status.md) - Fork counts in platform status

---

## Related Documentation

- [Platform API Overview](./overview/index.md)
- [Platform API Reference](../reference/platform-api-endpoints.md)
- [Forks Directory](../../forks/)
