use axum::extract::State;
use axum::http::StatusCode;
use grove_domain::event::{Event, EventCategory, EventEmitter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::extract::{Json, Path};
use crate::routes::helpers::resolve_workspace;
use crate::state::AppState;

#[derive(Serialize)]
pub struct EventResponse {
    pub id: Uuid,
    pub session_id: Uuid,
    pub workspace_id: Uuid,
    pub event_type: String,
    pub category: EventCategory,
    pub summary: String,
    pub data: serde_json::Value,
    pub emitted_by: EventEmitter,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<Event> for EventResponse {
    fn from(e: Event) -> Self {
        Self {
            id: e.id,
            session_id: e.session_id,
            workspace_id: e.workspace_id,
            event_type: e.event_type,
            category: e.category,
            summary: e.summary,
            data: e.data,
            emitted_by: e.emitted_by,
            created_at: e.created_at,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateEventRequest {
    pub event_type: String,
    pub category: EventCategory,
    pub summary: String,
    #[serde(default = "default_data")]
    pub data: serde_json::Value,
    #[serde(default = "default_emitted_by")]
    pub emitted_by: EventEmitter,
}

fn default_data() -> serde_json::Value {
    serde_json::json!({})
}

fn default_emitted_by() -> EventEmitter {
    EventEmitter::System
}

/// GET /orgs/{org_id}/workspaces/{ws_id}/sessions/{session_id}/events
pub async fn list(
    State(state): State<AppState>,
    Path((org_id, ws_id, session_id)): Path<(String, Uuid, Uuid)>,
) -> Result<axum::Json<Vec<EventResponse>>, ApiError> {
    resolve_workspace(&*state.workspace_repo, &org_id, ws_id).await?;

    // Verify session exists and belongs to this workspace
    state
        .session_repo
        .find_by_id(ws_id, session_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("session {session_id} not found")))?;

    let events = state.event_repo.find_all(session_id).await?;
    Ok(axum::Json(
        events.into_iter().map(EventResponse::from).collect(),
    ))
}

/// POST /orgs/{org_id}/workspaces/{ws_id}/sessions/{session_id}/events
pub async fn create(
    State(state): State<AppState>,
    Path((org_id, ws_id, session_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<CreateEventRequest>,
) -> Result<(StatusCode, axum::Json<EventResponse>), ApiError> {
    resolve_workspace(&*state.workspace_repo, &org_id, ws_id).await?;

    // Verify session exists and belongs to this workspace
    state
        .session_repo
        .find_by_id(ws_id, session_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("session {session_id} not found")))?;

    if body.event_type.trim().is_empty() {
        return Err(ApiError::BadRequest("event_type cannot be empty".into()));
    }
    if body.summary.trim().is_empty() {
        return Err(ApiError::BadRequest("summary cannot be empty".into()));
    }

    let event = Event {
        id: Uuid::now_v7(),
        session_id,
        workspace_id: ws_id,
        event_type: body.event_type.trim().to_string(),
        category: body.category,
        summary: body.summary.trim().to_string(),
        data: body.data,
        emitted_by: body.emitted_by,
        created_at: chrono::Utc::now(),
    };

    let created = state.event_repo.create(&event).await?;
    Ok((
        StatusCode::CREATED,
        axum::Json(EventResponse::from(created)),
    ))
}
