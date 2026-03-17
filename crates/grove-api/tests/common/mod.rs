// Each integration test file compiles as its own binary crate, so
// helpers used by some tests but not others trigger false dead_code warnings.
#![allow(dead_code)]

use std::sync::Arc;

use grove_api::config::AppConfig;
use grove_api::db::agent_repo::PgAgentRepo;
use grove_api::db::guardrail_repo::PgGuardrailRepo;
use grove_api::db::pool::create_pool;
use grove_api::db::workspace_repo::PgWorkspaceRepo;
use grove_api::state::AppState;
use sqlx::PgPool;

pub async fn test_state() -> AppState {
    let config = AppConfig::from_env();
    let pool = create_pool(&config.database_url)
        .await
        .expect("failed to connect to test database");

    run_migrations(&pool).await;

    AppState {
        workspace_repo: Arc::new(PgWorkspaceRepo::new(pool.clone())),
        agent_repo: Arc::new(PgAgentRepo::new(pool.clone())),
        guardrail_repo: Arc::new(PgGuardrailRepo::new(pool.clone())),
        pool,
        config,
    }
}

async fn run_migrations(pool: &PgPool) {
    let migrations: &[&str] = &[
        include_str!("../../migrations/001_initial_schema.sql"),
        include_str!("../../migrations/002_acp_schema.sql"),
        include_str!("../../migrations/003_collaborative_documents.sql"),
    ];

    for sql in migrations {
        match sqlx::raw_sql(sql).execute(pool).await {
            Ok(_) => {}
            Err(e) => {
                let msg = e.to_string();
                // Ignore "already exists" errors from re-running migrations
                if !msg.contains("already exists") && !msg.contains("duplicate") {
                    panic!("migration failed: {msg}");
                }
            }
        }
    }
}

pub fn unique_org_id() -> String {
    format!("test_org_{}", uuid::Uuid::now_v7())
}

pub async fn cleanup_org(pool: &PgPool, org_id: &str) {
    sqlx::query("DELETE FROM workspaces WHERE org_id = $1")
        .bind(org_id)
        .execute(pool)
        .await
        .ok();
}
