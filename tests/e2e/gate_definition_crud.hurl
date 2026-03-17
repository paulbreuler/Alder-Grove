# Gate Definition CRUD -- full lifecycle
#
# Requires: grove-api running on localhost:3001 with PostgreSQL
# Depends on: workspace creation to provide org/workspace context

# --- Create workspace for gate definition tests ---
POST http://localhost:3001/orgs/e2e_gatedef_org/workspaces
Content-Type: application/json
{
    "name": "Gate Definition E2E Workspace"
}
HTTP 201
[Captures]
ws_id: jsonpath "$.id"

# --- Create gate definition ---
POST http://localhost:3001/orgs/e2e_gatedef_org/workspaces/{{ws_id}}/gate-definitions
Content-Type: application/json
{
    "name": "Code Deletion Review",
    "description": "Requires approval before deleting code",
    "trigger_type": "automatic",
    "trigger_config": {
        "patterns": [{"event_type": "file_delete", "match": "**/*.rs"}]
    },
    "approval_type": "single",
    "timeout_minutes": 60,
    "timeout_action": "cancel"
}
HTTP 201
[Captures]
gatedef_id: jsonpath "$.id"
[Asserts]
jsonpath "$.name" == "Code Deletion Review"
jsonpath "$.trigger_type" == "automatic"
jsonpath "$.approval_type" == "single"
jsonpath "$.timeout_action" == "cancel"
jsonpath "$.timeout_minutes" == 60
jsonpath "$.enabled" == true
header "Content-Type" contains "application/json"

# --- Get gate definition by ID ---
GET http://localhost:3001/orgs/e2e_gatedef_org/workspaces/{{ws_id}}/gate-definitions/{{gatedef_id}}
HTTP 200
[Asserts]
jsonpath "$.id" == "{{gatedef_id}}"
jsonpath "$.name" == "Code Deletion Review"

# --- Create a second gate definition (disabled) ---
POST http://localhost:3001/orgs/e2e_gatedef_org/workspaces/{{ws_id}}/gate-definitions
Content-Type: application/json
{
    "name": "Manual Review Gate",
    "trigger_type": "manual",
    "enabled": false
}
HTTP 201
[Captures]
gatedef2_id: jsonpath "$.id"
[Asserts]
jsonpath "$.enabled" == false

# --- List all gate definitions ---
GET http://localhost:3001/orgs/e2e_gatedef_org/workspaces/{{ws_id}}/gate-definitions
HTTP 200
[Asserts]
jsonpath "$" count >= 2

# --- List with enabled filter ---
GET http://localhost:3001/orgs/e2e_gatedef_org/workspaces/{{ws_id}}/gate-definitions?enabled=true
HTTP 200
[Asserts]
jsonpath "$" count == 1
jsonpath "$[0].name" == "Code Deletion Review"

# --- Update gate definition ---
PUT http://localhost:3001/orgs/e2e_gatedef_org/workspaces/{{ws_id}}/gate-definitions/{{gatedef_id}}
Content-Type: application/json
{
    "name": "Updated Gate Def",
    "description": "Updated description",
    "trigger_type": "manual",
    "trigger_config": {},
    "approval_type": "all_of",
    "timeout_minutes": 120,
    "timeout_action": "escalate",
    "enabled": false,
    "sort_order": 1
}
HTTP 200
[Asserts]
jsonpath "$.name" == "Updated Gate Def"
jsonpath "$.trigger_type" == "manual"
jsonpath "$.approval_type" == "all_of"
jsonpath "$.timeout_action" == "escalate"
jsonpath "$.enabled" == false

# --- Delete gate definitions ---
DELETE http://localhost:3001/orgs/e2e_gatedef_org/workspaces/{{ws_id}}/gate-definitions/{{gatedef_id}}
HTTP 204

DELETE http://localhost:3001/orgs/e2e_gatedef_org/workspaces/{{ws_id}}/gate-definitions/{{gatedef2_id}}
HTTP 204

# --- Verify deleted ---
GET http://localhost:3001/orgs/e2e_gatedef_org/workspaces/{{ws_id}}/gate-definitions/{{gatedef_id}}
HTTP 404
[Asserts]
jsonpath "$.status" == 404

# --- Validation: empty name ---
POST http://localhost:3001/orgs/e2e_gatedef_org/workspaces/{{ws_id}}/gate-definitions
Content-Type: application/json
{
    "name": "  ",
    "trigger_type": "automatic"
}
HTTP 400
[Asserts]
jsonpath "$.status" == 400

# --- Cleanup: delete workspace ---
DELETE http://localhost:3001/orgs/e2e_gatedef_org/workspaces/{{ws_id}}
HTTP 204
