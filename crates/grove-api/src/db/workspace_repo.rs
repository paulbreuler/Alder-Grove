//! Workspace persistence adapter.
//!
//! All queries run as **superuser** (no `TenantTx`). The workspace table's RLS
//! policy (`id = current_workspace_id()`) is designed for sub-workspace entity
//! isolation, not workspace-level access control.
//!
//! **SECURITY GAP (pre-auth):** Org-level isolation currently relies solely on
//! `org_id` WHERE clauses — application-level filtering only, no database
//! enforcement. A missed WHERE clause would leak cross-org data. When Clerk JWT
//! auth lands, this must be hardened with:
//! 1. Auth middleware extracts `org_id` from JWT (not URL path)
//! 2. `SET LOCAL app.current_org_id` + RLS policy on workspaces
//! 3. Defense in depth: WHERE clauses remain, RLS is the safety net

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use grove_domain::error::DomainError;
use grove_domain::ports::WorkspaceRepository;
use grove_domain::workspace::Workspace;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PgWorkspaceRepo {
    pool: PgPool,
}

impl PgWorkspaceRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Private row type — maps 1:1 to SQL columns.
/// Separated from domain Workspace to decouple DB schema from domain types.
#[derive(sqlx::FromRow)]
struct WorkspaceRow {
    id: Uuid,
    org_id: String,
    name: String,
    description: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<WorkspaceRow> for Workspace {
    fn from(row: WorkspaceRow) -> Self {
        Self {
            id: row.id,
            org_id: row.org_id,
            name: row.name,
            description: row.description,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[async_trait]
impl WorkspaceRepository for PgWorkspaceRepo {
    async fn find_all(&self, org_id: &str) -> Result<Vec<Workspace>, DomainError> {
        // Runs as superuser — workspace listing bypasses RLS
        // (workspace table RLS is id = current_workspace_id(), but we need ALL for an org)
        let rows = sqlx::query_as::<_, WorkspaceRow>(
            "SELECT id, org_id, name, description, created_at, updated_at FROM workspaces WHERE org_id = $1 ORDER BY created_at",
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(Workspace::from).collect())
    }

    async fn find_by_id(&self, org_id: &str, id: Uuid) -> Result<Option<Workspace>, DomainError> {
        let row = sqlx::query_as::<_, WorkspaceRow>(
            "SELECT id, org_id, name, description, created_at, updated_at FROM workspaces WHERE id = $1 AND org_id = $2",
        )
        .bind(id)
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(row.map(Workspace::from))
    }

    async fn create(&self, workspace: &Workspace) -> Result<Workspace, DomainError> {
        let row = sqlx::query_as::<_, WorkspaceRow>(
            "INSERT INTO workspaces (id, org_id, name, description) VALUES ($1, $2, $3, $4) RETURNING id, org_id, name, description, created_at, updated_at",
        )
        .bind(workspace.id)
        .bind(&workspace.org_id)
        .bind(&workspace.name)
        .bind(&workspace.description)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(Workspace::from(row))
    }

    async fn update(&self, workspace: &Workspace) -> Result<Workspace, DomainError> {
        let row = sqlx::query_as::<_, WorkspaceRow>(
            "UPDATE workspaces SET name = $1, description = $2 WHERE id = $3 AND org_id = $4 RETURNING id, org_id, name, description, created_at, updated_at",
        )
        .bind(&workspace.name)
        .bind(&workspace.description)
        .bind(workspace.id)
        .bind(&workspace.org_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| DomainError::NotFound {
            entity: "workspace".into(),
            id: workspace.id.to_string(),
        })?;

        Ok(Workspace::from(row))
    }

    async fn delete(&self, org_id: &str, id: Uuid) -> Result<(), DomainError> {
        let result = sqlx::query("DELETE FROM workspaces WHERE id = $1 AND org_id = $2")
            .bind(id)
            .bind(org_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound {
                entity: "workspace".into(),
                id: id.to_string(),
            });
        }

        Ok(())
    }
}
