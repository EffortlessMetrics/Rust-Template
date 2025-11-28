---
id: DESIGN-TPL-BUNDLE-CONTRACT-001
title: Bundle Contract and Scoped Work Context
author: governance-system
doc_type: design_doc
date: 2025-11-28
status: published
stories: [US-TPL-PLT-001]
requirements: [REQ-TPL-BUNDLE-CONTRACT]
tags: [platform, devex, ai]
acs: [AC-TPL-BUNDLE-LAYOUT, AC-TPL-BUNDLE-CONTENT]
adrs: [ADR-0001, ADR-0005]
---

# Bundle Contract and Scoped Work Context

## Problem

Agents and developers need a clear, bounded context for focused work. Without bundles, they must scan the entire repository to understand a feature or requirement, leading to context explosion and inefficient work.

## Solution

Implement a "bundle" concept: a curated, minimal subset of files and specs relevant to a specific piece of work (e.g., implementing an AC or feature). A bundle includes:
1. **Spec ledger snippet**: Only the stories, requirements, and ACs relevant to the work
2. **Feature files**: Only the BDD scenarios tagged with the AC ID
3. **Implementation context**: Code files that will be touched (inferred from test failures or file analysis)
4. **References**: Links to relevant ADRs, docs, and CLI commands

## Implementation Approach

- **Bundle generation**: `cargo xtask bundle <ref>` (where ref is AC ID, REQ ID, or story)
- **Output format**: Structured JSON or markdown report
- **Contents**:
  - Filtered spec_ledger.yaml (stories → requirements → ACs)
  - Relevant .feature file scenarios
  - List of likely code files and tests
  - Contextual docs and ADR summaries
- **Agent use**: Agents load bundles to scope work without scanning the whole repo

## Notes

- Bundles are generated, not hand-authored
- They serve agents (primary) and developers (secondary use case)
- See `specs/tasks.yaml` for bundle template schema
- Reduces context window usage and improves focus
