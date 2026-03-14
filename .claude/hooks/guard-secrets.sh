#!/usr/bin/env bash
# Guard: Scan staged diffs for credential patterns before commit.
# Exit 0 = pass, Exit 2 = block (ask user).

set -euo pipefail

STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM 2>/dev/null || true)
if [ -z "$STAGED_FILES" ]; then
  exit 0
fi

STAGED_DIFF=$(git diff --cached -U0 2>/dev/null || true)
if [ -z "$STAGED_DIFF" ]; then
  exit 0
fi

# Generic credential patterns (case-insensitive, 8+ char values)
SECRET_PATTERNS=(
  "api_key"
  "secret_key"
  "password"
  "token"
  "credential"
  "private_key"
  "client_secret"
  "connection_string"
  "database_url"
)

# Project-specific patterns
EXPLICIT_PATTERNS=(
  "CLERK_SECRET_KEY"
  "CLERK_PUBLISHABLE_KEY"
  "DATABASE_URL"
  "VITE_CLERK_PUBLISHABLE_KEY"
  "POSTGRES_PASSWORD"
  "JWT_SECRET"
  "WEBHOOK_SECRET"
)

FOUND=""

for pattern in "${SECRET_PATTERNS[@]}"; do
  MATCHES=$(echo "$STAGED_DIFF" | grep -inE "^\+.*${pattern}\s*[=:]\s*.{8,}" 2>/dev/null || true)
  if [ -n "$MATCHES" ]; then
    FOUND="${FOUND}\n[generic] ${pattern}:\n${MATCHES}\n"
  fi
done

for pattern in "${EXPLICIT_PATTERNS[@]}"; do
  MATCHES=$(echo "$STAGED_DIFF" | grep -n "${pattern}" 2>/dev/null || true)
  if [ -n "$MATCHES" ]; then
    FOUND="${FOUND}\n[explicit] ${pattern}:\n${MATCHES}\n"
  fi
done

if [ -n "$FOUND" ]; then
  echo "BLOCKED: Possible secrets detected in staged changes."
  echo -e "$FOUND"
  echo ""
  echo "Review the matches above. If these are false positives (e.g. env.example), ask the user to confirm."
  exit 2
fi

exit 0
