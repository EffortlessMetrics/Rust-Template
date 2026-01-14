---
id: DESIGN-TPL-LLMIGNORE-001
doc_type: design_doc
title: ".llmignore Semantics Analysis and Recommendation"
author: platform-team
date: 2025-11-14
status: approved
stories: [US-TPL-PLT-001]
requirements: [REQ-TPL-BUNDLE-CONTRACT]
tags: [platform, bundle, agent]
acs: [AC-TPL-BUNDLE-MINIMAL-SCOPE]
adrs: []
---

# .llmignore Semantics Analysis and Recommendation

**Status:** Design Decision
**Date:** 2025-11-14
**Context:** Evaluating whether to adopt full gitignore semantics or document current minimal implementation

---

## Executive Summary

**Recommendation: Option A - Adopt Full Gitignore Semantics using the `ignore` crate**

The current `.llmignore` implementation is minimal and has semantic inconsistencies. Adopting standard gitignore semantics via the widely-used `ignore` crate provides:
- Industry-standard behavior users already understand
- Battle-tested implementation (81M+ downloads)
- No breaking changes (no existing .llmignore files in repo)
- Minimal implementation effort (~50 lines of code change)
- Future-proof extensibility

---

## Current Implementation Analysis

### Code Location

`../../crates/xtask/src/commands/bundle.rs`

### Current Semantics (lines 178-201)

```rust
fn should_ignore(path: &str, ignore_patterns: &[String]) -> bool {
    use std::path::Path as StdPath;

    for pattern in ignore_patterns {
        // Directory pattern: "foo/" should match "foo/bar.txt" but not "foobar.txt"
        if pattern.ends_with('/') {
            let dir_pattern = pattern.trim_end_matches('/');
            // Match if path starts with "dir/" or is exactly "dir"
            if path.starts_with(&format!("{}/", dir_pattern)) || path == dir_pattern {
                return true;
            }
        } else {
            // Component match: "foo" matches "foo", "bar/foo", or "bar/foo.txt"
            // but not "foobar" or "foobar.txt"
            if StdPath::new(path)
                .components()
                .any(|c| c.as_os_str().to_string_lossy() == pattern.as_str())
            {
                return true;
            }
        }
    }
    false
}
```

### Pattern Loading (lines 158-176)

```rust
fn load_ignore_patterns(workspace_root: &Path) -> Result<Vec<String>> {
    let ignore_path = workspace_root.join(".llm/.llmignore");

    if !ignore_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&ignore_path)
        .with_context(|| format!("Failed to read .llmignore: {}", ignore_path.display()))?;

    let patterns: Vec<String> = content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.to_string())
        .collect();

    Ok(patterns)
}
```

### What Works

1. **Directory patterns** (`foo/`): Correctly matches directories
2. **Component matching** (`foo`): Matches path component exactly
3. **Comments** (`#`): Properly ignored
4. **Empty lines**: Properly ignored
5. **Whitespace trimming**: Leading/trailing whitespace removed

### What Doesn't Work (Compared to gitignore)

1. **No glob patterns**: `*.log`, `test_*.rs` don't work
2. **No wildcards**: `?`, `*`, `**` not supported
3. **No path anchoring**: Cannot distinguish `foo` (anywhere) from `/foo` (at root)
4. **No negation**: Cannot use `!important.log` to un-ignore
5. **No character classes**: `[abc]`, `[0-9]` not supported
6. **No brace expansion**: `{a,b,c}` not supported
7. **No escaping**: Cannot match literal `#` or `!`

### Semantic Bugs in Current Implementation

**Bug 1: Overly broad component matching**
- Pattern: `test`
- Matches: `test`, `src/test`, `src/test.rs` [OK]
- Also matches: `src/test_utils.rs` (WRONG - should only match exact component)
- Current code matches if ANY component equals pattern, but doesn't handle partial filename matches correctly

**Bug 2: Directory pattern doesn't match recursively as expected**
- Pattern: `target/`
- Matches: `target/debug/app` [OK]
- Matches: `target` [OK]
- Does NOT match: `nested/target/debug/app` (inconsistent - gitignore WOULD match this)

---

## Available Rust Crates

### Option 1: `ignore` crate (RECOMMENDED)

**Stats:**
- Downloads: 81,820,905
- Maintained by: BurntSushi (ripgrep author)
- Documentation: <https://docs.rs/ignore>
- Used by: ripgrep, fd, numerous other tools

**Features:**
- Full gitignore semantics
- Multiple ignore file support (.gitignore, .ignore, custom)
- Whitelist/negation patterns
- Fast globset-based matching
- Well-documented precedence rules
- Handles global gitignore files

**Pros:**
- Industry standard - battle-tested
- Complete gitignore spec compliance
- Extremely well maintained
- Excellent documentation
- Used by CLI tools developers know
- Built-in support for walking directories (not needed for us, but shows maturity)

**Cons:**
- Slightly heavier dependency (but already used by ripgrep, so stable)
- More features than we strictly need (but won't hurt)

### Option 2: `globset` crate

**Stats:**
- Downloads: 116,248,645
- Maintained by: BurntSushi
- Documentation: <https://docs.rs/globset>

**Features:**
- Fast glob matching
- Glob set matching (multiple patterns at once)
- Character classes, wildcards, `**`
- Case-insensitive option

**Pros:**
- Even more widely used than `ignore`
- Lightweight - just glob matching
- Very fast

**Cons:**
- NOT gitignore semantics - just glob patterns
- No directory trailing slash semantics
- No negation patterns
- No comment support
- We'd have to implement gitignore-specific logic ourselves

### Option 3: `gitignore` crate

**Stats:**
- Downloads: Much less popular
- Various unmaintained forks exist

**Verdict:** Not recommended - fragmentary ecosystem, prefer `ignore` crate instead.

---

## Option Evaluation

### Option A: Adopt Full Gitignore Semantics (RECOMMENDED)

#### Implementation Approach

Use the `ignore` crate's `gitignore::Gitignore` module for pattern matching.

**Changes Required:**

1. Add dependency to `crates/xtask/Cargo.toml`:

```toml
ignore = "0.4"
```

2. Replace `load_ignore_patterns` and `should_ignore` functions:

```rust
use ignore::gitignore::GitignoreBuilder;

fn load_llmignore(workspace_root: &Path) -> Result<ignore::gitignore::Gitignore> {
    let ignore_path = workspace_root.join(".llm/.llmignore");

    let mut builder = GitignoreBuilder::new(workspace_root);

    if ignore_path.exists() {
        builder.add(&ignore_path);
    }

    let gitignore = builder.build()
        .context("Failed to build .llmignore matcher")?;

    Ok(gitignore)
}

// In resolve_files(), change:
// let ignore_patterns = load_ignore_patterns(workspace_root)?;
// to:
// let llmignore = load_llmignore(workspace_root)?;

// And replace:
// if should_ignore(path_str, &ignore_patterns) {
// with:
// if llmignore.matched(path_str, false).is_ignore() {
```

**Estimated effort:** 30 minutes to implement + 30 minutes to test

#### Supported Patterns After Implementation

All standard gitignore patterns:

1. **Glob patterns**
   - `*.log` - all .log files
   - `test_*.rs` - all test files
   - `**/*.tmp` - all .tmp files in any directory

2. **Directory patterns**
   - `target/` - ignore target directory
   - `node_modules/` - ignore node_modules

3. **Path anchoring**
   - `/foo` - only foo at root
   - `foo` - foo anywhere
   - `**/foo` - foo anywhere (explicit)

4. **Negation (whitelist)**
   - `!important.log` - don't ignore this specific file
   - `*.log` + `!error.log` - ignore all logs except error.log

5. **Character classes**
   - `test[0-9].rs` - test0.rs through test9.rs
   - `[abc]*.txt` - files starting with a, b, or c

6. **Wildcards**
   - `?` - single character
   - `*` - any characters except /
   - `**` - any characters including /

#### Breaking Changes

**NONE** - No existing `.llmignore` files in the repository (verified via `glob **/.llmignore`).

#### Pros

- **Users already understand it** - gitignore semantics are universal
- **Feature-complete** - supports all common use cases
- **Battle-tested** - 81M downloads, used in critical tools
- **Maintainable** - we don't own complex matching logic
- **Extensible** - easy to add `.llmignore-local` or other features later
- **Correct** - no semantic bugs like current implementation
- **Fast** - optimized globset matching
- **Well-documented** - users can reference gitignore docs

#### Cons

- **Dependency added** - but it's stable and widely used
- **Slightly more complex** - but abstracted behind clean API
- **Different from current** - but current has no users and is buggy

---

### Option B: Document Current Minimal Semantics

Keep the existing implementation and document exactly what it supports.

#### Required Documentation

**Location:** Add to `docs/reference/llmignore-spec.md`

```markdown
# .llmignore Pattern Reference

The `.llmignore` file uses a MINIMAL subset of gitignore-style patterns.
It is NOT fully gitignore-compatible.

## Location

`.llm/.llmignore` (relative to workspace root)

## Supported Patterns

### 1. Exact Component Match

Pattern: `foo`

Matches:
- `foo` (file at root)
- `bar/foo` (file in subdirectory)
- `bar/foo/baz.txt` (directory anywhere)

Does NOT match:
- `foobar` (partial match)
- `foo.txt` (different filename)

### 2. Directory Pattern

Pattern: `foo/`

Matches:
- `foo/bar.txt` (file in directory)
- `foo/baz/qux.rs` (file in subdirectory)

Does NOT match:
- `foo` (the directory itself)
- `bar/foo/baz.txt` (directory not at root)

### 3. Comments

Lines starting with `#` are ignored.

## NOT Supported

- Glob patterns: `*.log`, `test_*.rs`
- Wildcards: `?`, `*`, `**`
- Path anchoring: `/foo` vs `foo`
- Negation: `!important.log`
- Character classes: `[0-9]`, `[abc]`
- Brace expansion: `{a,b,c}`
- Escaping: `\#`, `\*`

## Examples

### Example 1: Ignore all test directories

```

# .llm/.llmignore

tests/

```

Matches:
- `tests/test1.rs`
- `tests/integration/test2.rs`

Does NOT match:
- `src/tests/unit.rs` (not at root)
- `src/my_tests/foo.rs` (different name)

### Example 2: Ignore target and node_modules

```

# .llm/.llmignore

target/
node_modules/

```

### Example 3: Ignore files with specific names

```

# .llm/.llmignore

Cargo.lock
.DS_Store

```

Matches:
- `Cargo.lock` (at root)
- `foo/Cargo.lock` (in any directory)
- `.DS_Store` (anywhere)

## Limitations

If you need glob patterns or more advanced matching, consider:
1. Using more specific `include` patterns in `contextpack.yaml`
2. Requesting gitignore semantics support (file a feature request)
```

#### Known Issues to Document

The current implementation has semantic bugs:

1. **Component matching is inconsistent**
   - `test` will match `test.rs` as a component, which may be unexpected
   - Need to clarify whether `foo` matches `foo.txt` or only exactly `foo`

2. **Directory patterns only match at root**
   - `target/` won't match `nested/target/`
   - This differs from gitignore behavior

#### Pros

- **No code changes** - zero risk
- **No new dependencies** - keeps dependencies lean
- **Simple implementation** - easy to understand and debug
- **Fully controlled** - we own all the logic

#### Cons

- **Semantic bugs remain** - current implementation is incorrect
- **User confusion** - doesn't match gitignore expectations
- **Limited functionality** - can't handle common use cases like `*.log`
- **Documentation burden** - must explain non-standard behavior
- **Future maintenance** - we own bug fixes and feature requests
- **Poor UX** - users expect gitignore semantics
- **Technical debt** - keeping known-buggy code

---

## Recommendation: Option A

### Why Option A (Adopt gitignore semantics)?

1. **Current implementation has bugs** - we should fix them
2. **Users expect gitignore semantics** - `.llmignore` name implies it
3. **No breaking changes** - no existing .llmignore files to migrate
4. **Low implementation cost** - ~1 hour of work
5. **High value** - enables powerful filtering patterns
6. **Industry standard** - used by ripgrep, fd, and countless tools
7. **Reduces maintenance** - we don't own matching logic
8. **Better UX** - users can reference standard gitignore docs

### Why NOT Option B (Document current semantics)?

1. **Bugs remain** - component matching is incorrect
2. **Limited value** - can't support common patterns like `*.log`
3. **User confusion** - non-standard semantics are surprising
4. **Documentation overhead** - have to explain "why not gitignore?"
5. **Future regret** - will likely migrate to gitignore later anyway

---

## Implementation Plan (Option A)

### Phase 1: Update Dependencies (5 min)

**File:** `crates/xtask/Cargo.toml`

```toml
[dependencies]
# ... existing deps ...
ignore = "0.4"
```

### Phase 2: Replace Implementation (20 min)

**File:** `crates/xtask/src/commands/bundle.rs`

**Remove:**
- `load_ignore_patterns()` function (lines 158-176)
- `should_ignore()` function (lines 178-201)

**Add:**

```rust
use ignore::gitignore::GitignoreBuilder;

/// Load .llmignore file and build gitignore matcher
fn load_llmignore(workspace_root: &Path) -> Result<ignore::gitignore::Gitignore> {
    let ignore_path = workspace_root.join(".llm/.llmignore");

    let mut builder = GitignoreBuilder::new(workspace_root);

    if ignore_path.exists() {
        builder.add(&ignore_path);
    }

    builder.build()
        .context("Failed to build .llmignore matcher")
}
```

**Update `resolve_files()` function:**

```rust
// Replace line 115:
// let ignore_patterns = load_ignore_patterns(workspace_root)?;
let llmignore = load_llmignore(workspace_root)?;

// Replace lines 143-145:
// if should_ignore(path_str, &ignore_patterns) {
//     continue;
// }
let path = Path::new(path_str);
if llmignore.matched(path, false).is_ignore() {
    continue;
}
```

### Phase 3: Test (30 min)

Create test `.llmignore` file:

```bash
mkdir -p .llm
cat > .llm/.llmignore << 'EOF'
# Test patterns
*.log
target/
test_*.rs
!important.log
EOF
```

Test bundle generation:

```bash
cargo run -p xtask -- bundle implement_ac
```

Verify:
- `*.log` files are excluded
- `target/` directory is excluded
- `test_*.rs` files are excluded
- `important.log` is included (negation works)

### Phase 4: Documentation (15 min)

**Update:** `docs/how-to/use-llm-bundles.md`

Add section:

```markdown
## .llmignore Patterns

The `.llm/.llmignore` file uses standard gitignore syntax to exclude files from bundles.

**Common patterns:**

```

# Ignore all log files

*.log

# Ignore build artifacts

target/
dist/

# Ignore test files

test_*.rs
*_test.go

# But keep important files

!error.log

```

For full syntax reference, see [gitignore documentation](https://git-scm.com/docs/gitignore).
```

**Update:** `docs/reference/xtask-commands.md`

Update bundle command section to mention gitignore syntax.

### Phase 5: Validate (10 min)

Run full test suite:

```bash
cargo run -p xtask -- check
cargo run -p xtask -- selftest
```

---

## Migration Path (If Needed Later)

If we ever need to support existing non-standard `.llmignore` files:

1. Check for simple patterns (no globs, no special chars)
2. These will work identically with gitignore semantics
3. Print deprecation warning if non-gitignore patterns detected
4. Provide migration guide in docs

---

## Conclusion

**Adopt Option A: Full gitignore semantics via the `ignore` crate.**

This decision:
- Fixes existing bugs
- Provides industry-standard UX
- Enables powerful pattern matching
- Requires minimal implementation effort
- Has no breaking changes
- Reduces long-term maintenance burden
- Aligns with user expectations

**Next Steps:**
1. Implement the changes per the plan above
2. Test with various .llmignore patterns
3. Update documentation
4. Commit changes

---

## Appendix: Example .llmignore Files

### Example 1: Rust Project

```
# Build artifacts
target/
Cargo.lock

# IDE files
.idea/
.vscode/
*.swp

# OS files
.DS_Store
Thumbs.db

# Test artifacts
*.profraw
*.profdata

# Large generated files
docs/assets/*.pdf
```

### Example 2: Multi-language Project

```
# Build directories
target/
dist/
build/
out/

# Dependencies
node_modules/
vendor/

# Logs and databases
*.log
*.db
*.sqlite

# But keep schema
!schema.sql
!migrations/*.sql
```

### Example 3: Documentation-Heavy Project

```
# Large binary files
docs/images/**/*.psd
docs/videos/

# Draft content
docs/drafts/
**/*-DRAFT.md

# But keep published drafts
!docs/drafts/published/
```

---

## References

- `ignore` crate documentation: <https://docs.rs/ignore>
- gitignore specification: <https://git-scm.com/docs/gitignore>
- ripgrep (uses `ignore` crate): <https://github.com/BurntSushi/ripgrep>
- Current implementation: `crates/xtask/src/commands/bundle.rs:158-201`
