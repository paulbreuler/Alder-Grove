---
paths:
  - "crates/grove-api/**/*.rs"
  - "crates/grove-domain/**/*.rs"
  - "crates/grove-sync/**/*.rs"
---

# API Rules (Rust Axum 0.8)

**Crates**: axum 0.8, sqlx 0.8 (PgPool), tower/tower-http, utoipa, tracing

## Hexagonal Layers

```
crates/grove-api/src/
  ├─ routes/        # HTTP handlers (adapters — inbound)
  ├─ db/            # Database queries (adapters — outbound)
  ├─ auth/          # Clerk JWT middleware
  ├─ acp/           # ACP WebSocket server, broker, session management
  └─ sync/          # CRDT sync handler, PostgreSQL bridge

crates/grove-domain/src/
  ├─ *.rs           # Entity structs (one per entity)
  ├─ ports.rs       # Port traits (repository interfaces)
  ├─ error.rs       # DomainError enum
  ├─ common.rs      # AiProvenance, shared types
  └─ acp.rs         # ACP protocol message types
```

### Domain
- Pure Rust structs and traits — no Axum, no sqlx, no HTTP types
- Define port traits here (e.g., `trait PersonaRepository`)
- Use `async_trait` or native async in traits (Rust 1.75+) for async ports
- Business rules and validation logic live here
- Store ports as `Arc<dyn PortTrait>` in Axum state for runtime polymorphism

### Routes (Inbound Adapters)
- Axum 0.8 handlers that extract request data and call domain/application logic
- Always extract `org_id` and `workspace_id` from auth middleware
- Return RFC 9457 Problem Details on errors via `ApiError` + `IntoResponse`
- Use Axum 0.8 path syntax: `/{org_id}/workspaces/{ws_id}/personas/{id}`
  (NOT `/:org_id` — that's the old 0.7 syntax)
- Use `axum::extract::State`, `Path`, `Query`, `Json` extractors
- For optional extractors, implement `OptionalFromRequestParts` (Axum 0.8)
- **Never** import concrete repo types (e.g., `PgWorkspaceRepo`) in route handlers —
  consume repos via `state.xxx_repo` (`Arc<dyn PortTrait>`) only

### DB (Outbound Adapters)
- Implement domain port traits using sqlx 0.8
- Every entity query MUST include `workspace_id` in WHERE clauses (tenant isolation)
- Use runtime `sqlx::query_as::<_, Row>()` — avoids compile-time DB dependency
- Use parameterized queries (`$1`, `$2`) — never string interpolation for data
- **Exception**: PostgreSQL `SET` commands do NOT support `$1` params — use
  `format!()` only with validated types like `Uuid` (hex + dashes only)
- Separate domain types from DB rows: private `XxxRow` struct with `FromRow`,
  then `impl From<XxxRow> for DomainType`
- Wire concrete adapters in `main.rs` and `tests/common/mod.rs` only —
  these are the composition roots

### Workspace Queries (Superuser)
- Workspace CRUD runs as **superuser** (no `TenantTx`)
- Workspace table RLS (`id = current_workspace_id()`) is for sub-workspace
  entity isolation — listing all workspaces for an org requires superuser
- **SECURITY GAP (pre-auth):** Org-level isolation currently relies solely on
  `org_id` WHERE clauses — application-level filtering, no database enforcement.
  When Clerk auth lands, harden with `SET LOCAL app.current_org_id` + an RLS
  policy on the workspaces table so that a missed WHERE clause cannot leak
  cross-org data

### Entity Queries (TenantTx)
- All sub-workspace entity queries (personas, journeys, specs, etc.) use `TenantTx`
- `TenantTx::begin(&pool, workspace_id)` sets `SET LOCAL ROLE grove_app` +
  `SET LOCAL app.current_workspace_id` within a transaction
- Context is automatically cleared on COMMIT or ROLLBACK
- Always call `tx.commit()` — dropping without commit rolls back

### Connection Pool Safety
- `create_pool()` registers an `after_release` callback
- Callback runs `RESET ROLE` + `RESET app.current_workspace_id` (two separate
  statements — prepared statements don't allow multi-command)
- Prevents leaked tenant context if a connection is reused outside a transaction

## Multi-Tenancy

- Every route is scoped: `/orgs/{org_id}/workspaces/{ws_id}/...`
- Sub-workspace entities enforced at database level via RLS (`TenantTx`)
- **NOT YET IMPLEMENTED:** Auth middleware to extract `org_id` from Clerk JWT
  (currently `org_id` comes from URL path — trusted only for dev)
- **NOT YET IMPLEMENTED:** RLS policy on workspaces for org-level isolation
  (`SET LOCAL app.current_org_id` + policy)
- Queries without workspace_id scoping are a security bug

## Middleware (Tower)

- Use `tower-http` for CORS, compression, request tracing
- Auth middleware as a Tower layer — runs before handlers
- Compose middleware with `ServiceBuilder` or Axum's `.layer()`

## Error Handling

- Use RFC 9457 Problem Details for all error responses
- `ApiError` enum implements `IntoResponse` — maps to status + JSON body
- Response MUST set `Content-Type: application/problem+json` (not `application/json`)
- Response body: `{ "type": "about:blank", "title": "...", "status": N, "detail": "..." }`
- `DomainError` maps to `ApiError` via `From` — keeps domain pure
- Never expose internal error details (stack traces, SQL errors) to clients
- Log errors with structured `tracing::error!`

## API Documentation

- All routes documented with `#[utoipa::path]` attributes
- OpenAPI spec served at `/api-docs`
- Swagger UI available in development
