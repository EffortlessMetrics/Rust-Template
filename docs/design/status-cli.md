---
doc_type: design_doc
id: DESIGN-PLT-STATUS
title: "Platform Status CLI Dashboard"
status: approved
owner: platform-team
stories:
  - US-TPL-PLT-001
requirements:
  - REQ-PLT-STATUS-CLI
adrs:
  - ADR-0005
---

# Platform Status CLI Dashboard

## Problem

Developers and agents need a quick way to assess governance health and get oriented without navigating the web UI or querying multiple API endpoints. Currently, understanding the platform state requires running multiple commands (`cargo xtask selftest`, checking spec counts manually, etc.) or visiting `http://localhost:8080/ui`.

## Solution

Provide a single `cargo xtask status` command that displays a consolidated dashboard of governance health metrics, including version information, REQ/AC/task counts, selftest status, and suggested next tasks. This becomes the "single pane of glass" for platform orientation.

## Implementation Approach

The `status` command will:
1. Query the governance graph (via `spec-runtime` crate) to count requirements, ACs, and tasks
2. Read the last selftest result from `target/selftest_status.json` (if available)
3. Check policy status from `target/policy_status.json`
4. Query `tasks.yaml` to find tasks in `Todo` or `InProgress` state
5. Format output as a structured CLI dashboard with clear sections

Output format will be human-readable text with clear visual separators, suitable for both terminal viewing and agent parsing.
