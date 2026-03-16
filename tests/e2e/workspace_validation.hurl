# Workspace validation — error cases

# --- Empty name rejected ---
POST http://localhost:3001/orgs/e2e_org/workspaces
Content-Type: application/json
{
    "name": "   "
}
HTTP 400
[Asserts]
jsonpath "$.status" == 400
jsonpath "$.detail" == "name cannot be empty"

# --- Missing body rejected ---
POST http://localhost:3001/orgs/e2e_org/workspaces
Content-Type: application/json
{}
HTTP 422

# --- Nonexistent workspace returns 404 ---
GET http://localhost:3001/orgs/e2e_org/workspaces/01970000-0000-7000-0000-000000000000
HTTP 404
