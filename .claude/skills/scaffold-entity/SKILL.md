---
name: scaffold-entity
description: Full-stack entity scaffold — Rust domain type + API route + TypeScript types + frontend adapter
user_invocable: true
---

# /scaffold-entity

Full-stack entity scaffold for Alder Grove. Creates the Rust domain type, API
routes, database repo, TypeScript types, and frontend adapter in one pass using
test-driven development.

## Workflow

### Step 1: Gather Requirements

Ask the user (or extract from `$ARGUMENTS`) for:

| Field             | Required | Default | Description                                      |
| ----------------- | -------- | ------- | ------------------------------------------------ |
| entity name       | yes      | —       | Singular, PascalCase (e.g., `Persona`)           |
| fields            | yes      | —       | Field definitions as `name:type` pairs           |
| workspace-scoped  | no       | yes     | Whether entity is scoped to a workspace          |
| parent extension  | yes      | —       | Frontend extension under `src/features/<name>/`  |

Standard fields added automatically (do not include in field list):
- `id: Uuid` (uuidv7 PK)
- `workspace_id: Uuid` (if workspace-scoped)
- `created_at: DateTime<Utc>`
- `updated_at: DateTime<Utc>`

### Step 2: Domain Layer (Rust) — Test First

**2a. Write domain test:**

Create or update `crates/grove-domain/src/<entity_snake>.rs` with a test module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_<entity_snake>_has_required_fields() {
        // Assert struct fields, derive traits, type constraints
    }
}
```

Run `cargo test -p grove-domain` — expect RED.

**2b. Implement domain type:**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct <EntityName> {
    pub id: Uuid,
    pub workspace_id: Uuid,  // if workspace-scoped
    // ... user-defined fields ...
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**2c. Add port trait:**

Add to `crates/grove-domain/src/ports.rs`:

```rust
/// Extends CrudRepository with any entity-specific queries.
/// For simple entities, just implement CrudRepository<EntityName> directly.
#[async_trait::async_trait]
pub trait <EntityName>Repository: CrudRepository<<EntityName>> {
    // Add entity-specific queries here, e.g.:
    // async fn find_by_status(&self, scope_id: Uuid, status: Status) -> Result<Vec<<EntityName>>, DomainError>;
}
```

If the entity has no special queries beyond CRUD, skip the custom trait entirely
and use `CrudRepository<<EntityName>>` directly (see `ports.rs` for the generic
trait definition).

**2d. Add DomainError variant** if needed in `crates/grove-domain/src/error.rs`.

**2e. Register module** in `crates/grove-domain/src/lib.rs`:

```rust
pub mod <entity_snake>;
```

Run `cargo test -p grove-domain` — expect GREEN.

### Step 3: API Layer (Rust) — Test First

**3a. Write integration test:**

Create test using `tower::ServiceExt::oneshot` pattern. See existing route tests
for the pattern (Clerk auth mock, test database, `TenantTx`).

Run `cargo test -p grove-api` — expect RED.

**3b. Create repo:**

`crates/grove-api/src/db/<entity_snake>_repo.rs`:

- Implement the port trait from grove-domain
- Use `TenantTx` for all queries (workspace isolation)
- Use runtime `sqlx::query_as::<_, Row>(...)` pattern (no compile-time DB coupling)
- Register in `crates/grove-api/src/db/mod.rs`

**3c. Create routes:**

`crates/grove-api/src/routes/<entity_snake>.rs`:

- Routes: `POST /`, `GET /`, `GET /{id}`, `PUT /{id}`, `DELETE /{id}`
- Full path: `/orgs/{org_id}/workspaces/{ws_id}/<entity_plural>`
- Use Clerk auth extractor
- Use `resolve_workspace()` for workspace ownership verification
- Inject repo via `Arc<dyn <EntityName>Repository>` from app state
- Register in `crates/grove-api/src/routes/mod.rs`

**3d. Wire into app:**

Update `crates/grove-api/src/lib.rs`:
- Add repo to `AppState`
- Add routes to router

Run `cargo test -p grove-api` — expect GREEN.

### Step 4: Generate TypeScript Types

```bash
pnpm generate:types
```

This runs `ts-rs` to export Rust types with `#[ts(export)]` to TypeScript
bindings. Verify the generated file includes the new entity type.

### Step 5: Frontend Adapter

Create `src/features/<extension>/adapters/<entityCamel>Api.ts`:

```typescript
import type { <EntityName> } from '@/generated/<EntityName>';

const BASE = '/orgs';

export const <entityCamel>Api = {
  async list(orgId: string, wsId: string): Promise<<EntityName>[]> {
    const res = await fetch(`${BASE}/${orgId}/workspaces/${wsId}/<entity_plural>`);
    if (!res.ok) throw new Error(`Failed to list <entity_plural>`);
    return res.json();
  },

  async getById(orgId: string, wsId: string, id: string): Promise<<EntityName>> {
    const res = await fetch(`${BASE}/${orgId}/workspaces/${wsId}/<entity_plural>/${id}`);
    if (!res.ok) throw new Error(`<EntityName> not found`);
    return res.json();
  },

  async create(orgId: string, wsId: string, data: Omit<<EntityName>, 'id' | 'created_at' | 'updated_at' | 'workspace_id'>): Promise<<EntityName>> {
    const res = await fetch(`${BASE}/${orgId}/workspaces/${wsId}/<entity_plural>`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });
    if (!res.ok) throw new Error(`Failed to create <entity_name>`);
    return res.json();
  },

  async update(orgId: string, wsId: string, id: string, data: Partial<Omit<<EntityName>, 'id' | 'created_at' | 'updated_at' | 'workspace_id'>>): Promise<<EntityName>> {
    const res = await fetch(`${BASE}/${orgId}/workspaces/${wsId}/<entity_plural>/${id}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });
    if (!res.ok) throw new Error(`Failed to update <entity_name>`);
    return res.json();
  },

  async delete(orgId: string, wsId: string, id: string): Promise<void> {
    const res = await fetch(`${BASE}/${orgId}/workspaces/${wsId}/<entity_plural>/${id}`, {
      method: 'DELETE',
    });
    if (!res.ok) throw new Error(`Failed to delete <entity_name>`);
  },
};
```

Adjust the import path for generated types based on the project's `tsconfig.json`
path aliases.

### Step 6: Frontend Domain (Optional)

If the entity needs frontend-specific types beyond what ts-rs generates
(e.g., form state, validation rules, display helpers), add them to
`src/features/<extension>/domain/<entityCamel>.ts`.

Write the test first: `src/features/<extension>/domain/<entityCamel>.test.ts`.

### Step 7: Verify

```bash
cargo test --workspace     # All Rust tests pass
pnpm check                 # TypeScript + ESLint pass
pnpm test                  # All frontend tests pass
```

## Checklist

Before marking complete, verify:

- [ ] `crates/grove-domain/src/<entity_snake>.rs` — domain type with feature-gated `#[cfg_attr(feature = "ts", derive(ts_rs::TS))]`
- [ ] Port trait added to `crates/grove-domain/src/ports.rs`
- [ ] Module registered in `crates/grove-domain/src/lib.rs`
- [ ] Domain tests pass (`cargo test -p grove-domain`)
- [ ] `crates/grove-api/src/db/<entity_snake>_repo.rs` — repo implementing port trait
- [ ] `crates/grove-api/src/routes/<entity_snake>.rs` — CRUD routes with Clerk auth
- [ ] Routes use `resolve_workspace()` for tenant isolation
- [ ] Repo registered in `crates/grove-api/src/db/mod.rs`
- [ ] Routes registered in `crates/grove-api/src/routes/mod.rs`
- [ ] Wired into `AppState` in `crates/grove-api/src/lib.rs`
- [ ] API tests pass (`cargo test -p grove-api`)
- [ ] TypeScript types generated (`pnpm generate:types`)
- [ ] Frontend adapter created in `src/features/<extension>/adapters/`
- [ ] `cargo test --workspace` passes
- [ ] `pnpm check` passes
- [ ] `pnpm test` passes

## Related Skills

- `/add-extension` — if the parent extension does not exist yet
- `/add-feature` — for adding sub-features within an extension
- `/add-api-endpoint` — for detailed API route patterns
- `/add-migration` — for database migration patterns

## Rules

- TDD is mandatory at every layer — test BEFORE implementation
- Domain types must use `#[derive(Debug, Clone, Serialize, Deserialize)]` with feature-gated `#[cfg_attr(feature = "ts", derive(ts_rs::TS))]` and `#[cfg_attr(feature = "ts", ts(export))]`
- All API routes require Clerk auth middleware
- All repo queries go through `TenantTx` for workspace isolation
- Handlers inject repos via `Arc<dyn PortTrait>` — never import concrete repo types in routes
- Frontend adapters return domain types, not raw API response shapes
- Use uuidv7 for all primary keys
- Follow existing naming conventions in the codebase for consistency
