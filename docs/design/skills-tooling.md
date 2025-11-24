---
id: DESIGN-TPL-SKILLS-TOOLING-001
title: Rust-native Skills Formatting and Linting
author: governance-system
doc_type: design_doc
date: 2025-11-22
status: draft
stories: [US-TPL-PLT-001]
requirements: [REQ-TPL-SKILLS-TOOLING]
tags: [platform, agent, devex]
acs: [AC-TPL-SKILLS-FMT, AC-TPL-SKILLS-LINT]
adrs: [ADR-0004, ADR-0005]
---

# Rust-native Skills Formatting and Linting

## Problem

Agent Skills are markdown files with YAML frontmatter that can drift from standards (inconsistent frontmatter, broken links to flows/APIs, malformed structure). Without automated checks, Skills quality degrades over time.

## Solution

Provide Rust-native xtask commands to format and lint Skills, eliminating external scripting dependencies and integrating with selftest workflow.

**Commands**:
1. `cargo xtask skills-fmt`: Normalizes SKILL.md files (frontmatter order, heading structure, link formatting)
2. `cargo xtask skills-lint`: Validates Skills (required frontmatter fields, references to valid flows/commands, structural rules)

## Implementation Approach

**Implementation**: `crates/xtask/src/commands/skills_{fmt,lint}.rs`

**skills-fmt logic**:
- Parse frontmatter and markdown sections
- Normalize frontmatter order: name, description, trigger, version
- Enforce heading hierarchy: # -> ## -> ###
- Format links to flows and APIs
- Write normalized content back to file

**skills-lint logic**:
- Validate required frontmatter: name, description present
- Check references: flows mentioned exist in `devex_flows.yaml`
- Verify commands referenced exist in `devex_flows.yaml`
- Check platform API endpoints mentioned are valid (`/platform/graph`, etc.)
- Fail on structural issues: missing headings, broken markdown

**Integration**: Add `skills-lint` to `cargo xtask selftest` as optional step (warns but doesn't fail initially).

**Benefits**: Consistent Skill format, catches drift early, prevents broken references, eliminates external script dependencies.
