# Session CRUD -- full lifecycle with status transitions
#
# Requires: grove-api running on localhost:3001 with PostgreSQL
# Depends on: workspace + agent creation to provide FK context

# --- Create workspace for session tests ---
POST http://localhost:3001/orgs/e2e_session_org/workspaces
Content-Type: application/json
{
    "name": "Session E2E Workspace"
}
HTTP 201
[Captures]
ws_id: jsonpath "$.id"

# --- Create agent (FK dependency for sessions) ---
POST http://localhost:3001/orgs/e2e_session_org/workspaces/{{ws_id}}/agents
Content-Type: application/json
{
    "name": "E2E Agent",
    "provider": "anthropic",
    "model": "claude-opus-4-20250514"
}
HTTP 201
[Captures]
agent_id: jsonpath "$.id"

# --- Create session ---
POST http://localhost:3001/orgs/e2e_session_org/workspaces/{{ws_id}}/sessions
Content-Type: application/json
{
    "agent_id": "{{agent_id}}",
    "title": "Implement feature X",
    "intent": "implement",
    "context": {"branch": "feature/x"},
    "initiated_by": "user_e2e"
}
HTTP 201
[Captures]
session_id: jsonpath "$.id"
[Asserts]
jsonpath "$.title" == "Implement feature X"
jsonpath "$.status" == "pending"
jsonpath "$.intent" == "implement"
jsonpath "$.initiated_by" == "user_e2e"
jsonpath "$.agent_id" == "{{agent_id}}"
jsonpath "$.started_at" == null
jsonpath "$.completed_at" == null
header "Content-Type" contains "application/json"

# --- Get session by ID ---
GET http://localhost:3001/orgs/e2e_session_org/workspaces/{{ws_id}}/sessions/{{session_id}}
HTTP 200
[Asserts]
jsonpath "$.id" == "{{session_id}}"
jsonpath "$.title" == "Implement feature X"
jsonpath "$.status" == "pending"

# --- List sessions for workspace ---
GET http://localhost:3001/orgs/e2e_session_org/workspaces/{{ws_id}}/sessions
HTTP 200
[Asserts]
jsonpath "$" count >= 1

# --- Update session ---
PUT http://localhost:3001/orgs/e2e_session_org/workspaces/{{ws_id}}/sessions/{{session_id}}
Content-Type: application/json
{
    "agent_id": "{{agent_id}}",
    "title": "Updated session title",
    "intent": "review",
    "context": {"branch": "feature/x", "updated": true},
    "result": null,
    "initiated_by": "user_e2e"
}
HTTP 200
[Asserts]
jsonpath "$.title" == "Updated session title"
jsonpath "$.intent" == "review"

# --- Status transition: pending -> active ---
PATCH http://localhost:3001/orgs/e2e_session_org/workspaces/{{ws_id}}/sessions/{{session_id}}/status
Content-Type: application/json
{
    "status": "active"
}
HTTP 200
[Asserts]
jsonpath "$.status" == "active"
jsonpath "$.started_at" != null

# --- Status transition: active -> completed ---
PATCH http://localhost:3001/orgs/e2e_session_org/workspaces/{{ws_id}}/sessions/{{session_id}}/status
Content-Type: application/json
{
    "status": "completed"
}
HTTP 200
[Asserts]
jsonpath "$.status" == "completed"
jsonpath "$.completed_at" != null

# --- Create a second session to test status filter ---
POST http://localhost:3001/orgs/e2e_session_org/workspaces/{{ws_id}}/sessions
Content-Type: application/json
{
    "agent_id": "{{agent_id}}",
    "title": "Active session",
    "intent": "analyze",
    "initiated_by": "user_e2e"
}
HTTP 201
[Captures]
session2_id: jsonpath "$.id"

# Transition second session to active
PATCH http://localhost:3001/orgs/e2e_session_org/workspaces/{{ws_id}}/sessions/{{session2_id}}/status
Content-Type: application/json
{
    "status": "active"
}
HTTP 200

# --- List sessions with status filter ---
GET http://localhost:3001/orgs/e2e_session_org/workspaces/{{ws_id}}/sessions?status=active
HTTP 200
[Asserts]
jsonpath "$" count == 1
jsonpath "$[0].status" == "active"

# --- Invalid status transition (completed -> active) ---
PATCH http://localhost:3001/orgs/e2e_session_org/workspaces/{{ws_id}}/sessions/{{session_id}}/status
Content-Type: application/json
{
    "status": "active"
}
HTTP 409

# --- Delete sessions ---
DELETE http://localhost:3001/orgs/e2e_session_org/workspaces/{{ws_id}}/sessions/{{session_id}}
HTTP 204

DELETE http://localhost:3001/orgs/e2e_session_org/workspaces/{{ws_id}}/sessions/{{session2_id}}
HTTP 204

# --- Verify deleted ---
GET http://localhost:3001/orgs/e2e_session_org/workspaces/{{ws_id}}/sessions/{{session_id}}
HTTP 404
[Asserts]
jsonpath "$.status" == 404

# --- Validation: agent_id from different workspace is rejected ---
POST http://localhost:3001/orgs/e2e_session_org/workspaces/{{ws_id}}/sessions
Content-Type: application/json
{
    "agent_id": "00000000-0000-0000-0000-000000000000",
    "title": "Foreign agent session",
    "intent": "implement",
    "initiated_by": "user_e2e"
}
HTTP 404
[Asserts]
jsonpath "$.status" == 404

# --- Validation: empty title ---
POST http://localhost:3001/orgs/e2e_session_org/workspaces/{{ws_id}}/sessions
Content-Type: application/json
{
    "agent_id": "{{agent_id}}",
    "title": "  ",
    "intent": "implement",
    "initiated_by": "user_e2e"
}
HTTP 400
[Asserts]
jsonpath "$.status" == 400

# --- Cleanup ---
DELETE http://localhost:3001/orgs/e2e_session_org/workspaces/{{ws_id}}
HTTP 204
