use axum::extract::{Query, State};
use axum::http::StatusCode;
use grove_domain::guardrail::{
    Guardrail, GuardrailCategory, GuardrailEnforcement, GuardrailRule, GuardrailScope,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::extract::{Json, Path};
use crate::state::AppState;

#[derive(Serialize)]
pub struct GuardrailResponse {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: GuardrailCategory,
    pub scope: GuardrailScope,
    pub enforcement: GuardrailEnforcement,
    pub rule: GuardrailRule,
    pub version: i32,
    pub sort_order: i32,
    pub enabled: bool,
}

impl From<Guardrail> for GuardrailResponse {
    fn from(g: Guardrail) -> Self {
        Self {
            id: g.id,
            workspace_id: g.workspace_id,
            name: g.name,
            description: g.description,
            category: g.category,
            scope: g.scope,
            enforcement: g.enforcement,
            rule: g.rule,
            version: g.version,
            sort_order: g.sort_order,
            enabled: g.enabled,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateGuardrailRequest {
    pub name: String,
    pub description: Option<String>,
    pub category: GuardrailCategory,
    #[serde(default = "default_scope")]
    pub scope: GuardrailScope,
    #[serde(default = "default_enforcement")]
    pub enforcement: GuardrailEnforcement,
    pub rule: GuardrailRule,
    #[serde(default = "default_version")]
    pub version: i32,
    #[serde(default)]
    pub sort_order: i32,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_scope() -> GuardrailScope {
    GuardrailScope::Workspace
}
fn default_enforcement() -> GuardrailEnforcement {
    GuardrailEnforcement::Enforced
}
fn default_version() -> i32 {
    1
}
fn default_enabled() -> bool {
    true
}

#[derive(Deserialize)]
pub struct UpdateGuardrailRequest {
    pub name: String,
    pub description: Option<String>,
    pub category: GuardrailCategory,
    #[serde(default = "default_scope")]
    pub scope: GuardrailScope,
    #[serde(default = "default_enforcement")]
    pub enforcement: GuardrailEnforcement,
    pub rule: GuardrailRule,
    #[serde(default = "default_version")]
    pub version: i32,
    #[serde(default)]
    pub sort_order: i32,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

#[derive(Deserialize)]
pub struct ListGuardrailsQuery {
    pub scope: Option<GuardrailScope>,
    pub enabled: Option<bool>,
}

/// Verify workspace exists and belongs to the org. Returns workspace_id.
async fn resolve_workspace(state: &AppState, org_id: &str, ws_id: Uuid) -> Result<Uuid, ApiError> {
    state
        .workspace_repo
        .find_by_id(org_id, ws_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("workspace {ws_id} not found")))?;
    Ok(ws_id)
}

/// GET /orgs/{org_id}/workspaces/{ws_id}/guardrails
///
/// Supports optional query params: `?scope=workspace&enabled=true`
/// When both scope and enabled=true are provided, uses optimized find_enabled_by_scope.
pub async fn list(
    State(state): State<AppState>,
    Path((org_id, ws_id)): Path<(String, Uuid)>,
    Query(query): Query<ListGuardrailsQuery>,
) -> Result<axum::Json<Vec<GuardrailResponse>>, ApiError> {
    let ws_id = resolve_workspace(&state, &org_id, ws_id).await?;

    let guardrails = match (query.scope, query.enabled) {
        // Optimized path: enabled + scope filter uses dedicated index
        (Some(scope), Some(true)) => {
            state
                .guardrail_repo
                .find_enabled_by_scope(ws_id, scope)
                .await?
        }
        _ => {
            let mut all = state.guardrail_repo.find_all(ws_id).await?;

            // Apply client-side filters for non-optimized combinations
            if let Some(scope) = query.scope {
                all.retain(|g| g.scope == scope);
            }
            if let Some(enabled) = query.enabled {
                all.retain(|g| g.enabled == enabled);
            }

            all
        }
    };

    Ok(axum::Json(
        guardrails
            .into_iter()
            .map(GuardrailResponse::from)
            .collect(),
    ))
}

/// GET /orgs/{org_id}/workspaces/{ws_id}/guardrails/{guardrail_id}
pub async fn get(
    State(state): State<AppState>,
    Path((org_id, ws_id, guardrail_id)): Path<(String, Uuid, Uuid)>,
) -> Result<axum::Json<GuardrailResponse>, ApiError> {
    let ws_id = resolve_workspace(&state, &org_id, ws_id).await?;
    let guardrail = state
        .guardrail_repo
        .find_by_id(ws_id, guardrail_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("guardrail {guardrail_id} not found")))?;
    Ok(axum::Json(GuardrailResponse::from(guardrail)))
}

/// POST /orgs/{org_id}/workspaces/{ws_id}/guardrails
pub async fn create(
    State(state): State<AppState>,
    Path((org_id, ws_id)): Path<(String, Uuid)>,
    Json(body): Json<CreateGuardrailRequest>,
) -> Result<(StatusCode, axum::Json<GuardrailResponse>), ApiError> {
    let ws_id = resolve_workspace(&state, &org_id, ws_id).await?;

    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name cannot be empty".into()));
    }

    let guardrail = Guardrail {
        id: Uuid::now_v7(),
        workspace_id: ws_id,
        name: body.name.trim().to_string(),
        description: body.description,
        category: body.category,
        scope: body.scope,
        enforcement: body.enforcement,
        rule: body.rule,
        version: body.version,
        sort_order: body.sort_order,
        enabled: body.enabled,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let created = state.guardrail_repo.create(&guardrail).await?;
    Ok((
        StatusCode::CREATED,
        axum::Json(GuardrailResponse::from(created)),
    ))
}

/// PUT /orgs/{org_id}/workspaces/{ws_id}/guardrails/{guardrail_id}
pub async fn update(
    State(state): State<AppState>,
    Path((org_id, ws_id, guardrail_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<UpdateGuardrailRequest>,
) -> Result<axum::Json<GuardrailResponse>, ApiError> {
    let ws_id = resolve_workspace(&state, &org_id, ws_id).await?;

    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name cannot be empty".into()));
    }

    let existing = state
        .guardrail_repo
        .find_by_id(ws_id, guardrail_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("guardrail {guardrail_id} not found")))?;

    let updated_guardrail = Guardrail {
        name: body.name.trim().to_string(),
        description: body.description,
        category: body.category,
        scope: body.scope,
        enforcement: body.enforcement,
        rule: body.rule,
        version: body.version,
        sort_order: body.sort_order,
        enabled: body.enabled,
        ..existing
    };

    let result = state.guardrail_repo.update(&updated_guardrail).await?;
    Ok(axum::Json(GuardrailResponse::from(result)))
}

/// DELETE /orgs/{org_id}/workspaces/{ws_id}/guardrails/{guardrail_id}
pub async fn delete(
    State(state): State<AppState>,
    Path((org_id, ws_id, guardrail_id)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let ws_id = resolve_workspace(&state, &org_id, ws_id).await?;
    state.guardrail_repo.delete(ws_id, guardrail_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
