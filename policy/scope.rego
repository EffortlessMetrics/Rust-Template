package main

# Scope Guard Policy
# Validates that PR changes respect module boundaries and scope constraints

# Scope definitions based on directory structure
deny[msg] {
    some file
    file := input.changed_files[_]

    # Check for cross-scope violations
    # Example: Changing domain code in a platform-only PR
    is_cross_scope_violation(file, input.pr_scope)

    msg := sprintf(
        "File '%s' is outside scope '%s'. PR scope should be limited to relevant directories.",
        [file, input.pr_scope]
    )
}

# Detect cross-scope violations based on file path
is_cross_scope_violation(file, pr_scope) {
    # If PR scope is specified, validate files are within scope
    pr_scope != "any"

    # Map files to scope categories
    file_scope := categorize_file(file)

    # Check if file is outside declared PR scope
    not file_in_scope(file_scope, pr_scope)
}

# Categorize a file path into a scope category
categorize_file(file) = "platform" {
    # Platform/infrastructure files
    startswith(file, ".github/")
    or
    startswith(file, "infra/")
    or
    startswith(file, "scripts/")
    or
    file == "flake.nix"
    or
    file == "Cargo.toml"
    or
    file == "Cargo.lock"
    or
    file == "deny.toml"
    or
    file == "clippy.toml"
}

categorize_file(file) = "platform" {
    # Template configuration files
    startswith(file, "specs/")
    or
    startswith(file, "policy/")
    or
    startswith(file, "flags/")
    or
    startswith(file, "config/")
    or
    startswith(file, ".devcontainer/")
    or
    startswith(file, ".vscode/")
    or
    startswith(file, ".claude/")
}

categorize_file(file) = "docs" {
    # Documentation files
    startswith(file, "docs/")
    or
    endswith(file, ".md")
    or
    endswith(file, ".txt")
}

categorize_file(file) = "tests" {
    # Test files
    contains(file, "/tests/")
    or
    contains(file, "/test_")
    or
    endswith(file, "_test.rs")
    or
    endswith(file, "_tests.rs")
    or
    endswith(file, ".feature")
    or
    startswith(file, "benches/")
}

categorize_file(file) = "domain" {
    # Domain/application code
    startswith(file, "crates/")
    or
    startswith(file, "src/")
    or
    startswith(file, "lib/")
    or
    startswith(file, "apps/")
}

categorize_file(file) = "examples" {
    # Example code
    startswith(file, "examples/")
}

categorize_file(file) = "other" {
    # Other files
    true
}

# Check if a file's scope is within the allowed PR scope
file_in_scope(file_scope, pr_scope) {
    # If PR scope is "any", all files are allowed
    pr_scope == "any"

    # Otherwise, check if file scope matches PR scope
    or
    file_scope == pr_scope

    # Allow some cross-scope combinations
    or
    allowed_cross_scope(file_scope, pr_scope)
}

# Define allowed cross-scope combinations
# These are cases where it's acceptable to touch multiple scopes
allowed_cross_scope(file_scope, pr_scope) {
    # Platform changes can touch docs (e.g., updating docs for platform changes)
    pr_scope == "platform"
    file_scope == "docs"

    # Domain changes can touch tests (e.g., adding tests for domain code)
    or
    pr_scope == "domain"
    file_scope == "tests"

    # Examples can touch tests
    or
    pr_scope == "examples"
    file_scope == "tests"

    # Docs changes can touch specs (docs and specs are closely related)
    or
    pr_scope == "docs"
    file_scope == "platform"
}
