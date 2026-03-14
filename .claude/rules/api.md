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
- Return RFC 9457 Problem Details on errors via `AppError` + `IntoResponse`
- Use Axum 0.8 path syntax: `/{org_id}/workspaces/{ws_id}/personas/{id}`
  (NOT `/:org_id` — that's the old 0.7 syntax)
- Use `axum::extract::State`, `Path`, `Query`, `Json` extractors
- For optional extractors, implement `OptionalFromRequestParts` (Axum 0.8)

### DB (Outbound Adapters)
- Implement domain port traits using sqlx 0.8
- Every query MUST include `workspace_id` in WHERE clauses (tenant isolation)
- Use `sqlx::query!` / `sqlx::query_as!` for compile-time checked SQL
- Use parameterized queries (`$1`, `$2`) — never string interpolation
- Get connections via `PgPool::acquire()` — return to pool on drop

## Multi-Tenancy

- Every route is scoped: `/orgs/{org_id}/workspaces/{ws_id}/...`
- Auth middleware extracts org_id from Clerk JWT claims
- Workspace membership is verified before any data access
- Queries without workspace_id scoping are a security bug

## Middleware (Tower)

- Use `tower-http` for CORS, compression, request tracing
- Auth middleware as a Tower layer — runs before handlers
- Compose middleware with `ServiceBuilder` or Axum's `.layer()`

## Error Handling

- Use RFC 9457 Problem Details for all error responses
- `AppError` enum implements `IntoResponse` — maps to status + JSON body
- Never expose internal error details (stack traces, SQL errors) to clients
- Log errors with structured `tracing::error!`

## API Documentation

- All routes documented with `#[utoipa::path]` attributes
- OpenAPI spec served at `/api-docs`
- Swagger UI available in development
