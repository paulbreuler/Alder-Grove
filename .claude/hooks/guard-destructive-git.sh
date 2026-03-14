#!/usr/bin/env bash
# Guard: Block destructive git commands unless user explicitly approves.
# Exit 0 = pass, Exit 2 = block (ask user).

set -euo pipefail

INPUT="${1:-}"

# Destructive patterns to block
DESTRUCTIVE_PATTERNS=(
  "push.*--force"
  "push.*-f"
  "reset.*--hard"
  "checkout \."
  "restore \."
  "clean.*-f"
  "clean.*-fd"
  "branch.*-D"
  "stash drop"
  "stash clear"
)

for pattern in "${DESTRUCTIVE_PATTERNS[@]}"; do
  if echo "$INPUT" | grep -qE "$pattern" 2>/dev/null; then
    echo "BLOCKED: Destructive git command detected: $pattern"
    echo "This operation could cause data loss. Ask the user to confirm."
    exit 2
  fi
done

exit 0
