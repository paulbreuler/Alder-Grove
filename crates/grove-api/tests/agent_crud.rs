mod common;

use grove_domain::agent::{Agent, AgentStatus};
use uuid::Uuid;

/// Helper: create a workspace and return its id.
async fn create_workspace(pool: &sqlx::PgPool, org_id: &str) -> Uuid {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO workspaces (org_id, name) VALUES ($1, 'agent-test-ws') RETURNING id",
    )
    .bind(org_id)
    .fetch_one(pool)
    .await
    .unwrap();
    row.0
}

fn make_agent(workspace_id: Uuid) -> Agent {
    Agent {
        id: Uuid::now_v7(),
        workspace_id,
        name: "Claude Code".into(),
        provider: "anthropic".into(),
        model: Some("claude-opus-4-20250514".into()),
        description: Some("Code generation agent".into()),
        capabilities: vec!["code_generation".into(), "code_review".into()],
        config: serde_json::json!({"max_tokens": 8192}),
        status: AgentStatus::Active,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

#[tokio::test]
async fn agent_crud_lifecycle() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();
    let ws_id = create_workspace(&pool, &org_id).await;
    let repo = &state.agent_repo;

    // Create
    let agent = make_agent(ws_id);
    let created = repo.create(&agent).await.unwrap();
    assert_eq!(created.name, "Claude Code");
    assert_eq!(created.provider, "anthropic");
    assert_eq!(created.workspace_id, ws_id);
    assert_eq!(created.status, AgentStatus::Active);
    assert_eq!(created.capabilities.len(), 2);

    // Read
    let found = repo.find_by_id(ws_id, created.id).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.id, created.id);
    assert_eq!(found.name, "Claude Code");

    // List
    let all = repo.find_all(ws_id).await.unwrap();
    assert!(all.iter().any(|a| a.id == created.id));

    // Update
    let mut updated_agent = found.clone();
    updated_agent.name = "Updated Agent".into();
    updated_agent.status = AgentStatus::Disabled;
    let updated = repo.update(&updated_agent).await.unwrap();
    assert_eq!(updated.name, "Updated Agent");
    assert_eq!(updated.status, AgentStatus::Disabled);

    // Delete
    repo.delete(ws_id, created.id).await.unwrap();
    let gone = repo.find_by_id(ws_id, created.id).await.unwrap();
    assert!(gone.is_none());

    // Cleanup
    common::cleanup_org(&pool, &org_id).await;
}

#[tokio::test]
async fn agent_find_by_id_nonexistent_returns_none() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();
    let ws_id = create_workspace(&pool, &org_id).await;

    let result = state
        .agent_repo
        .find_by_id(ws_id, Uuid::now_v7())
        .await
        .unwrap();
    assert!(result.is_none());

    common::cleanup_org(&pool, &org_id).await;
}

#[tokio::test]
async fn agent_delete_nonexistent_returns_not_found() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();
    let ws_id = create_workspace(&pool, &org_id).await;

    let result = state.agent_repo.delete(ws_id, Uuid::now_v7()).await;
    assert!(result.is_err());

    common::cleanup_org(&pool, &org_id).await;
}

#[tokio::test]
async fn agent_rls_isolates_across_workspaces() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_a = common::unique_org_id();
    let org_b = common::unique_org_id();
    let ws_a = create_workspace(&pool, &org_a).await;
    let ws_b = create_workspace(&pool, &org_b).await;
    let repo = &state.agent_repo;

    // Create agent in workspace A
    let agent = make_agent(ws_a);
    let created = repo.create(&agent).await.unwrap();

    // Should NOT be visible from workspace B
    let found = repo.find_by_id(ws_b, created.id).await.unwrap();
    assert!(
        found.is_none(),
        "RLS should isolate agents across workspaces"
    );

    let list_b = repo.find_all(ws_b).await.unwrap();
    assert!(
        !list_b.iter().any(|a| a.id == created.id),
        "Agent from ws_a should not appear in ws_b list"
    );

    common::cleanup_org(&pool, &org_a).await;
    common::cleanup_org(&pool, &org_b).await;
}
