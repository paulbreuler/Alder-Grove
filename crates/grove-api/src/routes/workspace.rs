use axum::extract::State;
use axum::http::StatusCode;
use grove_domain::workspace::Workspace;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::extract::{Json, Path};
use crate::state::AppState;

#[derive(Serialize)]
pub struct WorkspaceResponse {
    pub id: Uuid,
    pub org_id: String,
    pub name: String,
    pub description: Option<String>,
}

impl From<Workspace> for WorkspaceResponse {
    fn from(ws: Workspace) -> Self {
        Self {
            id: ws.id,
            org_id: ws.org_id,
            name: ws.name,
            description: ws.description,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateWorkspaceRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateWorkspaceRequest {
    pub name: String,
    pub description: Option<String>,
}

/// GET /orgs/{org_id}/workspaces
pub async fn list(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<axum::Json<Vec<WorkspaceResponse>>, ApiError> {
    let workspaces = state.workspace_repo.find_all(&org_id).await?;
    Ok(axum::Json(workspaces.into_iter().map(WorkspaceResponse::from).collect()))
}

/// GET /orgs/{org_id}/workspaces/{ws_id}
pub async fn get(
    State(state): State<AppState>,
    Path((org_id, ws_id)): Path<(String, Uuid)>,
) -> Result<axum::Json<WorkspaceResponse>, ApiError> {
    let ws = state
        .workspace_repo
        .find_by_id(&org_id, ws_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("workspace {ws_id} not found")))?;
    Ok(axum::Json(WorkspaceResponse::from(ws)))
}

/// POST /orgs/{org_id}/workspaces
pub async fn create(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    Json(body): Json<CreateWorkspaceRequest>,
) -> Result<(StatusCode, axum::Json<WorkspaceResponse>), ApiError> {
    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name cannot be empty".into()));
    }

    let workspace = Workspace {
        id: Uuid::now_v7(),
        org_id,
        name: body.name.trim().to_string(),
        description: body.description,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let created = state.workspace_repo.create(&workspace).await?;
    Ok((StatusCode::CREATED, axum::Json(WorkspaceResponse::from(created))))
}

/// PUT /orgs/{org_id}/workspaces/{ws_id}
pub async fn update(
    State(state): State<AppState>,
    Path((org_id, ws_id)): Path<(String, Uuid)>,
    Json(body): Json<UpdateWorkspaceRequest>,
) -> Result<axum::Json<WorkspaceResponse>, ApiError> {
    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name cannot be empty".into()));
    }

    let existing = state
        .workspace_repo
        .find_by_id(&org_id, ws_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("workspace {ws_id} not found")))?;

    let updated_ws = Workspace {
        name: body.name.trim().to_string(),
        description: body.description,
        ..existing
    };

    let result = state.workspace_repo.update(&updated_ws).await?;
    Ok(axum::Json(WorkspaceResponse::from(result)))
}

/// DELETE /orgs/{org_id}/workspaces/{ws_id}
pub async fn delete(
    State(state): State<AppState>,
    Path((org_id, ws_id)): Path<(String, Uuid)>,
) -> Result<StatusCode, ApiError> {
    state.workspace_repo.delete(&org_id, ws_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
