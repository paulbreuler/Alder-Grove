use std::sync::Arc;

use grove_api::config::AppConfig;
use grove_api::db::agent_repo::PgAgentRepo;
use grove_api::db::guardrail_repo::PgGuardrailRepo;
use grove_api::db::pool::create_pool;
use grove_api::db::workspace_repo::PgWorkspaceRepo;
use grove_api::state::AppState;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = AppConfig::from_env();
    let pool = create_pool(&config.database_url).await?;

    let state = AppState {
        workspace_repo: Arc::new(PgWorkspaceRepo::new(pool.clone())),
        agent_repo: Arc::new(PgAgentRepo::new(pool.clone())),
        guardrail_repo: Arc::new(PgGuardrailRepo::new(pool.clone())),
        pool,
        config: config.clone(),
    };

    let app = grove_api::create_app(state);
    let addr = format!("127.0.0.1:{}", config.port);

    tracing::info!("grove-api listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
