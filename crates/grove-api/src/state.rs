use std::sync::Arc;

use grove_domain::ports::WorkspaceRepository;
use sqlx::PgPool;

use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: AppConfig,
    pub workspace_repo: Arc<dyn WorkspaceRepository>,
}
