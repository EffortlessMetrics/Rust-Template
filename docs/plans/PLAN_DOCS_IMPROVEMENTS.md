# Plan: Documentation Registration and Improvements

**Date:** 2025-12-02
**Status:** Ready for Implementation
**Related ACs:** AC-TPL-PLATFORM-DOCS, AC-PLT-DOC-INDEX-FRONTMATTER

## Scope

**Files in scope:**
- `specs/doc_index.yaml` - Register all existing documentation
- `docs/reference/` (new directory) - Create API reference pages
- `docs/api/` (new directory) - Platform API documentation
- Existing docs/* files - Add/update frontmatter with story/requirement/AC references

**Documentation gaps identified:**
- Platform API endpoints (e.g., `/platform/status`, `/platform/graph`, `/platform/tasks`)
- CLI command references (xtask subcommands)
- Skills API (agent interface documentation)
- Configuration schema reference
- BDD test writing guide
- Governance artifact schemas (questions, friction, forks)

## Goals

1. Register all existing documentation in `specs/doc_index.yaml`
2. Create missing API reference documentation
3. Add frontmatter to all docs with proper REQ/AC linkage
4. Ensure bidirectional alignment between doc_index and frontmatter
5. Validate with `cargo xtask docs-check`

## Implementation Steps

### 1. Audit existing documentation

**Discover all docs:**
```bash
find docs/ -name "*.md" | sort > docs_inventory.txt
```

**Check which are registered:**
```bash
grep -E "^  - id:" specs/doc_index.yaml | sed 's/.*id: //' | sort > registered_docs.txt
comm -23 docs_inventory.txt registered_docs.txt > unregistered_docs.txt
```

### 2. Create missing API reference documentation

**Platform API reference** (`docs/api/platform-endpoints.md`):
- Document all `/platform/*` endpoints
- Include request/response schemas
- Add usage examples with curl
- Link to REQ-TPL-PLATFORM-INTROSPECTION and related ACs

**CLI reference** (`docs/reference/xtask-commands.md`):
- Document all xtask subcommands
- Include flags, examples, exit codes
- Link to REQ-PLT-DEVEX-CONTRACT

**Skills reference** (`docs/reference/skills-api.md`):
- Document agent Skills structure
- Include SKILL.md template and validation rules
- Link to REQ-TPL-SKILLS-GOVERNANCE

**Configuration reference** (`docs/reference/config-schema.md`):
- Document config_schema.yaml structure
- Include environment variable mapping
- Link to REQ-TPL-CONFIG-INTEGRITY

**BDD guide** (`docs/how-to/write-bdd-tests.md`):
- Document Gherkin conventions
- Include step definition patterns
- Link to REQ-PLT-ONBOARDING

**Governance artifacts reference** (`docs/reference/governance-artifacts.md`):
- Document question, friction, fork schemas
- Include CLI commands for managing artifacts
- Link to REQ-TPL-GOV-ARTIFACTS

### 3. Update doc_index.yaml with all documents

**Add entries for each document:**
```yaml
  - id: API-PLATFORM-ENDPOINTS
    path: docs/api/platform-endpoints.md
    title: "Platform API Endpoints Reference"
    category: reference
    tags: [api, platform, introspection]
    stories: [US-TPL-PLT-001]
    requirements: [REQ-TPL-PLATFORM-INTROSPECTION]
    acs: [AC-TPL-PLATFORM-GRAPH, AC-TPL-PLATFORM-DEVEX, AC-TPL-PLATFORM-DOCS]

  - id: REF-XTASK-COMMANDS
    path: docs/reference/xtask-commands.md
    title: "xtask CLI Command Reference"
    category: reference
    tags: [cli, devex, commands]
    requirements: [REQ-PLT-DEVEX-CONTRACT]
    acs: [AC-PLT-014, AC-PLT-015]

  # ... add entries for all other docs
```

### 4. Add/update frontmatter in all documentation files

**Example frontmatter pattern:**
```markdown
---
title: "Platform API Endpoints Reference"
category: reference
stories: [US-TPL-PLT-001]
requirements: [REQ-TPL-PLATFORM-INTROSPECTION]
acs: [AC-TPL-PLATFORM-GRAPH, AC-TPL-PLATFORM-DEVEX, AC-TPL-PLATFORM-DOCS]
tags: [api, platform, introspection]
last_updated: 2025-12-02
---
```

**Process:**
1. For each doc in docs/, add frontmatter if missing
2. Populate stories/requirements/acs by tracing content back to spec_ledger.yaml
3. Ensure consistency with doc_index.yaml entry

### 5. Validate bidirectional alignment

**Run docs-check:**
```bash
cargo xtask docs-check
```

**Validation should confirm:**
- All docs in doc_index have corresponding files
- All docs have frontmatter with matching IDs
- All stories/requirements/acs in frontmatter are in doc_index
- All stories/requirements/acs in doc_index are in frontmatter

### 6. Update MISSING_MANUAL.md or create DOC_ROADMAP.md

**Track remaining doc gaps:**
- Document which areas still need deep-dive guides
- Link to spec_ledger ACs that lack doc coverage
- Provide templates for future documentation

## Verification Commands

```bash
# Validate doc_index and frontmatter alignment
cargo xtask docs-check

# Check which docs are unregistered
cargo xtask docs-check --verbose

# Verify platform API docs are accessible
curl http://localhost:8080/platform/docs/index | jq '.docs[] | select(.id == "API-PLATFORM-ENDPOINTS")'

# Run BDD test for doc index frontmatter AC
CUCUMBER_TAG_EXPRESSION="@AC-PLT-DOC-INDEX-FRONTMATTER" cargo test -p acceptance --test acceptance

# Full validation
cargo xtask selftest
```

## Definition of Done

- [ ] All existing docs in docs/ are registered in doc_index.yaml
- [ ] New API reference docs created:
  - docs/api/platform-endpoints.md
  - docs/reference/xtask-commands.md
  - docs/reference/skills-api.md
  - docs/reference/config-schema.md
  - docs/how-to/write-bdd-tests.md
  - docs/reference/governance-artifacts.md
- [ ] All docs have frontmatter with proper story/requirement/AC linkage
- [ ] `cargo xtask docs-check` passes (bidirectional alignment validated)
- [ ] `GET /platform/docs/index` returns all registered docs
- [ ] AC-PLT-DOC-INDEX-FRONTMATTER BDD test passes
- [ ] `cargo xtask ac-status` shows doc-related ACs as PASS
- [ ] No other ACs flip to FAIL

## Notes

- **Estimated Effort:** 2-3 hours (discovery + writing + validation)
- **Priority:** Medium - improves discoverability and governance alignment
- **Risk Level:** Low - additive changes only
- **Dependencies:** None - can be done independently
- **Follow-up:** Consider creating a living doc roadmap for future improvements
