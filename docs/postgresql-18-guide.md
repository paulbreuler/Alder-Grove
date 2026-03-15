# PostgreSQL 18 Guide for Alder Grove

> Features, patterns, and gotchas specific to PostgreSQL 18 that affect
> Alder Grove's database layer.

---

## PG18 Features We Use

### uuidv7() — Time-Sortable Primary Keys

```sql
id UUID PRIMARY KEY DEFAULT uuidv7()
```

Native PG18 function. Generates RFC 9562 UUIDs with embedded timestamp —
naturally ordered by creation time without a separate `created_at` index for
ordering. All 19 tables use this as the default PK.

**Why not UUIDv4?** UUIDv4 is random, causing B-tree index fragmentation on
high-insert tables. UUIDv7 inserts are append-only to the index, matching
auto-increment performance with distributed-system compatibility.

### Virtual Generated Columns (Default in PG18)

PG18 changed the default from `STORED` to `VIRTUAL` for generated columns.
Virtual columns compute at read time and occupy no storage.

**Current schema uses STORED explicitly:**

```sql
acceptance_count INTEGER GENERATED ALWAYS AS (jsonb_array_length(requirements->'acceptance')) STORED,
```

We use `STORED` because:
- `jsonb_array_length()` on a JSONB column is cheap but not free
- These counts are read far more often than written (dashboard queries)
- STORED columns can be indexed; VIRTUAL cannot

**Rule:** Always declare `STORED` or `VIRTUAL` explicitly. Never rely on the
default — it changed between PG17 (stored) and PG18 (virtual).

### AS RESTRICTIVE Row Level Security Policies

```sql
CREATE POLICY events_no_update ON events AS RESTRICTIVE FOR UPDATE USING (false);
```

RESTRICTIVE policies use AND logic — they must pass alongside permissive
policies. This is the correct way to block specific operations (UPDATE/DELETE)
while still allowing others (SELECT/INSERT) through permissive policies.

**Do NOT use triggers for operation blocking when RESTRICTIVE policies exist.**
Available since PG15, this is the database-native mechanism.

### FORCE ROW LEVEL SECURITY

```sql
ALTER TABLE personas FORCE ROW LEVEL SECURITY;
```

Without FORCE, the table owner bypasses RLS. With FORCE, RLS applies to
everyone including the table owner. Critical for defense-in-depth when the
application connects as the table owner and uses `SET ROLE` to downgrade.

---

## PG18 Features to Adopt (Future Chunks)

### OLD/NEW in RETURNING Clause

PG18 adds access to previous and current values in RETURNING:

```sql
UPDATE sessions SET status = 'completed'
WHERE id = $1
RETURNING OLD.status AS previous_status, NEW.status AS current_status;
```

**Relevant for:** Session state machine transitions — the API can return both
the old and new status in a single query without a separate SELECT. Adopt in
Chunk B when wiring up session lifecycle endpoints.

### Temporal Constraints (WITHOUT OVERLAPS)

```sql
CREATE TABLE schedules (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    resource_id UUID NOT NULL,
    valid_during TSTZRANGE NOT NULL,
    EXCLUDE USING GIST (resource_id WITH =, valid_during WITH &&)
);
```

PG18 adds native `WITHOUT OVERLAPS` for PRIMARY KEY and UNIQUE constraints,
and `PERIOD` for foreign keys. Not currently needed but relevant if we add:
- Gate timeout windows (non-overlapping expiration ranges)
- Session scheduling (non-overlapping agent execution windows)
- Versioned guardrails with validity periods

### Async I/O (AIO)

PG18's new asynchronous I/O subsystem provides up to 3x read performance for
sequential scans, bitmap heap scans, and vacuum. This is transparent — no
schema changes needed. Benefits large table scans (events, audit logs).

**Docker configuration:** AIO is enabled by default in PG18. No action needed.

### Skip Scan for Multicolumn B-tree Indexes

PG18 can skip leading columns in multicolumn indexes when the leading column
has few distinct values. Our composite indexes like
`(workspace_id, status)` on sessions benefit automatically — a query filtering
only on `status` can now use this index via skip scan.

**No action needed** — the query planner uses this automatically.

### OAuth Authentication

PG18 supports OAuth 2.0 for database authentication. Not relevant for
Alder Grove (we use Clerk JWT → application-layer auth, not direct DB auth).

---

## Docker Volume Mount (Breaking Change)

PG18 changed the default `PGDATA` from `/var/lib/postgresql/data` to
`/var/lib/postgresql/18/docker`. This breaks existing Docker volume mounts.

**Our docker-compose.yml:**

```yaml
volumes:
  - grove_data:/var/lib/postgresql  # NOT /var/lib/postgresql/data
```

Mount at `/var/lib/postgresql` (the parent), not the data subdirectory.
PG18 creates the data directory at `18/docker/` beneath the mount point.

**If upgrading from PG17 volumes:** The container will refuse to start.
Either remove the old volume (`docker compose down -v`) or set
`PGDATA=/var/lib/postgresql/data` as an environment variable.

---

## Patterns for Alder Grove

### Connection Pool Safety (sqlx)

```rust
PgPoolOptions::new()
    .after_release(|conn, _meta| Box::pin(async move {
        // Reset any lingering tenant context when connection returns to pool
        sqlx::query("RESET ROLE; RESET app.current_workspace_id")
            .execute(&mut *conn).await?;
        Ok(true)
    }))
```

**Why:** `SET LOCAL` is transaction-scoped, but if a query accidentally runs
outside a transaction, the connection retains the previous tenant's context.
The `after_release` callback ensures clean connections.

### Transaction Pattern for RLS

```rust
// CORRECT: all queries within a transaction with tenant context
let mut tx = pool.begin().await?;
sqlx::query("SET LOCAL ROLE grove_app").execute(&mut *tx).await?;
sqlx::query("SET LOCAL app.current_workspace_id = $1")
    .bind(workspace_id.to_string())
    .execute(&mut *tx).await?;

// Queries here see only the active workspace's data
let personas = sqlx::query_as!(Persona, "SELECT * FROM personas")
    .fetch_all(&mut *tx).await?;

tx.commit().await?;
// Context automatically cleared — SET LOCAL is transaction-scoped
```

```rust
// WRONG: query outside transaction bypasses RLS
let personas = sqlx::query_as!(Persona, "SELECT * FROM personas")
    .fetch_one(&pool)  // ← Runs as superuser, sees ALL data
    .await?;
```

### SECURITY DEFINER Helper Function

```sql
CREATE OR REPLACE FUNCTION current_workspace_id()
RETURNS uuid LANGUAGE sql STABLE SECURITY DEFINER
SET search_path = pg_catalog, public
AS $$
    SELECT NULLIF(current_setting('app.current_workspace_id', true), '')::uuid
$$;
```

- `SECURITY DEFINER` — executes with the privileges of the function owner
  (superuser), preventing search_path injection attacks
- `SET search_path = pg_catalog, public` — pins the search_path within the
  function body so malicious schema objects can't be resolved
- `current_setting('...', true)` — the `true` parameter returns NULL instead
  of raising an error when the setting doesn't exist
- `NULLIF('', '')` — PG18's `RESET` command sets GUC variables to empty
  string, not NULL. NULLIF converts empty string → NULL for the fail-safe

---

## Index Strategy

### Composite Indexes for RLS-Scoped Queries

Every workspace-scoped table should have `(workspace_id, id)` for efficient
RLS-filtered primary key lookups:

```sql
CREATE INDEX idx_<table>_workspace_id ON <table> (workspace_id, id);
```

PG18's skip scan means a query filtering only on `id` can still use this
index (skipping the `workspace_id` leading column), so the composite index
serves both RLS-scoped and direct-lookup queries.

### Partial Indexes for Status Filtering

```sql
CREATE INDEX idx_gates_pending_expiry ON gates (expires_at) WHERE status = 'pending';
```

Partial indexes are small and fast — they only index rows matching the WHERE
condition. Ideal for queries that filter by status (active sessions, pending
gates, enabled guardrails).

---

## References

- [PostgreSQL 18 Release Notes](https://www.postgresql.org/docs/current/release-18.html)
- [PostgreSQL 18 Press Kit](https://www.postgresql.org/about/press/presskit18/)
- [PostgreSQL 18: Generated Columns](https://www.postgresql.org/docs/current/ddl-generated-columns.html)
- [PostgreSQL 18: Row Security Policies](https://www.postgresql.org/docs/current/ddl-rowsecurity.html)
- [PostgreSQL 18: UUID Functions](https://www.postgresql.org/docs/current/functions-uuid.html)
- [Docker PG18 PGDATA Change](https://github.com/docker-library/postgres/pull/1259)
- [sqlx Multi-Tenancy Patterns](https://github.com/launchbadge/sqlx/discussions/2783)
