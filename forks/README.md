<!-- doclint:disable orphan-version -->
# Fork Registry

This directory tracks known forks of the Rust-as-Spec template kernel. Forks represent instances of this template adapted for specific knowledge domains or use cases.

## Purpose

The fork registry serves several purposes:

1. **Track Template Usage**: Understand how the template is being used across different domains
2. **Identify Patterns**: Discover common pain points that should be addressed in the kernel
3. **Coordinate Updates**: Know which kernel versions forks are based on for maintenance planning
4. **Enable Collaboration**: Connect fork maintainers for knowledge sharing

## Structure

- **`fork_schema.yaml`**: Schema definition for fork entries
- **`fork_registry.yaml`**: Human-readable index/summary of all forks
- **`FORK-*.yaml`**: Individual fork entry files (machine-readable)

## Commands

### List Forks

```bash
# List all registered forks
cargo xtask fork-list

# Filter by status
cargo xtask fork-list --status active
cargo xtask fork-list --status experimental
cargo xtask fork-list --status archived

# Filter by domain substring
cargo xtask fork-list --domain rust
cargo xtask fork-list --domain ml
```

### Register a Fork

```bash
cargo xtask fork-register \
  --name "My Fork Name" \
  --domain "my-domain" \
  --kernel-version "vX.Y.Z" \
  --maintainer-name "Your Name" \
  --maintainer-contact "you@example.com" \
  --status "experimental" \
  --url "https://github.com/org/repo" \
  --notes "Brief description of fork's purpose"
```

**Required fields:**

- `--name`: Human-readable fork name
- `--domain`: Knowledge domain or purpose (e.g., "rust-sdk", "python-ml", "knowledge-hub")
- `--kernel-version`: Which kernel version this fork is based on (e.g., "v3.3.4")

**Optional fields:**

- `--url`: Repository URL (may be private/omitted)
- `--maintainer-name`: Maintainer name or team
- `--maintainer-contact`: Contact method (email, GitHub handle, etc.)
- `--status`: Fork lifecycle status (active, archived, experimental) - defaults to "experimental"
- `--notes`: Free-form notes about the fork

After registration, you can edit the generated YAML file to add:

- `features`: Key capabilities added beyond the kernel
- `pain_points`: Known friction areas (link to friction entries or issues)
- `related_items`: Links to kernel issues, ADRs, or friction entries

## Fork Lifecycle

### Status Values

- **experimental**: Fork is in early development, exploring patterns
- **active**: Fork is actively maintained and in production use
- **archived**: Fork is no longer maintained or has been sunset

### Maintenance

Update fork entries when:

- Fork syncs with new kernel version (update `kernel_version` and `last_synced`)
- Fork status changes (experimental → active, active → archived)
- New pain points or features are discovered
- Maintainer contact information changes

## Integration with Kernel Maintenance

See [KERNEL_MAINTENANCE.md](../KERNEL_MAINTENANCE.md) for how fork feedback influences kernel development:

1. **Backporting**: Features/fixes validated in forks may be backported to the kernel
2. **Pain Point Tracking**: Fork pain points inform kernel improvement priorities
3. **Breaking Changes**: Active forks are notified of breaking kernel changes
4. **Pattern Discovery**: Common fork features suggest gaps in the kernel

## Examples

### Knowledge Hub Fork

```yaml
id: FORK-KHUB-001
name: "Knowledge Hub"
domain: "ml-documentation"
url: "https://github.com/org/knowledge-hub"
maintainer:
  name: "ML Team"
  contact: "ml-team@example.com"
kernel_version: v3.3.3
forked_at: "2024-01-15"
last_synced: "2024-03-20"
status: active
features:
  - "GraphQL API for ML model documentation"
  - "Extended BDD scenarios for ML validation"
  - "Custom platform endpoints for training metadata"
pain_points:
  - "FRICTION-BUNDLE-001: Slow bundle generation on large repos"
  - "https://github.com/kernel/repo/issues/123"
notes: >
  Production ML documentation platform. Heavy use of bundle command and
  platform API. Discovered performance issues with large spec_ledger files.
```

### SDK Template Fork

```yaml
id: FORK-SDK-001
name: "Rust SDK Template"
domain: "rust-sdk"
maintainer:
  name: "SDK Team"
kernel_version: v3.2.0
forked_at: "2023-11-10"
status: active
features:
  - "Client library ACs and validation rules"
  - "API contract testing patterns"
  - "SDK-specific docs structure"
notes: >
  Template for Rust client libraries. Standardizes SDK governance across
  the organization. Plans to sync with v3.3.3 kernel soon.
```

## Privacy

- Fork registration is **optional** - forks can use the template without registering
- Fork URLs can be omitted if the repository is private
- Maintainer contact information is optional
- Fork entries serve as a feedback mechanism, not an enforcement tool

## Questions?

- Open a GitHub issue in the kernel repo
- Create a friction entry if you encounter process issues
- Reach out to kernel maintainers via the documented channels
