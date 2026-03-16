# Org isolation — workspaces are not visible across orgs

# --- Create workspace in org_alpha ---
POST http://localhost:3001/orgs/e2e_org_alpha/workspaces
Content-Type: application/json
{
    "name": "Alpha's Workspace"
}
HTTP 201
[Captures]
alpha_ws_id: jsonpath "$.id"

# --- List from org_beta — should not see org_alpha's workspace ---
GET http://localhost:3001/orgs/e2e_org_beta/workspaces
HTTP 200
[Asserts]
jsonpath "$" count == 0

# --- Cleanup: delete alpha workspace ---
DELETE http://localhost:3001/orgs/e2e_org_alpha/workspaces/{{alpha_ws_id}}
HTTP 204
