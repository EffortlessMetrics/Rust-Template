---
id: GUIDE-TPL-PORT-INTEGRATION-001
doc_type: how_to
title: "Integrating Platform Status with Port.io"
status: published
audience: platform-engineers, idp-implementers
tags: [idp, port.io, integration, governance]
stories: [US-TPL-PLT-001]
requirements:
  - REQ-TPL-PLATFORM-INTROSPECTION
  - REQ-TPL-AI-IDP-COMPAT
acs:
  - AC-TPL-CLI-JSON-CORE
adrs: [ADR-0016]
last_updated: 2025-12-01
---
<!-- doclint:disable orphan-version -->

# Integrating Platform Status with Port.io

This guide shows how to integrate a Rust-as-Spec platform cell with Port.io IDP (Internal Developer Portal) to visualize governance health, AC coverage, documentation status, and task progress in your developer portal.

## Why Port.io Integration?

Port.io provides a flexible catalog-based IDP that can display:
- **Service health tiles**: Real-time governance status from `/platform/status`
- **Quality scorecards**: AC coverage, test health, documentation completeness
- **Task tracking**: Work queue derived from `/platform/agent/hints`
- **Documentation catalog**: Doc inventory from `/platform/docs/index`

This integration leverages the platform's stable JSON contracts (see [JSON Contracts](../explanation/json-contracts.md) and [ADR-0016](../adr/0016-idp-tiles-json-contracts.md)) to ensure reliability.

---

## Prerequisites

Before starting, ensure you have:

1. **Port.io account**: Team or self-hosted instance with API access
2. **Port.io API credentials**: Client ID and secret from Port.io settings
3. **Platform service running**: Your Rust-as-Spec service deployed and accessible
4. **Network access**: Port.io can reach your platform's `/platform/*` endpoints (or use GitHub Actions for polling)
5. **Platform version**: Template v3.3.6 or later (for stable JSON contracts)

**Verify prerequisites:**
```bash
# Check platform is running
curl http://localhost:8080/platform/status

# Verify Port.io credentials
export PORT_CLIENT_ID="your-client-id"
export PORT_CLIENT_SECRET="your-client-secret"

# Test Port.io API access
curl -X POST https://api.getport.io/v1/auth/access_token \
  -H "Content-Type: application/json" \
  -d "{\"clientId\":\"$PORT_CLIENT_ID\",\"clientSecret\":\"$PORT_CLIENT_SECRET\"}"
```

---

## Step 1: Define Port.io Blueprint for Services

Port.io uses **blueprints** to define entity schemas. We'll create a `rust-template-service` blueprint to model platform cells.

### 1.1 Create Blueprint YAML

Save this as `port-blueprint-service.yaml`:

```yaml
identifier: rust-template-service
title: Rust-as-Spec Service
description: Platform cell following Rust-as-Spec governance model
icon: Rust
schema:
  properties:
    # Basic metadata
    service_id:
      title: Service ID
      type: string
      description: Unique service identifier from service_metadata.yaml

    template_version:
      title: Template Version
      type: string
      description: Platform cell template version (SemVer)

    display_name:
      title: Display Name
      type: string
      description: Human-readable service name

    description:
      title: Description
      type: string
      description: Service description

    # Governance health
    governance_passing:
      title: Governance Passing
      type: boolean
      description: All governance checks passing (from /platform/status)

    ac_total:
      title: Total ACs
      type: number
      description: Total acceptance criteria count

    ac_passing:
      title: Passing ACs
      type: number
      description: Acceptance criteria with all tests passing

    ac_failing:
      title: Failing ACs
      type: number
      description: Acceptance criteria with failing tests

    ac_coverage_pct:
      title: AC Coverage %
      type: number
      description: Percentage of ACs passing (calculated)

    # Documentation health
    docs_total:
      title: Total Docs
      type: number
      description: Total documentation count

    docs_with_issues:
      title: Docs with Issues
      type: number
      description: Documents failing doc_type contracts

    # DevEx metrics
    friction_total:
      title: Friction Log Entries
      type: number
      description: Total DevEx friction entries

    friction_open:
      title: Open Friction
      type: number
      description: Unresolved friction entries

    questions_open:
      title: Open Questions
      type: number
      description: Unresolved design questions

    # Task tracking
    tasks_total:
      title: Total Tasks
      type: number
      description: All tasks in specs/tasks.yaml

    tasks_todo:
      title: Todo Tasks
      type: number
      description: Tasks in Todo status

    tasks_in_progress:
      title: In Progress Tasks
      type: number
      description: Tasks actively being worked on

    # Links
    platform_url:
      title: Platform URL
      type: string
      format: url
      description: Base URL for platform endpoints

    repo_url:
      title: Repository URL
      type: string
      format: url
      description: Git repository URL

  required:
    - service_id
    - template_version
    - platform_url

calculationProperties:
  ac_coverage_pct:
    title: AC Coverage Percentage
    calculation: |
      (.properties.ac_passing / .properties.ac_total) * 100
    type: number

relations: {}

mirrorProperties: {}
```

### 1.2 Import Blueprint to Port.io

```bash
# Get Port.io access token
export PORT_TOKEN=$(curl -X POST https://api.getport.io/v1/auth/access_token \
  -H "Content-Type: application/json" \
  -d "{\"clientId\":\"$PORT_CLIENT_ID\",\"clientSecret\":\"$PORT_CLIENT_SECRET\"}" \
  | jq -r '.accessToken')

# Create blueprint
curl -X POST https://api.getport.io/v1/blueprints \
  -H "Authorization: Bearer $PORT_TOKEN" \
  -H "Content-Type: application/json" \
  -d @port-blueprint-service.yaml
```

**Expected result:** Blueprint `rust-template-service` visible in Port.io UI under "Builder" → "Blueprints"

---

## Step 2: Create Entity Sync Script

This script polls `/platform/status` and syncs data to Port.io.

### 2.1 Python Sync Script

Save as `port-sync.py`:

```python
#!/usr/bin/env python3
"""
Port.io sync script for Rust-as-Spec platform cells.

Polls /platform/status, /platform/docs/index, /platform/tasks
and syncs governance health to Port.io entities.

Usage:
  export PORT_CLIENT_ID="..."
  export PORT_CLIENT_SECRET="..."
  export PLATFORM_URL="http://localhost:8080"
  python3 port-sync.py
"""

import os
import sys
import requests
from typing import Dict, Any, Optional

# Configuration from environment
PORT_CLIENT_ID = os.getenv("PORT_CLIENT_ID")
PORT_CLIENT_SECRET = os.getenv("PORT_CLIENT_SECRET")
PLATFORM_URL = os.getenv("PLATFORM_URL", "http://localhost:8080")
PORT_API_URL = "https://api.getport.io/v1"

def get_port_token() -> str:
    """Authenticate with Port.io and get access token."""
    response = requests.post(
        f"{PORT_API_URL}/auth/access_token",
        json={
            "clientId": PORT_CLIENT_ID,
            "clientSecret": PORT_CLIENT_SECRET,
        },
    )
    response.raise_for_status()
    return response.json()["accessToken"]

def fetch_platform_status() -> Dict[str, Any]:
    """Fetch /platform/status from Rust-as-Spec service."""
    response = requests.get(f"{PLATFORM_URL}/platform/status", timeout=10)
    response.raise_for_status()
    return response.json()

def fetch_platform_docs() -> Dict[str, Any]:
    """Fetch /platform/docs/index for documentation health."""
    response = requests.get(f"{PLATFORM_URL}/platform/docs/index", timeout=10)
    response.raise_for_status()
    return response.json()

def fetch_platform_tasks() -> Dict[str, Any]:
    """Fetch /platform/tasks for task counts."""
    response = requests.get(f"{PLATFORM_URL}/platform/tasks", timeout=10)
    response.raise_for_status()
    return response.json()

def calculate_ac_coverage(governance: Dict[str, Any]) -> Dict[str, int]:
    """Calculate AC coverage from governance data."""
    # Note: This depends on /platform/coverage endpoint
    # For now, derive from /platform/status if available
    try:
        response = requests.get(f"{PLATFORM_URL}/platform/coverage", timeout=10)
        response.raise_for_status()
        coverage = response.json()

        total = coverage.get("summary", {}).get("total", 0)
        passing = coverage.get("summary", {}).get("passing", 0)
        failing = coverage.get("summary", {}).get("failing", 0)

        return {
            "ac_total": total,
            "ac_passing": passing,
            "ac_failing": failing,
        }
    except Exception as e:
        print(f"Warning: Could not fetch AC coverage: {e}", file=sys.stderr)
        return {"ac_total": 0, "ac_passing": 0, "ac_failing": 0}

def calculate_task_counts(tasks: Dict[str, Any]) -> Dict[str, int]:
    """Calculate task status counts."""
    task_list = tasks.get("tasks", [])
    counts = {
        "total": len(task_list),
        "todo": 0,
        "in_progress": 0,
        "review": 0,
        "done": 0,
    }

    for task in task_list:
        status = task.get("status", "").lower()
        if status == "todo":
            counts["todo"] += 1
        elif status == "inprogress":
            counts["in_progress"] += 1
        elif status == "review":
            counts["review"] += 1
        elif status == "done":
            counts["done"] += 1

    return counts

def build_port_entity(
    status: Dict[str, Any],
    docs: Dict[str, Any],
    tasks: Dict[str, Any],
    ac_coverage: Dict[str, int],
    task_counts: Dict[str, int],
) -> Dict[str, Any]:
    """Build Port.io entity from platform data."""
    service = status.get("service", {})
    governance = status.get("governance", {})

    # Map /platform/status fields to Port.io properties
    entity = {
        "identifier": service.get("service_id", "unknown"),
        "title": service.get("display_name", "Unknown Service"),
        "blueprint": "rust-template-service",
        "properties": {
            "service_id": service.get("service_id", "unknown"),
            "template_version": service.get("template_version", "unknown"),
            "display_name": service.get("display_name", "Unknown Service"),
            "description": service.get("description", ""),

            # Governance health
            "governance_passing": governance.get("policies", {}).get("status") == "passing",
            "ac_total": ac_coverage["ac_total"],
            "ac_passing": ac_coverage["ac_passing"],
            "ac_failing": ac_coverage["ac_failing"],

            # Documentation health
            "docs_total": docs.get("summary", {}).get("total", 0),
            "docs_with_issues": docs.get("summary", {}).get("with_issues", 0),

            # DevEx metrics
            "friction_total": governance.get("friction", {}).get("total", 0),
            "friction_open": governance.get("friction", {}).get("open", 0),
            "questions_open": governance.get("questions", {}).get("open", 0),

            # Task tracking
            "tasks_total": task_counts["total"],
            "tasks_todo": task_counts["todo"],
            "tasks_in_progress": task_counts["in_progress"],

            # Links
            "platform_url": PLATFORM_URL,
            "repo_url": service.get("links", {}).get("repository", ""),
        },
        "relations": {},
    }

    return entity

def upsert_port_entity(token: str, entity: Dict[str, Any]) -> None:
    """Create or update entity in Port.io."""
    identifier = entity["identifier"]
    blueprint = entity["blueprint"]

    # Port.io upsert: PUT to /blueprints/{blueprint}/entities/{identifier}
    url = f"{PORT_API_URL}/blueprints/{blueprint}/entities/{identifier}"

    response = requests.put(
        url,
        headers={
            "Authorization": f"Bearer {token}",
            "Content-Type": "application/json",
        },
        json=entity,
    )

    if response.status_code in (200, 201):
        print(f"✓ Synced entity {identifier} to Port.io")
    else:
        print(f"✗ Failed to sync {identifier}: {response.status_code} {response.text}", file=sys.stderr)
        response.raise_for_status()

def main():
    """Main sync workflow."""
    # Validate environment
    if not PORT_CLIENT_ID or not PORT_CLIENT_SECRET:
        print("Error: PORT_CLIENT_ID and PORT_CLIENT_SECRET must be set", file=sys.stderr)
        sys.exit(1)

    print(f"Syncing platform {PLATFORM_URL} to Port.io...")

    try:
        # 1. Authenticate with Port.io
        token = get_port_token()
        print("✓ Authenticated with Port.io")

        # 2. Fetch platform data
        status = fetch_platform_status()
        print("✓ Fetched /platform/status")

        docs = fetch_platform_docs()
        print("✓ Fetched /platform/docs/index")

        tasks = fetch_platform_tasks()
        print("✓ Fetched /platform/tasks")

        ac_coverage = calculate_ac_coverage(status.get("governance", {}))
        print("✓ Calculated AC coverage")

        task_counts = calculate_task_counts(tasks)
        print("✓ Calculated task counts")

        # 3. Build entity
        entity = build_port_entity(status, docs, tasks, ac_coverage, task_counts)

        # 4. Sync to Port.io
        upsert_port_entity(token, entity)

        print("✓ Sync complete!")

    except requests.exceptions.RequestException as e:
        print(f"✗ Network error: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"✗ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
```

### 2.2 Make Script Executable

```bash
chmod +x port-sync.py
```

### 2.3 Test Sync Locally

```bash
# Set environment variables
export PORT_CLIENT_ID="your-client-id"
export PORT_CLIENT_SECRET="your-client-secret"
export PLATFORM_URL="http://localhost:8080"

# Start platform service (if not already running)
cargo run -p app-http &

# Run sync
python3 port-sync.py
```

**Expected output:**
```
Syncing platform http://localhost:8080 to Port.io...
✓ Authenticated with Port.io
✓ Fetched /platform/status
✓ Fetched /platform/docs/index
✓ Fetched /platform/tasks
✓ Calculated AC coverage
✓ Calculated task counts
✓ Synced entity rust-template to Port.io
✓ Sync complete!
```

---

## Step 3: Map `/platform/status` Fields to Port Properties

The sync script maps platform JSON contracts to Port.io properties. Here's the full mapping:

| Platform Field | Port.io Property | Source Endpoint | Type |
|---------------|-----------------|----------------|------|
| `service.service_id` | `service_id` | `/platform/status` | string |
| `service.template_version` | `template_version` | `/platform/status` | string |
| `service.display_name` | `display_name` | `/platform/status` | string |
| `service.description` | `description` | `/platform/status` | string |
| `governance.policies.status` | `governance_passing` | `/platform/status` | boolean |
| `summary.total` | `ac_total` | `/platform/coverage` | number |
| `summary.passing` | `ac_passing` | `/platform/coverage` | number |
| `summary.failing` | `ac_failing` | `/platform/coverage` | number |
| `summary.total` | `docs_total` | `/platform/docs/index` | number |
| `summary.with_issues` | `docs_with_issues` | `/platform/docs/index` | number |
| `governance.friction.total` | `friction_total` | `/platform/status` | number |
| `governance.friction.open` | `friction_open` | `/platform/status` | number |
| `governance.questions.open` | `questions_open` | `/platform/status` | number |
| `tasks[].status` (counted) | `tasks_todo` | `/platform/tasks` | number |
| `tasks[].status` (counted) | `tasks_in_progress` | `/platform/tasks` | number |

**Calculated fields:**
- `ac_coverage_pct`: `(ac_passing / ac_total) * 100` (Port.io calculation property)

**Versioning:**
- Platform contracts are stable within MAJOR version (see [ADR-0016](../adr/0016-idp-tiles-json-contracts.md))
- Adding new fields is non-breaking (additive-only)
- Sync script should gracefully handle missing fields (use `.get()` with defaults)

---

## Step 4: Create Scorecards for Governance Compliance

Port.io scorecards evaluate entity health against rules. We'll create scorecards for:
1. **Governance Health**: All ACs passing
2. **Documentation Coverage**: No docs with issues
3. **DevEx Quality**: Low friction and open questions

### 4.1 Governance Health Scorecard

Create in Port.io UI: **Scorecards** → **New Scorecard**

**Name:** Governance Health

**Rules:**
1. **All ACs Passing**
   - Level: Gold
   - Condition: `governance_passing == true`
   - Weight: 40%

2. **High AC Coverage**
   - Level: Silver
   - Condition: `ac_coverage_pct >= 90`
   - Weight: 30%

3. **No Failing ACs**
   - Level: Bronze
   - Condition: `ac_failing == 0`
   - Weight: 30%

### 4.2 Documentation Quality Scorecard

**Name:** Documentation Quality

**Rules:**
1. **No Doc Issues**
   - Level: Gold
   - Condition: `docs_with_issues == 0`
   - Weight: 50%

2. **Comprehensive Coverage**
   - Level: Silver
   - Condition: `docs_total >= 50`
   - Weight: 30%

3. **Up-to-date Docs**
   - Level: Bronze
   - Condition: `docs_with_issues <= 2`
   - Weight: 20%

### 4.3 DevEx Health Scorecard

**Name:** Developer Experience

**Rules:**
1. **Low Friction**
   - Level: Gold
   - Condition: `friction_open == 0`
   - Weight: 40%

2. **Questions Resolved**
   - Level: Silver
   - Condition: `questions_open <= 1`
   - Weight: 30%

3. **Active Maintenance**
   - Level: Bronze
   - Condition: `friction_total <= 10`
   - Weight: 30%

---

## Step 5: Set Up Scheduled Sync (GitHub Actions)

For production, run the sync script on a schedule using GitHub Actions.

### 5.1 Create Workflow File

Save as `.github/workflows/port-sync.yml`:

```yaml
name: Port.io Sync

on:
  # Run every 15 minutes
  schedule:
    - cron: '*/15 * * * *'

  # Allow manual trigger
  workflow_dispatch:

  # Sync on push to main (optional)
  push:
    branches: [main]

jobs:
  sync:
    name: Sync to Port.io
    runs-on: ubuntu-latest

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: '3.11'

      - name: Install dependencies
        run: pip install requests

      - name: Start platform service
        run: |
          # Build and start service in background
          cargo build --release -p app-http
          ./target/release/app-http &

          # Wait for service to be ready
          timeout 30 bash -c 'until curl -f http://localhost:8080/platform/status; do sleep 1; done'

      - name: Run Port.io sync
        env:
          PORT_CLIENT_ID: ${{ secrets.PORT_CLIENT_ID }}
          PORT_CLIENT_SECRET: ${{ secrets.PORT_CLIENT_SECRET }}
          PLATFORM_URL: "http://localhost:8080"
        run: python3 port-sync.py

      - name: Notify on failure
        if: failure()
        run: echo "Port.io sync failed! Check logs."
```

### 5.2 Add Secrets to GitHub

In your GitHub repository:
1. Go to **Settings** → **Secrets and variables** → **Actions**
2. Add secrets:
   - `PORT_CLIENT_ID`: Your Port.io client ID
   - `PORT_CLIENT_SECRET`: Your Port.io client secret

### 5.3 Test Workflow

```bash
# Trigger manually
gh workflow run port-sync.yml

# Check status
gh run list --workflow=port-sync.yml
```

**Alternative: Direct API Sync**

For deployed services with public endpoints:

```yaml
      - name: Sync deployed service
        env:
          PORT_CLIENT_ID: ${{ secrets.PORT_CLIENT_ID }}
          PORT_CLIENT_SECRET: ${{ secrets.PORT_CLIENT_SECRET }}
          PLATFORM_URL: "https://api.example.com"  # Your production URL
        run: python3 port-sync.py
```

---

## Step 6: Validation and Testing

### 6.1 Verify Entity in Port.io

1. Open Port.io UI: https://app.getport.io
2. Navigate to **Catalog** → **rust-template-service**
3. Find your service entity (e.g., `rust-template`)
4. Verify properties are populated:
   - `template_version`: Should match your platform version
   - `governance_passing`: Should reflect real status
   - `ac_coverage_pct`: Should be calculated correctly

### 6.2 Test Scorecard Evaluation

1. Open entity in Port.io
2. Check **Scorecards** tab
3. Verify:
   - **Governance Health**: Shows Gold/Silver/Bronze based on AC status
   - **Documentation Quality**: Reflects doc health
   - **DevEx Health**: Shows friction and question counts

### 6.3 Test Sync Updates

Make a change that affects governance:

```bash
# Fail a test locally
cargo test -- --ignored failing_test

# Re-run sync
python3 port-sync.py

# Verify in Port.io:
# - governance_passing should be false
# - ac_failing should increase
# - Scorecard should downgrade
```

Restore and verify recovery:

```bash
# Fix test
git checkout .

# Re-sync
python3 port-sync.py

# Verify Port.io reflects green state
```

### 6.4 Monitor Sync Health

Add monitoring to your workflow:

```yaml
      - name: Verify sync success
        run: |
          # Fetch entity from Port.io to verify sync worked
          TOKEN=$(curl -X POST https://api.getport.io/v1/auth/access_token \
            -H "Content-Type: application/json" \
            -d "{\"clientId\":\"$PORT_CLIENT_ID\",\"clientSecret\":\"$PORT_CLIENT_SECRET\"}" \
            | jq -r '.accessToken')

          curl -f -H "Authorization: Bearer $TOKEN" \
            "https://api.getport.io/v1/blueprints/rust-template-service/entities/rust-template"
```

---

## Troubleshooting

### Sync Script Fails with 401 Unauthorized

**Cause:** Invalid Port.io credentials

**Fix:**
```bash
# Verify credentials
echo $PORT_CLIENT_ID
echo $PORT_CLIENT_SECRET

# Test authentication manually
curl -X POST https://api.getport.io/v1/auth/access_token \
  -H "Content-Type: application/json" \
  -d "{\"clientId\":\"$PORT_CLIENT_ID\",\"clientSecret\":\"$PORT_CLIENT_SECRET\"}"
```

### Entity Not Appearing in Port.io

**Cause:** Blueprint doesn't exist or identifier mismatch

**Fix:**
```bash
# Check blueprint exists
curl -H "Authorization: Bearer $PORT_TOKEN" \
  https://api.getport.io/v1/blueprints/rust-template-service

# Check entity identifier matches service_id
curl http://localhost:8080/platform/status | jq '.service.service_id'
```

### Fields Missing or Null

**Cause:** Platform endpoints returning different schema

**Fix:**
```bash
# Inspect actual response
curl http://localhost:8080/platform/status | jq '.'

# Check template version (must be v3.3.6+)
curl http://localhost:8080/platform/status | jq '.service.template_version'
```

### Scorecard Always Fails

**Cause:** Calculation property syntax error

**Fix:**
1. Go to Port.io UI → **Builder** → **Blueprints** → `rust-template-service`
2. Check **Calculation Properties**
3. Verify `ac_coverage_pct` formula: `(.properties.ac_passing / .properties.ac_total) * 100`
4. Test with sample data: Set `ac_total = 10`, `ac_passing = 9` → Should show `90`

---

## Advanced: Multi-Service Sync

To sync multiple platform cells:

```python
# port-sync-multi.py

SERVICES = [
    {"url": "http://service-a:8080", "identifier": "service-a"},
    {"url": "http://service-b:8080", "identifier": "service-b"},
]

for service in SERVICES:
    PLATFORM_URL = service["url"]
    # Run sync logic (same as above)
```

---

## Related Documentation

- [JSON Contracts](../explanation/json-contracts.md) - Full schema reference
- [ADR-0016: IDP Tiles and JSON Contracts](../adr/0016-idp-tiles-json-contracts.md) - Design rationale
- [IDP Positioning](../explanation/idp-positioning.md) - Integration strategies
- [Platform Introspection Design](../design/platform-introspection.md) - API architecture
- [Port.io Documentation](https://docs.getport.io/) - Official Port.io guides

---

## Summary

This integration provides:
- **Real-time governance visibility**: Port.io dashboard shows AC health, doc status, and DevEx metrics
- **Automated scorecards**: Quality gates based on platform contracts
- **Scheduled sync**: GitHub Actions keeps Port.io updated every 15 minutes
- **Stable contracts**: Additive-only JSON evolution ensures long-term reliability

**Next steps:**
1. Customize scorecards for your team's quality standards
2. Add alerts (e.g., Slack notifications when governance fails)
3. Create Port.io dashboards aggregating multiple services
4. Extend sync script to include custom metrics
