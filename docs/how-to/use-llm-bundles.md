# How-to: Use LLM Context Bundles

This guide shows you how to use the template's LLM context bundler to get AI assistance with focused, relevant context.

**Time:** 5 minutes
**Prerequisites:** Template cloned, xtask working

---

## What Are Context Bundles?

Context bundles are curated collections of files packaged as markdown, designed to give LLMs exactly the context they need for specific tasks without overwhelming them with irrelevant code.

**Benefits:**
- **Focused**: Only includes files relevant to the task
- **Bounded**: Respects token limits via `max_bytes`
- **Versioned**: Generated from specific git commits
- **Repeatable**: Same task → same bundle structure

---

## Available Tasks

Check `.llm/contextpack.yaml` for configured tasks:

```bash
cargo run -p xtask -- bundle implement_ac       # AC implementation context
cargo run -p xtask -- bundle implement_feature  # Feature development context
cargo run -p xtask -- bundle debug_tests        # Test debugging context
```

Each task defines:
- **include**: Glob patterns for files to bundle
- **max_bytes**: Size limit (default: 250KB)
- **description**: What the bundle is for

---

## When to Use LLM Bundles

### ✅ **Good Use Cases**

**1. Implementing or changing an AC**
```bash
# Generate context
cargo run -p xtask -- bundle implement_ac

# Paste bundle into LLM with prompt:
"Here is our ledger, specs, features, and core code.
Implement behavior to satisfy AC-123: 'Customer can view refund status'.
Show me the diffs you'd make to:
1. specs/spec_ledger.yaml
2. specs/features/*.feature
3. crates/core/src/*.rs
4. crates/acceptance/src/steps/*.rs"
```

**2. Investigating test failures**
```bash
# Generate debugging context
cargo run -p xtask -- bundle debug_tests

# Paste bundle with:
"The BDD scenario for AC-123 is failing with error: [paste error].
What's the likely cause and how should I fix it?"
```

**3. Understanding feature-AC relationships**
```bash
cargo run -p xtask -- bundle implement_feature

# Ask:
"Show me all ACs related to the refunds feature and their current implementation status."
```

### ❌ **Bad Use Cases**

**Don't use bundles for:**
- **Architecture changes** - too broad, needs human design
- **Policy changes** - requires careful human review and approval
- **Inventing new IDs** - AC/FT/flag IDs must come from the ledger
- **Cross-cutting refactors** - bundler can't capture all dependencies

---

## Best Practices

### 1. **Always verify LLM output**

LLMs can hallucinate. Before applying changes:
- ✅ Check AC IDs exist in `specs/spec_ledger.yaml`
- ✅ Verify BDD tags match ledger references
- ✅ Run `xtask check` after changes
- ✅ Run `xtask bdd` to validate scenarios

### 2. **Use specific prompts**

**Bad prompt:**
> "Fix this code"

**Good prompt:**
> "AC-123 requires that refund status is returned in the response.
> Current code at crates/core/src/refunds.rs:42 doesn't include status.
> Show me the minimal diff to add status field to RefundResponse."

### 3. **Don't let LLMs invent governance artifacts**

**Never accept:**
- New AC IDs not in the ledger
- New flag keys without owner/rollout plan
- New PII fields without retention policy

**Always:**
- Add new ACs to ledger first, then ask LLM to implement
- Define flags in `flags/registry.yaml` before referencing
- Add PII fields to `specs/privacy.yaml` with owner + retention

### 4. **Use bundles iteratively**

If the LLM's first attempt isn't quite right:
- Regenerate the bundle (it's fast)
- Provide the LLM's previous attempt as context
- Add specific constraints or examples

---

## Common Workflows

### Workflow 1: Add a new AC

```bash
# 1. Human adds AC to ledger
vim specs/spec_ledger.yaml
# Add: AC-125: "Customer receives email when refund is approved"

# 2. Generate bundle
cargo run -p xtask -- bundle implement_ac

# 3. Prompt LLM
"Implement AC-125. Show me:
1. New Gherkin scenario with @AC-125 tag
2. Step definitions in crates/acceptance/src/steps/refunds.rs
3. Core logic in crates/core/src/refunds.rs"

# 4. Apply changes, validate
cargo run -p xtask -- bdd
cargo run -p xtask -- ac-status  # Check AC-125 shows as passing
```

### Workflow 2: Debug a failing scenario

```bash
# 1. Scenario fails
cargo run -p xtask -- bdd
# Error: Expected 201, got 400

# 2. Generate debug bundle
cargo run -p xtask -- bundle debug_tests

# 3. Prompt LLM
"Scenario 'Create a refund' (@AC-123) fails with:
Expected 201, got 400 with body: {\"error\":\"Invalid amount\"}

Looking at the step definition and core logic, what's wrong?"

# 4. Apply fix, re-test
cargo run -p xtask -- bdd
```

### Workflow 3: Understand what an AC actually does

```bash
# Generate context
cargo run -p xtask -- bundle implement_ac

# Ask LLM:
"What does AC-123 actually require?
Show me:
1. The AC text from the ledger
2. The Gherkin scenario
3. The core code that implements it"
```

---

## Customizing Tasks

Edit `.llm/contextpack.yaml` to add new tasks:

```yaml
tasks:
  my_custom_task:
    max_bytes: 150000
    include:
      - specs/spec_ledger.yaml
      - flags/*.yaml
      - crates/core/src/flags.rs
    description: "Context for working with feature flags"
```

Then use it:
```bash
cargo run -p xtask -- bundle my_custom_task
```

---

## Excluding Files with .llmignore

The `.llm/.llmignore` file lets you exclude files from bundles using **standard gitignore syntax**.

### Location

Create or edit: `.llm/.llmignore`

### Syntax

`.llmignore` uses **full gitignore semantics** - the same patterns you use in `.gitignore` work here.

For complete syntax reference, see [gitignore documentation](https://git-scm.com/docs/gitignore).

### Common Patterns

**1. Wildcard patterns:**
```
# Ignore all log files
*.log

# Ignore test files
test_*.rs
*_test.go
```

**2. Directory patterns:**
```
# Ignore build directories
target/
dist/
node_modules/
```

**3. Path anchoring:**
```
# Only at root
/ROOT_FILE.txt

# Anywhere in tree
Cargo.lock
```

**4. Recursive wildcards:**
```
# All .draft files in docs subdirectories
docs/**/*.draft

# All temporary files anywhere
**/*.tmp
```

**5. Negation (whitelist):**
```
# Ignore all logs except error.log
*.log
!error.log
```

**6. Character classes:**
```
# Ignore test0.rs through test9.rs
test[0-9].rs

# Files starting with a, b, or c
[abc]*.txt
```

### Example .llmignore

```
# Build artifacts
target/
*.lock
*.tmp

# IDE and OS files
.idea/
.vscode/
.DS_Store
*.swp

# Test and development files
*_test.go
test_*.rs

# Logs and databases
*.log
*.db

# Environment files
.env*

# Documentation drafts
docs/**/*.draft
*.bak
```

### How It Works

1. Files matched by `include` patterns in contextpack.yaml
2. `.llmignore` patterns are applied to exclude files (using gitignore semantics)
3. Remaining files are added to bundle (up to `max_bytes`)

### Tips

**Balance include patterns with .llmignore:**

```yaml
# Good: Use include for broad categories, .llmignore for exclusions
include:
  - crates/**/*.rs

# .llmignore:
# test_*.rs
# *_test.rs
# target/
```

**Be specific when possible:**

```yaml
# Even better: Be specific in contextpack.yaml
include:
  - crates/core/src/**/*.rs
  - crates/app-http/src/**/*.rs
  # More specific = less filtering needed
```

---

## Troubleshooting

**Bundle is empty or missing files:**
- Check that `include` globs match files tracked by git
- Use `git ls-files <pattern>` to test your globs
- Check `.llm/.llmignore` isn't excluding files you want

**Bundle exceeds max_bytes:**
- Reduce `max_bytes` limit
- Make `include` patterns more specific
- Split into multiple smaller tasks

**LLM gives bad suggestions:**
- Make your prompt more specific
- Include examples of desired output format
- Add explicit constraints ("Do not invent new AC IDs")

---

## Summary

**Do:**
- ✅ Use bundles for focused, well-defined tasks
- ✅ Verify all LLM output before applying
- ✅ Keep prompts specific and constrained
- ✅ Add governance artifacts (ACs, flags, PII) to specs *first*

**Don't:**
- ❌ Let LLMs invent AC/FT/flag IDs
- ❌ Accept policy changes without review
- ❌ Use bundles for architecture-level changes
- ❌ Skip running `xtask check` and `xtask bdd` after changes

LLM bundles are a **tool to amplify your productivity**, not a replacement for understanding your system. Use them to accelerate implementation of decisions you've already made.
