---
name: add-api-endpoint
description: Scaffold a REST API endpoint with hexagonal layers and integration tests
user_invocable: true
---

# /add-api-endpoint

Scaffold a new REST API endpoint across all hexagonal layers with
test-first development. Produces domain types, port traits, route handlers,
database repos, and integration tests — all wired and passing.

## Dependency Flow

```
grove-domain (pure types, ports)
      ^
      |  depends on
      |
grove-api (routes, db repos, TenantTx, error mapping)
```

Domain never imports from grove-api. Route handlers never import concrete
repo types — they consume `Arc<dyn PortTrait>` via `AppState`.

## Reference Files

| Purpose                     | Path                                            |
| --------------------------- | ----------------------------------------------- |
| Domain entity example       | `crates/grove-domain/src/agent.rs`              |
| Domain error                | `crates/grove-domain/src/error.rs`              |
| Port traits                 | `crates/grove-domain/src/ports.rs`              |
| Domain lib (module exports) | `crates/grove-domain/src/lib.rs`                |
| Route handler example       | `crates/grove-api/src/routes/agent.rs`          |
| Route module registry       | `crates/grove-api/src/routes/mod.rs`            |
| DB repo example             | `crates/grove-api/src/db/agent_repo.rs`         |
| DB module registry          | `crates/grove-api/src/db/mod.rs`                |
| TenantTx (RLS isolation)    | `crates/grove-api/src/db/tenant.rs`             |
| AppState (DI root)          | `crates/grove-api/src/state.rs`                 |
| Router wiring               | `crates/grove-api/src/lib.rs`                   |
| ApiError (RFC 9457)         | `crates/grove-api/src/error.rs`                 |
| Custom extractors           | `crates/grove-api/src/extract.rs`               |
| resolve_workspace helper    | `crates/grove-api/src/routes/helpers.rs`        |
| Test helper / state factory | `crates/grove-api/tests/common/mod.rs`          |
| Integration test example    | `crates/grove-api/tests/agent_routes.rs`        |
| Tenant isolation test       | `crates/grove-api/tests/tenant_isolation.rs`    |

---

## Step 0: Gather Requirements

Before writing any code, clarify:

- **Entity name** (singular, snake_case): e.g. `persona`, `journey`
- **HTTP methods**: which of GET (list), GET (by ID), POST, PUT, DELETE
- **Route path**: follows `/orgs/{org_id}/workspaces/{ws_id}/<entities>` for
  workspace-scoped resources, or nested under a parent
  (`/sessions/{session_id}/events`)
- **Request body fields** (for POST/PUT): names, types, required vs optional
- **Response fields**: what to expose (may differ from internal entity)
- **Query parameters**: filtering, pagination, etc.
- **Parent scoping**: is this scoped directly by workspace, or nested under
  another entity (e.g. events under sessions)?
- **DomainError variants**: does this entity need new error variants, or do
  existing ones (`NotFound`, `Validation`, `Conflict`) suffice?

---

## Step 1: Create Domain Types

**File:** `crates/grove-domain/src/<entity>.rs`

Domain types are pure data — no framework dependencies. Use `serde` and `uuid`
only.

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct MyEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    // ... domain fields ...
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_roundtrip() {
        let entity = MyEntity {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            name: "test".into(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&entity).unwrap();
        let back: MyEntity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity.id, back.id);
    }
}
```

Register the module in `crates/grove-domain/src/lib.rs`:

```rust
pub mod my_entity;
```

If the entity has enum fields (status, type), add `#[serde(rename_all = "snake_case")]`
and test serialization in unit tests.

If new `DomainError` variants are needed, add them to
`crates/grove-domain/src/error.rs`.

---

## Step 2: Create Port Trait

**File:** `crates/grove-domain/src/ports.rs`

For standard CRUD entities, implement `CrudRepository<T>`:

```rust
// No new trait needed — use the generic CrudRepository:
// CrudRepository<MyEntity> provides find_all, find_by_id, create, update, delete

// If additional query methods are needed, extend the generic trait:
#[async_trait::async_trait]
pub trait MyEntityRepository: CrudRepository<MyEntity> {
    async fn find_by_status(
        &self,
        workspace_id: Uuid,
        status: MyEntityStatus,
    ) -> Result<Vec<MyEntity>, DomainError>;
}
```

Add the required import to the `use` block at the top of `ports.rs`.

For non-CRUD patterns (append-only, upsert), define a standalone trait with
`Send + Sync` bounds. See `EventRepository` for the append-only pattern.

---

## Step 3: Write Integration Test FIRST (RED)

**File:** `crates/grove-api/tests/<entity>_routes.rs`

Write the test before any handler or repo code exists. This test drives the
implementation.

```rust
mod common;

use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

fn json_body(json: serde_json::Value) -> axum::body::Body {
    axum::body::Body::from(serde_json::to_vec(&json).unwrap())
}

/// Helper: create a workspace via HTTP and return ws_id
async fn create_workspace(app: &axum::Router, org_id: &str) -> String {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/orgs/{org_id}/workspaces"))
                .header("content-type", "application/json")
                .body(json_body(serde_json::json!({
                    "name": "test-ws"
                })))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    created["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn my_entity_route_crud_lifecycle() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();
    let app = grove_api::create_app(state);
    let ws_id = create_workspace(&app, &org_id).await;

    // POST — create
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/orgs/{org_id}/workspaces/{ws_id}/my-entities"))
                .header("content-type", "application/json")
                .body(json_body(serde_json::json!({
                    "name": "Test Entity"
                })))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(created["name"], "Test Entity");
    let entity_id = created["id"].as_str().unwrap();

    // GET — by ID
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/orgs/{org_id}/workspaces/{ws_id}/my-entities/{entity_id}"
                ))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // GET — list
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/orgs/{org_id}/workspaces/{ws_id}/my-entities"))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let list: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(list.iter().any(|e| e["id"].as_str() == Some(entity_id)));

    // DELETE
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/orgs/{org_id}/workspaces/{ws_id}/my-entities/{entity_id}"
                ))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Cleanup
    common::cleanup_org(&pool, &org_id).await;
}

#[tokio::test]
async fn my_entity_route_returns_404_for_nonexistent_workspace() {
    let state = common::test_state().await;
    let app = grove_api::create_app(state);
    let fake_ws = uuid::Uuid::now_v7();

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/orgs/no_org/workspaces/{fake_ws}/my-entities"))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
```

### Tenant Isolation Test

Also verify that data from one workspace is invisible to another:

```rust
#[tokio::test]
async fn my_entity_cross_workspace_isolation() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_a = common::unique_org_id();
    let org_b = common::unique_org_id();
    let app = grove_api::create_app(state);
    let ws_a = create_workspace(&app, &org_a).await;
    let ws_b = create_workspace(&app, &org_b).await;

    // Create entity in workspace A
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/orgs/{org_a}/workspaces/{ws_a}/my-entities"))
                .header("content-type", "application/json")
                .body(json_body(serde_json::json!({ "name": "WS-A only" })))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // List from workspace B — should not see workspace A's entity
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/orgs/{org_b}/workspaces/{ws_b}/my-entities"))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let list: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(
        list.iter().all(|e| e["name"] != "WS-A only"),
        "workspace B must not see workspace A data"
    );

    common::cleanup_org(&pool, &org_a).await;
    common::cleanup_org(&pool, &org_b).await;
}
```

---

## Step 4: Run Test to Verify RED

```bash
cargo test -p grove-api --test <entity>_routes 2>&1 | head -50
```

The test must fail to compile or fail at runtime. This confirms you are testing
real behavior, not a tautology.

---

## Step 5: Create Route Handler

**File:** `crates/grove-api/src/routes/<entity>.rs`

Follow the established handler pattern. Key rules:

- Use custom extractors from `crate::extract` (`Json`, `Path`, `Query`) — not
  `axum::extract` directly — so rejections produce RFC 9457 Problem Details
- Call `resolve_workspace()` as the first operation in every handler
- Consume repos via `state.xxx_repo` — never import concrete repo types
- Return `ApiError` on all error paths (it converts to Problem Details via
  `IntoResponse`)
- Use `Uuid::now_v7()` for new IDs
- Trim string inputs before persisting

```rust
use axum::extract::State;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::extract::{Json, Path};
use crate::routes::helpers::resolve_workspace;
use crate::state::AppState;

use grove_domain::my_entity::MyEntity;

#[derive(Serialize)]
pub struct MyEntityResponse {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    // ... public fields ...
}

impl From<MyEntity> for MyEntityResponse {
    fn from(e: MyEntity) -> Self {
        Self {
            id: e.id,
            workspace_id: e.workspace_id,
            name: e.name,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateMyEntityRequest {
    pub name: String,
    // ... input fields ...
}

/// GET /orgs/{org_id}/workspaces/{ws_id}/my-entities
#[tracing::instrument(skip(state))]
pub async fn list(
    State(state): State<AppState>,
    Path((org_id, ws_id)): Path<(String, Uuid)>,
) -> Result<axum::Json<Vec<MyEntityResponse>>, ApiError> {
    resolve_workspace(&*state.workspace_repo, &org_id, ws_id).await?;
    let entities = state.my_entity_repo.find_all(ws_id).await?;
    Ok(axum::Json(
        entities.into_iter().map(MyEntityResponse::from).collect(),
    ))
}

/// GET /orgs/{org_id}/workspaces/{ws_id}/my-entities/{entity_id}
#[tracing::instrument(skip(state))]
pub async fn get(
    State(state): State<AppState>,
    Path((org_id, ws_id, entity_id)): Path<(String, Uuid, Uuid)>,
) -> Result<axum::Json<MyEntityResponse>, ApiError> {
    resolve_workspace(&*state.workspace_repo, &org_id, ws_id).await?;
    let entity = state
        .my_entity_repo
        .find_by_id(ws_id, entity_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("my_entity {entity_id} not found")))?;
    Ok(axum::Json(MyEntityResponse::from(entity)))
}

/// POST /orgs/{org_id}/workspaces/{ws_id}/my-entities
#[tracing::instrument(skip(state))]
pub async fn create(
    State(state): State<AppState>,
    Path((org_id, ws_id)): Path<(String, Uuid)>,
    Json(body): Json<CreateMyEntityRequest>,
) -> Result<(StatusCode, axum::Json<MyEntityResponse>), ApiError> {
    resolve_workspace(&*state.workspace_repo, &org_id, ws_id).await?;

    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name cannot be empty".into()));
    }

    let entity = MyEntity {
        id: Uuid::now_v7(),
        workspace_id: ws_id,
        name: body.name.trim().to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let created = state.my_entity_repo.create(&entity).await?;
    Ok((
        StatusCode::CREATED,
        axum::Json(MyEntityResponse::from(created)),
    ))
}

/// DELETE /orgs/{org_id}/workspaces/{ws_id}/my-entities/{entity_id}
#[tracing::instrument(skip(state))]
pub async fn delete(
    State(state): State<AppState>,
    Path((org_id, ws_id, entity_id)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    resolve_workspace(&*state.workspace_repo, &org_id, ws_id).await?;
    state.my_entity_repo.delete(ws_id, entity_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
```

Register the module in `crates/grove-api/src/routes/mod.rs`:

```rust
pub mod my_entity;
```

---

## Step 6: Create DB Repo

**File:** `crates/grove-api/src/db/<entity>_repo.rs`

All queries run inside a `TenantTx` for RLS isolation. The private `Row`
struct decouples the SQL schema from domain types.

```rust
//! MyEntity persistence adapter.
//!
//! All queries use `TenantTx` for RLS-scoped workspace isolation.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use grove_domain::error::DomainError;
use grove_domain::my_entity::MyEntity;
use grove_domain::ports::CrudRepository;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::tenant::TenantTx;

pub struct PgMyEntityRepo {
    pool: PgPool,
}

impl PgMyEntityRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Private row type — maps 1:1 to SQL columns.
/// Separated from domain MyEntity to decouple DB schema from domain types.
#[derive(sqlx::FromRow)]
struct MyEntityRow {
    id: Uuid,
    workspace_id: Uuid,
    name: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<MyEntityRow> for MyEntity {
    fn from(row: MyEntityRow) -> Self {
        Self {
            id: row.id,
            workspace_id: row.workspace_id,
            name: row.name,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[async_trait]
impl CrudRepository<MyEntity> for PgMyEntityRepo {
    async fn find_all(&self, workspace_id: Uuid) -> Result<Vec<MyEntity>, DomainError> {
        let mut tx = TenantTx::begin(&self.pool, workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let rows = sqlx::query_as::<_, MyEntityRow>(
            "SELECT id, workspace_id, name, created_at, updated_at \
             FROM my_entities ORDER BY created_at",
        )
        .fetch_all(tx.conn())
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(MyEntity::from).collect())
    }

    async fn find_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<MyEntity>, DomainError> {
        let mut tx = TenantTx::begin(&self.pool, workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let row = sqlx::query_as::<_, MyEntityRow>(
            "SELECT id, workspace_id, name, created_at, updated_at \
             FROM my_entities WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(tx.conn())
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(row.map(MyEntity::from))
    }

    async fn create(&self, entity: &MyEntity) -> Result<MyEntity, DomainError> {
        let mut tx = TenantTx::begin(&self.pool, entity.workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let row = sqlx::query_as::<_, MyEntityRow>(
            "INSERT INTO my_entities (id, workspace_id, name) \
             VALUES ($1, $2, $3) \
             RETURNING id, workspace_id, name, created_at, updated_at",
        )
        .bind(entity.id)
        .bind(entity.workspace_id)
        .bind(&entity.name)
        .fetch_one(tx.conn())
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(MyEntity::from(row))
    }

    async fn update(&self, entity: &MyEntity) -> Result<MyEntity, DomainError> {
        let mut tx = TenantTx::begin(&self.pool, entity.workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let row = sqlx::query_as::<_, MyEntityRow>(
            "UPDATE my_entities SET name = $1 \
             WHERE id = $2 \
             RETURNING id, workspace_id, name, created_at, updated_at",
        )
        .bind(&entity.name)
        .bind(entity.id)
        .fetch_optional(tx.conn())
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| DomainError::NotFound {
            entity: "my_entity".into(),
            id: entity.id.to_string(),
        })?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(MyEntity::from(row))
    }

    async fn delete(&self, workspace_id: Uuid, id: Uuid) -> Result<(), DomainError> {
        let mut tx = TenantTx::begin(&self.pool, workspace_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let result = sqlx::query("DELETE FROM my_entities WHERE id = $1")
            .bind(id)
            .execute(tx.conn())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound {
                entity: "my_entity".into(),
                id: id.to_string(),
            });
        }

        Ok(())
    }
}
```

Register in `crates/grove-api/src/db/mod.rs`:

```rust
pub mod my_entity_repo;
```

### Row Mapping Rules

- Use `From<Row>` when the mapping is infallible (all fields are direct copies)
- Use `TryFrom<Row>` when you need to parse enums or deserialize JSON
  (see `agent_repo.rs` for the `TryFrom` pattern with `AgentRow`)
- The `Row` struct field types must match SQL column types exactly
  (`String` for `TEXT/VARCHAR`, `serde_json::Value` for `JSONB`,
  `DateTime<Utc>` for `TIMESTAMPTZ`, etc.)

---

## Step 7: Wire Into AppState and Router

### AppState

**File:** `crates/grove-api/src/state.rs`

Add the repo field:

```rust
pub my_entity_repo: Arc<dyn CrudRepository<MyEntity>>,
```

Add the import for the domain type and port trait.

### Router

**File:** `crates/grove-api/src/lib.rs`

Register routes using Axum 0.8 path syntax (`{param}` NOT `:param`):

```rust
.route(
    "/orgs/{org_id}/workspaces/{ws_id}/my-entities",
    get(routes::my_entity::list).post(routes::my_entity::create),
)
.route(
    "/orgs/{org_id}/workspaces/{ws_id}/my-entities/{entity_id}",
    get(routes::my_entity::get)
        .put(routes::my_entity::update)
        .delete(routes::my_entity::delete),
)
```

### Test State Factory

**File:** `crates/grove-api/tests/common/mod.rs`

Wire the concrete repo into `test_state()`:

```rust
use grove_api::db::my_entity_repo::PgMyEntityRepo;

// Inside test_state():
my_entity_repo: Arc::new(PgMyEntityRepo::new(pool.clone())),
```

### Production Wiring

**File:** `crates/grove-api/src/main.rs`

Wire the concrete repo the same way as in `test_state()`, but using the
production pool.

---

## Step 8: Verify GREEN

```bash
cargo test -p grove-api --test <entity>_routes
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

All three must pass. If a clippy lint fires, fix it before proceeding.

---

## NON-NEGOTIABLE: Security Rules

These rules apply to every endpoint. Violating any of them is a blocking issue.

1. **Never trust client-supplied tenant IDs.** The `workspace_id` comes from
   the URL path, validated by `resolve_workspace()`. The request body must
   never contain `workspace_id` — the handler sets it from the path.

2. **Always call `resolve_workspace()` first.** This confirms the workspace
   exists and belongs to the org. Without it, a user could guess workspace
   UUIDs and access other tenants' data.

3. **Always use `TenantTx` for sub-workspace queries.** The RLS policy on
   every sub-workspace table enforces
   `workspace_id = current_setting('app.current_workspace_id')::uuid`.
   Bypassing `TenantTx` bypasses RLS.

4. **Never expose internal IDs or stack traces in error responses.** Use
   `ApiError` variants which map to RFC 9457 Problem Details. Log internal
   details at `tracing::error` level, return a generic message to the client.

5. **Parameterize all SQL values.** Use `$1`, `$2`, etc. The only exception
   is the `SET LOCAL app.current_workspace_id` in `TenantTx::begin`, which
   is safe because `Uuid` cannot contain SQL injection payloads.

6. **Workspace CRUD runs as superuser.** The `WorkspaceRepository` does NOT
   use `TenantTx` because workspaces are the RLS scope boundary. This is
   intentional — do not "fix" it.

---

## Checklist

Before marking this endpoint as done, verify:

- [ ] Domain type in `crates/grove-domain/src/<entity>.rs` with serde roundtrip test
- [ ] Module registered in `crates/grove-domain/src/lib.rs`
- [ ] Port trait in `crates/grove-domain/src/ports.rs` (or reuse `CrudRepository<T>`)
- [ ] Integration test in `crates/grove-api/tests/<entity>_routes.rs`
- [ ] Tests written BEFORE handler/repo (RED phase confirmed)
- [ ] Route handler in `crates/grove-api/src/routes/<entity>.rs`
- [ ] Route module registered in `crates/grove-api/src/routes/mod.rs`
- [ ] DB repo in `crates/grove-api/src/db/<entity>_repo.rs`
- [ ] DB module registered in `crates/grove-api/src/db/mod.rs`
- [ ] Repo uses `TenantTx` for all queries (except workspace CRUD)
- [ ] `AppState` has `Arc<dyn PortTrait>` field
- [ ] Routes wired in `crates/grove-api/src/lib.rs` with `{param}` syntax
- [ ] `test_state()` wires concrete repo
- [ ] `main.rs` wires concrete repo
- [ ] `resolve_workspace()` called in every handler
- [ ] Request body does NOT accept `workspace_id`
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] Tenant isolation test verifies cross-workspace data is invisible
- [ ] If routes changed, `./scripts/e2e.sh` passes
