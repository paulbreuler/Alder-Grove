//! Guardrail persistence adapter.
//!
//! All queries use `TenantTx` for RLS-scoped workspace isolation.
//! The guardrails table has `workspace_id = current_workspace_id()` RLS policy.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use grove_domain::error::DomainError;
use grove_domain::guardrail::{
    Guardrail, GuardrailCategory, GuardrailEnforcement, GuardrailRule, GuardrailScope,
};
use grove_domain::ports::{CrudRepository, GuardrailRepository};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::tenant::TenantTx;

pub struct PgGuardrailRepo {
    pool: PgPool,
}

impl PgGuardrailRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Private row type -- maps 1:1 to SQL columns.
/// Separated from domain Guardrail to decouple DB schema from domain types.
#[derive(sqlx::FromRow)]
struct GuardrailRow {
    id: Uuid,
    workspace_id: Uuid,
    name: String,
    description: Option<String>,
    category: String,
    scope: String,
    enforcement: String,
    rule: serde_json::Value,
    version: i32,
    sort_order: i32,
    enabled: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

fn parse_category(s: &str) -> Result<GuardrailCategory, DomainError> {
    match s {
        "prohibition" => Ok(GuardrailCategory::Prohibition),
        "requirement" => Ok(GuardrailCategory::Requirement),
        "boundary" => Ok(GuardrailCategory::Boundary),
        "preference" => Ok(GuardrailCategory::Preference),
        other => Err(DomainError::Internal(format!(
            "invalid guardrail category: {other}"
        ))),
    }
}

fn parse_scope(s: &str) -> Result<GuardrailScope, DomainError> {
    match s {
        "workspace" => Ok(GuardrailScope::Workspace),
        "session" => Ok(GuardrailScope::Session),
        other => Err(DomainError::Internal(format!(
            "invalid guardrail scope: {other}"
        ))),
    }
}

fn parse_enforcement(s: &str) -> Result<GuardrailEnforcement, DomainError> {
    match s {
        "enforced" => Ok(GuardrailEnforcement::Enforced),
        "advisory" => Ok(GuardrailEnforcement::Advisory),
        other => Err(DomainError::Internal(format!(
            "invalid guardrail enforcement: {other}"
        ))),
    }
}

fn category_to_str(c: &GuardrailCategory) -> &'static str {
    match c {
        GuardrailCategory::Prohibition => "prohibition",
        GuardrailCategory::Requirement => "requirement",
        GuardrailCategory::Boundary => "boundary",
        GuardrailCategory::Preference => "preference",
    }
}

fn scope_to_str(s: &GuardrailScope) -> &'static str {
    match s {
        GuardrailScope::Workspace => "workspace",
        GuardrailScope::Session => "session",
    }
}

fn enforcement_to_str(e: &GuardrailEnforcement) -> &'static str {
    match e {
        GuardrailEnforcement::Enforced => "enforced",
        GuardrailEnforcement::Advisory => "advisory",
    }
}

impl TryFrom<GuardrailRow> for Guardrail {
    type Error = DomainError;

    fn try_from(row: GuardrailRow) -> Result<Self, Self::Error> {
        let rule: GuardrailRule = serde_json::from_value(row.rule)
            .map_err(|e| DomainError::Internal(format!("invalid guardrail rule JSON: {e}")))?;
        Ok(Self {
            id: row.id,
            workspace_id: row.workspace_id,
            name: row.name,
            description: row.description,
            category: parse_category(&row.category)?,
            scope: parse_scope(&row.scope)?,
            enforcement: parse_enforcement(&row.enforcement)?,
            rule,
            version: row.version,
            sort_order: row.sort_order,
            enabled: row.enabled,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

const SELECT_COLS: &str = "id, workspace_id, name, description, category, scope, enforcement, rule, version, sort_order, enabled, created_at, updated_at";

#[async_trait]
impl CrudRepository<Guardrail> for PgGuardrailRepo {
    async fn find_all(&self, workspace_id: Uuid) -> Result<Vec<Guardrail>, DomainError> {
        let mut tx = TenantTx::begin(&self.pool, workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let query = format!("SELECT {SELECT_COLS} FROM guardrails ORDER BY sort_order, created_at");
        let rows = sqlx::query_as::<_, GuardrailRow>(&query)
            .fetch_all(tx.conn())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        rows.into_iter().map(Guardrail::try_from).collect()
    }

    async fn find_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Guardrail>, DomainError> {
        let mut tx = TenantTx::begin(&self.pool, workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let query = format!("SELECT {SELECT_COLS} FROM guardrails WHERE id = $1");
        let row = sqlx::query_as::<_, GuardrailRow>(&query)
            .bind(id)
            .fetch_optional(tx.conn())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        row.map(Guardrail::try_from).transpose()
    }

    async fn create(&self, guardrail: &Guardrail) -> Result<Guardrail, DomainError> {
        let mut tx = TenantTx::begin(&self.pool, guardrail.workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let rule_json = serde_json::to_value(&guardrail.rule)
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let query = format!(
            "INSERT INTO guardrails (id, workspace_id, name, description, category, scope, enforcement, rule, version, sort_order, enabled) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) \
             RETURNING {SELECT_COLS}"
        );
        let row = sqlx::query_as::<_, GuardrailRow>(&query)
            .bind(guardrail.id)
            .bind(guardrail.workspace_id)
            .bind(&guardrail.name)
            .bind(&guardrail.description)
            .bind(category_to_str(&guardrail.category))
            .bind(scope_to_str(&guardrail.scope))
            .bind(enforcement_to_str(&guardrail.enforcement))
            .bind(&rule_json)
            .bind(guardrail.version)
            .bind(guardrail.sort_order)
            .bind(guardrail.enabled)
            .fetch_one(tx.conn())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Guardrail::try_from(row)
    }

    async fn update(&self, guardrail: &Guardrail) -> Result<Guardrail, DomainError> {
        let mut tx = TenantTx::begin(&self.pool, guardrail.workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let rule_json = serde_json::to_value(&guardrail.rule)
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let query = format!(
            "UPDATE guardrails SET name = $1, description = $2, category = $3, scope = $4, \
             enforcement = $5, rule = $6, version = $7, sort_order = $8, enabled = $9 \
             WHERE id = $10 \
             RETURNING {SELECT_COLS}"
        );
        let row = sqlx::query_as::<_, GuardrailRow>(&query)
            .bind(&guardrail.name)
            .bind(&guardrail.description)
            .bind(category_to_str(&guardrail.category))
            .bind(scope_to_str(&guardrail.scope))
            .bind(enforcement_to_str(&guardrail.enforcement))
            .bind(&rule_json)
            .bind(guardrail.version)
            .bind(guardrail.sort_order)
            .bind(guardrail.enabled)
            .bind(guardrail.id)
            .fetch_optional(tx.conn())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .ok_or_else(|| DomainError::NotFound {
                entity: "guardrail".into(),
                id: guardrail.id.to_string(),
            })?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Guardrail::try_from(row)
    }

    async fn delete(&self, workspace_id: Uuid, id: Uuid) -> Result<(), DomainError> {
        let mut tx = TenantTx::begin(&self.pool, workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let result = sqlx::query("DELETE FROM guardrails WHERE id = $1")
            .bind(id)
            .execute(tx.conn())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound {
                entity: "guardrail".into(),
                id: id.to_string(),
            });
        }

        Ok(())
    }
}

#[async_trait]
impl GuardrailRepository for PgGuardrailRepo {
    async fn find_enabled_by_scope(
        &self,
        workspace_id: Uuid,
        scope: GuardrailScope,
    ) -> Result<Vec<Guardrail>, DomainError> {
        let mut tx = TenantTx::begin(&self.pool, workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let query = format!(
            "SELECT {SELECT_COLS} FROM guardrails WHERE enabled = true AND scope = $1 ORDER BY sort_order, created_at"
        );
        let rows = sqlx::query_as::<_, GuardrailRow>(&query)
            .bind(scope_to_str(&scope))
            .fetch_all(tx.conn())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        rows.into_iter().map(Guardrail::try_from).collect()
    }

    async fn find_filtered(
        &self,
        workspace_id: Uuid,
        scope: Option<GuardrailScope>,
        enabled: Option<bool>,
    ) -> Result<Vec<Guardrail>, DomainError> {
        let mut tx = TenantTx::begin(&self.pool, workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        // Build WHERE clause dynamically based on provided filters.
        let mut conditions = Vec::new();
        let mut bind_idx: u32 = 1;

        if scope.is_some() {
            conditions.push(format!("scope = ${bind_idx}"));
            bind_idx += 1;
        }
        if enabled.is_some() {
            conditions.push(format!("enabled = ${bind_idx}"));
            // bind_idx not incremented — last possible filter
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };

        let query_str = format!(
            "SELECT {SELECT_COLS} FROM guardrails{where_clause} ORDER BY sort_order, created_at"
        );
        let mut query = sqlx::query_as::<_, GuardrailRow>(&query_str);

        if let Some(scope) = scope {
            query = query.bind(scope_to_str(&scope));
        }
        if let Some(enabled) = enabled {
            query = query.bind(enabled);
        }

        let rows = query
            .fetch_all(tx.conn())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        rows.into_iter().map(Guardrail::try_from).collect()
    }
}
