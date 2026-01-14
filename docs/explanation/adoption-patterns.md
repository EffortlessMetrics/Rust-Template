<!-- doclint:disable orphan-version -->
<!-- Illustrative: This document uses version examples for explanation purposes. -->
# Explanation: Template Adoption Patterns

This document explains three common patterns for adopting this Rust service template in your organization. Each has trade-offs around control, maintenance, and template updates.

**Choose based on your needs:**
- **Pattern A:** Best for single services, teams that want full control
- **Pattern B:** Best for multiple services that want to stay current with template improvements
- **Pattern C:** Best for platform teams managing many services at scale

---

## Pattern A: Clone and Detach (Single Service)

**Use when:** Building one service and template is just a starting point.

### How It Works

1. **Clone the template:**

   ```bash
   git clone https://github.com/your-org/rust-template.git my-service
   cd my-service
   ```

2. **Remove Git history:**

   ```bash
   rm -rf .git
   ```

3. **Initialize as new repo:**

   ```bash
   git init
   git add .
   git commit -m "Initial commit from Rust template v1.0.0"
   git remote add origin https://github.com/your-org/my-service.git
   git push -u origin main
   ```

4. **Customize:**
   - Update `Cargo.toml` metadata
   - Replace example ACs with your domain
   - Remove unnecessary crates (e.g., if no gRPC, remove proto specs)
   - Adjust policies to match your governance needs

### Pros

✅ **Full control** - Template is just a starting point, you own everything
✅ **No merge conflicts** - No upstream changes to reconcile
✅ **Simplified workflow** - No tracking of template versions
✅ **Easy customization** - Delete what you don't need without worrying about upstream

### Cons

❌ **No template updates** - Bug fixes and improvements in template don't flow to you
❌ **Manual migration** - If template adds valuable features, you manually port them
❌ **Divergence** - Your service and template drift over time

### When to Use

- **One-off service** - Not planning to spin up many services
- **Experimental/prototype** - Testing the template, might not keep it
- **Highly customized** - Planning significant deviations from template structure
- **Small team** - Can manage all template code themselves

### Upgrade Strategy

**None.** You've forked the template. Updates are manual:

1. Watch template releases: <https://github.com/your-org/rust-template/releases>
2. Read changelog, identify useful changes
3. Manually apply changes to your service
4. Test and commit

**Example:**

```bash
# Template releases v1.1.0 with improved bundler

# In your service:
git diff https://github.com/your-org/rust-template/v1.0.0..v1.1.0 -- crates/xtask/src/commands/bundle.rs

# Manually apply relevant changes
vim crates/xtask/src/commands/bundle.rs
cargo test
git commit -m "Backport bundler improvements from template v1.1.0"
```

---

## Pattern B: Template as Upstream

**Use when:** Building multiple services and want to benefit from template improvements.

### How It Works

1. **Clone with template as remote:**

   ```bash
   git clone https://github.com/your-org/rust-template.git my-service
   cd my-service
   git remote rename origin template
   git remote add origin https://github.com/your-org/my-service.git
   ```

2. **Create your main branch:**

   ```bash
   git checkout -b main
   git push -u origin main
   ```

3. **Customize on your branch:**

   ```bash
   # Update metadata
   vim Cargo.toml
   vim .github/CODEOWNERS

   # Add your domain features
   vim specs/spec_ledger.yaml

   git add .
   git commit -m "Customize for my-service"
   git push origin main
   ```

4. **Pull template updates:**

   ```bash
   # When template releases v1.1.0
   git fetch template
   git checkout main
   git merge template/main
   # Resolve conflicts
   git commit
   git push origin main
   ```

### Pros

✅ **Get template updates** - Bug fixes and improvements flow automatically
✅ **Selective adoption** - Can cherry-pick changes instead of full merge
✅ **Clear provenance** - Git history shows what came from template
✅ **Multiple services benefit** - Repeat for each service

### Cons

❌ **Merge conflicts** - Template changes may conflict with your customizations
❌ **Requires discipline** - Need clear conventions about what to change where
❌ **Conflict resolution overhead** - Someone must own merges
❌ **Testing required** - Template changes might break your features

### When to Use

- **Multiple services** (3-10+) - Worth the merge overhead
- **Active template** - Template is being improved, you want updates
- **Moderate customization** - Not diverging wildly from template structure
- **Skilled team** - Can handle Git merge conflicts and testing

### Upgrade Strategy

**Periodic rebases or merges:**

#### Option 1: Merge (preserves history)

```bash
# Check template for new releases
git fetch template

# See what changed
git log main..template/main --oneline

# Merge into your service
git checkout main
git merge template/main

# Resolve conflicts (see conflict resolution strategy below)
# Test thoroughly
cargo run -p xtask -- selftest

# Commit and push
git push origin main
```

#### Option 2: Rebase (cleaner history)

```bash
git fetch template
git checkout main
git rebase template/main

# Resolve conflicts one commit at a time
# Test thoroughly
cargo run -p xtask -- selftest

git push origin main --force-with-lease
```

### Conflict Resolution Strategy

**Common conflict areas:**

1. **`specs/spec_ledger.yaml`** - You added your ACs, template added examples
   - **Resolution:** Keep both, organize by story

2. **`crates/app-http/src/lib.rs`** - You added routes, template improved middleware
   - **Resolution:** Keep your routes, take template middleware changes

3. **`.github/workflows/*.yml`** - You customized CI, template added new checks
   - **Resolution:** Merge both changes, test CI

4. **`Cargo.toml`** - Metadata and dependency conflicts
   - **Resolution:** Keep your metadata, merge dependency updates

**Best practice:**

```bash
# Before merging, create a safety branch
git checkout -b upgrade-to-v1.1.0
git merge template/main

# Resolve conflicts, test
cargo run -p xtask -- selftest

# If successful:
git checkout main
git merge upgrade-to-v1.1.0

# If failed:
git checkout main
# Try cherry-picking specific commits instead
```

### Conventions to Minimize Conflicts

**Template ownership zones:**
- `crates/xtask/` - Let template own this
- `policy/*.rego` - Let template own base policies
- `.github/workflows/` - Let template own CI structure

**Your ownership zones:**
- `specs/userstories/` - Your domain stories
- `specs/spec_ledger.yaml` - Your ACs (after template core)
- `crates/core/` - Your domain logic
- `specs/features/` - Your BDD scenarios (after template_core.feature)

**Shared zones (careful!):**
- `crates/app-http/src/lib.rs` - Template provides structure, you add routes
- `specs/openapi/` - Template provides example, you define your API

**Use comments to mark changes:**

```rust
// BEGIN MY-SERVICE CUSTOMIZATION
.route("/tasks", post(create_task))
.route("/tasks/:id", get(get_task))
// END MY-SERVICE CUSTOMIZATION
```

This helps during merge conflict resolution.

---

## Pattern C: Generator-Based (Platform Team)

**Use when:** Managing many services (10+) and want centralized template evolution.

### How It Works

Build a **generator tool** that stamps out services from the template:

```bash
# Platform team maintains generator
cargo install --path ./template-generator

# Developers use it
template-generator new my-service --domain tasks
cd my-service
git init
git add .
git commit -m "Generated from template v1.0.0"
```

### Architecture

```
┌─────────────────────────────────────────────┐
│  Template Repository                        │
│  - Canonical source of truth                │
│  - Tagged releases (v1.0.0, v1.1.0, ...)    │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│  Generator Tool                             │
│  - Reads template                           │
│  - Applies transformations                  │
│  - Renders service                          │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│  Generated Service                          │
│  - No link to template (detached)           │
│  - Fully customized for domain              │
│  - Team owns it                             │
└─────────────────────────────────────────────┘
```

### Generator Features

1. **Substitution:**
   - Replace `{{SERVICE_NAME}}` in files
   - Update package names in `Cargo.toml`
   - Set ownership in `CODEOWNERS`, `flags/registry.yaml`, `specs/privacy.yaml`

2. **Optional features:**
   - `--no-grpc` - Remove proto specs and dependencies
   - `--no-graphql` - Remove GraphQL schema
   - `--minimal` - Only HTTP, no other protocols
   - `--domain <name>` - Pre-populate with domain example

3. **Validation:**
   - Run `cargo run -p xtask -- selftest` after generation
   - Ensure generated service builds and tests pass
   - Pre-commit hooks installed

### Example Generator (Rust)

```rust
// template-generator/src/main.rs
use clap::Parser;
use std::fs;
use std::path::Path;

#[derive(Parser)]
struct Args {
    /// Service name (e.g., "user-service")
    name: String,

    /// Domain example to include
    #[arg(long)]
    domain: Option<String>,

    /// Exclude gRPC support
    #[arg(long)]
    no_grpc: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Copy template
    let template_path = Path::new("../rust-template");
    let output_path = Path::new(&args.name);

    fs::create_dir_all(output_path)?;
    copy_dir_recursive(template_path, output_path)?;

    // Apply transformations
    substitute_vars(output_path, &[
        ("{{SERVICE_NAME}}", &args.name),
        ("{{CRATE_NAME}}", &args.name.replace("-", "_")),
    ])?;

    // Remove optional features
    if args.no_grpc {
        fs::remove_dir_all(output_path.join("specs/proto"))?;
        remove_crate(output_path, "app-grpc")?;
    }

    // Add domain example
    if let Some(domain) = args.domain {
        add_domain_example(output_path, &domain)?;
    }

    // Validate
    validate_generated_service(output_path)?;

    println!("✅ Service '{}' generated successfully!", args.name);
    Ok(())
}
```

### Pros

✅ **Centralized control** - Platform team owns template evolution
✅ **Consistency** - All services start from same baseline
✅ **Easy upgrades** - Regenerate with newer template version
✅ **Customizable** - Generator can add/remove features
✅ **Governance** - Platform team enforces standards

### Cons

❌ **Tooling investment** - Need to build and maintain generator
❌ **Regeneration challenges** - Can't easily re-generate existing services
❌ **Generator complexity** - More features = more generator code
❌ **Service divergence** - Services still drift from template over time

### When to Use

- **Many services** (10+) - ROI on building generator
- **Platform team** - Has capacity to own template and generator
- **Standardization goals** - Want consistent service structure
- **Frequent service creation** - Spinning up new services regularly

### Upgrade Strategy

**Option 1: One-time generation (Pattern A)**

Services are generated once and detached. No upgrades.

**Option 2: Regeneration with overwrite protection**

```bash
# In service repo, mark customizations
git tag before-regen

# Regenerate from new template
template-generator new my-service --template v1.1.0 --output /tmp/my-service-v1.1.0

# Diff and merge
diff -r /tmp/my-service-v1.1.0 . > changes.diff
# Review changes.diff, apply selectively
```

**Option 3: Generator outputs migration scripts**

```bash
# Generator knows what changed from v1.0.0 → v1.1.0
template-generator upgrade my-service --from v1.0.0 --to v1.1.0

# Outputs migration script:
# migrate-v1.0.0-to-v1.1.0.sh
# - Updates dependencies
# - Adds new workflows
# - Preserves your customizations
```

---

## Choosing a Pattern

| Factor | Pattern A | Pattern B | Pattern C |
|--------|-----------|-----------|-----------|
| **Number of services** | 1-2 | 3-10 | 10+ |
| **Template updates** | Manual | Automatic (with effort) | Platform-managed |
| **Customization flexibility** | High | Medium | Medium (generator-limited) |
| **Merge conflict overhead** | None | Medium | Low (one-time generation) |
| **Tooling investment** | None | Low | High (build generator) |
| **Platform team required** | No | No | Yes |
| **Best for** | Independent teams | Active template, multiple services | Platform standardization |

### Decision Tree

```
Start here
    │
    ├─ Building 1-2 services?
    │   └─ Pattern A (Clone & Detach)
    │
    ├─ Building 3-10 services with shared template improvements?
    │   └─ Pattern B (Template as Upstream)
    │
    └─ Building 10+ services with platform team?
        └─ Pattern C (Generator-Based)
```

### Hybrid Approaches

**Combine patterns:**

- **Start with B, migrate to C**
  - Use Pattern B for first 5 services
  - Once you hit 10 services, invest in generator (Pattern C)
  - New services use generator, existing use Pattern B

- **A for experimental, B for production**
  - Prototypes use Pattern A (full control)
  - Production services use Pattern B (stay current)

- **C with optional B**
  - Generator creates service (Pattern C)
  - Service optionally keeps `template` remote (Pattern B) for manual updates

---

## Migration Between Patterns

### A → B (Detached → Upstream)

```bash
# In your service repo
git remote add template https://github.com/your-org/rust-template.git
git fetch template

# Identify your starting template version
# (check commit message, Cargo.toml, or README)

git tag my-changes-start

# Try merging template
git merge template/main
# Resolve conflicts
# Test thoroughly
```

### B → A (Upstream → Detached)

```bash
# Remove template remote
git remote remove template

# Clean up git history (optional)
git filter-branch --prune-empty --subdirectory-filter HEAD

# Now fully independent
```

### A/B → C (Manual → Generator)

**Platform team:**

1. Build generator from current template
2. Test generator on greenfield service
3. Document migration path

**Service teams:**

1. Leave existing services as-is (Pattern A or B)
2. New services use generator (Pattern C)
3. Optional: regenerate existing services when major refactor needed

---

## Template Versioning Best Practices

Regardless of pattern, follow semantic versioning for the template:

- **v1.0.0** - Initial stable release
- **v1.1.0** - Backward-compatible improvements (new features, non-breaking)
- **v1.0.1** - Backward-compatible bug fixes
- **v2.0.0** - Breaking changes (require migration)

### Changelog Discipline

**For each release, document:**

```markdown
## v1.1.0 (2025-11-20)

### Added
- LLM bundler now supports custom exclusion patterns
- New `xtask deploy` command for Kubernetes deployment

### Changed
- BDD step definitions refactored for better reusability
- Improved error messages in policy tests

### Fixed
- AC status report now handles scenarios with multiple tags

### Migration Notes
- Pattern A: No changes required
- Pattern B: Merge conflicts expected in `crates/xtask/src/commands/bundle.rs`
  - Resolution: Keep your custom exclusions, add new features
- Pattern C: Regenerate with `--template v1.1.0`
```

---

## Summary

**Pattern A: Clone and Detach**
- One-time fork, full control, no updates
- Best for: 1-2 services, experimental, highly customized

**Pattern B: Template as Upstream**
- Periodic merges, get updates, manage conflicts
- Best for: 3-10 services, active template, moderate customization

**Pattern C: Generator-Based**
- Central tool, consistent generation, platform-managed
- Best for: 10+ services, platform team, standardization

**Choose based on:**
- Number of services
- Template update frequency
- Team size and skill
- Governance requirements

**You can migrate between patterns** as your needs evolve. Start simple (Pattern A), grow to B when you have multiple services, invest in C when you have a platform team.

---

## See Also

- **[New Service from Template](../how-to/new-service-from-template.md)** - Step-by-step setup (uses Pattern A)
- **[Template Contracts](TEMPLATE-CONTRACTS.md)** - What's stable, what can change
- **[Architecture](architecture.md)** - Design decisions and principles
