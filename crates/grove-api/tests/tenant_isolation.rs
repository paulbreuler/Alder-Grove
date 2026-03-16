mod common;

use grove_api::db::tenant::TenantTx;
use uuid::Uuid;

#[tokio::test]
async fn tenant_tx_isolates_data_via_rls() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_a = common::unique_org_id();
    let org_b = common::unique_org_id();

    // Create two workspaces as superuser
    let ws_a: (Uuid,) =
        sqlx::query_as("INSERT INTO workspaces (org_id, name) VALUES ($1, 'WS-A') RETURNING id")
            .bind(&org_a)
            .fetch_one(&pool)
            .await
            .unwrap();

    let ws_b: (Uuid,) =
        sqlx::query_as("INSERT INTO workspaces (org_id, name) VALUES ($1, 'WS-B') RETURNING id")
            .bind(&org_b)
            .fetch_one(&pool)
            .await
            .unwrap();

    // Insert personas as superuser
    sqlx::query("INSERT INTO personas (workspace_id, name) VALUES ($1, 'Dev')")
        .bind(ws_a.0)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO personas (workspace_id, name) VALUES ($1, 'Designer')")
        .bind(ws_b.0)
        .execute(&pool)
        .await
        .unwrap();

    // Query within TenantTx for workspace A — should only see 'Dev'
    let mut tx_a = TenantTx::begin(&pool, ws_a.0).await.unwrap();
    let rows: Vec<(String,)> = sqlx::query_as("SELECT name FROM personas")
        .fetch_all(tx_a.conn())
        .await
        .unwrap();
    tx_a.commit().await.unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, "Dev");

    // Query within TenantTx for workspace B — should only see 'Designer'
    let mut tx_b = TenantTx::begin(&pool, ws_b.0).await.unwrap();
    let rows: Vec<(String,)> = sqlx::query_as("SELECT name FROM personas")
        .fetch_all(tx_b.conn())
        .await
        .unwrap();
    tx_b.commit().await.unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, "Designer");

    // Cleanup
    common::cleanup_org(&pool, &org_a).await;
    common::cleanup_org(&pool, &org_b).await;
}

#[tokio::test]
async fn tenant_tx_blocks_cross_workspace_insert() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_a = common::unique_org_id();
    let org_b = common::unique_org_id();

    let ws_a: (Uuid,) =
        sqlx::query_as("INSERT INTO workspaces (org_id, name) VALUES ($1, 'WS-A') RETURNING id")
            .bind(&org_a)
            .fetch_one(&pool)
            .await
            .unwrap();

    let ws_b: (Uuid,) =
        sqlx::query_as("INSERT INTO workspaces (org_id, name) VALUES ($1, 'WS-B') RETURNING id")
            .bind(&org_b)
            .fetch_one(&pool)
            .await
            .unwrap();

    // Try inserting into workspace B while scoped to workspace A
    let mut tx = TenantTx::begin(&pool, ws_a.0).await.unwrap();
    let result = sqlx::query("INSERT INTO personas (workspace_id, name) VALUES ($1, 'Hacker')")
        .bind(ws_b.0)
        .execute(tx.conn())
        .await;

    assert!(result.is_err(), "RLS should block cross-workspace insert");

    // Cleanup (tx auto-rolled back on drop)
    common::cleanup_org(&pool, &org_a).await;
    common::cleanup_org(&pool, &org_b).await;
}
