package main

# Scope Guard Policy - Advisory by default, hard-fail on danger zones
#
# Philosophy:
#   - "Scope enforcement" means bounded intent + reviewability, not small PRs
#   - Large PRs are fine; unclear intent is the risk
#   - Classifications describe what changed, not how much

# =============================================================================
# PR Types (classification, not size limit)
# =============================================================================
#
# mechanical    - renames, formatting, refactors, dependency bumps, code motion
# behavior      - anything that changes runtime outputs/contracts
# governance    - specs/ACs/policy/doc contracts
# release       - tagging, baseline changes
# docs          - documentation-only changes
# =============================================================================

# Danger zones - these require explicit scope declaration
danger_zone_paths := [
    "specs/spec_ledger.yaml",
    "specs/devex_flows.yaml",
    "specs/service_metadata.yaml",
    "policy/",
    ".github/workflows/",
    "specs/openapi.yaml",
    "docs/feature_status.md",
    "CLAUDE.md",
    "CHANGELOG.md"
]

# Check if file is in a danger zone (prefix match or exact match)
is_danger_zone(f) {
    danger_zone_paths[_] == f
}

is_danger_zone(f) {
    startswith(f, danger_zone_paths[_])
}

# =============================================================================
# Advisory warnings (never block, just inform)
# =============================================================================

# Warn when scope block is missing entirely
warn[msg] {
    not input.scope_declared
    msg := "PR body missing ## Scope block. Consider adding for reviewer clarity."
}

# Warn when detected scope doesn't match declared type
warn[msg] {
    input.scope_declared
    input.declared_type != "any"
    detected := categorize_changes(input.changed_files)
    input.declared_type != detected.primary
    msg := sprintf(
        "Declared type '%s' may not match changes (detected: %s). Review scope accuracy.",
        [input.declared_type, detected.primary]
    )
}

# Warn on large PRs without scope block (advisory, not blocking)
warn[msg] {
    not input.scope_declared
    count(input.changed_files) > 50
    msg := sprintf(
        "Large PR (%d files) without scope declaration. Consider adding ## Scope for reviewers.",
        [count(input.changed_files)]
    )
}

# =============================================================================
# Hard failures (danger zone + missing declaration)
# =============================================================================

# Fail when danger zone touched without scope declaration
deny[msg] {
    changed_file := input.changed_files[_]
    is_danger_zone(changed_file)
    not input.scope_declared
    msg := sprintf(
        "Danger zone file '%s' modified without ## Scope declaration. Add scope block to PR body.",
        [changed_file]
    )
}

# Fail when danger zone touched with mismatched type (governance files need governance type)
deny[msg] {
    input.scope_declared
    gov_file := input.changed_files[_]
    is_danger_zone(gov_file)
    is_governance_file(gov_file)
    not type_allows_governance(input.declared_type)
    msg := sprintf(
        "File '%s' is a governance artifact but PR type is '%s'. Use type: governance or mechanical.",
        [gov_file, input.declared_type]
    )
}

# =============================================================================
# Helpers
# =============================================================================

is_governance_file(f) {
    startswith(f, "specs/")
}

is_governance_file(f) {
    startswith(f, "policy/")
}

is_governance_file(f) {
    f == "CLAUDE.md"
}

type_allows_governance(t) {
    t == "governance"
}

type_allows_governance(t) {
    t == "mechanical"
}

type_allows_governance(t) {
    t == "release"
}

# Categorize a file into a bucket
file_bucket(path) = "ci" {
    startswith(path, ".github/workflows/")
}

file_bucket(path) = "specs" {
    startswith(path, "specs/")
}

file_bucket(path) = "policy" {
    startswith(path, "policy/")
}

file_bucket(path) = "docs" {
    startswith(path, "docs/")
}

file_bucket(path) = "docs" {
    endswith(path, ".md")
    not startswith(path, "docs/")
}

file_bucket(path) = "runtime" {
    startswith(path, "crates/")
}

file_bucket(path) = "runtime" {
    endswith(path, ".rs")
    not startswith(path, "crates/")
}

file_bucket(path) = "config" {
    startswith(path, "config/")
}

file_bucket(path) = "config" {
    startswith(path, "flags/")
}

file_bucket(path) = "examples" {
    startswith(path, "examples/")
}

file_bucket(path) = "other" {
    not startswith(path, ".github/workflows/")
    not startswith(path, "specs/")
    not startswith(path, "policy/")
    not startswith(path, "docs/")
    not endswith(path, ".md")
    not startswith(path, "crates/")
    not endswith(path, ".rs")
    not startswith(path, "config/")
    not startswith(path, "flags/")
    not startswith(path, "examples/")
}

# Categorize all changes to determine primary type
categorize_changes(files) = result {
    buckets := {bucket | some idx; files[idx]; bucket := file_bucket(files[idx])}

    result := {
        "buckets": buckets,
        "primary": determine_primary(buckets),
        "touches_danger": touches_any_danger_zone(files)
    }
}

determine_primary(buckets) = "governance" {
    buckets["specs"]
}

determine_primary(buckets) = "governance" {
    buckets["policy"]
    not buckets["specs"]
}

determine_primary(buckets) = "behavior" {
    buckets["runtime"]
    not buckets["specs"]
    not buckets["policy"]
}

determine_primary(buckets) = "docs" {
    buckets["docs"]
    not buckets["runtime"]
    not buckets["specs"]
    not buckets["policy"]
}

determine_primary(buckets) = "mechanical" {
    not buckets["runtime"]
    not buckets["specs"]
    not buckets["policy"]
    not buckets["docs"]
}

touches_any_danger_zone(files) {
    is_danger_zone(files[_])
}
