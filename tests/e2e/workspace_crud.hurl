# Workspace CRUD — full lifecycle
#
# Requires: grove-api running on localhost:3001 with PostgreSQL

# --- Create workspace ---
POST http://localhost:3001/orgs/e2e_org/workspaces
Content-Type: application/json
{
    "name": "E2E Workspace",
    "description": "Created by Hurl e2e test"
}
HTTP 201
[Captures]
ws_id: jsonpath "$.id"
[Asserts]
jsonpath "$.name" == "E2E Workspace"
jsonpath "$.org_id" == "e2e_org"
jsonpath "$.description" == "Created by Hurl e2e test"

# --- Get workspace by ID ---
GET http://localhost:3001/orgs/e2e_org/workspaces/{{ws_id}}
HTTP 200
[Asserts]
jsonpath "$.id" == "{{ws_id}}"
jsonpath "$.name" == "E2E Workspace"

# --- List workspaces for org ---
GET http://localhost:3001/orgs/e2e_org/workspaces
HTTP 200
[Asserts]
jsonpath "$" count >= 1

# --- Update workspace ---
PUT http://localhost:3001/orgs/e2e_org/workspaces/{{ws_id}}
Content-Type: application/json
{
    "name": "Updated Workspace",
    "description": "Modified by Hurl"
}
HTTP 200
[Asserts]
jsonpath "$.name" == "Updated Workspace"
jsonpath "$.description" == "Modified by Hurl"

# --- Delete workspace ---
DELETE http://localhost:3001/orgs/e2e_org/workspaces/{{ws_id}}
HTTP 204

# --- Verify deleted ---
GET http://localhost:3001/orgs/e2e_org/workspaces/{{ws_id}}
HTTP 404
[Asserts]
jsonpath "$.status" == 404
