mod common;

use grove_domain::event::{Event, EventCategory, EventEmitter};
use uuid::Uuid;

/// Helper: create a workspace and return its id.
async fn create_workspace(pool: &sqlx::PgPool, org_id: &str) -> Uuid {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO workspaces (org_id, name) VALUES ($1, 'event-test-ws') RETURNING id",
    )
    .bind(org_id)
    .fetch_one(pool)
    .await
    .unwrap();
    row.0
}

/// Helper: create an agent in the workspace (required FK for sessions).
async fn create_agent(pool: &sqlx::PgPool, workspace_id: Uuid) -> Uuid {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO agents (workspace_id, name, provider) \
         VALUES ($1, 'Test Agent', 'anthropic') RETURNING id",
    )
    .bind(workspace_id)
    .fetch_one(pool)
    .await
    .unwrap();
    row.0
}

/// Helper: create a session in the workspace (required FK for events).
async fn create_session(pool: &sqlx::PgPool, workspace_id: Uuid, agent_id: Uuid) -> Uuid {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO sessions (workspace_id, agent_id, title, intent, initiated_by) \
         VALUES ($1, $2, 'Test Session', 'implement', 'user_test') RETURNING id",
    )
    .bind(workspace_id)
    .bind(agent_id)
    .fetch_one(pool)
    .await
    .unwrap();
    row.0
}

#[tokio::test]
async fn event_create_and_list() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();
    let ws_id = create_workspace(&pool, &org_id).await;
    let agent_id = create_agent(&pool, ws_id).await;
    let session_id = create_session(&pool, ws_id, agent_id).await;
    let repo = &state.event_repo;

    // Create lifecycle event
    let e1 = Event::lifecycle(session_id, ws_id, "session_started", "Session started");
    let created1 = repo.create(&e1).await.unwrap();
    assert_eq!(created1.session_id, session_id);
    assert_eq!(created1.workspace_id, ws_id);
    assert_eq!(created1.event_type, "session_started");
    assert_eq!(created1.category, EventCategory::Lifecycle);
    assert_eq!(created1.emitted_by, EventEmitter::System);

    // Create action event
    let e2 = Event::action(
        session_id,
        ws_id,
        "file_modify",
        "Modified main.rs",
        serde_json::json!({"path": "src/main.rs", "lines_changed": 42}),
    );
    let created2 = repo.create(&e2).await.unwrap();
    assert_eq!(created2.category, EventCategory::Action);

    // List events for session — workspace_id required for RLS
    let events = repo.find_all(ws_id, session_id).await.unwrap();
    assert_eq!(events.len(), 2);
    assert!(events.iter().any(|e| e.id == created1.id));
    assert!(events.iter().any(|e| e.id == created2.id));

    // Cleanup
    common::cleanup_org(&pool, &org_id).await;
}

#[tokio::test]
async fn event_scoped_to_session() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();
    let ws_id = create_workspace(&pool, &org_id).await;
    let agent_id = create_agent(&pool, ws_id).await;
    let session_a = create_session(&pool, ws_id, agent_id).await;
    let session_b = create_session(&pool, ws_id, agent_id).await;
    let repo = &state.event_repo;

    // Create event in session A
    let event = Event::lifecycle(session_a, ws_id, "session_started", "Session A started");
    let created = repo.create(&event).await.unwrap();

    // Events for session A should include it
    let events_a = repo.find_all(ws_id, session_a).await.unwrap();
    assert!(events_a.iter().any(|e| e.id == created.id));

    // Events for session B should NOT include it
    let events_b = repo.find_all(ws_id, session_b).await.unwrap();
    assert!(
        !events_b.iter().any(|e| e.id == created.id),
        "Event from session_a should not appear in session_b list"
    );

    // Cleanup
    common::cleanup_org(&pool, &org_id).await;
}

#[tokio::test]
async fn event_list_empty_session_returns_empty() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();
    let ws_id = create_workspace(&pool, &org_id).await;
    let agent_id = create_agent(&pool, ws_id).await;
    let session_id = create_session(&pool, ws_id, agent_id).await;
    let repo = &state.event_repo;

    // List events for a session with no events
    let events = repo.find_all(ws_id, session_id).await.unwrap();
    assert!(events.is_empty());

    // Cleanup
    common::cleanup_org(&pool, &org_id).await;
}

#[tokio::test]
async fn event_preserves_data_payload() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();
    let ws_id = create_workspace(&pool, &org_id).await;
    let agent_id = create_agent(&pool, ws_id).await;
    let session_id = create_session(&pool, ws_id, agent_id).await;
    let repo = &state.event_repo;

    let data = serde_json::json!({
        "path": "src/main.rs",
        "lines_changed": 42,
        "nested": {"key": "value"}
    });
    let event = Event::action(
        session_id,
        ws_id,
        "file_modify",
        "Modified main.rs",
        data.clone(),
    );
    let created = repo.create(&event).await.unwrap();
    assert_eq!(created.data, data);

    // Verify persistence via find_all (workspace-scoped)
    let events = repo.find_all(ws_id, session_id).await.unwrap();
    let found = events.iter().find(|e| e.id == created.id).unwrap();
    assert_eq!(found.data, data);

    // Cleanup
    common::cleanup_org(&pool, &org_id).await;
}

#[tokio::test]
async fn event_rls_isolates_across_workspaces() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_a = common::unique_org_id();
    let org_b = common::unique_org_id();
    let ws_a = create_workspace(&pool, &org_a).await;
    let ws_b = create_workspace(&pool, &org_b).await;
    let agent_a = create_agent(&pool, ws_a).await;
    let session_a = create_session(&pool, ws_a, agent_a).await;
    let repo = &state.event_repo;

    // Create event in workspace A
    let event = Event::lifecycle(session_a, ws_a, "session_started", "Started in ws_a");
    let created = repo.create(&event).await.unwrap();

    // Events for ws_a/session_a should include it
    let events_a = repo.find_all(ws_a, session_a).await.unwrap();
    assert!(
        events_a.iter().any(|e| e.id == created.id),
        "Event should be visible in its own workspace"
    );

    // Querying with ws_b context should NOT return ws_a's events (RLS isolation)
    let events_wrong_ws = repo.find_all(ws_b, session_a).await.unwrap();
    assert!(
        !events_wrong_ws.iter().any(|e| e.id == created.id),
        "Event from ws_a should not be visible under ws_b RLS context"
    );

    common::cleanup_org(&pool, &org_a).await;
    common::cleanup_org(&pool, &org_b).await;
}
