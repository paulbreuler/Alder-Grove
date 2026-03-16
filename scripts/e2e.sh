#!/usr/bin/env bash
set -euo pipefail

# E2E test runner for grove-api using Hurl.
# Requires: hurl, cargo, docker compose (PostgreSQL running)
#
# Usage:
#   ./scripts/e2e.sh              # Run all e2e tests
#   ./scripts/e2e.sh health.hurl  # Run a single test file

export API_PORT="${API_PORT:-3001}"
export DATABASE_URL="${DATABASE_URL:-postgres://grove:grove_dev@localhost:5432/grove_dev}"
API_URL="http://localhost:${API_PORT}"
E2E_DIR="$(cd "$(dirname "$0")/../tests/e2e" && pwd)"
API_PID=""

cleanup() {
    if [ -n "$API_PID" ] && kill -0 "$API_PID" 2>/dev/null; then
        echo "Stopping grove-api (PID $API_PID)..."
        kill "$API_PID" 2>/dev/null || true
        wait "$API_PID" 2>/dev/null || true
    fi
}
trap cleanup EXIT

# Check prerequisites
command -v hurl >/dev/null 2>&1 || { echo "Error: hurl not found. Install with: brew install hurl"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "Error: cargo not found."; exit 1; }

# Build the API
echo "Building grove-api..."
cargo build -p grove-api --quiet

# Start the API server in the background
echo "Starting grove-api on port ${API_PORT}..."
cargo run -p grove-api --quiet &
API_PID=$!

# Wait for the API to be ready
echo "Waiting for API..."
for i in $(seq 1 30); do
    if curl -sf "${API_URL}/health" >/dev/null 2>&1; then
        echo "API ready."
        break
    fi
    if ! kill -0 "$API_PID" 2>/dev/null; then
        echo "Error: grove-api exited unexpectedly."
        exit 1
    fi
    sleep 0.5
done

# Verify API is actually responding
if ! curl -sf "${API_URL}/health" >/dev/null 2>&1; then
    echo "Error: API did not become ready within 15 seconds."
    exit 1
fi

# Run Hurl tests
echo ""
if [ $# -gt 0 ]; then
    # Run specific test files
    for file in "$@"; do
        echo "Running: ${file}"
        hurl --test "${E2E_DIR}/${file}"
    done
else
    # Run all .hurl files
    echo "Running all e2e tests in ${E2E_DIR}/"
    hurl --test "${E2E_DIR}"/*.hurl
fi

echo ""
echo "All e2e tests passed."
