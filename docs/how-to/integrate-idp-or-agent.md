---
id: GUIDE-TPL-INTEGRATE-IDP-001
title: Integrate with IDP or Agent
doc_type: how-to
status: published
audience: platform-engineers, operators, integration-developers
tags: [api, integration, idp, agent, platform, portal]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-PLATFORM-APIS]
acs: [AC-PLT-006, AC-PLT-019, AC-PLT-024]
adrs: [ADR-0004]
last_updated: 2025-11-27
---
<!-- doclint:disable orphan-version -->

# Integrate with IDP or Agent

**Audience:** Portal/IDP operators, platform engineers, or agents integrating this service into a larger system.

**Goal:** Use the `/platform/*` APIs and CLI commands to drive dashboards, health checks, task management, and autonomous workflows.

---

## TL;DR: Three API Families

| API | Endpoint | Use Case |
|-----|----------|----------|
| **Governance Health** | `/platform/status` | Portal widget: "Is this service governed?" |
| **Governance Graph** | `/platform/graph` | Dashboard: Show REQ/AC/doc relationships |
| **Tasks & Hints** | `/platform/tasks`, `/platform/agent/hints` | Task management: "What should I work on?" |
| **Docs Index** | `/platform/docs/index` | Doc discovery: "What docs exist?" |
| **Flows & Commands** | `/platform/devex/flows` | Automation: "What workflows can I run?" |

All responses are JSON. No database required—same data CI validates.

---

## The Contract

**What you integrate against:**

1. **`/platform/*` HTTP endpoints** – All platform APIs live under this path. The endpoints listed above are the kernel contract; their response schemas are stable.
2. **OpenAPI spec** – Machine-readable contract at `/platform/openapi` and in `specs/openapi/openapi.yaml`. Use this for codegen.
3. **CLI JSON mode** – All xtask commands support `--json` for scripting (`cargo xtask status --json`, etc.).

**Reference implementation:**

The Backstage plugin in [`examples/backstage-plugin/`](../../examples/backstage-plugin/) demonstrates:

- TypeScript types generated from OpenAPI (see `src/types/PlatformClient.ts`)
- React components consuming `/platform/status` and `/platform/docs/index`
- Wiring into Backstage catalog EntityPage

**After changing `/platform/*` or OpenAPI:**

```bash
cargo xtask idp-check   # Validate OpenAPI + TypeScript client alignment
```

This ensures your schema, Rust handlers, and TypeScript types stay in sync.

---

## Recipe 1: Portal Widget (Service Health)

**Goal:** Show "governance health" in Backstage, Port.io, or custom portal.

**Endpoint:**
```
GET /platform/status
```

**Response:**
```json
{
  "service": {
    "name": "Rust Template",
    "version": "3.3.6",
    "id": "template"
  },
  "governance": {
    "ac_total": 65,
    "ac_pass": 65,
    "ac_fail": 0,
    "ac_unknown": 0,
    "selftest_status": "pass",
    "selftest_gates": [
      { "name": "spec_validation", "status": "pass" },
      { "name": "code_compilation", "status": "pass" },
      { "name": "unit_tests", "status": "pass" },
      { "name": "bdd_acceptance", "status": "pass" },
      { "name": "docs_consistency", "status": "pass" },
      { "name": "security_scan", "status": "pass" },
      { "name": "policy_tests", "status": "pass" },
      { "name": "graph_integrity", "status": "pass" }
    ]
  },
  "policy": {
    "policies_total": 5,
    "policies_pass": 5,
    "policies_fail": 0
  }
}
```

**Portal integration:**

```python
# Pseudocode for a Backstage catalog plugin
response = requests.get("http://localhost:8080/platform/status")
status = response.json()

if status["governance"]["selftest_status"] == "pass":
    health = "HEALTHY"
    color = "green"
else:
    health = "AT RISK"
    color = "red"

widget_text = f"""
Governance: {health}
ACs: {status['governance']['ac_pass']}/{status['governance']['ac_total']}
Version: {status['service']['version']}
"""
```

**Display options:**

- **Traffic light:** ✅ green if all gates pass, 🟡 yellow if some ACs unknown, ❌ red if any fail
- **Scorecard:** Show "8/8 selftest gates green"
- **Trend:** Track health over time (call weekly, log results)

---

## Recipe 2: Governance Graph Dashboard

**Goal:** Visualize the relationship between stories, requirements, ACs, docs, and commands.

**Endpoint:**
```
GET /platform/graph
```

**Response:** (Simplified)
```json
{
  "stories": [
    { "id": "US-TPL-001", "title": "Service can be governed", "requirements": ["REQ-TPL-PLT-001"] }
  ],
  "requirements": [
    { "id": "REQ-TPL-PLT-001", "title": "...", "acs": ["AC-TPL-001"] }
  ],
  "acceptance_criteria": [
    { "id": "AC-TPL-001", "text": "Service exposes /health", "status": "pass", "tests": [...] }
  ],
  "docs": [
    { "id": "GUIDE-TPL-QUICKSTART-001", "title": "Quick Start Guide", "acs": ["AC-TPL-001"] }
  ],
  "commands": [
    { "name": "xtask selftest", "flows": ["selftest-gate"] }
  ]
}
```

**Mermaid rendering:**

```html
<!-- Portal displays as interactive graph -->
<div id="governance-graph"></div>
<script>
  fetch("http://localhost:8080/platform/graph")
    .then(r => r.json())
    .then(graph => {
      // Render Mermaid diagram from graph structure
      let mermaid = "graph LR\n";
      graph.stories.forEach(s => {
        mermaid += `  ${s.id}["${s.title}"]\n`;
      });
      // ... etc
      document.getElementById("governance-graph").innerHTML = mermaid;
    });
</script>
```

**Use cases:**

- **Impact analysis:** "If I change this requirement, what ACs and docs are affected?"
- **Coverage dashboard:** "Which requirements lack ACs?"
- **Compliance audit:** "Show the full chain from law/policy → REQ → AC → test"

---

## Recipe 3: Task Management Integration

**Goal:** Populate a kanban board or task manager with work items and agent suggestions.

**Endpoint 1: Task List**
```
GET /platform/tasks
```

**Response:**
```json
{
  "tasks": [
    {
      "id": "TASK-TPL-001",
      "title": "Implement health endpoint",
      "status": "in_progress",
      "owner": "alice",
      "requirement": "REQ-TPL-HEALTH",
      "acs": ["AC-TPL-001", "AC-TPL-002"],
      "labels": ["kernel", "health-check"]
    }
  ]
}
```

**Endpoint 2: Agent Hints (What Should I Work On?)**
```
GET /platform/agent/hints
```

**Response:**
```json
{
  "hints": [
    {
      "task_id": "TASK-TPL-002",
      "title": "Add metrics endpoint",
      "status": "todo",
      "owner": "anyone",
      "requirement": "REQ-TPL-METRICS",
      "acs": ["AC-TPL-015"],
      "recommended_flow": "governed-feature-dev",
      "suggested_commands": [
        "cargo xtask bundle implement_ac",
        "cargo xtask test-ac AC-TPL-015",
        "cargo xtask selftest"
      ],
      "priority": 1
    }
  ]
}
```

**Portal integration:**

```python
# Backlog board
response = requests.get("http://localhost:8080/platform/agent/hints")
hints = response.json()["hints"]

for hint in sorted(hints, key=lambda h: h["priority"]):
    print(f"[{hint['priority']}] {hint['title']}")
    print(f"  Flow: {hint['recommended_flow']}")
    print(f"  Commands: {' → '.join(hint['suggested_commands'])}")
```

**Agent integration:**

```python
# Claude Code or other agent
hints = get_agent_hints()

for hint in hints:
    if should_work_on(hint):
        # Execute suggested commands
        for cmd in hint["suggested_commands"]:
            run_command(cmd)
        # Agent knows where to focus
        context = generate_bundle(hint["task_id"])
        work = do_work(context)
        return work
```

---

## Recipe 4: Documentation Inventory

**Goal:** Auto-populate a docs portal or generate docs indexing.

**Endpoint:**
```
GET /platform/docs/index
```

**Response:**
```json
{
  "docs": [
    {
      "id": "GUIDE-TPL-QUICKSTART-001",
      "title": "Quick Start Guide",
      "type": "guide",
      "status": "published",
      "audience": ["developers", "team-leads"],
      "path": "docs/QUICKSTART.md",
      "tags": ["onboarding", "quickstart"],
      "acs": ["AC-PLT-001", "AC-PLT-015"],
      "requirements": ["REQ-PLT-ONBOARDING"],
      "related_docs": ["GUIDE-TPL-FIRST-FORK-001", "GUIDE-TPL-PRE-FORK-001"]
    }
  ]
}
```

**Portal integration:**

```html
<!-- Dynamic docs sidebar -->
<nav id="docs-nav"></nav>
<script>
  fetch("http://localhost:8080/platform/docs/index")
    .then(r => r.json())
    .then(docs => {
      let nav = "";
      docs.docs
        .filter(d => d.status === "published")
        .forEach(d => {
          nav += `<a href="/docs/${d.id}">${d.title}</a>\n`;
        });
      document.getElementById("docs-nav").innerHTML = nav;
    });
</script>
```

**Use cases:**

- **Knowledge base sync:** Auto-fetch docs from multiple service cells into central portal
- **Audience filtering:** "Show me docs for platform engineers"
- **AC coverage:** "Which docs cover this requirement?"

---

## Recipe 5: DevEx Flows & Commands

**Goal:** Drive automation: discover available workflows and what commands to run.

**Endpoint:**
```
GET /platform/devex/flows
```

**Response:**
```json
{
  "flows": [
    {
      "id": "governed-feature-dev",
      "title": "Governed Feature Development",
      "description": "Implement a feature following spec-as-code",
      "steps": [
        { "name": "Start from REQ + AC", "command": "cargo xtask ac-status" },
        { "name": "Create bundle", "command": "cargo xtask bundle implement_ac" },
        { "name": "Test AC", "command": "cargo xtask test-ac AC-XXX" },
        { "name": "Full validation", "command": "cargo xtask selftest" }
      ]
    }
  ]
}
```

**Agent integration:**

```python
# Ask: "What workflows does this service support?"
flows = requests.get("http://localhost:8080/platform/devex/flows").json()

for flow in flows["flows"]:
    print(f"Flow: {flow['title']}")
    for step in flow["steps"]:
        print(f"  → {step['command']}")
```

---

## Recipe 6: CLI Commands for Integration

Use these commands when you can't (or prefer not to) make HTTP requests:

```bash
# Health status (JSON)
cargo xtask status --json
# → { "governance": { "ac_pass": 65, ... } }

# AC coverage (JSON)
cargo xtask ac-status --json
# → [{ "id": "AC-TPL-001", "status": "pass", ... }]

# Service version (JSON)
cargo xtask version --json
# → { "version": "3.3.6", "kernel": "v3.3.6-kernel" }

# Friction log (JSON)
cargo xtask friction-list --json
# → [{ "id": "FRICTION-001", "title": "...", "refs": ["AC-123"] }]

# Questions log (JSON)
cargo xtask questions-list --json
# → [{ "id": "Q-001", "title": "...", "refs": ["REQ-123"] }]

# Fork registry (JSON)
cargo xtask fork-list --json
# → { "total": 3, "forks": [...] }
```

**Integration example:**

```bash
#!/bin/bash
# Portal health check script

SERVICE_URL="http://localhost:8080"
AC_STATUS=$(curl -s $SERVICE_URL/platform/status | jq -r '.governance.selftest_status')

if [ "$AC_STATUS" == "pass" ]; then
    echo "HEALTHY"
    exit 0
else
    echo "AT RISK"
    exit 1
fi
```

---

## Recipe 7: Custom Integration Flow

**Scenario:** You're building a custom workflow that combines multiple APIs.

**Example:** "Show me all failing ACs that have docs explaining how to fix them"

```bash
#!/bin/bash
# Find fixable failures

# Get AC status
ACS=$(curl -s http://localhost:8080/platform/status | \
      jq '.governance | select(.ac_fail > 0)')

# Get docs
DOCS=$(curl -s http://localhost:8080/platform/docs/index)

# Find ACs with associated docs
curl -s http://localhost:8080/platform/graph | \
  jq '.acceptance_criteria[] | select(.status == "fail") | .id' | \
  while read -r AC_ID; do
    HAS_DOC=$(echo "$DOCS" | jq ".docs[] | select(.acs[] | contains($AC_ID))")
    if [ ! -z "$HAS_DOC" ]; then
      echo "Fixable: $AC_ID"
    fi
  done
```

---

## Authentication & Security

By default, all `/platform/*` endpoints are open (read-only).

**If you need auth:**

```bash
export PLATFORM_AUTH_MODE=basic
export PLATFORM_AUTH_TOKEN="secret-token"
cargo run -p app-http
```

Then include token in requests:

```bash
curl -H "Authorization: Bearer secret-token" \
  http://localhost:8080/platform/status
```

For write operations (task updates, friction creation), auth is always required when enabled.

---

## Testing Your Integration

### 1. Start the service

```bash
cargo run -p app-http &
sleep 2
```

### 2. Test each recipe

```bash
# Health widget
curl http://localhost:8080/platform/status | jq '.'

# Graph
curl http://localhost:8080/platform/graph | jq '.stories[0]'

# Tasks
curl http://localhost:8080/platform/tasks | jq '.'

# Docs
curl http://localhost:8080/platform/docs/index | jq '.docs[0]'

# Flows
curl http://localhost:8080/platform/devex/flows | jq '.flows[0].steps'
```

### 3. Verify no errors

```bash
# Check console for warnings
# Expected: No 404s, all responses valid JSON
```

---

## Common Integration Patterns

### Pattern 1: Periodic Health Check

```bash
# Cron job every 5 minutes
*/5 * * * * curl -s http://localhost:8080/platform/status | \
  jq -e '.governance.selftest_status == "pass"' \
  || send_alert
```

### Pattern 2: Task Sync

```python
# Sync tasks to Jira every hour
import requests
import json

while True:
    tasks = requests.get("http://localhost:8080/platform/tasks").json()
    for task in tasks["tasks"]:
        jira.upsert_issue(task["id"], task)
    time.sleep(3600)
```

### Pattern 3: Governance Report

```python
# Weekly report
status = requests.get("http://localhost:8080/platform/status").json()
graph = requests.get("http://localhost:8080/platform/graph").json()

report = f"""
## Governance Report

- ACs: {status['governance']['ac_pass']}/{status['governance']['ac_total']} pass
- Selftest: {status['governance']['selftest_status']}
- Stories: {len(graph['stories'])}
- Requirements: {len(graph['requirements'])}
- Docs: {len(graph['docs'])}
"""
send_email(report)
```

### Pattern 4: Agent Orchestration

```python
# Agent decides what to work on
hints = requests.get("http://localhost:8080/platform/agent/hints").json()

for hint in sorted(hints["hints"], key=lambda h: h["priority"]):
    if is_available_to_work():
        work = do_work_on(hint)
        mark_task_complete(hint["task_id"])
```

---

## Troubleshooting

### 404 from /platform/status

→ Service not running. Start with `cargo run -p app-http`.

### 401 Unauthorized

→ Auth mode enabled but token not provided. See **Authentication & Security** above.

### Stale data in response

→ Call `/platform/status` to trigger refresh, or restart service.

### "What endpoints exist?"

→ Call `/platform/openapi` for OpenAPI spec, or see `docs/reference/platform-support.md`.

---

## See Also

- [docs/AGENT_GUIDE.md](../AGENT_GUIDE.md) – Detailed agent integration patterns
- [docs/design/platform-runtime-contract.md](../design/platform-runtime-contract.md) – API contract details
- [docs/design/platform-introspection.md](../design/platform-introspection.md) – How introspection works
- [docs/explanation/idp-positioning.md](../explanation/idp-positioning.md) – Relationship to portals/IDPs
