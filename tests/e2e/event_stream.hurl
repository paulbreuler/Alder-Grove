# Event stream -- append-only event creation and listing
#
# Requires: grove-api running on localhost:3001 with PostgreSQL
# Depends on: workspace + agent + session creation (FK chain)

# --- Create workspace for event tests ---
POST http://localhost:3001/orgs/e2e_event_org/workspaces
Content-Type: application/json
{
    "name": "Event E2E Workspace"
}
HTTP 201
[Captures]
ws_id: jsonpath "$.id"

# --- Create agent (FK dependency for sessions) ---
POST http://localhost:3001/orgs/e2e_event_org/workspaces/{{ws_id}}/agents
Content-Type: application/json
{
    "name": "E2E Agent",
    "provider": "anthropic"
}
HTTP 201
[Captures]
agent_id: jsonpath "$.id"

# --- Create session (FK dependency for events) ---
POST http://localhost:3001/orgs/e2e_event_org/workspaces/{{ws_id}}/sessions
Content-Type: application/json
{
    "agent_id": "{{agent_id}}",
    "title": "Event test session",
    "intent": "implement",
    "initiated_by": "user_e2e"
}
HTTP 201
[Captures]
session_id: jsonpath "$.id"

# --- List events (should be empty) ---
GET http://localhost:3001/orgs/e2e_event_org/workspaces/{{ws_id}}/sessions/{{session_id}}/events
HTTP 200
[Asserts]
jsonpath "$" count == 0

# --- Create lifecycle event ---
POST http://localhost:3001/orgs/e2e_event_org/workspaces/{{ws_id}}/sessions/{{session_id}}/events
Content-Type: application/json
{
    "event_type": "session_started",
    "category": "lifecycle",
    "summary": "Session started"
}
HTTP 201
[Captures]
event1_id: jsonpath "$.id"
[Asserts]
jsonpath "$.event_type" == "session_started"
jsonpath "$.category" == "lifecycle"
jsonpath "$.summary" == "Session started"
jsonpath "$.session_id" == "{{session_id}}"
jsonpath "$.emitted_by" == "system"
header "Content-Type" contains "application/json"

# --- Create action event with data payload ---
POST http://localhost:3001/orgs/e2e_event_org/workspaces/{{ws_id}}/sessions/{{session_id}}/events
Content-Type: application/json
{
    "event_type": "file_modify",
    "category": "action",
    "summary": "Modified main.rs",
    "data": {"path": "src/main.rs", "lines_changed": 42},
    "emitted_by": "agent"
}
HTTP 201
[Captures]
event2_id: jsonpath "$.id"
[Asserts]
jsonpath "$.category" == "action"
jsonpath "$.emitted_by" == "agent"
jsonpath "$.data.path" == "src/main.rs"
jsonpath "$.data.lines_changed" == 42

# --- Create error event ---
POST http://localhost:3001/orgs/e2e_event_org/workspaces/{{ws_id}}/sessions/{{session_id}}/events
Content-Type: application/json
{
    "event_type": "api_error",
    "category": "error",
    "summary": "API call failed",
    "data": {"error": "connection refused", "retries": 3}
}
HTTP 201

# --- List events for session (should contain all 3) ---
GET http://localhost:3001/orgs/e2e_event_org/workspaces/{{ws_id}}/sessions/{{session_id}}/events
HTTP 200
[Asserts]
jsonpath "$" count == 3

# --- Verify events are not returned for wrong session ---
POST http://localhost:3001/orgs/e2e_event_org/workspaces/{{ws_id}}/sessions
Content-Type: application/json
{
    "agent_id": "{{agent_id}}",
    "title": "Other session",
    "intent": "review",
    "initiated_by": "user_e2e"
}
HTTP 201
[Captures]
session2_id: jsonpath "$.id"

GET http://localhost:3001/orgs/e2e_event_org/workspaces/{{ws_id}}/sessions/{{session2_id}}/events
HTTP 200
[Asserts]
jsonpath "$" count == 0

# --- Validation: empty event_type ---
POST http://localhost:3001/orgs/e2e_event_org/workspaces/{{ws_id}}/sessions/{{session_id}}/events
Content-Type: application/json
{
    "event_type": "  ",
    "category": "lifecycle",
    "summary": "Test"
}
HTTP 400
[Asserts]
jsonpath "$.status" == 400

# --- Validation: empty summary ---
POST http://localhost:3001/orgs/e2e_event_org/workspaces/{{ws_id}}/sessions/{{session_id}}/events
Content-Type: application/json
{
    "event_type": "test",
    "category": "lifecycle",
    "summary": "  "
}
HTTP 400
[Asserts]
jsonpath "$.status" == 400

# --- Cleanup: delete workspace (cascades to sessions, events) ---
DELETE http://localhost:3001/orgs/e2e_event_org/workspaces/{{ws_id}}
HTTP 204
