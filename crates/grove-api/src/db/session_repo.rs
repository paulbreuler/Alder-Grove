//! Session persistence adapter.
//!
//! All queries use `TenantTx` for RLS-scoped workspace isolation.
//! The sessions table has `workspace_id = current_workspace_id()` RLS policy.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use grove_domain::error::DomainError;
use grove_domain::ports::{CrudRepository, SessionRepository};
use grove_domain::session::{Session, SessionIntent, SessionStatus, SessionTargetType};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::tenant::TenantTx;

pub struct PgSessionRepo {
    pool: PgPool,
}

impl PgSessionRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Private row type -- maps 1:1 to SQL columns.
/// Separated from domain Session to decouple DB schema from domain types.
#[derive(sqlx::FromRow)]
struct SessionRow {
    id: Uuid,
    workspace_id: Uuid,
    agent_id: Uuid,
    title: String,
    status: String,
    intent: String,
    target_type: Option<String>,
    target_id: Option<Uuid>,
    context: serde_json::Value,
    result: Option<serde_json::Value>,
    initiated_by: String,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

fn parse_status(s: &str) -> Result<SessionStatus, DomainError> {
    match s {
        "pending" => Ok(SessionStatus::Pending),
        "active" => Ok(SessionStatus::Active),
        "completed" => Ok(SessionStatus::Completed),
        "failed" => Ok(SessionStatus::Failed),
        "cancelled" => Ok(SessionStatus::Cancelled),
        "gated" => Ok(SessionStatus::Gated),
        "timed_out" => Ok(SessionStatus::TimedOut),
        other => Err(DomainError::Internal(format!(
            "invalid session status: {other}"
        ))),
    }
}

fn parse_intent(s: &str) -> Result<SessionIntent, DomainError> {
    match s {
        "implement" => Ok(SessionIntent::Implement),
        "review" => Ok(SessionIntent::Review),
        "assess" => Ok(SessionIntent::Assess),
        "analyze" => Ok(SessionIntent::Analyze),
        "author" => Ok(SessionIntent::Author),
        "execute" => Ok(SessionIntent::Execute),
        other => Err(DomainError::Internal(format!(
            "invalid session intent: {other}"
        ))),
    }
}

fn parse_target_type(s: &str) -> Result<SessionTargetType, DomainError> {
    match s {
        "specification" => Ok(SessionTargetType::Specification),
        "task" => Ok(SessionTargetType::Task),
        "journey" => Ok(SessionTargetType::Journey),
        "step" => Ok(SessionTargetType::Step),
        "snapshot" => Ok(SessionTargetType::Snapshot),
        "repository" => Ok(SessionTargetType::Repository),
        other => Err(DomainError::Internal(format!(
            "invalid session target_type: {other}"
        ))),
    }
}

fn status_to_str(s: &SessionStatus) -> &'static str {
    match s {
        SessionStatus::Pending => "pending",
        SessionStatus::Active => "active",
        SessionStatus::Completed => "completed",
        SessionStatus::Failed => "failed",
        SessionStatus::Cancelled => "cancelled",
        SessionStatus::Gated => "gated",
        SessionStatus::TimedOut => "timed_out",
    }
}

fn intent_to_str(i: &SessionIntent) -> &'static str {
    match i {
        SessionIntent::Implement => "implement",
        SessionIntent::Review => "review",
        SessionIntent::Assess => "assess",
        SessionIntent::Analyze => "analyze",
        SessionIntent::Author => "author",
        SessionIntent::Execute => "execute",
    }
}

fn target_type_to_str(t: &SessionTargetType) -> &'static str {
    match t {
        SessionTargetType::Specification => "specification",
        SessionTargetType::Task => "task",
        SessionTargetType::Journey => "journey",
        SessionTargetType::Step => "step",
        SessionTargetType::Snapshot => "snapshot",
        SessionTargetType::Repository => "repository",
    }
}

impl TryFrom<SessionRow> for Session {
    type Error = DomainError;

    fn try_from(row: SessionRow) -> Result<Self, Self::Error> {
        let target_type = row
            .target_type
            .as_deref()
            .map(parse_target_type)
            .transpose()?;

        Session::new(
            row.id,
            row.workspace_id,
            row.agent_id,
            row.title,
            parse_status(&row.status)?,
            parse_intent(&row.intent)?,
            target_type,
            row.target_id,
            row.context,
            row.result,
            row.initiated_by,
            row.started_at,
            row.completed_at,
            row.created_at,
            row.updated_at,
        )
    }
}

const SELECT_COLS: &str = "\
    id, workspace_id, agent_id, title, status, intent, \
    target_type, target_id, context, result, initiated_by, \
    started_at, completed_at, created_at, updated_at";

#[async_trait]
impl CrudRepository<Session> for PgSessionRepo {
    async fn find_all(&self, workspace_id: Uuid) -> Result<Vec<Session>, DomainError> {
        let mut tx = TenantTx::begin(&self.pool, workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let query = format!("SELECT {SELECT_COLS} FROM sessions ORDER BY created_at");
        let rows = sqlx::query_as::<_, SessionRow>(&query)
            .fetch_all(tx.conn())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        rows.into_iter().map(Session::try_from).collect()
    }

    async fn find_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Session>, DomainError> {
        let mut tx = TenantTx::begin(&self.pool, workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let query = format!("SELECT {SELECT_COLS} FROM sessions WHERE id = $1");
        let row = sqlx::query_as::<_, SessionRow>(&query)
            .bind(id)
            .fetch_optional(tx.conn())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        row.map(Session::try_from).transpose()
    }

    async fn create(&self, session: &Session) -> Result<Session, DomainError> {
        let mut tx = TenantTx::begin(&self.pool, session.workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let query = format!(
            "INSERT INTO sessions (id, workspace_id, agent_id, title, status, intent, \
             target_type, target_id, context, result, initiated_by, started_at, completed_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) \
             RETURNING {SELECT_COLS}"
        );
        let row = sqlx::query_as::<_, SessionRow>(&query)
            .bind(session.id)
            .bind(session.workspace_id)
            .bind(session.agent_id)
            .bind(&session.title)
            .bind(status_to_str(&session.status))
            .bind(intent_to_str(&session.intent))
            .bind(session.target_type.as_ref().map(target_type_to_str))
            .bind(session.target_id)
            .bind(&session.context)
            .bind(&session.result)
            .bind(&session.initiated_by)
            .bind(session.started_at)
            .bind(session.completed_at)
            .fetch_one(tx.conn())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Session::try_from(row)
    }

    async fn update(&self, session: &Session) -> Result<Session, DomainError> {
        let mut tx = TenantTx::begin(&self.pool, session.workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let query = format!(
            "UPDATE sessions SET title = $1, status = $2, intent = $3, \
             target_type = $4, target_id = $5, context = $6, result = $7, \
             initiated_by = $8, started_at = $9, completed_at = $10 \
             WHERE id = $11 \
             RETURNING {SELECT_COLS}"
        );
        let row = sqlx::query_as::<_, SessionRow>(&query)
            .bind(&session.title)
            .bind(status_to_str(&session.status))
            .bind(intent_to_str(&session.intent))
            .bind(session.target_type.as_ref().map(target_type_to_str))
            .bind(session.target_id)
            .bind(&session.context)
            .bind(&session.result)
            .bind(&session.initiated_by)
            .bind(session.started_at)
            .bind(session.completed_at)
            .bind(session.id)
            .fetch_optional(tx.conn())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .ok_or_else(|| DomainError::NotFound {
                entity: "session".into(),
                id: session.id.to_string(),
            })?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Session::try_from(row)
    }

    async fn delete(&self, workspace_id: Uuid, id: Uuid) -> Result<(), DomainError> {
        let mut tx = TenantTx::begin(&self.pool, workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let result = sqlx::query("DELETE FROM sessions WHERE id = $1")
            .bind(id)
            .execute(tx.conn())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound {
                entity: "session".into(),
                id: id.to_string(),
            });
        }

        Ok(())
    }
}

#[async_trait]
impl SessionRepository for PgSessionRepo {
    async fn find_by_status(
        &self,
        workspace_id: Uuid,
        status: SessionStatus,
    ) -> Result<Vec<Session>, DomainError> {
        let mut tx = TenantTx::begin(&self.pool, workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let query =
            format!("SELECT {SELECT_COLS} FROM sessions WHERE status = $1 ORDER BY created_at");
        let rows = sqlx::query_as::<_, SessionRow>(&query)
            .bind(status_to_str(&status))
            .fetch_all(tx.conn())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        rows.into_iter().map(Session::try_from).collect()
    }
}
