mod common;

use grove_domain::session::{Session, SessionIntent, SessionStatus};
use uuid::Uuid;

/// Helper: create a workspace and return its id.
async fn create_workspace(pool: &sqlx::PgPool, org_id: &str) -> Uuid {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO workspaces (org_id, name) VALUES ($1, 'session-test-ws') RETURNING id",
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

fn make_session(workspace_id: Uuid, agent_id: Uuid) -> Session {
    let now = chrono::Utc::now();
    Session::new(
        Uuid::now_v7(),
        workspace_id,
        agent_id,
        "Implement feature X".into(),
        SessionStatus::Pending,
        SessionIntent::Implement,
        None,
        None,
        serde_json::json!({"key": "value"}),
        None,
        "user_test".into(),
        None,
        None,
        now,
        now,
    )
    .unwrap()
}

#[tokio::test]
async fn session_crud_lifecycle() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();
    let ws_id = create_workspace(&pool, &org_id).await;
    let agent_id = create_agent(&pool, ws_id).await;
    let repo = &state.session_repo;

    // Create
    let session = make_session(ws_id, agent_id);
    let created = repo.create(&session).await.unwrap();
    assert_eq!(created.title, "Implement feature X");
    assert_eq!(created.status, SessionStatus::Pending);
    assert_eq!(created.intent, SessionIntent::Implement);
    assert_eq!(created.workspace_id, ws_id);
    assert_eq!(created.agent_id, agent_id);
    assert_eq!(created.initiated_by, "user_test");
    assert!(created.started_at.is_none());
    assert!(created.completed_at.is_none());

    // Read
    let found = repo.find_by_id(ws_id, created.id).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.id, created.id);
    assert_eq!(found.title, "Implement feature X");

    // List
    let all = repo.find_all(ws_id).await.unwrap();
    assert!(all.iter().any(|s| s.id == created.id));

    // Update
    let now = chrono::Utc::now();
    let updated_session = Session::new(
        found.id,
        found.workspace_id,
        found.agent_id,
        "Updated title".into(),
        found.status,
        SessionIntent::Review,
        None,
        None,
        serde_json::json!({"updated": true}),
        None,
        found.initiated_by.clone(),
        found.started_at,
        found.completed_at,
        found.created_at,
        now,
    )
    .unwrap();
    let updated = repo.update(&updated_session).await.unwrap();
    assert_eq!(updated.title, "Updated title");
    assert_eq!(updated.intent, SessionIntent::Review);

    // Delete
    repo.delete(ws_id, created.id).await.unwrap();
    let gone = repo.find_by_id(ws_id, created.id).await.unwrap();
    assert!(gone.is_none());

    // Cleanup
    common::cleanup_org(&pool, &org_id).await;
}

#[tokio::test]
async fn session_find_by_status() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();
    let ws_id = create_workspace(&pool, &org_id).await;
    let agent_id = create_agent(&pool, ws_id).await;
    let repo = &state.session_repo;

    // Create a pending session
    let s1 = make_session(ws_id, agent_id);
    repo.create(&s1).await.unwrap();

    // Create a second session and transition it to active
    let mut s2 = make_session(ws_id, agent_id);
    s2.id = Uuid::now_v7();
    s2.title = "Active session".into();
    let created_s2 = repo.create(&s2).await.unwrap();
    let mut active_session = created_s2;
    active_session.transition_to(SessionStatus::Active).unwrap();
    repo.update(&active_session).await.unwrap();

    // find_by_status(Pending) should include s1 but not s2
    let pending = repo
        .find_by_status(ws_id, SessionStatus::Pending)
        .await
        .unwrap();
    assert!(pending.iter().any(|s| s.id == s1.id));
    assert!(!pending.iter().any(|s| s.id == s2.id));

    // find_by_status(Active) should include s2 but not s1
    let active = repo
        .find_by_status(ws_id, SessionStatus::Active)
        .await
        .unwrap();
    assert!(active.iter().any(|s| s.id == s2.id));
    assert!(!active.iter().any(|s| s.id == s1.id));

    // Cleanup
    common::cleanup_org(&pool, &org_id).await;
}

#[tokio::test]
async fn session_status_transition_via_update() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();
    let ws_id = create_workspace(&pool, &org_id).await;
    let agent_id = create_agent(&pool, ws_id).await;
    let repo = &state.session_repo;

    let session = make_session(ws_id, agent_id);
    let mut created = repo.create(&session).await.unwrap();
    assert_eq!(created.status, SessionStatus::Pending);

    // Pending -> Active
    created.transition_to(SessionStatus::Active).unwrap();
    let updated = repo.update(&created).await.unwrap();
    assert_eq!(updated.status, SessionStatus::Active);
    assert!(updated.started_at.is_some());

    // Active -> Completed
    let mut active = updated;
    active.transition_to(SessionStatus::Completed).unwrap();
    let completed = repo.update(&active).await.unwrap();
    assert_eq!(completed.status, SessionStatus::Completed);
    assert!(completed.completed_at.is_some());

    // Cleanup
    common::cleanup_org(&pool, &org_id).await;
}

#[tokio::test]
async fn session_rls_isolates_across_workspaces() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_a = common::unique_org_id();
    let org_b = common::unique_org_id();
    let ws_a = create_workspace(&pool, &org_a).await;
    let ws_b = create_workspace(&pool, &org_b).await;
    let agent_a = create_agent(&pool, ws_a).await;
    let repo = &state.session_repo;

    // Create session in workspace A
    let session = make_session(ws_a, agent_a);
    let created = repo.create(&session).await.unwrap();

    // Should NOT be visible from workspace B
    let found = repo.find_by_id(ws_b, created.id).await.unwrap();
    assert!(
        found.is_none(),
        "RLS should isolate sessions across workspaces"
    );

    let list_b = repo.find_all(ws_b).await.unwrap();
    assert!(
        !list_b.iter().any(|s| s.id == created.id),
        "Session from ws_a should not appear in ws_b list"
    );

    common::cleanup_org(&pool, &org_a).await;
    common::cleanup_org(&pool, &org_b).await;
}

#[tokio::test]
async fn session_find_by_id_nonexistent_returns_none() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();
    let ws_id = create_workspace(&pool, &org_id).await;

    let result = state
        .session_repo
        .find_by_id(ws_id, Uuid::now_v7())
        .await
        .unwrap();
    assert!(result.is_none());

    common::cleanup_org(&pool, &org_id).await;
}

#[tokio::test]
async fn session_delete_nonexistent_returns_not_found() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();
    let ws_id = create_workspace(&pool, &org_id).await;

    let result = state.session_repo.delete(ws_id, Uuid::now_v7()).await;
    assert!(result.is_err());

    common::cleanup_org(&pool, &org_id).await;
}
