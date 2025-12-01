# Example Fork: MyService

This is a minimal example demonstrating how to fork the Rust-as-Spec template for a specific service domain.

## What This Example Shows

This fork demonstrates:

1. **Service Identity** - How to define service metadata (`service_metadata.yaml`)
2. **Domain Stories & Requirements** - How to add domain-specific specs (`spec_additions.yaml`)
3. **BDD Scenarios** - How to write acceptance criteria as executable tests (`features/myservice.feature`)
4. **Fork Registration** - How to register your fork in the template's fork registry

## Structure

```
forks/example-myservice/
├── README.md                  # This file - overview and documentation
├── service_metadata.yaml      # Service identity and metadata
├── spec_additions.yaml        # Domain-specific stories, requirements, and ACs
├── features/
│   └── myservice.feature      # BDD scenarios for domain acceptance criteria
└── FORK-MYSERV-001.yaml      # Fork registry entry (optional)
```

## How to Use This Pattern

### 1. Copy and Customize

```bash
# Copy this example as a starting point
cp -r forks/example-myservice forks/my-actual-service

# Update service_metadata.yaml with your service details
# Update spec_additions.yaml with your domain requirements
# Write your BDD scenarios in features/
```

### 2. Service Metadata

`service_metadata.yaml` defines your service's identity:

- `service_id`: Unique identifier (lowercase, no spaces)
- `name`: Human-readable service name
- `description`: Brief description of what the service does
- `tags`: Categorization tags
- `version`: Service version (independent of template version)
- `owner_team`: Team responsible for this service
- `template_version`: Which template version you forked from

### 3. Domain Specifications

`spec_additions.yaml` extends the template's core specs with your domain:

- **Stories (US-*)**: User-facing capabilities
- **Requirements (REQ-*)**: Technical requirements for each story
- **Acceptance Criteria (AC-*)**: Testable conditions that define "done"

Follow the naming convention:
- Use a domain prefix (e.g., `MYSERV` for MyService)
- Stories: `US-MYSERV-NNN`
- Requirements: `REQ-MYSERV-FEATURE`
- Acceptance Criteria: `AC-MYSERV-NNN`

### 4. BDD Features

Write executable scenarios in `features/*.feature`:

- Tag scenarios with their AC IDs: `@AC-MYSERV-001`
- Use Given/When/Then format
- Keep scenarios focused and testable
- Link back to spec_additions.yaml

### 5. Integration

The fork system integrates with the template:

- **Platform API**: `GET /platform/forks` exposes fork metadata
- **xtask commands**: `cargo xtask fork-list`, `cargo xtask fork-register`
- **Governance**: Your specs and BDD scenarios are validated by `cargo xtask selftest`

## Example: Echo Endpoint

This example implements a simple echo endpoint:

- **Story**: US-MYSERV-001 - MyService Core Capabilities
- **Requirement**: REQ-MYSERV-ECHO - Echo endpoint for testing
- **AC**: AC-MYSERV-001 - GET /api/echo returns the provided message
- **BDD**: Scenario in `features/myservice.feature`

## Registering Your Fork

Optionally register your fork in the template's fork registry:

1. Create `forks/example-myservice/FORK-MYSERV-001.yaml` (see example in this directory)
2. Add entry to `forks/fork_registry.yaml` if contributing back to kernel
3. Use `cargo xtask fork-register` command (when available)

## Next Steps

1. **Implement**: Write the code that makes your BDD scenarios pass
2. **Validate**: Run `cargo xtask selftest` to ensure governance compliance
3. **Iterate**: Add more stories, requirements, and ACs as your service grows
4. **Sync**: Periodically sync with template kernel updates

## References

- Template Core Docs: `/docs/`
- Fork System: `/forks/README.md`
- Fork Schema: `/forks/fork_schema.yaml`
- Spec Ledger: `/specs/spec_ledger.yaml`
- BDD Guide: `/docs/how-to/`
