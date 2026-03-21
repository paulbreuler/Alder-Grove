---
name: add-migration
description: Scaffold a SQL migration with RLS policy, rollback plan, and test seed data
user_invocable: true
---

# /add-migration

Scaffold a PostgreSQL migration for Alder Grove with RLS, rollback plan, seed
data, and a Rust integration test verifying tenant isolation.

## Step 1: Gather Requirements

Collect these before writing any SQL:

| Field              | Description                                               |
| ------------------ | --------------------------------------------------------- |
| Table name         | Plural, snake_case (e.g., `personas`, `gate_definitions`) |
| Columns            | Name, type, nullability, defaults                         |
| Relationships      | FK references (always via `workspace_id` composite FKs)   |
| Workspace-scoped   | Default YES -- all content tables have `workspace_id`     |
| Indexes            | Query patterns that need indexed access                   |
| Status column      | State machine values (use CHECK, not ENUM)                |
| JSONB columns      | Strategic denormalization fields with defaults             |
| AI provenance      | `ai_authored`, `ai_confidence`, `ai_rationale` if content |
| Seed data          | Representative rows for development and testing           |

## Step 2: Determine Next Migration Number

List existing files to find the next sequential number:

```bash
ls crates/grove-api/migrations/
```

Migration files use sequential numeric prefixes: `001_`, `002_`, `003_`, etc.
The next migration is `NNN+1`.

## Step 3: Create Migration File

Create `crates/grove-api/migrations/NNN_<description>.sql`.

### Migration template

```sql
-- Migration NNN: <Description>
-- <One-line purpose of this migration.>
--
-- Rollback plan:
--   DROP POLICY IF EXISTS workspace_isolation ON <table_name>;
--   DROP TABLE IF EXISTS <table_name>;
--   -- If adding columns to existing table:
--   ALTER TABLE <table> DROP COLUMN IF EXISTS <column>;

-- ============================================================
-- <table_name>
-- ============================================================
CREATE TABLE <table_name> (
    id              UUID        PRIMARY KEY DEFAULT uuidv7(),
    workspace_id    UUID        NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,

    -- Domain columns
    name            TEXT        NOT NULL,
    description     TEXT,
    status          TEXT        NOT NULL DEFAULT 'draft'
                    CHECK (status IN ('draft', 'active', 'archived')),

    -- AI provenance (include for content entities)
    ai_authored     BOOLEAN     NOT NULL DEFAULT false,
    ai_confidence   REAL        CHECK (ai_confidence BETWEEN 0.0 AND 1.0),
    ai_rationale    TEXT,

    -- Audit timestamps
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

COMMENT ON TABLE <table_name> IS '<One-line description>';
COMMENT ON COLUMN <table_name>.status IS 'State machine: draft | active | archived';

CREATE TRIGGER <table_name>_updated_at BEFORE UPDATE ON <table_name>
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- Indexes
CREATE INDEX idx_<table_name>_ws ON <table_name> (workspace_id);
CREATE UNIQUE INDEX idx_<table_name>_ws_id ON <table_name> (workspace_id, id);

-- Row-Level Security
ALTER TABLE <table_name> ENABLE ROW LEVEL SECURITY;
ALTER TABLE <table_name> FORCE ROW LEVEL SECURITY;
CREATE POLICY workspace_isolation ON <table_name> FOR ALL
    USING (workspace_id = current_workspace_id())
    WITH CHECK (workspace_id = current_workspace_id());

-- Grant application role access
GRANT SELECT, INSERT, UPDATE, DELETE ON <table_name> TO grove_app;
```

### SQL conventions

| Convention                | Detail                                                         |
| ------------------------- | -------------------------------------------------------------- |
| Primary keys              | `UUID DEFAULT uuidv7()`                                        |
| Tenant column             | `workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE` |
| Status columns            | `CHECK (status IN (...))` -- never `CREATE TYPE ... AS ENUM`   |
| Named constraints         | Use explicit constraint names for complex checks               |
| Composite foreign keys    | For child tables: denormalize `workspace_id`, use `(workspace_id, parent_id)` FK referencing parent's `(workspace_id, id)` unique index |
| JSONB columns             | Strategic use with sensible defaults: `NOT NULL DEFAULT '{}'`  |
| Timestamps                | `created_at` + `updated_at` with `set_updated_at()` trigger   |
| Append-only tables        | Omit `updated_at`, add restrictive UPDATE/DELETE policies      |
| RLS function              | `current_workspace_id()` -- already defined in `001_initial_schema.sql` |
| Application role          | `grove_app` -- already created in `001_initial_schema.sql`     |

### Composite FK pattern (child tables)

When a table is a child of a workspace-scoped parent (e.g., `steps` under
`journeys`), denormalize `workspace_id` and use a composite FK:

```sql
CREATE TABLE child_items (
    id              UUID        PRIMARY KEY DEFAULT uuidv7(),
    parent_id       UUID        NOT NULL,
    workspace_id    UUID        NOT NULL,

    -- columns...

    FOREIGN KEY (workspace_id, parent_id)
        REFERENCES parents(workspace_id, id) ON DELETE CASCADE
);
```

This requires the parent table to have a unique index on `(workspace_id, id)`,
which all Grove tables include as `idx_<table>_ws_id`.

### M:N join table pattern

```sql
CREATE TABLE entity_a_entity_b (
    entity_a_id  UUID NOT NULL,
    entity_b_id  UUID NOT NULL,
    workspace_id UUID NOT NULL,
    PRIMARY KEY (entity_a_id, entity_b_id),
    FOREIGN KEY (workspace_id, entity_a_id)
        REFERENCES entity_a(workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, entity_b_id)
        REFERENCES entity_b(workspace_id, id) ON DELETE CASCADE
);

CREATE INDEX idx_entity_a_entity_b_ws ON entity_a_entity_b (workspace_id);

ALTER TABLE entity_a_entity_b ENABLE ROW LEVEL SECURITY;
ALTER TABLE entity_a_entity_b FORCE ROW LEVEL SECURITY;
CREATE POLICY workspace_isolation ON entity_a_entity_b FOR ALL
    USING (workspace_id = current_workspace_id())
    WITH CHECK (workspace_id = current_workspace_id());
```

## Step 4: Create Seed Data Migration (optional)

Create a separate file: `crates/grove-api/migrations/NNN+1_seed_<table_name>.sql`.

```sql
-- Migration NNN+1: Seed data for <table_name>
-- Development and testing seed data. Idempotent via ON CONFLICT.
--
-- Rollback plan:
--   DELETE FROM <table_name> WHERE name LIKE '%-seed';

-- Requires a workspace to exist. Use with test fixtures.
-- These INSERTs run as superuser (no RLS), so workspace_id must be provided.

INSERT INTO <table_name> (id, workspace_id, name, description)
VALUES
    ('019577a0-0000-7000-8000-000000000001', :'ws_id', 'Example Item-seed', 'Seed description')
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    description = EXCLUDED.description;
```

### Seed data conventions

- Suffix names with `-seed` so they are identifiable
- Use deterministic UUIDs with the `019577a0-0000-7000-8000-` prefix for seeds
- Always use `ON CONFLICT DO UPDATE` for idempotency
- Reference workspace via psql variable `:'ws_id'` or a known test fixture
- No permission/RBAC seeds -- Clerk handles authorization

## Step 5: Write RLS Integration Test

Add a test to `crates/grove-api/tests/` or extend the existing
`tenant_isolation.rs` to verify cross-workspace isolation for the new table.

### Test template

```rust
#[tokio::test]
async fn <table_name>_rls_isolates_workspaces() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_a = common::unique_org_id();
    let org_b = common::unique_org_id();

    // Create two workspaces as superuser
    let ws_a: (Uuid,) =
        sqlx::query_as("INSERT INTO workspaces (org_id, name) VALUES ($1, 'WS-A') RETURNING id")
            .bind(&org_a)
            .fetch_one(&pool)
            .await
            .unwrap();

    let ws_b: (Uuid,) =
        sqlx::query_as("INSERT INTO workspaces (org_id, name) VALUES ($1, 'WS-B') RETURNING id")
            .bind(&org_b)
            .fetch_one(&pool)
            .await
            .unwrap();

    // Insert rows as superuser (bypasses RLS)
    sqlx::query("INSERT INTO <table_name> (workspace_id, name) VALUES ($1, 'Item-A')")
        .bind(ws_a.0)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO <table_name> (workspace_id, name) VALUES ($1, 'Item-B')")
        .bind(ws_b.0)
        .execute(&pool)
        .await
        .unwrap();

    // TenantTx for workspace A -- should only see Item-A
    let mut tx_a = TenantTx::begin(&pool, ws_a.0).await.unwrap();
    let rows: Vec<(String,)> = sqlx::query_as("SELECT name FROM <table_name>")
        .fetch_all(tx_a.conn())
        .await
        .unwrap();
    tx_a.commit().await.unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, "Item-A");

    // TenantTx for workspace B -- should only see Item-B
    let mut tx_b = TenantTx::begin(&pool, ws_b.0).await.unwrap();
    let rows: Vec<(String,)> = sqlx::query_as("SELECT name FROM <table_name>")
        .fetch_all(tx_b.conn())
        .await
        .unwrap();
    tx_b.commit().await.unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, "Item-B");

    // Verify cross-workspace insert is blocked
    let mut tx_cross = TenantTx::begin(&pool, ws_a.0).await.unwrap();
    let result = sqlx::query(
        "INSERT INTO <table_name> (workspace_id, name) VALUES ($1, 'Intruder')",
    )
    .bind(ws_b.0)
    .execute(tx_cross.conn())
    .await;
    assert!(result.is_err(), "RLS should block cross-workspace insert");

    // Cleanup
    common::cleanup_org(&pool, &org_a).await;
    common::cleanup_org(&pool, &org_b).await;
}
```

### Test conventions

- Use `common::test_state()` to get a pool with migrations applied
- Create isolated test orgs with `common::unique_org_id()`
- Always clean up with `common::cleanup_org()`
- Test both read isolation (SELECT returns only own workspace rows) and
  write isolation (INSERT to another workspace is rejected by RLS)

## Step 6: Register Migration in Test Helper

If the new migration file is not automatically picked up, add it to
`crates/grove-api/tests/common/mod.rs` in the `run_migrations` function:

```rust
let migrations: &[&str] = &[
    include_str!("../../migrations/001_initial_schema.sql"),
    include_str!("../../migrations/002_acp_schema.sql"),
    include_str!("../../migrations/003_collaborative_documents.sql"),
    include_str!("../../migrations/NNN_<description>.sql"),  // <-- add
];
```

## Step 7: Verify

```bash
cargo test --workspace
```

All tests must pass, including the new RLS isolation test.

## Reference Files

| File                                          | Purpose                                     |
| --------------------------------------------- | ------------------------------------------- |
| `crates/grove-api/migrations/`                | All migration files (sequential numbering)  |
| `crates/grove-api/migrations/001_initial_schema.sql` | Foundation: uuidv7, set_updated_at, current_workspace_id, grove_app role |
| `crates/grove-api/migrations/002_acp_schema.sql`     | ACP tables pattern reference                |
| `crates/grove-api/src/db/tenant.rs`           | TenantTx implementation                     |
| `crates/grove-api/tests/tenant_isolation.rs`  | Existing RLS test patterns                  |
| `crates/grove-api/tests/common/mod.rs`        | Test helper: pool, migrations, cleanup      |
| `crates/grove-domain/src/`                    | Domain types the new table maps to          |
| `docs/architecture-reference.md`              | Entity model and schema decisions           |

## Checklist

Before declaring done:

- [ ] Requirements gathered (table name, columns, FKs, indexes, seed data)
- [ ] Next migration number determined by listing existing files
- [ ] Migration file created at `crates/grove-api/migrations/NNN_<description>.sql`
- [ ] Rollback plan documented in migration header comments
- [ ] `workspace_id` column with FK to `workspaces(id)` (unless table IS workspaces)
- [ ] `created_at` and `updated_at` timestamps with `set_updated_at()` trigger
- [ ] `COMMENT ON TABLE` and `COMMENT ON COLUMN` for non-obvious columns
- [ ] Status columns use `CHECK` constraints, not ENUMs
- [ ] Composite FKs for child tables (denormalized `workspace_id`)
- [ ] RLS enabled and forced: `ENABLE ROW LEVEL SECURITY` + `FORCE ROW LEVEL SECURITY`
- [ ] RLS policy uses `current_workspace_id()` in both `USING` and `WITH CHECK`
- [ ] `GRANT SELECT, INSERT, UPDATE, DELETE ON <table> TO grove_app`
- [ ] Unique index on `(workspace_id, id)` for composite FK support
- [ ] Seed data in separate migration file with `ON CONFLICT DO UPDATE`
- [ ] RLS integration test written (read isolation + write isolation)
- [ ] Migration registered in `tests/common/mod.rs` `run_migrations`
- [ ] `cargo test --workspace` passes
