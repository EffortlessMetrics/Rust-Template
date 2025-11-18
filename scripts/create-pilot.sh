#!/usr/bin/env bash
# Create a new greenfield pilot project from the Rust Template v2.3.0
#
# Usage:
#   ./scripts/create-pilot.sh <project-name> [target-directory]
#
# Example:
#   ./scripts/create-pilot.sh task-api-service ~/pilots/
#   ./scripts/create-pilot.sh my-service .

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMPLATE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

error() {
    echo -e "${RED}ERROR: $1${NC}" >&2
    exit 1
}

info() {
    echo -e "${BLUE}INFO: $1${NC}"
}

success() {
    echo -e "${GREEN}SUCCESS: $1${NC}"
}

warn() {
    echo -e "${YELLOW}WARN: $1${NC}"
}

usage() {
    cat <<EOF
Create a new greenfield pilot project from Rust Template v2.3.0

USAGE:
    $0 <project-name> [target-directory]

ARGUMENTS:
    <project-name>        Name of the new pilot project (e.g., 'task-api-service')
    [target-directory]    Optional directory to create project in (default: current directory)

EXAMPLE:
    $0 my-pilot-service ~/projects/
    $0 task-api .

The script will:
  1. Copy the template to a new directory
  2. Initialize a fresh git repository
  3. Set up the FRICTION_LOG.md from template
  4. Run initial selftest to verify setup
  5. Provide next steps for Day 1 development

EOF
    exit 1
}

# Parse arguments
if [ $# -lt 1 ]; then
    usage
fi

PROJECT_NAME="$1"
TARGET_DIR="${2:-.}"

# Validate project name (basic check for valid directory name)
if [[ ! "$PROJECT_NAME" =~ ^[a-zA-Z0-9_-]+$ ]]; then
    error "Project name must contain only letters, numbers, hyphens, and underscores"
fi

# Resolve target directory
TARGET_DIR="$(cd "$TARGET_DIR" && pwd)"
PROJECT_PATH="$TARGET_DIR/$PROJECT_NAME"

info "Creating pilot project: $PROJECT_NAME"
info "Target location: $PROJECT_PATH"

# Check if target already exists
if [ -d "$PROJECT_PATH" ]; then
    error "Directory already exists: $PROJECT_PATH"
fi

# Check we're running from template root
if [ ! -f "$TEMPLATE_ROOT/Cargo.toml" ]; then
    error "Must run from Rust Template root directory"
fi

# Check for v2.3.0 tag
CURRENT_TAG=$(cd "$TEMPLATE_ROOT" && git describe --tags --exact-match 2>/dev/null || echo "")
if [ "$CURRENT_TAG" != "v2.3.0" ]; then
    warn "Not on v2.3.0 tag (current: ${CURRENT_TAG:-untagged})"
    warn "Consider checking out v2.3.0: git checkout v2.3.0"
    read -p "Continue anyway? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 0
    fi
fi

# Create pilot project
info "Copying template to $PROJECT_PATH..."
mkdir -p "$PROJECT_PATH"
rsync -a \
    --exclude='.git' \
    --exclude='target' \
    --exclude='.llm/bundle' \
    --exclude='*.log' \
    --exclude='.DS_Store' \
    "$TEMPLATE_ROOT/" "$PROJECT_PATH/"

# Initialize fresh git repo
info "Initializing fresh git repository..."
cd "$PROJECT_PATH"
git init
git add .
git commit -m "Initial commit from Rust-Template v2.3.0" --quiet

# Set up friction log
info "Setting up friction log..."
if [ -f "docs/templates/FRICTION_LOG.md" ]; then
    cp docs/templates/FRICTION_LOG.md FRICTION_LOG.md
    # Fill in template variables
    sed -i "s/\[Project Name\]/$PROJECT_NAME/g" FRICTION_LOG.md
    sed -i "s/YYYY-MM-DD/$(date +%Y-%m-%d)/g" FRICTION_LOG.md
    sed -i "s/\[Team\/Developer Name\]/$(git config user.name || echo 'Unknown')/g" FRICTION_LOG.md
    sed -i "s/\[Greenfield \/ Brownfield\]/Greenfield/g" FRICTION_LOG.md
    git add FRICTION_LOG.md
    git commit -m "docs: set up friction log for pilot" --quiet
fi

# Run initial selftest
info "Running initial selftest..."
echo
if cargo run -p xtask -- selftest; then
    success "Selftest passed! ✓"
else
    warn "Selftest failed - check output above"
    warn "This may indicate environment issues or template problems"
fi

# Print next steps
cat <<EOF

${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}
${GREEN}✓${NC} Pilot project created: ${BLUE}$PROJECT_NAME${NC}
${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}

Location: ${BLUE}$PROJECT_PATH${NC}

${YELLOW}NEXT STEPS:${NC}

${BLUE}1. Navigate to project${NC}
   cd $PROJECT_PATH

${BLUE}2. Define your first feature${NC}
   # Edit specs/spec_ledger.yaml to add acceptance criteria
   # Example: AC-TSK-001 "As a user I can create a task"

${BLUE}3. Add BDD scenario${NC}
   # Create specs/features/tasks.feature with Gherkin scenarios

${BLUE}4. Generate context bundle${NC}
   cargo run -p xtask -- bundle implement_ac

${BLUE}5. Feed to LLM${NC}
   # Open .llm/bundle/implement_ac.md in your LLM tool
   # Apply suggested changes

${BLUE}6. Validate${NC}
   cargo run -p xtask -- selftest

${BLUE}7. Record friction${NC}
   # Document pain points in FRICTION_LOG.md
   # Every rough edge, missing doc, confusing behavior

${YELLOW}FRICTION LOG:${NC}
   Track all friction in: ${BLUE}FRICTION_LOG.md${NC}

   ${GREEN}After 1-2 weeks of development:${NC}
   - Review friction log entries
   - Classify: 🔴 Blockers / 🟡 Annoyances / 🟢 Nice-to-have
   - Determine if template needs v2.3.1 patch or v2.4.0 features

${YELLOW}RESOURCES:${NC}
   - Release Playbook: docs/RELEASE_PLAYBOOK.md
   - How-to Guides: docs/how-to/
   - Template README: README.md

${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}

${GREEN}Happy building!${NC} Remember: you're validating the ${BLUE}template${NC}, not perfecting it.

EOF
