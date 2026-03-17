use axum::extract::State;
use axum::http::StatusCode;
use grove_domain::gate::{ApprovalType, GateDefinition, TimeoutAction, TriggerType};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::extract::{Json, Path, Query};
use crate::routes::helpers::resolve_workspace;
use crate::state::AppState;

#[derive(Serialize)]
pub struct GateDefinitionResponse {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: TriggerType,
    pub trigger_config: serde_json::Value,
    pub approval_type: ApprovalType,
    pub timeout_minutes: Option<i32>,
    pub timeout_action: TimeoutAction,
    pub enabled: bool,
    pub sort_order: i32,
}

impl From<GateDefinition> for GateDefinitionResponse {
    fn from(gd: GateDefinition) -> Self {
        Self {
            id: gd.id,
            workspace_id: gd.workspace_id,
            name: gd.name,
            description: gd.description,
            trigger_type: gd.trigger_type,
            trigger_config: gd.trigger_config,
            approval_type: gd.approval_type,
            timeout_minutes: gd.timeout_minutes,
            timeout_action: gd.timeout_action,
            enabled: gd.enabled,
            sort_order: gd.sort_order,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateGateDefinitionRequest {
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: TriggerType,
    #[serde(default = "default_trigger_config")]
    pub trigger_config: serde_json::Value,
    #[serde(default = "default_approval_type")]
    pub approval_type: ApprovalType,
    #[serde(default = "default_timeout_minutes")]
    pub timeout_minutes: Option<i32>,
    #[serde(default = "default_timeout_action")]
    pub timeout_action: TimeoutAction,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub sort_order: i32,
}

fn default_trigger_config() -> serde_json::Value {
    serde_json::json!({})
}
fn default_approval_type() -> ApprovalType {
    ApprovalType::Single
}
fn default_timeout_minutes() -> Option<i32> {
    Some(60)
}
fn default_timeout_action() -> TimeoutAction {
    TimeoutAction::Cancel
}
fn default_enabled() -> bool {
    true
}

#[derive(Deserialize)]
pub struct UpdateGateDefinitionRequest {
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: TriggerType,
    pub trigger_config: serde_json::Value,
    pub approval_type: ApprovalType,
    pub timeout_minutes: Option<i32>,
    pub timeout_action: TimeoutAction,
    pub enabled: bool,
    pub sort_order: i32,
}

#[derive(Deserialize)]
pub struct ListGateDefinitionsQuery {
    pub enabled: Option<bool>,
}

/// GET /orgs/{org_id}/workspaces/{ws_id}/gate-definitions
///
/// Supports optional query param: `?enabled=true`
pub async fn list(
    State(state): State<AppState>,
    Path((org_id, ws_id)): Path<(String, Uuid)>,
    Query(query): Query<ListGateDefinitionsQuery>,
) -> Result<axum::Json<Vec<GateDefinitionResponse>>, ApiError> {
    resolve_workspace(&*state.workspace_repo, &org_id, ws_id).await?;

    let gate_defs = if query.enabled == Some(true) {
        state.gate_definition_repo.find_enabled(ws_id).await?
    } else {
        state.gate_definition_repo.find_all(ws_id).await?
    };

    Ok(axum::Json(
        gate_defs
            .into_iter()
            .map(GateDefinitionResponse::from)
            .collect(),
    ))
}

/// GET /orgs/{org_id}/workspaces/{ws_id}/gate-definitions/{gate_def_id}
pub async fn get(
    State(state): State<AppState>,
    Path((org_id, ws_id, gate_def_id)): Path<(String, Uuid, Uuid)>,
) -> Result<axum::Json<GateDefinitionResponse>, ApiError> {
    resolve_workspace(&*state.workspace_repo, &org_id, ws_id).await?;
    let gd = state
        .gate_definition_repo
        .find_by_id(ws_id, gate_def_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("gate definition {gate_def_id} not found")))?;
    Ok(axum::Json(GateDefinitionResponse::from(gd)))
}

/// POST /orgs/{org_id}/workspaces/{ws_id}/gate-definitions
pub async fn create(
    State(state): State<AppState>,
    Path((org_id, ws_id)): Path<(String, Uuid)>,
    Json(body): Json<CreateGateDefinitionRequest>,
) -> Result<(StatusCode, axum::Json<GateDefinitionResponse>), ApiError> {
    resolve_workspace(&*state.workspace_repo, &org_id, ws_id).await?;

    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name cannot be empty".into()));
    }

    let gd = GateDefinition {
        id: Uuid::now_v7(),
        workspace_id: ws_id,
        name: body.name.trim().to_string(),
        description: body.description,
        trigger_type: body.trigger_type,
        trigger_config: body.trigger_config,
        approval_type: body.approval_type,
        timeout_minutes: body.timeout_minutes,
        timeout_action: body.timeout_action,
        enabled: body.enabled,
        sort_order: body.sort_order,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let created = state.gate_definition_repo.create(&gd).await?;
    Ok((
        StatusCode::CREATED,
        axum::Json(GateDefinitionResponse::from(created)),
    ))
}

/// PUT /orgs/{org_id}/workspaces/{ws_id}/gate-definitions/{gate_def_id}
pub async fn update(
    State(state): State<AppState>,
    Path((org_id, ws_id, gate_def_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<UpdateGateDefinitionRequest>,
) -> Result<axum::Json<GateDefinitionResponse>, ApiError> {
    resolve_workspace(&*state.workspace_repo, &org_id, ws_id).await?;

    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name cannot be empty".into()));
    }

    let existing = state
        .gate_definition_repo
        .find_by_id(ws_id, gate_def_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("gate definition {gate_def_id} not found")))?;

    let updated_gd = GateDefinition {
        name: body.name.trim().to_string(),
        description: body.description,
        trigger_type: body.trigger_type,
        trigger_config: body.trigger_config,
        approval_type: body.approval_type,
        timeout_minutes: body.timeout_minutes,
        timeout_action: body.timeout_action,
        enabled: body.enabled,
        sort_order: body.sort_order,
        ..existing
    };

    let result = state.gate_definition_repo.update(&updated_gd).await?;
    Ok(axum::Json(GateDefinitionResponse::from(result)))
}

/// DELETE /orgs/{org_id}/workspaces/{ws_id}/gate-definitions/{gate_def_id}
pub async fn delete(
    State(state): State<AppState>,
    Path((org_id, ws_id, gate_def_id)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    resolve_workspace(&*state.workspace_repo, &org_id, ws_id).await?;
    state
        .gate_definition_repo
        .delete(ws_id, gate_def_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
