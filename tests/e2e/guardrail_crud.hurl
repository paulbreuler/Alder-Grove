# Guardrail CRUD -- full lifecycle
#
# Requires: grove-api running on localhost:3001 with PostgreSQL
# Depends on: workspace creation to provide org/workspace context

# --- Create workspace for guardrail tests ---
POST http://localhost:3001/orgs/e2e_guardrail_org/workspaces
Content-Type: application/json
{
    "name": "Guardrail E2E Workspace"
}
HTTP 201
[Captures]
ws_id: jsonpath "$.id"

# --- Create guardrail ---
POST http://localhost:3001/orgs/e2e_guardrail_org/workspaces/{{ws_id}}/guardrails
Content-Type: application/json
{
    "name": "No migration edits",
    "description": "Prevent accidental migration changes",
    "category": "prohibition",
    "scope": "workspace",
    "enforcement": "enforced",
    "rule": {
        "type": "prohibition",
        "description": "Do not modify migration files",
        "patterns": ["migrations/"],
        "actions": ["file_modify", "file_delete"]
    }
}
HTTP 201
[Captures]
guardrail_id: jsonpath "$.id"
[Asserts]
jsonpath "$.name" == "No migration edits"
jsonpath "$.category" == "prohibition"
jsonpath "$.scope" == "workspace"
jsonpath "$.enforcement" == "enforced"
jsonpath "$.enabled" == true
jsonpath "$.version" == 1
header "Content-Type" contains "application/json"

# --- Get guardrail by ID ---
GET http://localhost:3001/orgs/e2e_guardrail_org/workspaces/{{ws_id}}/guardrails/{{guardrail_id}}
HTTP 200
[Asserts]
jsonpath "$.id" == "{{guardrail_id}}"
jsonpath "$.name" == "No migration edits"

# --- Create a second guardrail (session-scoped) ---
POST http://localhost:3001/orgs/e2e_guardrail_org/workspaces/{{ws_id}}/guardrails
Content-Type: application/json
{
    "name": "Session boundary",
    "category": "boundary",
    "scope": "session",
    "enforcement": "enforced",
    "rule": {
        "type": "boundary",
        "description": "Stay in src",
        "allowed_paths": ["src/"],
        "denied_paths": [".env"]
    }
}
HTTP 201
[Captures]
guardrail2_id: jsonpath "$.id"

# --- List all guardrails ---
GET http://localhost:3001/orgs/e2e_guardrail_org/workspaces/{{ws_id}}/guardrails
HTTP 200
[Asserts]
jsonpath "$" count >= 2

# --- List with scope filter ---
GET http://localhost:3001/orgs/e2e_guardrail_org/workspaces/{{ws_id}}/guardrails?scope=workspace&enabled=true
HTTP 200
[Asserts]
jsonpath "$" count == 1
jsonpath "$[0].name" == "No migration edits"

# --- List session-scoped ---
GET http://localhost:3001/orgs/e2e_guardrail_org/workspaces/{{ws_id}}/guardrails?scope=session&enabled=true
HTTP 200
[Asserts]
jsonpath "$" count == 1
jsonpath "$[0].name" == "Session boundary"

# --- Update guardrail ---
PUT http://localhost:3001/orgs/e2e_guardrail_org/workspaces/{{ws_id}}/guardrails/{{guardrail_id}}
Content-Type: application/json
{
    "name": "Updated guardrail",
    "category": "prohibition",
    "scope": "workspace",
    "enforcement": "advisory",
    "rule": {
        "type": "prohibition",
        "description": "Do not modify migration files",
        "patterns": ["migrations/"],
        "actions": ["file_modify"]
    },
    "enabled": false
}
HTTP 200
[Asserts]
jsonpath "$.name" == "Updated guardrail"
jsonpath "$.enforcement" == "advisory"
jsonpath "$.enabled" == false

# --- Delete guardrails ---
DELETE http://localhost:3001/orgs/e2e_guardrail_org/workspaces/{{ws_id}}/guardrails/{{guardrail_id}}
HTTP 204

DELETE http://localhost:3001/orgs/e2e_guardrail_org/workspaces/{{ws_id}}/guardrails/{{guardrail2_id}}
HTTP 204

# --- Verify deleted ---
GET http://localhost:3001/orgs/e2e_guardrail_org/workspaces/{{ws_id}}/guardrails/{{guardrail_id}}
HTTP 404
[Asserts]
jsonpath "$.status" == 404

# --- Validation: empty name ---
POST http://localhost:3001/orgs/e2e_guardrail_org/workspaces/{{ws_id}}/guardrails
Content-Type: application/json
{
    "name": "  ",
    "category": "prohibition",
    "rule": {
        "type": "prohibition",
        "description": "test",
        "patterns": [],
        "actions": []
    }
}
HTTP 400
[Asserts]
jsonpath "$.status" == 400

# --- Cleanup: delete workspace ---
DELETE http://localhost:3001/orgs/e2e_guardrail_org/workspaces/{{ws_id}}
HTTP 204
