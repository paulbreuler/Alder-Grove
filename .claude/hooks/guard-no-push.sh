#!/usr/bin/env bash
# Guard: Block git push unless user explicitly approves.
# Exit 0 = pass, Exit 2 = block (ask user).

set -euo pipefail

echo "BLOCKED: git push requires explicit user approval."
echo "Agents must not push code without review."
exit 2
