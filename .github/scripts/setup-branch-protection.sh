#!/usr/bin/env bash
# Setup GitHub branch protection for the Rust-as-Spec template
#
# This script configures branch protection rules on the 'main' branch to enforce:
# - Required pull request reviews
# - Required status checks (tier1-selftest, ci-security, ci-docs, ci-policy-verify)
# - No direct pushes to main
# - No force pushes or branch deletion
# - Enforcement for administrators
#
# Prerequisites:
# - gh CLI installed and authenticated (gh auth login)
# - Repository admin permissions
#
# Usage:
#   .github/scripts/setup-branch-protection.sh [OWNER/REPO]
#
# If OWNER/REPO is not provided, it will be detected from git remote.

set -euo pipefail

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print functions
print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Detect repository from git remote or use argument
detect_repo() {
    if [ $# -eq 1 ]; then
        echo "$1"
        return
    fi

    if ! command -v git &> /dev/null; then
        print_error "git is not installed"
        exit 1
    fi

    local remote_url
    remote_url=$(git remote get-url origin 2>/dev/null || echo "")

    if [ -z "$remote_url" ]; then
        print_error "No git remote 'origin' found and no OWNER/REPO argument provided"
        echo "Usage: $0 [OWNER/REPO]"
        exit 1
    fi

    # Extract OWNER/REPO from git@github.com:OWNER/REPO.git or https://github.com/OWNER/REPO.git
    local repo_path
    repo_path=$(echo "$remote_url" | sed -E 's#^git@github\.com:##; s#^https://github\.com/##; s#\.git$##')

    echo "$repo_path"
}

# Check if gh CLI is installed and authenticated
check_gh_cli() {
    if ! command -v gh &> /dev/null; then
        print_error "gh CLI is not installed"
        echo ""
        echo "Install gh CLI:"
        echo "  - Ubuntu/Debian: sudo apt install gh"
        echo "  - macOS: brew install gh"
        echo "  - Other: https://github.com/cli/cli#installation"
        echo ""
        exit 1
    fi

    if ! gh auth status &> /dev/null; then
        print_error "gh CLI is not authenticated"
        echo ""
        echo "Authenticate with: gh auth login"
        echo ""
        exit 1
    fi

    print_success "gh CLI is installed and authenticated"
}

# Check if the repository exists and is accessible
check_repo_access() {
    local repo=$1

    print_info "Checking access to repository: $repo"

    if ! gh api "/repos/$repo" &> /dev/null; then
        print_error "Cannot access repository: $repo"
        echo ""
        echo "Possible causes:"
        echo "  - Repository does not exist"
        echo "  - You don't have access to the repository"
        echo "  - Your token lacks 'repo' scope"
        echo ""
        echo "Try: gh auth refresh -s repo"
        exit 1
    fi

    print_success "Repository is accessible"
}

# Check if main branch exists
check_main_branch() {
    local repo=$1

    print_info "Checking if 'main' branch exists"

    if ! gh api "/repos/$repo/branches/main" &> /dev/null; then
        print_error "Branch 'main' does not exist in $repo"
        echo ""
        echo "Create the main branch first by pushing at least one commit."
        exit 1
    fi

    print_success "Branch 'main' exists"
}

# Configure branch protection
configure_protection() {
    local repo=$1

    print_info "Configuring branch protection for 'main'..."

    # Create the protection payload
    local payload
    payload=$(cat <<'EOF'
{
  "required_status_checks": {
    "strict": true,
    "contexts": [
      "tier1-selftest",
      "ci-security",
      "ci-docs",
      "ci-policy-verify"
    ]
  },
  "enforce_admins": true,
  "required_pull_request_reviews": {
    "dismissal_restrictions": {},
    "dismiss_stale_reviews": true,
    "require_code_owner_reviews": false,
    "required_approving_review_count": 1,
    "require_last_push_approval": false
  },
  "restrictions": null,
  "required_conversation_resolution": true,
  "allow_force_pushes": false,
  "allow_deletions": false,
  "block_creations": false,
  "required_linear_history": false,
  "lock_branch": false
}
EOF
)

    # Apply the protection
    if gh api \
        --method PUT \
        -H "Accept: application/vnd.github+json" \
        "/repos/$repo/branches/main/protection" \
        --input - <<< "$payload" > /dev/null 2>&1; then
        print_success "Branch protection configured successfully"
    else
        print_warning "Failed to configure branch protection (see below for details)"
        echo ""

        # Try again with verbose output for troubleshooting
        print_info "Detailed error output:"
        gh api \
            --method PUT \
            -H "Accept: application/vnd.github+json" \
            "/repos/$repo/branches/main/protection" \
            --input - <<< "$payload" || true

        echo ""
        print_warning "Common issues:"
        echo "  - Required status checks not found (they must run at least once first)"
        echo "  - Insufficient permissions (token needs 'repo' scope)"
        echo "  - Repository is not owned by you or your organization"
        echo ""
        exit 1
    fi
}

# Verify protection is active
verify_protection() {
    local repo=$1

    print_info "Verifying branch protection settings..."

    if ! gh api "/repos/$repo/branches/main/protection" > /tmp/protection.json 2>/dev/null; then
        print_error "Failed to verify branch protection"
        exit 1
    fi

    # Check key settings
    local enforce_admins
    enforce_admins=$(jq -r '.enforce_admins.enabled' /tmp/protection.json 2>/dev/null || echo "false")

    local required_reviews
    required_reviews=$(jq -r '.required_pull_request_reviews.required_approving_review_count' /tmp/protection.json 2>/dev/null || echo "0")

    local required_checks
    required_checks=$(jq -r '.required_status_checks.contexts | length' /tmp/protection.json 2>/dev/null || echo "0")

    echo ""
    print_info "Branch Protection Summary:"
    echo "  - Enforce for admins: $enforce_admins"
    echo "  - Required approvals: $required_reviews"
    echo "  - Required status checks: $required_checks"
    echo ""

    if [ "$enforce_admins" = "true" ] && [ "$required_reviews" -ge 1 ] && [ "$required_checks" -ge 4 ]; then
        print_success "Branch protection is properly configured"
    else
        print_warning "Branch protection is active but may not match expected settings"
    fi

    rm -f /tmp/protection.json
}

# Main function
main() {
    echo ""
    echo "=============================================="
    echo "  GitHub Branch Protection Setup"
    echo "  Rust-as-Spec Template"
    echo "=============================================="
    echo ""

    # Check prerequisites
    check_gh_cli

    # Detect or parse repository
    local repo
    repo=$(detect_repo "$@")
    print_info "Target repository: $repo"
    echo ""

    # Validate access
    check_repo_access "$repo"
    check_main_branch "$repo"
    echo ""

    # Configure protection
    configure_protection "$repo"
    echo ""

    # Verify
    verify_protection "$repo"
    echo ""

    # Done
    print_success "Setup complete!"
    echo ""
    echo "Next steps:"
    echo "  1. Test direct push (should be blocked):"
    echo "     git push origin main"
    echo ""
    echo "  2. Create a test PR:"
    echo "     git checkout -b test-branch"
    echo "     git push origin test-branch"
    echo "     gh pr create --title 'Test PR' --body 'Testing branch protection'"
    echo ""
    echo "  3. View protection settings:"
    echo "     gh api /repos/$repo/branches/main/protection | jq"
    echo ""
    echo "For more details, see: docs/how-to/setup-branch-protection.md"
    echo ""
}

# Run main function with all arguments
main "$@"
