use grove_domain::ports::WorkspaceRepository;
use uuid::Uuid;

use crate::error::ApiError;

/// Verify workspace exists and belongs to the org.
///
/// Shared by all sub-workspace route modules to avoid duplicating
/// the lookup-or-404 pattern.
pub async fn resolve_workspace(
    workspace_repo: &dyn WorkspaceRepository,
    org_id: &str,
    ws_id: Uuid,
) -> Result<(), ApiError> {
    workspace_repo
        .find_by_id(org_id, ws_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("workspace {ws_id} not found")))?;
    Ok(())
}
