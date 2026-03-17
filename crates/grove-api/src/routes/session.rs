use axum::extract::State;
use axum::http::StatusCode;
use grove_domain::session::{Session, SessionIntent, SessionStatus, SessionTargetType};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::extract::{Json, Path, Query};
use crate::routes::helpers::resolve_workspace;
use crate::state::AppState;

#[derive(Serialize)]
pub struct SessionResponse {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub agent_id: Uuid,
    pub title: String,
    pub status: SessionStatus,
    pub intent: SessionIntent,
    pub target_type: Option<SessionTargetType>,
    pub target_id: Option<Uuid>,
    pub context: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub initiated_by: String,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<Session> for SessionResponse {
    fn from(s: Session) -> Self {
        Self {
            id: s.id,
            workspace_id: s.workspace_id,
            agent_id: s.agent_id,
            title: s.title,
            status: s.status,
            intent: s.intent,
            target_type: s.target_type,
            target_id: s.target_id,
            context: s.context,
            result: s.result,
            initiated_by: s.initiated_by,
            started_at: s.started_at,
            completed_at: s.completed_at,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateSessionRequest {
    pub agent_id: Uuid,
    pub title: String,
    pub intent: SessionIntent,
    pub target_type: Option<SessionTargetType>,
    pub target_id: Option<Uuid>,
    #[serde(default = "default_context")]
    pub context: serde_json::Value,
    pub initiated_by: String,
}

fn default_context() -> serde_json::Value {
    serde_json::json!({})
}

#[derive(Deserialize)]
pub struct UpdateSessionRequest {
    pub agent_id: Uuid,
    pub title: String,
    pub intent: SessionIntent,
    pub target_type: Option<SessionTargetType>,
    pub target_id: Option<Uuid>,
    pub context: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub initiated_by: String,
}

#[derive(Deserialize)]
pub struct StatusTransitionRequest {
    pub status: SessionStatus,
}

#[derive(Deserialize)]
pub struct ListSessionsQuery {
    pub status: Option<SessionStatus>,
}

/// GET /orgs/{org_id}/workspaces/{ws_id}/sessions
///
/// Supports optional query param: `?status=active`
pub async fn list(
    State(state): State<AppState>,
    Path((org_id, ws_id)): Path<(String, Uuid)>,
    Query(query): Query<ListSessionsQuery>,
) -> Result<axum::Json<Vec<SessionResponse>>, ApiError> {
    resolve_workspace(&*state.workspace_repo, &org_id, ws_id).await?;

    let sessions = if let Some(status) = query.status {
        state.session_repo.find_by_status(ws_id, status).await?
    } else {
        state.session_repo.find_all(ws_id).await?
    };

    Ok(axum::Json(
        sessions.into_iter().map(SessionResponse::from).collect(),
    ))
}

/// GET /orgs/{org_id}/workspaces/{ws_id}/sessions/{session_id}
pub async fn get(
    State(state): State<AppState>,
    Path((org_id, ws_id, session_id)): Path<(String, Uuid, Uuid)>,
) -> Result<axum::Json<SessionResponse>, ApiError> {
    resolve_workspace(&*state.workspace_repo, &org_id, ws_id).await?;
    let session = state
        .session_repo
        .find_by_id(ws_id, session_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("session {session_id} not found")))?;
    Ok(axum::Json(SessionResponse::from(session)))
}

/// POST /orgs/{org_id}/workspaces/{ws_id}/sessions
pub async fn create(
    State(state): State<AppState>,
    Path((org_id, ws_id)): Path<(String, Uuid)>,
    Json(body): Json<CreateSessionRequest>,
) -> Result<(StatusCode, axum::Json<SessionResponse>), ApiError> {
    resolve_workspace(&*state.workspace_repo, &org_id, ws_id).await?;

    if body.title.trim().is_empty() {
        return Err(ApiError::BadRequest("title cannot be empty".into()));
    }
    if body.initiated_by.trim().is_empty() {
        return Err(ApiError::BadRequest("initiated_by cannot be empty".into()));
    }

    let now = chrono::Utc::now();
    let session = Session::new(
        Uuid::now_v7(),
        ws_id,
        body.agent_id,
        body.title.trim().to_string(),
        SessionStatus::Pending,
        body.intent,
        body.target_type,
        body.target_id,
        body.context,
        None,
        body.initiated_by.trim().to_string(),
        None,
        None,
        now,
        now,
    )
    .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let created = state.session_repo.create(&session).await?;
    Ok((
        StatusCode::CREATED,
        axum::Json(SessionResponse::from(created)),
    ))
}

/// PUT /orgs/{org_id}/workspaces/{ws_id}/sessions/{session_id}
///
/// Full replacement — all fields required (no defaults).
pub async fn update(
    State(state): State<AppState>,
    Path((org_id, ws_id, session_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<UpdateSessionRequest>,
) -> Result<axum::Json<SessionResponse>, ApiError> {
    resolve_workspace(&*state.workspace_repo, &org_id, ws_id).await?;

    if body.title.trim().is_empty() {
        return Err(ApiError::BadRequest("title cannot be empty".into()));
    }
    if body.initiated_by.trim().is_empty() {
        return Err(ApiError::BadRequest("initiated_by cannot be empty".into()));
    }

    let existing = state
        .session_repo
        .find_by_id(ws_id, session_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("session {session_id} not found")))?;

    // Validate target_type/target_id consistency via Session::new
    let updated_session = Session::new(
        existing.id,
        existing.workspace_id,
        body.agent_id,
        body.title.trim().to_string(),
        existing.status,
        body.intent,
        body.target_type,
        body.target_id,
        body.context,
        body.result,
        body.initiated_by.trim().to_string(),
        existing.started_at,
        existing.completed_at,
        existing.created_at,
        existing.updated_at,
    )
    .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let result = state.session_repo.update(&updated_session).await?;
    Ok(axum::Json(SessionResponse::from(result)))
}

/// PATCH /orgs/{org_id}/workspaces/{ws_id}/sessions/{session_id}/status
///
/// Transitions session status using domain validation rules.
pub async fn transition_status(
    State(state): State<AppState>,
    Path((org_id, ws_id, session_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<StatusTransitionRequest>,
) -> Result<axum::Json<SessionResponse>, ApiError> {
    resolve_workspace(&*state.workspace_repo, &org_id, ws_id).await?;

    let mut session = state
        .session_repo
        .find_by_id(ws_id, session_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("session {session_id} not found")))?;

    session.transition_to(body.status)?;

    let result = state.session_repo.update(&session).await?;
    Ok(axum::Json(SessionResponse::from(result)))
}

/// DELETE /orgs/{org_id}/workspaces/{ws_id}/sessions/{session_id}
pub async fn delete(
    State(state): State<AppState>,
    Path((org_id, ws_id, session_id)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    resolve_workspace(&*state.workspace_repo, &org_id, ws_id).await?;
    state.session_repo.delete(ws_id, session_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
