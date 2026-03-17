//! Event persistence adapter (append-only).
//!
//! Events are immutable once created — no update or delete operations.
//! All queries use `TenantTx` for RLS-scoped workspace isolation.
//! The events table has `workspace_id = current_workspace_id()` RLS policy
//! plus restrictive policies preventing UPDATE and DELETE.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use grove_domain::error::DomainError;
use grove_domain::event::{Event, EventCategory, EventEmitter};
use grove_domain::ports::EventRepository;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::tenant::TenantTx;

pub struct PgEventRepo {
    pool: PgPool,
}

impl PgEventRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Private row type -- maps 1:1 to SQL columns.
/// Separated from domain Event to decouple DB schema from domain types.
#[derive(sqlx::FromRow)]
struct EventRow {
    id: Uuid,
    session_id: Uuid,
    workspace_id: Uuid,
    event_type: String,
    category: String,
    summary: String,
    data: serde_json::Value,
    emitted_by: String,
    created_at: DateTime<Utc>,
}

fn parse_category(s: &str) -> Result<EventCategory, DomainError> {
    match s {
        "lifecycle" => Ok(EventCategory::Lifecycle),
        "action" => Ok(EventCategory::Action),
        "gate" => Ok(EventCategory::Gate),
        "content" => Ok(EventCategory::Content),
        "error" => Ok(EventCategory::Error),
        "metric" => Ok(EventCategory::Metric),
        other => Err(DomainError::Internal(format!(
            "invalid event category: {other}"
        ))),
    }
}

fn parse_emitted_by(s: &str) -> Result<EventEmitter, DomainError> {
    match s {
        "agent" => Ok(EventEmitter::Agent),
        "system" => Ok(EventEmitter::System),
        "human" => Ok(EventEmitter::Human),
        other => Err(DomainError::Internal(format!(
            "invalid event emitted_by: {other}"
        ))),
    }
}

fn category_to_str(c: &EventCategory) -> &'static str {
    match c {
        EventCategory::Lifecycle => "lifecycle",
        EventCategory::Action => "action",
        EventCategory::Gate => "gate",
        EventCategory::Content => "content",
        EventCategory::Error => "error",
        EventCategory::Metric => "metric",
    }
}

fn emitted_by_to_str(e: &EventEmitter) -> &'static str {
    match e {
        EventEmitter::Agent => "agent",
        EventEmitter::System => "system",
        EventEmitter::Human => "human",
    }
}

impl TryFrom<EventRow> for Event {
    type Error = DomainError;

    fn try_from(row: EventRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            session_id: row.session_id,
            workspace_id: row.workspace_id,
            event_type: row.event_type,
            category: parse_category(&row.category)?,
            summary: row.summary,
            data: row.data,
            emitted_by: parse_emitted_by(&row.emitted_by)?,
            created_at: row.created_at,
        })
    }
}

const SELECT_COLS: &str = "\
    id, session_id, workspace_id, event_type, category, \
    summary, data, emitted_by, created_at";

#[async_trait]
impl EventRepository for PgEventRepo {
    async fn find_all(
        &self,
        workspace_id: Uuid,
        session_id: Uuid,
    ) -> Result<Vec<Event>, DomainError> {
        let mut tx = TenantTx::begin(&self.pool, workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let rows = sqlx::query_as::<_, EventRow>(&format!(
            "SELECT {SELECT_COLS} FROM events WHERE session_id = $1 ORDER BY created_at"
        ))
        .bind(session_id)
        .fetch_all(tx.conn())
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        rows.into_iter().map(Event::try_from).collect()
    }

    async fn create(&self, event: &Event) -> Result<Event, DomainError> {
        let mut tx = TenantTx::begin(&self.pool, event.workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let query = format!(
            "INSERT INTO events \
             (id, session_id, workspace_id, event_type, category, summary, data, emitted_by) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             RETURNING {SELECT_COLS}"
        );
        let row = sqlx::query_as::<_, EventRow>(&query)
            .bind(event.id)
            .bind(event.session_id)
            .bind(event.workspace_id)
            .bind(&event.event_type)
            .bind(category_to_str(&event.category))
            .bind(&event.summary)
            .bind(&event.data)
            .bind(emitted_by_to_str(&event.emitted_by))
            .fetch_one(tx.conn())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Event::try_from(row)
    }
}
