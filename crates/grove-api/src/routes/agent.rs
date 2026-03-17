use axum::extract::State;
use axum::http::StatusCode;
use grove_domain::agent::{Agent, AgentStatus};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::extract::{Json, Path};
use crate::routes::helpers::resolve_workspace;
use crate::state::AppState;

#[derive(Serialize)]
pub struct AgentResponse {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub provider: String,
    pub model: Option<String>,
    pub description: Option<String>,
    pub capabilities: Vec<String>,
    pub config: serde_json::Value,
    pub status: AgentStatus,
}

impl From<Agent> for AgentResponse {
    fn from(a: Agent) -> Self {
        Self {
            id: a.id,
            workspace_id: a.workspace_id,
            name: a.name,
            provider: a.provider,
            model: a.model,
            description: a.description,
            capabilities: a.capabilities,
            config: a.config,
            status: a.status,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub provider: String,
    pub model: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default = "default_config")]
    pub config: serde_json::Value,
    #[serde(default = "default_active")]
    pub status: AgentStatus,
}

fn default_config() -> serde_json::Value {
    serde_json::json!({})
}

fn default_active() -> AgentStatus {
    AgentStatus::Active
}

#[derive(Deserialize)]
pub struct UpdateAgentRequest {
    pub name: String,
    pub provider: String,
    pub model: Option<String>,
    pub description: Option<String>,
    pub capabilities: Vec<String>,
    pub config: serde_json::Value,
    pub status: AgentStatus,
}

/// GET /orgs/{org_id}/workspaces/{ws_id}/agents
pub async fn list(
    State(state): State<AppState>,
    Path((org_id, ws_id)): Path<(String, Uuid)>,
) -> Result<axum::Json<Vec<AgentResponse>>, ApiError> {
    resolve_workspace(&*state.workspace_repo, &org_id, ws_id).await?;
    let agents = state.agent_repo.find_all(ws_id).await?;
    Ok(axum::Json(
        agents.into_iter().map(AgentResponse::from).collect(),
    ))
}

/// GET /orgs/{org_id}/workspaces/{ws_id}/agents/{agent_id}
pub async fn get(
    State(state): State<AppState>,
    Path((org_id, ws_id, agent_id)): Path<(String, Uuid, Uuid)>,
) -> Result<axum::Json<AgentResponse>, ApiError> {
    resolve_workspace(&*state.workspace_repo, &org_id, ws_id).await?;
    let agent = state
        .agent_repo
        .find_by_id(ws_id, agent_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("agent {agent_id} not found")))?;
    Ok(axum::Json(AgentResponse::from(agent)))
}

/// POST /orgs/{org_id}/workspaces/{ws_id}/agents
pub async fn create(
    State(state): State<AppState>,
    Path((org_id, ws_id)): Path<(String, Uuid)>,
    Json(body): Json<CreateAgentRequest>,
) -> Result<(StatusCode, axum::Json<AgentResponse>), ApiError> {
    resolve_workspace(&*state.workspace_repo, &org_id, ws_id).await?;

    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name cannot be empty".into()));
    }
    if body.provider.trim().is_empty() {
        return Err(ApiError::BadRequest("provider cannot be empty".into()));
    }

    let agent = Agent {
        id: Uuid::now_v7(),
        workspace_id: ws_id,
        name: body.name.trim().to_string(),
        provider: body.provider.trim().to_string(),
        model: body.model,
        description: body.description,
        capabilities: body.capabilities,
        config: body.config,
        status: body.status,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let created = state.agent_repo.create(&agent).await?;
    Ok((
        StatusCode::CREATED,
        axum::Json(AgentResponse::from(created)),
    ))
}

/// PUT /orgs/{org_id}/workspaces/{ws_id}/agents/{agent_id}
pub async fn update(
    State(state): State<AppState>,
    Path((org_id, ws_id, agent_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<UpdateAgentRequest>,
) -> Result<axum::Json<AgentResponse>, ApiError> {
    resolve_workspace(&*state.workspace_repo, &org_id, ws_id).await?;

    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name cannot be empty".into()));
    }
    if body.provider.trim().is_empty() {
        return Err(ApiError::BadRequest("provider cannot be empty".into()));
    }

    let existing = state
        .agent_repo
        .find_by_id(ws_id, agent_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("agent {agent_id} not found")))?;

    let updated_agent = Agent {
        name: body.name.trim().to_string(),
        provider: body.provider.trim().to_string(),
        model: body.model,
        description: body.description,
        capabilities: body.capabilities,
        config: body.config,
        status: body.status,
        ..existing
    };

    let result = state.agent_repo.update(&updated_agent).await?;
    Ok(axum::Json(AgentResponse::from(result)))
}

/// DELETE /orgs/{org_id}/workspaces/{ws_id}/agents/{agent_id}
pub async fn delete(
    State(state): State<AppState>,
    Path((org_id, ws_id, agent_id)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    resolve_workspace(&*state.workspace_repo, &org_id, ws_id).await?;
    state.agent_repo.delete(ws_id, agent_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
