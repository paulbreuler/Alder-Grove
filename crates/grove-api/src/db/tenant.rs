use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::error::ApiError;

/// A database transaction with RLS tenant context applied.
///
/// Uses SET LOCAL (transaction-scoped) for both ROLE and workspace_id.
/// Context is automatically cleared on COMMIT or ROLLBACK.
///
/// Usage:
/// ```ignore
/// let mut tx = TenantTx::begin(&pool, workspace_id).await?;
/// let rows = sqlx::query("SELECT * FROM personas")
///     .fetch_all(tx.conn())
///     .await?;
/// tx.commit().await?;
/// ```
pub struct TenantTx {
    tx: Transaction<'static, Postgres>,
}

impl TenantTx {
    /// Begin a transaction with tenant context.
    /// Sets ROLE to grove_app (RLS-enforced) and workspace_id for policies.
    pub async fn begin(pool: &PgPool, workspace_id: Uuid) -> Result<Self, ApiError> {
        let mut tx = pool.begin().await.map_err(ApiError::internal)?;

        sqlx::query("SET LOCAL ROLE grove_app")
            .execute(&mut *tx)
            .await
            .map_err(ApiError::internal)?;

        // SET doesn't support parameterized queries — format directly.
        // Safe: workspace_id is a validated Uuid (no SQL injection possible).
        sqlx::query(&format!(
            "SET LOCAL app.current_workspace_id = '{workspace_id}'"
        ))
        .execute(&mut *tx)
        .await
        .map_err(ApiError::internal)?;

        Ok(Self { tx })
    }

    /// Get a mutable reference to the underlying connection for queries.
    pub fn conn(&mut self) -> &mut sqlx::PgConnection {
        &mut self.tx
    }

    /// Commit the transaction. Context is cleared automatically.
    pub async fn commit(self) -> Result<(), ApiError> {
        self.tx.commit().await.map_err(ApiError::internal)
    }
}
