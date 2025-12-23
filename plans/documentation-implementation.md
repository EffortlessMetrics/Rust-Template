# Documentation Implementation Guide

**Status**: 🔄 IDENTIFIED - Some documentation exists, some needs to be created

## Overview

This guide covers the implementation of missing documentation components needed to complete the Rust template's documentation suite. While some documentation already exists (platform-api-contract.md and agent-skills-reference.md), the add-database.md guide is missing and ADR reference inconsistencies need to be resolved.

## Current Documentation Status

### ✅ Already Exists
- [`docs/reference/platform_api_contract.md`](docs/reference/platform_api_contract.md) - Platform API contract documentation
- [`docs/AGENT_SKILLS.md`](docs/AGENT_SKILLS.md) - Agent skills reference guide
- [`docs/SKILLS_GOVERNANCE.md`](docs/SKILLS_GOVERNANCE.md) - Skills governance documentation

### ❌ Missing Documentation
- [`docs/how-to/add-database.md`](docs/how-to/add-database.md) - Database integration guide

### 🔄 Needs Updates
- ADR reference inconsistencies across documentation files
- Version reference updates needed across multiple docs
- Integration examples for new documentation components

## Implementation Tasks

### 1. Create Missing add-database.md Guide

**Location**: [`docs/how-to/add-database.md`](docs/how-to/add-database.md)

**Purpose**: Comprehensive guide for adding database integration to Rust template services

**Required Sections**:
```markdown
# Adding Database Integration to Rust Template

## Overview
This guide covers adding database support to Rust template services...

## Prerequisites
- Rust template v3.3.8+ installed
- Basic understanding of SQLx and database patterns
- Development environment set up

## Database Options
### PostgreSQL Integration
- Connection pooling configuration
- Migration management with Atlas
- Performance optimization patterns

### SQLite Integration  
- Embedded database option
- Configuration for different environments
- Testing strategies for database code

## Implementation Steps
### 1. Database Adapter Configuration
- Add database dependencies to Cargo.toml
- Configure connection pooling
- Set up environment-specific settings

### 2. Migration Management
- Create initial migration files
- Set up Atlas integration
- Configure migration runner

### 3. Service Integration
- Add database state management
- Implement repository pattern
- Add health check for database connectivity

### 4. Testing
- Unit tests for database operations
- Integration tests with testcontainers
- Performance testing

### 5. Documentation
- Update API documentation
- Add database configuration examples
- Create troubleshooting guide

## Configuration Examples
```yaml
# config/local.yaml
database:
  url: "postgresql://user:password@localhost:5432/template_db"
  max_connections: 10
  timeout_seconds: 30
  ssl_mode: "require"
  
  # SQLite for development
  sqlite:
    path: "./data/template.db"
    readonly: false
```

## Integration Commands
```bash
# Add database dependencies
cargo add sqlx postgres sqlx-cli --workspace

# Run database migrations
sqlx migrate run --database-url $DATABASE_URL

# Test database integration
cargo test -p adapters-db-sqlx

# Set up database for development
cargo run -p app-http --database-setup
```

### 2. Fix ADR Reference Inconsistencies

**Files to Update**:
- [`docs/ROADMAP.md`](docs/ROADMAP.md)
- [`docs/feature_status.md`](docs/feature_status.md)
- [`README.md`](README.md)
- [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md)

**ADR Reference Standards**:
```markdown
## References to ADRs
- Use consistent format: `ADR-XXXX-title`
- Include brief description of ADR purpose
- Link to related ADRs where appropriate
- Reference implementation status

## Consistent Reference Format
```markdown
See [ADR-0005](docs/adr/0005-xtask-selftest-single-gate.md) for governance model.
See [Security Implementation Guide](plans/security-implementation.md) for security decisions.
```

### 3. Update Version References

**Files to Update**:
- All documentation files with version mentions
- Configuration templates with version examples
- README files with version compatibility

**Version Update Process**:
```bash
# Update version references in documentation
find docs/ -name "*.md" -exec grep -l "v3\.3\." {} \;

# Update README with current version
sed -i 's/v3\.3\./v3.3.8/g' README.md
```

## Implementation Commands

### Documentation Creation Commands

```bash
# Create add-database.md
touch docs/how-to/add-database.md

# Add comprehensive content
cat > docs/how-to/add-database.md << 'EOF'
[Content from section above]
EOF

# Update ADR references
find docs/ -name "*.md" -exec sed -i 's/ADR-[0-9]\{4\}/ADR-XXXX/g' {} \;

# Update version references
find docs/ -name "*.md" -exec sed -i 's/v3\.3\.[0-9]/v3.3.8/g' {} \;
```

### Documentation Validation Commands

```bash
# Check documentation completeness
cargo xtask docs-check

# Validate internal links
find docs/ -name "*.md" -exec grep -l "ADR-XXXX" {} \; | head -10

# Test documentation examples
# Test code examples in documentation
cargo test --doc
```

## Testing Strategy

### Documentation Testing

```bash
# Test add-database.md examples
cargo test --doc add_database_integration

# Validate documentation links
markdown-link-check docs/

# Test ADR reference consistency
grep -r "ADR-[0-9]" docs/ | wc -l
```

### Integration Testing

```bash
# Test database integration with new documentation
cargo test -p adapters-db-sqlx integration_test

# End-to-end documentation workflow
# 1. Add database
# 2. Update documentation  
# 3. Test complete flow
```

## Success Criteria

### Documentation Implementation Success Metrics

- ✅ add-database.md created and comprehensive
- ✅ ADR reference inconsistencies resolved
- ✅ Version references updated across all documentation
- ✅ Documentation validation passing (`cargo xtask docs-check`)
- ✅ Integration tests passing with new database components
- ✅ Complete examples working with documented steps

### Verification Checklist

- [ ] add-database.md exists: `test -f docs/how-to/add-database.md`
- [ ] ADR references consistent: `grep -r "ADR-" docs/ | grep -v "inconsistent"`
- [ ] Version references updated: `grep -r "v3.3." docs/ | grep -v "v3.3.8"`
- [ ] Documentation validation passing: `cargo xtask docs-check`
- [ ] Integration tests passing: `cargo test -p adapters-db-sqlx`
- [ ] Examples tested: `cargo test --doc add_database_integration`

## Maintenance Procedures

### Daily Documentation Checks

```bash
# Validate documentation completeness
cargo xtask docs-check

# Check for broken internal links
markdown-link-check docs/

# Verify all examples are current
cargo test --doc
```

### Weekly Documentation Reviews

```bash
# Comprehensive documentation audit
find docs/ -name "*.md" -exec grep -l "TODO\|FIXME\|XXX" {} \;

# Update examples based on code changes
cargo test --doc

# Review and update ADR references
grep -r "ADR-" docs/ | sort | uniq -c
```

### Monthly Documentation Tasks

```bash
# Documentation coverage analysis
find docs/ -name "*.md" | wc -l | sort -nr

# Update documentation based on template changes
# Sync with latest template improvements

# Review and improve documentation organization
tree docs/ -d
```

## Troubleshooting

### Common Documentation Issues

**Missing Documentation**:
- **Problem**: add-database.md doesn't exist
- **Solution**: Run `touch docs/how-to/add-database.md` and add comprehensive content

**ADR Reference Issues**:
- **Problem**: Inconsistent ADR reference formats
- **Solution**: Use standardized format and validate with scripts

**Version Reference Issues**:
- **Problem**: Outdated version references in documentation
- **Solution**: Run version update script to synchronize all references

**Documentation Validation Failures**:
- **Problem**: `cargo xtask docs-check` failing
- **Solution**: Fix broken links and missing references

## Related Files

- [Missing Documentation](docs/how-to/add-database.md) - TO BE CREATED
- [Platform API Contract](docs/reference/platform_api_contract.md) - ✅ EXISTS
- [Agent Skills Reference](docs/AGENT_SKILLS.md) - ✅ EXISTS
- [Skills Governance](docs/SKILLS_GOVERNANCE.md) - ✅ EXISTS
- [Documentation Validation](scripts/docs-check.sh) - Check for existence
- [ADR References](docs/adr/) - Multiple ADR files
- [Version References](README.md, docs/ROADMAP.md) - Multiple files with version info

## Next Steps

The documentation implementation needs to be completed to achieve full template release readiness. Once add-database.md is created and ADR references are consistent, the template will have comprehensive documentation coverage.

## Implementation Priority

1. **HIGH**: Create missing add-database.md guide
2. **MEDIUM**: Fix ADR reference inconsistencies  
3. **LOW**: Update version references across documentation

The missing add-database.md guide is the highest priority item as it represents a significant gap in the template's documentation coverage.