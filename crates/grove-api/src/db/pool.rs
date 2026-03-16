use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

/// Create a PgPool with after_release safety callback.
/// Resets tenant context when connections return to the pool,
/// preventing leaked workspace_id if a query runs outside a transaction.
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .after_release(|conn, _meta| {
            Box::pin(async move {
                // Split into separate statements — prepared statements
                // don't allow multiple commands in a single query.
                sqlx::query("RESET ROLE")
                    .execute(&mut *conn)
                    .await?;
                sqlx::query("RESET app.current_workspace_id")
                    .execute(&mut *conn)
                    .await?;
                Ok(true)
            })
        })
        .connect(database_url)
        .await
}
