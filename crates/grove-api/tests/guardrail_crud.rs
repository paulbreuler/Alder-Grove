mod common;

use grove_domain::guardrail::{
    Guardrail, GuardrailCategory, GuardrailEnforcement, GuardrailRule, GuardrailScope,
};
use uuid::Uuid;

/// Helper: create a workspace and return its id.
async fn create_workspace(pool: &sqlx::PgPool, org_id: &str) -> Uuid {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO workspaces (org_id, name) VALUES ($1, 'guardrail-test-ws') RETURNING id",
    )
    .bind(org_id)
    .fetch_one(pool)
    .await
    .unwrap();
    row.0
}

fn make_guardrail(workspace_id: Uuid) -> Guardrail {
    Guardrail {
        id: Uuid::now_v7(),
        workspace_id,
        name: "No migration edits".into(),
        description: Some("Prevent accidental migration changes".into()),
        category: GuardrailCategory::Prohibition,
        scope: GuardrailScope::Workspace,
        enforcement: GuardrailEnforcement::Enforced,
        rule: GuardrailRule::Prohibition {
            description: "Do not modify migration files".into(),
            patterns: vec!["migrations/".into()],
            actions: vec!["file_modify".into(), "file_delete".into()],
        },
        version: 1,
        sort_order: 0,
        enabled: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

#[tokio::test]
async fn guardrail_crud_lifecycle() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();
    let ws_id = create_workspace(&pool, &org_id).await;
    let repo = &state.guardrail_repo;

    // Create
    let guardrail = make_guardrail(ws_id);
    let created = repo.create(&guardrail).await.unwrap();
    assert_eq!(created.name, "No migration edits");
    assert_eq!(created.category, GuardrailCategory::Prohibition);
    assert_eq!(created.scope, GuardrailScope::Workspace);
    assert_eq!(created.enforcement, GuardrailEnforcement::Enforced);
    assert!(created.enabled);

    // Read
    let found = repo.find_by_id(ws_id, created.id).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.id, created.id);
    assert_eq!(found.name, "No migration edits");

    // List
    let all = repo.find_all(ws_id).await.unwrap();
    assert!(all.iter().any(|g| g.id == created.id));

    // Update
    let mut updated_guardrail = found.clone();
    updated_guardrail.name = "Updated guardrail".into();
    updated_guardrail.enabled = false;
    updated_guardrail.enforcement = GuardrailEnforcement::Advisory;
    let updated = repo.update(&updated_guardrail).await.unwrap();
    assert_eq!(updated.name, "Updated guardrail");
    assert!(!updated.enabled);
    assert_eq!(updated.enforcement, GuardrailEnforcement::Advisory);

    // Delete
    repo.delete(ws_id, created.id).await.unwrap();
    let gone = repo.find_by_id(ws_id, created.id).await.unwrap();
    assert!(gone.is_none());

    // Cleanup
    common::cleanup_org(&pool, &org_id).await;
}

#[tokio::test]
async fn guardrail_find_enabled_by_scope() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();
    let ws_id = create_workspace(&pool, &org_id).await;
    let repo = &state.guardrail_repo;

    // Create workspace-scoped enabled guardrail
    let mut g1 = make_guardrail(ws_id);
    g1.scope = GuardrailScope::Workspace;
    g1.enabled = true;
    repo.create(&g1).await.unwrap();

    // Create session-scoped enabled guardrail
    let mut g2 = make_guardrail(ws_id);
    g2.id = Uuid::now_v7();
    g2.name = "Session boundary".into();
    g2.scope = GuardrailScope::Session;
    g2.enabled = true;
    g2.category = GuardrailCategory::Boundary;
    g2.rule = GuardrailRule::Boundary {
        description: "Stay in src".into(),
        allowed_paths: vec!["src/".into()],
        denied_paths: vec![],
    };
    repo.create(&g2).await.unwrap();

    // Create workspace-scoped disabled guardrail
    let mut g3 = make_guardrail(ws_id);
    g3.id = Uuid::now_v7();
    g3.name = "Disabled rule".into();
    g3.enabled = false;
    repo.create(&g3).await.unwrap();

    // find_enabled_by_scope(workspace) should return only g1
    let workspace_enabled = repo
        .find_enabled_by_scope(ws_id, GuardrailScope::Workspace)
        .await
        .unwrap();
    assert_eq!(workspace_enabled.len(), 1);
    assert_eq!(workspace_enabled[0].name, "No migration edits");

    // find_enabled_by_scope(session) should return only g2
    let session_enabled = repo
        .find_enabled_by_scope(ws_id, GuardrailScope::Session)
        .await
        .unwrap();
    assert_eq!(session_enabled.len(), 1);
    assert_eq!(session_enabled[0].name, "Session boundary");

    // Cleanup
    common::cleanup_org(&pool, &org_id).await;
}

#[tokio::test]
async fn guardrail_rls_isolates_across_workspaces() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_a = common::unique_org_id();
    let org_b = common::unique_org_id();
    let ws_a = create_workspace(&pool, &org_a).await;
    let ws_b = create_workspace(&pool, &org_b).await;
    let repo = &state.guardrail_repo;

    // Create guardrail in workspace A
    let guardrail = make_guardrail(ws_a);
    let created = repo.create(&guardrail).await.unwrap();

    // Should NOT be visible from workspace B
    let found = repo.find_by_id(ws_b, created.id).await.unwrap();
    assert!(
        found.is_none(),
        "RLS should isolate guardrails across workspaces"
    );

    let list_b = repo.find_all(ws_b).await.unwrap();
    assert!(
        !list_b.iter().any(|g| g.id == created.id),
        "Guardrail from ws_a should not appear in ws_b list"
    );

    common::cleanup_org(&pool, &org_a).await;
    common::cleanup_org(&pool, &org_b).await;
}

#[tokio::test]
async fn guardrail_delete_nonexistent_returns_not_found() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();
    let ws_id = create_workspace(&pool, &org_id).await;

    let result = state.guardrail_repo.delete(ws_id, Uuid::now_v7()).await;
    assert!(result.is_err());

    common::cleanup_org(&pool, &org_id).await;
}
