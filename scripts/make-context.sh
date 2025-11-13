#!/usr/bin/env bash
# LLM Context Bundler
# Reads .llm/contextpack.yaml and generates context bundles for LLM consumption

set -euo pipefail

TASK="${1:-}"
WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTEXTPACK="${WORKSPACE_ROOT}/.llm/contextpack.yaml"
LLMIGNORE="${WORKSPACE_ROOT}/.llm/.llmignore"
BUNDLE_DIR="${WORKSPACE_ROOT}/.llm/bundle"
GIT_SHA="$(git -C "${WORKSPACE_ROOT}" rev-parse --short HEAD 2>/dev/null || echo 'unknown')"

# Validate task argument
if [ -z "$TASK" ]; then
    echo "Error: Task name required" >&2
    echo "Usage: $0 <task>" >&2
    echo "" >&2
    echo "Available tasks:" >&2
    yq -r '.tasks | keys[]' "$CONTEXTPACK" 2>/dev/null || echo "  (none defined)" >&2
    exit 1
fi

# Check if task exists
if ! yq -e ".tasks.$TASK" "$CONTEXTPACK" >/dev/null 2>&1; then
    echo "Error: Task '$TASK' not found in $CONTEXTPACK" >&2
    echo "" >&2
    echo "Available tasks:" >&2
    yq -r '.tasks | keys[]' "$CONTEXTPACK" 2>/dev/null || echo "  (none)" >&2
    exit 1
fi

# Read task configuration
MAX_BYTES=$(yq -r ".tasks.$TASK.max_bytes" "$CONTEXTPACK")
DESCRIPTION=$(yq -r ".tasks.$TASK.description // \"\"" "$CONTEXTPACK")
INCLUDES=$(yq -r ".tasks.$TASK.include[]" "$CONTEXTPACK")

echo "Building context bundle: $TASK"
echo "  Max size: $MAX_BYTES bytes"
[ -n "$DESCRIPTION" ] && echo "  Description: $DESCRIPTION"

# Create bundle directory
mkdir -p "$BUNDLE_DIR"
BUNDLE_FILE="${BUNDLE_DIR}/${TASK}.md"

# Write header
cat > "$BUNDLE_FILE" <<EOF
# Context Bundle: $TASK

Generated from commit: $GIT_SHA

## Purpose

$DESCRIPTION

## Instructions

- AC IDs (AC-####) are the atomic units of behavior
- Specs live under \`specs/\`, features under \`features/\`
- Do not invent new IDs; use the ones present here
- All paths are relative to the repository root

---

EOF

# Collect files
TOTAL_BYTES=$(stat -c%s "$BUNDLE_FILE")
FILE_COUNT=0

while IFS= read -r pattern; do
    # Skip empty patterns
    [ -z "$pattern" ] && continue

    # Find files matching pattern (tracked by git)
    while IFS= read -r file; do
        # Skip if file doesn't exist
        [ ! -f "$WORKSPACE_ROOT/$file" ] && continue

        # Check against .llmignore if it exists
        if [ -f "$LLMIGNORE" ]; then
            if grep -qxF "$file" "$LLMIGNORE" 2>/dev/null; then
                continue
            fi
        fi

        # Check size limit
        FILE_SIZE=$(stat -c%s "$WORKSPACE_ROOT/$file")
        NEW_TOTAL=$((TOTAL_BYTES + FILE_SIZE + 100))  # +100 for headers

        if [ $NEW_TOTAL -gt $MAX_BYTES ]; then
            echo "  Warning: Size limit reached, skipping remaining files" >&2
            break 2
        fi

        # Add file to bundle
        {
            echo ""
            echo "---"
            echo "# FILE: $file"
            echo ""
            cat "$WORKSPACE_ROOT/$file"
            echo ""
        } >> "$BUNDLE_FILE"

        TOTAL_BYTES=$NEW_TOTAL
        FILE_COUNT=$((FILE_COUNT + 1))

    done < <(cd "$WORKSPACE_ROOT" && git ls-files "$pattern" 2>/dev/null || true)
done <<< "$INCLUDES"

# Final stats
FINAL_SIZE=$(stat -c%s "$BUNDLE_FILE")

echo "  Files included: $FILE_COUNT"
echo "  Bundle size: $FINAL_SIZE bytes"
echo ""
echo "Bundle written to: $BUNDLE_FILE"
