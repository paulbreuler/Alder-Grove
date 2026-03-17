//! Gate definition persistence adapter.
//!
//! All queries use `TenantTx` for RLS-scoped workspace isolation.
//! The gate_definitions table has `workspace_id = current_workspace_id()` RLS policy.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use grove_domain::error::DomainError;
use grove_domain::gate::{ApprovalType, GateDefinition, TimeoutAction, TriggerType};
use grove_domain::ports::{CrudRepository, GateDefinitionRepository};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::tenant::TenantTx;

pub struct PgGateDefinitionRepo {
    pool: PgPool,
}

impl PgGateDefinitionRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Private row type -- maps 1:1 to SQL columns.
/// Separated from domain GateDefinition to decouple DB schema from domain types.
#[derive(sqlx::FromRow)]
struct GateDefinitionRow {
    id: Uuid,
    workspace_id: Uuid,
    name: String,
    description: Option<String>,
    trigger_type: String,
    trigger_config: serde_json::Value,
    approval_type: String,
    timeout_minutes: Option<i32>,
    timeout_action: String,
    enabled: bool,
    sort_order: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

fn parse_trigger_type(s: &str) -> Result<TriggerType, DomainError> {
    match s {
        "automatic" => Ok(TriggerType::Automatic),
        "manual" => Ok(TriggerType::Manual),
        "threshold" => Ok(TriggerType::Threshold),
        other => Err(DomainError::Internal(format!(
            "invalid trigger_type: {other}"
        ))),
    }
}

fn parse_approval_type(s: &str) -> Result<ApprovalType, DomainError> {
    match s {
        "single" => Ok(ApprovalType::Single),
        "any_of" => Ok(ApprovalType::AnyOf),
        "all_of" => Ok(ApprovalType::AllOf),
        other => Err(DomainError::Internal(format!(
            "invalid approval_type: {other}"
        ))),
    }
}

fn parse_timeout_action(s: &str) -> Result<TimeoutAction, DomainError> {
    match s {
        "cancel" => Ok(TimeoutAction::Cancel),
        "approve" => Ok(TimeoutAction::Approve),
        "escalate" => Ok(TimeoutAction::Escalate),
        other => Err(DomainError::Internal(format!(
            "invalid timeout_action: {other}"
        ))),
    }
}

fn trigger_type_to_str(t: &TriggerType) -> &'static str {
    match t {
        TriggerType::Automatic => "automatic",
        TriggerType::Manual => "manual",
        TriggerType::Threshold => "threshold",
    }
}

fn approval_type_to_str(a: &ApprovalType) -> &'static str {
    match a {
        ApprovalType::Single => "single",
        ApprovalType::AnyOf => "any_of",
        ApprovalType::AllOf => "all_of",
    }
}

fn timeout_action_to_str(t: &TimeoutAction) -> &'static str {
    match t {
        TimeoutAction::Cancel => "cancel",
        TimeoutAction::Approve => "approve",
        TimeoutAction::Escalate => "escalate",
    }
}

impl TryFrom<GateDefinitionRow> for GateDefinition {
    type Error = DomainError;

    fn try_from(row: GateDefinitionRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            workspace_id: row.workspace_id,
            name: row.name,
            description: row.description,
            trigger_type: parse_trigger_type(&row.trigger_type)?,
            trigger_config: row.trigger_config,
            approval_type: parse_approval_type(&row.approval_type)?,
            timeout_minutes: row.timeout_minutes,
            timeout_action: parse_timeout_action(&row.timeout_action)?,
            enabled: row.enabled,
            sort_order: row.sort_order,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

const SELECT_COLS: &str = "\
    id, workspace_id, name, description, trigger_type, trigger_config, \
    approval_type, timeout_minutes, timeout_action, enabled, sort_order, \
    created_at, updated_at";

#[async_trait]
impl CrudRepository<GateDefinition> for PgGateDefinitionRepo {
    async fn find_all(&self, workspace_id: Uuid) -> Result<Vec<GateDefinition>, DomainError> {
        let mut tx = TenantTx::begin(&self.pool, workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let query =
            format!("SELECT {SELECT_COLS} FROM gate_definitions ORDER BY sort_order, created_at");
        let rows = sqlx::query_as::<_, GateDefinitionRow>(&query)
            .fetch_all(tx.conn())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        rows.into_iter().map(GateDefinition::try_from).collect()
    }

    async fn find_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<GateDefinition>, DomainError> {
        let mut tx = TenantTx::begin(&self.pool, workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let query = format!("SELECT {SELECT_COLS} FROM gate_definitions WHERE id = $1");
        let row = sqlx::query_as::<_, GateDefinitionRow>(&query)
            .bind(id)
            .fetch_optional(tx.conn())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        row.map(GateDefinition::try_from).transpose()
    }

    async fn create(&self, gd: &GateDefinition) -> Result<GateDefinition, DomainError> {
        let mut tx = TenantTx::begin(&self.pool, gd.workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let query = format!(
            "INSERT INTO gate_definitions \
             (id, workspace_id, name, description, trigger_type, trigger_config, \
              approval_type, timeout_minutes, timeout_action, enabled, sort_order) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) \
             RETURNING {SELECT_COLS}"
        );
        let row = sqlx::query_as::<_, GateDefinitionRow>(&query)
            .bind(gd.id)
            .bind(gd.workspace_id)
            .bind(&gd.name)
            .bind(&gd.description)
            .bind(trigger_type_to_str(&gd.trigger_type))
            .bind(&gd.trigger_config)
            .bind(approval_type_to_str(&gd.approval_type))
            .bind(gd.timeout_minutes)
            .bind(timeout_action_to_str(&gd.timeout_action))
            .bind(gd.enabled)
            .bind(gd.sort_order)
            .fetch_one(tx.conn())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        GateDefinition::try_from(row)
    }

    async fn update(&self, gd: &GateDefinition) -> Result<GateDefinition, DomainError> {
        let mut tx = TenantTx::begin(&self.pool, gd.workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let query = format!(
            "UPDATE gate_definitions SET name = $1, description = $2, trigger_type = $3, \
             trigger_config = $4, approval_type = $5, timeout_minutes = $6, \
             timeout_action = $7, enabled = $8, sort_order = $9 \
             WHERE id = $10 \
             RETURNING {SELECT_COLS}"
        );
        let row = sqlx::query_as::<_, GateDefinitionRow>(&query)
            .bind(&gd.name)
            .bind(&gd.description)
            .bind(trigger_type_to_str(&gd.trigger_type))
            .bind(&gd.trigger_config)
            .bind(approval_type_to_str(&gd.approval_type))
            .bind(gd.timeout_minutes)
            .bind(timeout_action_to_str(&gd.timeout_action))
            .bind(gd.enabled)
            .bind(gd.sort_order)
            .bind(gd.id)
            .fetch_optional(tx.conn())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .ok_or_else(|| DomainError::NotFound {
                entity: "gate_definition".into(),
                id: gd.id.to_string(),
            })?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        GateDefinition::try_from(row)
    }

    async fn delete(&self, workspace_id: Uuid, id: Uuid) -> Result<(), DomainError> {
        let mut tx = TenantTx::begin(&self.pool, workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let result = sqlx::query("DELETE FROM gate_definitions WHERE id = $1")
            .bind(id)
            .execute(tx.conn())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound {
                entity: "gate_definition".into(),
                id: id.to_string(),
            });
        }

        Ok(())
    }
}

#[async_trait]
impl GateDefinitionRepository for PgGateDefinitionRepo {
    async fn find_enabled(&self, workspace_id: Uuid) -> Result<Vec<GateDefinition>, DomainError> {
        let mut tx = TenantTx::begin(&self.pool, workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let query = format!(
            "SELECT {SELECT_COLS} FROM gate_definitions \
             WHERE enabled = true ORDER BY sort_order, created_at"
        );
        let rows = sqlx::query_as::<_, GateDefinitionRow>(&query)
            .fetch_all(tx.conn())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        rows.into_iter().map(GateDefinition::try_from).collect()
    }
}
