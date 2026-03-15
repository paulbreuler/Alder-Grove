#!/usr/bin/env bash
# Hook: auto-format staged Rust files before commit.
# Exit 0 = allow (always), formats staged .rs files.

set -euo pipefail

# Get staged .rs files (excluding deleted files)
STAGED_RS=$(git diff --cached --name-only --diff-filter=d -- '*.rs' 2>/dev/null || true)

if [[ -z "$STAGED_RS" ]]; then
  exit 0
fi

if ! command -v rustfmt &>/dev/null; then
  # Formatter not installed — skip silently
  exit 0
fi

# Count files for the message
FILE_COUNT=$(echo "$STAGED_RS" | wc -l | tr -d ' ')

# Format only the staged .rs files
if echo "$STAGED_RS" | xargs rustfmt --edition 2024 2>/dev/null; then
  # Re-stage the modified .rs files
  echo "$STAGED_RS" | while IFS= read -r file; do
    if [[ -f "$file" ]]; then
      git add "$file"
    fi
  done
  echo "Auto-formatted $FILE_COUNT Rust file(s)"
else
  echo "rustfmt skipped — formatter returned an error"
fi

exit 0
