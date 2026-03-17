mod common;

use grove_domain::gate::{ApprovalType, GateDefinition, TimeoutAction, TriggerType};
use uuid::Uuid;

/// Helper: create a workspace and return its id.
async fn create_workspace(pool: &sqlx::PgPool, org_id: &str) -> Uuid {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO workspaces (org_id, name) VALUES ($1, 'gatedef-test-ws') RETURNING id",
    )
    .bind(org_id)
    .fetch_one(pool)
    .await
    .unwrap();
    row.0
}

fn make_gate_definition(workspace_id: Uuid) -> GateDefinition {
    GateDefinition {
        id: Uuid::now_v7(),
        workspace_id,
        name: "Code Deletion Review".into(),
        description: Some("Requires approval before deleting code".into()),
        trigger_type: TriggerType::Automatic,
        trigger_config: serde_json::json!({
            "patterns": [{"event_type": "file_delete", "match": "**/*.rs"}]
        }),
        approval_type: ApprovalType::Single,
        timeout_minutes: Some(60),
        timeout_action: TimeoutAction::Cancel,
        enabled: true,
        sort_order: 0,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

#[tokio::test]
async fn gate_definition_crud_lifecycle() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();
    let ws_id = create_workspace(&pool, &org_id).await;
    let repo = &state.gate_definition_repo;

    // Create
    let gd = make_gate_definition(ws_id);
    let created = repo.create(&gd).await.unwrap();
    assert_eq!(created.name, "Code Deletion Review");
    assert_eq!(created.trigger_type, TriggerType::Automatic);
    assert_eq!(created.approval_type, ApprovalType::Single);
    assert_eq!(created.timeout_action, TimeoutAction::Cancel);
    assert!(created.enabled);
    assert_eq!(created.workspace_id, ws_id);

    // Read
    let found = repo.find_by_id(ws_id, created.id).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.id, created.id);
    assert_eq!(found.name, "Code Deletion Review");

    // List
    let all = repo.find_all(ws_id).await.unwrap();
    assert!(all.iter().any(|g| g.id == created.id));

    // Update
    let updated_gd = GateDefinition {
        name: "Updated Gate Def".into(),
        description: Some("Updated description".into()),
        trigger_type: TriggerType::Manual,
        approval_type: ApprovalType::AllOf,
        timeout_action: TimeoutAction::Escalate,
        enabled: false,
        ..found
    };
    let updated = repo.update(&updated_gd).await.unwrap();
    assert_eq!(updated.name, "Updated Gate Def");
    assert_eq!(updated.trigger_type, TriggerType::Manual);
    assert_eq!(updated.approval_type, ApprovalType::AllOf);
    assert_eq!(updated.timeout_action, TimeoutAction::Escalate);
    assert!(!updated.enabled);

    // Delete
    repo.delete(ws_id, created.id).await.unwrap();
    let gone = repo.find_by_id(ws_id, created.id).await.unwrap();
    assert!(gone.is_none());

    // Cleanup
    common::cleanup_org(&pool, &org_id).await;
}

#[tokio::test]
async fn gate_definition_find_enabled() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();
    let ws_id = create_workspace(&pool, &org_id).await;
    let repo = &state.gate_definition_repo;

    // Create an enabled gate definition
    let g1 = make_gate_definition(ws_id);
    repo.create(&g1).await.unwrap();

    // Create a disabled gate definition
    let mut g2 = make_gate_definition(ws_id);
    g2.id = Uuid::now_v7();
    g2.name = "Disabled Gate".into();
    g2.enabled = false;
    repo.create(&g2).await.unwrap();

    // find_enabled should return only g1
    let enabled = repo.find_enabled(ws_id).await.unwrap();
    assert_eq!(enabled.len(), 1);
    assert_eq!(enabled[0].name, "Code Deletion Review");

    // Cleanup
    common::cleanup_org(&pool, &org_id).await;
}

#[tokio::test]
async fn gate_definition_rls_isolates_across_workspaces() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_a = common::unique_org_id();
    let org_b = common::unique_org_id();
    let ws_a = create_workspace(&pool, &org_a).await;
    let ws_b = create_workspace(&pool, &org_b).await;
    let repo = &state.gate_definition_repo;

    // Create gate definition in workspace A
    let gd = make_gate_definition(ws_a);
    let created = repo.create(&gd).await.unwrap();

    // Should NOT be visible from workspace B
    let found = repo.find_by_id(ws_b, created.id).await.unwrap();
    assert!(
        found.is_none(),
        "RLS should isolate gate definitions across workspaces"
    );

    let list_b = repo.find_all(ws_b).await.unwrap();
    assert!(
        !list_b.iter().any(|g| g.id == created.id),
        "Gate definition from ws_a should not appear in ws_b list"
    );

    common::cleanup_org(&pool, &org_a).await;
    common::cleanup_org(&pool, &org_b).await;
}

#[tokio::test]
async fn gate_definition_find_by_id_nonexistent_returns_none() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();
    let ws_id = create_workspace(&pool, &org_id).await;

    let result = state
        .gate_definition_repo
        .find_by_id(ws_id, Uuid::now_v7())
        .await
        .unwrap();
    assert!(result.is_none());

    common::cleanup_org(&pool, &org_id).await;
}

#[tokio::test]
async fn gate_definition_filter_by_disabled() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();
    let ws_id = create_workspace(&pool, &org_id).await;
    let repo = &state.gate_definition_repo;

    // Create an enabled gate definition
    let g1 = make_gate_definition(ws_id);
    repo.create(&g1).await.unwrap();

    // Create a disabled gate definition
    let mut g2 = make_gate_definition(ws_id);
    g2.id = Uuid::now_v7();
    g2.name = "Disabled Gate".into();
    g2.enabled = false;
    repo.create(&g2).await.unwrap();

    // find_disabled should return only g2
    let disabled = repo.find_disabled(ws_id).await.unwrap();
    assert_eq!(disabled.len(), 1);
    assert_eq!(disabled[0].name, "Disabled Gate");
    assert!(!disabled[0].enabled);

    // Cleanup
    common::cleanup_org(&pool, &org_id).await;
}

#[tokio::test]
async fn gate_definition_delete_nonexistent_returns_not_found() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();
    let ws_id = create_workspace(&pool, &org_id).await;

    let result = state
        .gate_definition_repo
        .delete(ws_id, Uuid::now_v7())
        .await;
    assert!(result.is_err());

    common::cleanup_org(&pool, &org_id).await;
}
