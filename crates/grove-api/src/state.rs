use std::sync::Arc;

use grove_domain::agent::Agent;
use grove_domain::ports::{
    CrudRepository, EventRepository, GateDefinitionRepository, GuardrailRepository,
    SessionRepository, WorkspaceRepository,
};
use sqlx::PgPool;

use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: AppConfig,
    pub workspace_repo: Arc<dyn WorkspaceRepository>,
    pub agent_repo: Arc<dyn CrudRepository<Agent>>,
    pub guardrail_repo: Arc<dyn GuardrailRepository>,
    pub session_repo: Arc<dyn SessionRepository>,
    pub gate_definition_repo: Arc<dyn GateDefinitionRepository>,
    pub event_repo: Arc<dyn EventRepository>,
}
