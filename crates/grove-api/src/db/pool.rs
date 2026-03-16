use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

/// Create a PgPool with after_release safety callback.
///
/// Defense-in-depth: resets tenant context when connections return to the pool.
/// Since TenantTx uses SET LOCAL (transaction-scoped), context is already cleared
/// on commit/rollback. This callback guards against any future code path that
/// might use session-scoped SET without a transaction.
///
/// Tradeoff: adds two SQL round-trips per connection release. Acceptable at
/// current scale; revisit if pool throughput becomes a bottleneck.
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .after_release(|conn, _meta| {
            Box::pin(async move {
                // Split into separate statements — prepared statements
                // don't allow multiple commands in a single query.
                sqlx::query("RESET ROLE").execute(&mut *conn).await?;
                sqlx::query("RESET app.current_workspace_id")
                    .execute(&mut *conn)
                    .await?;
                Ok(true)
            })
        })
        .connect(database_url)
        .await
}
