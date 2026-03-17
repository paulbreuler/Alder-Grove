# Agent CRUD -- full lifecycle
#
# Requires: grove-api running on localhost:3001 with PostgreSQL
# Depends on: workspace creation to provide org/workspace context

# --- Create workspace for agent tests ---
POST http://localhost:3001/orgs/e2e_agent_org/workspaces
Content-Type: application/json
{
    "name": "Agent E2E Workspace"
}
HTTP 201
[Captures]
ws_id: jsonpath "$.id"

# --- Create agent ---
POST http://localhost:3001/orgs/e2e_agent_org/workspaces/{{ws_id}}/agents
Content-Type: application/json
{
    "name": "Claude Code",
    "provider": "anthropic",
    "model": "claude-opus-4-20250514",
    "description": "Code generation agent",
    "capabilities": ["code_generation", "code_review"],
    "config": {"max_tokens": 8192}
}
HTTP 201
[Captures]
agent_id: jsonpath "$.id"
[Asserts]
jsonpath "$.name" == "Claude Code"
jsonpath "$.provider" == "anthropic"
jsonpath "$.model" == "claude-opus-4-20250514"
jsonpath "$.status" == "active"
jsonpath "$.capabilities" count == 2
header "Content-Type" contains "application/json"

# --- Get agent by ID ---
GET http://localhost:3001/orgs/e2e_agent_org/workspaces/{{ws_id}}/agents/{{agent_id}}
HTTP 200
[Asserts]
jsonpath "$.id" == "{{agent_id}}"
jsonpath "$.name" == "Claude Code"
jsonpath "$.provider" == "anthropic"

# --- List agents for workspace ---
GET http://localhost:3001/orgs/e2e_agent_org/workspaces/{{ws_id}}/agents
HTTP 200
[Asserts]
jsonpath "$" count >= 1

# --- Update agent ---
PUT http://localhost:3001/orgs/e2e_agent_org/workspaces/{{ws_id}}/agents/{{agent_id}}
Content-Type: application/json
{
    "name": "Updated Agent",
    "provider": "anthropic",
    "model": "claude-opus-4-20250514",
    "status": "disabled"
}
HTTP 200
[Asserts]
jsonpath "$.name" == "Updated Agent"
jsonpath "$.status" == "disabled"

# --- Delete agent ---
DELETE http://localhost:3001/orgs/e2e_agent_org/workspaces/{{ws_id}}/agents/{{agent_id}}
HTTP 204

# --- Verify deleted ---
GET http://localhost:3001/orgs/e2e_agent_org/workspaces/{{ws_id}}/agents/{{agent_id}}
HTTP 404
[Asserts]
jsonpath "$.status" == 404

# --- Validation: empty name ---
POST http://localhost:3001/orgs/e2e_agent_org/workspaces/{{ws_id}}/agents
Content-Type: application/json
{
    "name": "  ",
    "provider": "anthropic"
}
HTTP 400
[Asserts]
jsonpath "$.status" == 400

# --- Cleanup: delete workspace ---
DELETE http://localhost:3001/orgs/e2e_agent_org/workspaces/{{ws_id}}
HTTP 204
