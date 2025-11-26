# Template Versioning Strategy

## Overview

The template includes explicit version metadata across all specification files to track:
1. **Template version** - Which release of the template (v2.3.0)
2. **Schema version** - Spec file format version (v1.0)
3. **Last updated** - When the spec was last modified

## Where Version Metadata Appears

### 1. spec_ledger.yaml
```yaml
metadata:
  schema_version: "1.0"
  template_version: "2.4.0"
  last_updated: "2025-11-18"
  description: "Template core capabilities - v2.4.0 includes health, version, errors, and metrics"
```

**Purpose:**
- Tracks which template version introduced which ACs
- Enables compatibility checking across template updates
- Documents the feature set at this version

### 2. Feature Files (.feature)
```gherkin
# Template Version: v2.4.0
# Schema: spec_ledger.yaml v1.0
# Last Updated: 2025-11-18

Feature: Template Core Endpoints
```

**Purpose:**
- Documents when each feature was added
- Links to spec_ledger schema version
- Helps track breaking changes in BDD scenarios

### 3. OpenAPI Specification
```yaml
info:
  description: |
    Template-based service API (Template v2.3.0).
    ...
  x-template-version: "2.3.0"
  x-schema-version: "1.0"
  x-last-updated: "2025-11-17"
```

**Purpose:**
- API consumers know which template version
- Extension fields (x-*) for tooling to read
- Clear documentation of template core vs domain endpoints

## Benefits

### For Template Users
1. **Know what you have** - Clear indication of template version in use
2. **Track changes** - See when endpoints/ACs were added
3. **Migration path** - Understand what changed between template versions

### For Template Maintainers
1. **Breaking change detection** - Schema version bumps signal compatibility issues
2. **Feature tracking** - Know which version introduced which capabilities
3. **Documentation** - Self-documenting specs

### For LLMs
1. **Context awareness** - LLM knows template version when generating code
2. **Compatibility** - Can check if suggested changes match template version
3. **Documentation** - Clear metadata about what exists

## Version Semantics

### template_version
- Follows template releases: v2.3.0, v2.4.0, etc.
- Updated when template is upgraded
- Indicates feature set available

### schema_version
- Follows SemVer for spec file format changes
- v1.0 = current format
- Bump to v2.0 if spec_ledger.yaml structure changes (breaking)
- Bump to v1.1 if new optional fields added (non-breaking)

### last_updated
- ISO date: YYYY-MM-DD
- Updated whenever spec content changes
- Helps track freshness

## Maintenance Guidelines

### When to Update

**template_version:**
- When upgrading template to new release
- When backporting features from newer template versions
- When template is forked and versioned independently

**schema_version:**
- When spec_ledger.yaml structure changes (v1.0 → v2.0)
- When BDD feature file format expectations change
- When OpenAPI extension fields change meaning

**last_updated:**
- Every time you edit the file
- After adding new ACs
- After modifying feature files

### Example: Adding a New AC

```yaml
# In specs/spec_ledger.yaml
metadata:
  schema_version: "1.0"          # No change (format same)
  template_version: "2.3.0"       # No change (your version)
  last_updated: "2025-11-18"      # Update to today
  description: "..."              # Update if needed
```

## Integration with Tools

### Current xtask Commands
- `ac-status` - Reads spec_ledger.yaml metadata
- `bundle` - Includes specs with version metadata
- `selftest` - Validates all specs including metadata

### Future Enhancements
```bash
# Possible future commands
cargo xtask version              # Show template version from specs
cargo xtask migrate --to v2.4.0  # Migrate specs to new template version
cargo xtask validate-schema      # Check schema_version compatibility
```

## Examples

### Current Template (v2.3.0)
All specs show:
- template_version: "2.3.0"
- schema_version: "1.0"
- Includes: health, version, errors, metrics

### Hypothetical v2.4.0
If v2.4.0 adds OTLP tracing to core:
```yaml
metadata:
  template_version: "2.4.0"       # Bumped
  schema_version: "1.0"           # Same format
  last_updated: "2025-12-01"      # New date
  description: "Template core capabilities - v2.4.0 adds OTLP tracing"
```

### Breaking Schema Change
If spec_ledger.yaml format changes significantly:
```yaml
metadata:
  schema_version: "2.0"           # Breaking change
  template_version: "3.0.0"       # Coordinated bump
  last_updated: "2026-01-01"
  description: "New ledger format with feature flags integration"
```

## Decision Rationale

**Why add this now?**
- Template is at v2.3.0, good time to establish versioning
- Pilot phase will generate forks - need version tracking
- LLM context benefits from explicit version metadata

**Why these fields?**
- `template_version` - Users need to know what they're running
- `schema_version` - Tools need to know format compatibility
- `last_updated` - Helps track staleness and freshness

**Why in comments vs YAML?**
- Feature files: Comments (Gherkin doesn't support metadata blocks)
- YAML files: Structured metadata (machine-readable by tools)
- OpenAPI: Extension fields (standard practice for custom metadata)

## Summary

Version metadata provides:
- ✅ **Clarity** - Know what template version you have
- ✅ **Traceability** - Track when features were added
- ✅ **Tooling** - Enable version-aware automation
- ✅ **Migration** - Smooth template upgrades
- ✅ **Documentation** - Self-describing specs

All specs now carry v2.3.0 metadata, establishing a baseline for future evolution.
