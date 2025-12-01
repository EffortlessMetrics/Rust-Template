#!/usr/bin/env python3
"""
Port.io Sync Script for Rust-as-Spec Platform Cells

Syncs governance data from /platform/idp/snapshot to Port.io catalog.
Designed for incremental, idempotent execution.

Usage:
    python3 sync_to_port.py [--verbose] [--force]

Environment:
    PLATFORM_URL: Base URL for platform (default: http://localhost:8080)
    PORT_CLIENT_ID: Port.io API client ID
    PORT_CLIENT_SECRET: Port.io API client secret
"""

import os
import sys
import json
import argparse
from datetime import datetime, timezone
from typing import Optional

try:
    import requests
except ImportError:
    print("Error: requests library required. Install with: pip install requests")
    sys.exit(1)

# Configuration
PLATFORM_URL = os.environ.get("PLATFORM_URL", "http://localhost:8080")
PORT_API_URL = "https://api.getport.io/v1"
BLUEPRINT_ID = "rust-template-service"

def get_port_token() -> str:
    """Authenticate with Port.io and return access token."""
    client_id = os.environ.get("PORT_CLIENT_ID")
    client_secret = os.environ.get("PORT_CLIENT_SECRET")

    if not client_id or not client_secret:
        raise ValueError("PORT_CLIENT_ID and PORT_CLIENT_SECRET must be set")

    response = requests.post(
        f"{PORT_API_URL}/auth/access_token",
        json={"clientId": client_id, "clientSecret": client_secret}
    )
    response.raise_for_status()
    return response.json()["accessToken"]

def fetch_idp_snapshot() -> dict:
    """Fetch governance snapshot from platform."""
    response = requests.get(f"{PLATFORM_URL}/platform/idp/snapshot")
    response.raise_for_status()
    return response.json()

def fetch_status() -> dict:
    """Fetch full status from platform."""
    response = requests.get(f"{PLATFORM_URL}/platform/status")
    response.raise_for_status()
    return response.json()

def transform_to_port_entity(snapshot: dict, status: dict) -> dict:
    """Transform platform data to Port.io entity format."""
    # Extract governance health
    health = snapshot.get("governance_health", {})
    ac_coverage = health.get("ac_coverage", {})
    docs = snapshot.get("documentation", {})
    task_hints = snapshot.get("task_hints", {})

    # Calculate AC coverage percentage
    total = ac_coverage.get("total", 0)
    passing = ac_coverage.get("passing", 0)
    coverage_pct = (passing / total * 100) if total > 0 else 0

    # Determine governance status
    governance_status = status.get("governance", {})
    policies = governance_status.get("policies", {})
    governance_passing = policies.get("status") == "passing"

    # Get service ID
    service_id = snapshot.get("service_id") or status.get("service_id") or "template"

    return {
        "identifier": service_id,
        "title": f"Rust-as-Spec: {service_id}",
        "blueprint": BLUEPRINT_ID,
        "properties": {
            "service_id": service_id,
            "template_version": snapshot.get("template_version", "unknown"),
            "display_name": service_id,
            "description": f"Platform cell running template v{snapshot.get('template_version', 'unknown')}",
            "governance_passing": governance_passing,
            "ac_coverage_percent": round(coverage_pct, 1),
            "ac_total": total,
            "ac_passing": passing,
            "ac_failing": ac_coverage.get("failing", 0),
            "docs_total": docs.get("total", 0),
            "docs_valid": docs.get("valid", 0),
            "docs_with_issues": docs.get("with_issues", 0),
            "tasks_pending": task_hints.get("total_pending", 0),
            "tasks_in_progress": task_hints.get("total_in_progress", 0),
            "friction_count": task_hints.get("friction_count", 0),
            "platform_url": PLATFORM_URL,
            "last_synced": datetime.now(tz=timezone.utc).isoformat()
        }
    }

def upsert_entity(token: str, entity: dict, verbose: bool = False) -> dict:
    """Create or update entity in Port.io."""
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json"
    }

    response = requests.post(
        f"{PORT_API_URL}/blueprints/{BLUEPRINT_ID}/entities",
        headers=headers,
        json=entity,
        params={"upsert": "true"}
    )

    if verbose:
        print(f"Port.io response: {response.status_code}")
        print(json.dumps(response.json(), indent=2))

    response.raise_for_status()
    return response.json()

def main():
    parser = argparse.ArgumentParser(description="Sync platform data to Port.io")
    parser.add_argument("--verbose", "-v", action="store_true", help="Verbose output")
    parser.add_argument("--force", "-f", action="store_true", help="Force full sync")
    parser.add_argument("--dump-only", "-d", action="store_true",
                        help="Fetch from platform and dump entity JSON (no Port sync)")
    args = parser.parse_args()

    try:
        # Fetch platform data
        if args.verbose or args.dump_only:
            print(f"Fetching from {PLATFORM_URL}...")

        snapshot = fetch_idp_snapshot()
        status = fetch_status()

        if args.verbose:
            print(f"Template version: {snapshot.get('template_version')}")
            print(f"Governance: {snapshot.get('governance_health', {}).get('status')}")

        # Transform to Port.io format
        entity = transform_to_port_entity(snapshot, status)

        if args.verbose or args.dump_only:
            print(f"Entity: {json.dumps(entity, indent=2)}")

        # If dump-only, exit without syncing
        if args.dump_only:
            print(f"\n--dump-only mode: would sync entity '{entity['identifier']}' to Port.io")
            return 0

        # Sync to Port.io
        print(f"Syncing {entity['identifier']} to Port.io...")
        token = get_port_token()
        result = upsert_entity(token, entity, args.verbose)

        print(f"Success! Entity synced: {result.get('entity', {}).get('identifier')}")
        return 0

    except requests.exceptions.ConnectionError as e:
        print(f"Error: Could not connect to platform at {PLATFORM_URL}")
        print(f"  Ensure the service is running: cargo run -p app-http")
        return 1
    except requests.exceptions.HTTPError as e:
        print(f"Error: HTTP {e.response.status_code}: {e.response.text}")
        return 1
    except ValueError as e:
        print(f"Error: {e}")
        return 1
    except Exception as e:
        print(f"Error: {e}")
        return 1

if __name__ == "__main__":
    sys.exit(main())
