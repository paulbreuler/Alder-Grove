//! Agent persistence adapter.
//!
//! All queries use `TenantTx` for RLS-scoped workspace isolation.
//! The agents table has `workspace_id = current_workspace_id()` RLS policy.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use grove_domain::agent::{Agent, AgentStatus};
use grove_domain::error::DomainError;
use grove_domain::ports::CrudRepository;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::tenant::TenantTx;

pub struct PgAgentRepo {
    pool: PgPool,
}

impl PgAgentRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Private row type -- maps 1:1 to SQL columns.
/// Separated from domain Agent to decouple DB schema from domain types.
#[derive(sqlx::FromRow)]
struct AgentRow {
    id: Uuid,
    workspace_id: Uuid,
    name: String,
    provider: String,
    model: Option<String>,
    description: Option<String>,
    capabilities: serde_json::Value,
    config: serde_json::Value,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<AgentRow> for Agent {
    type Error = DomainError;

    fn try_from(row: AgentRow) -> Result<Self, Self::Error> {
        let status = match row.status.as_str() {
            "active" => AgentStatus::Active,
            "disabled" => AgentStatus::Disabled,
            other => {
                return Err(DomainError::Internal(format!(
                    "invalid agent status: {other}"
                )));
            }
        };
        let capabilities: Vec<String> = serde_json::from_value(row.capabilities)
            .map_err(|e| DomainError::Internal(format!("invalid agent capabilities JSON: {e}")))?;
        Ok(Self {
            id: row.id,
            workspace_id: row.workspace_id,
            name: row.name,
            provider: row.provider,
            model: row.model,
            description: row.description,
            capabilities,
            config: row.config,
            status,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

fn status_to_str(status: &AgentStatus) -> &'static str {
    match status {
        AgentStatus::Active => "active",
        AgentStatus::Disabled => "disabled",
    }
}

#[async_trait]
impl CrudRepository<Agent> for PgAgentRepo {
    async fn find_all(&self, workspace_id: Uuid) -> Result<Vec<Agent>, DomainError> {
        let mut tx = TenantTx::begin(&self.pool, workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let rows = sqlx::query_as::<_, AgentRow>(
            "SELECT id, workspace_id, name, provider, model, description, capabilities, config, status, created_at, updated_at FROM agents ORDER BY created_at",
        )
        .fetch_all(tx.conn())
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        rows.into_iter().map(Agent::try_from).collect()
    }

    async fn find_by_id(&self, workspace_id: Uuid, id: Uuid) -> Result<Option<Agent>, DomainError> {
        let mut tx = TenantTx::begin(&self.pool, workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let row = sqlx::query_as::<_, AgentRow>(
            "SELECT id, workspace_id, name, provider, model, description, capabilities, config, status, created_at, updated_at FROM agents WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(tx.conn())
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        row.map(Agent::try_from).transpose()
    }

    async fn create(&self, agent: &Agent) -> Result<Agent, DomainError> {
        let mut tx = TenantTx::begin(&self.pool, agent.workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let capabilities = serde_json::to_value(&agent.capabilities)
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let row = sqlx::query_as::<_, AgentRow>(
            "INSERT INTO agents (id, workspace_id, name, provider, model, description, capabilities, config, status) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
             RETURNING id, workspace_id, name, provider, model, description, capabilities, config, status, created_at, updated_at",
        )
        .bind(agent.id)
        .bind(agent.workspace_id)
        .bind(&agent.name)
        .bind(&agent.provider)
        .bind(&agent.model)
        .bind(&agent.description)
        .bind(&capabilities)
        .bind(&agent.config)
        .bind(status_to_str(&agent.status))
        .fetch_one(tx.conn())
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Agent::try_from(row)
    }

    async fn update(&self, agent: &Agent) -> Result<Agent, DomainError> {
        let mut tx = TenantTx::begin(&self.pool, agent.workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let capabilities = serde_json::to_value(&agent.capabilities)
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let row = sqlx::query_as::<_, AgentRow>(
            "UPDATE agents SET name = $1, provider = $2, model = $3, description = $4, \
             capabilities = $5, config = $6, status = $7 \
             WHERE id = $8 \
             RETURNING id, workspace_id, name, provider, model, description, capabilities, config, status, created_at, updated_at",
        )
        .bind(&agent.name)
        .bind(&agent.provider)
        .bind(&agent.model)
        .bind(&agent.description)
        .bind(&capabilities)
        .bind(&agent.config)
        .bind(status_to_str(&agent.status))
        .bind(agent.id)
        .fetch_optional(tx.conn())
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| DomainError::NotFound {
            entity: "agent".into(),
            id: agent.id.to_string(),
        })?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Agent::try_from(row)
    }

    async fn delete(&self, workspace_id: Uuid, id: Uuid) -> Result<(), DomainError> {
        let mut tx = TenantTx::begin(&self.pool, workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let result = sqlx::query("DELETE FROM agents WHERE id = $1")
            .bind(id)
            .execute(tx.conn())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound {
                entity: "agent".into(),
                id: id.to_string(),
            });
        }

        Ok(())
    }
}
