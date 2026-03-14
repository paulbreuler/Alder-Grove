#!/usr/bin/env bash
# Hook: Auto-format staged markdown files before commit.
# Requires markdownlint-cli2 to be installed.
# Exit 0 = pass (always — formatting is best-effort).

set -euo pipefail

STAGED_MD=$(git diff --cached --name-only --diff-filter=ACM -- '*.md' 2>/dev/null || true)
if [ -z "$STAGED_MD" ]; then
  exit 0
fi

if ! command -v markdownlint-cli2 &>/dev/null; then
  # Linter not installed — skip silently
  exit 0
fi

# Format staged markdown files
echo "$STAGED_MD" | xargs markdownlint-cli2 --fix 2>/dev/null || true

# Re-stage formatted files
echo "$STAGED_MD" | xargs git add 2>/dev/null || true

exit 0
