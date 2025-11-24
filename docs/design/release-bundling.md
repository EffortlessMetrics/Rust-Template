---
doc_type: design_doc
id: DESIGN-TPL-REL-BUNDLE
title: "Release Evidence Bundle Generation"
status: approved
owner: platform-team
stories:
  - US-TPL-PLT-001
requirements:
  - REQ-TPL-REL-BUNDLE
adrs:
  - ADR-0005
---

# Release Evidence Bundle Generation

## Problem

The release process needs comprehensive, structured evidence to support generating accurate release notes. Manual collection of tasks, requirements, ACs, ADRs, and git history is error-prone and time-consuming. Without automated bundling, release notes may miss important changes or fail to properly credit completed work.

## Solution

Implement `cargo xtask release-bundle X.Y.Z` command that automatically generates a structured markdown file at `release_evidence/vX.Y.Z.md`. The bundle aggregates all governance artifacts relevant to the release: completed tasks with their linked requirements and ACs, ADRs created or modified, git commit history since the last tag, selftest validation status, and policy compliance signals. The output format provides distinct sections optimized for downstream processing by LLMs into Keep a Changelog format.

## Implementation Approach

The release-bundle command will:
1. Parse `specs/tasks.yaml` to identify tasks marked as completed since the last release
2. Traverse the governance graph to collect linked requirements, ACs, and ADRs for each task
3. Execute `git log --pretty=format` to extract commit history between tags
4. Read `target/policy_status.json` and selftest results to capture governance health signals
5. Generate a markdown file with standardized sections (Tasks, Requirements/ACs, ADRs, Git Log, Governance Signals)
6. Write the output to `release_evidence/vX.Y.Z.md` with timestamp metadata

The structured format enables automated CHANGELOG generation while preserving human-readable evidence for audit trails and retrospectives.
