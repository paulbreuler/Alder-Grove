---
name: check-architecture
description: Verify hexagonal architecture constraints across the codebase
user_invocable: true
---

# /check-architecture

Verify that the codebase follows hexagonal architecture constraints.

## Checks

### 1. Frontend Dependency Direction

For each feature under `src/features/`:

- **Domain** must NOT import from `application/`, `adapters/`, or `ui/`
- **Domain** must NOT import React, Zustand, fetch, or any framework
- **Application** must NOT import from `ui/`
- **Application** may import from `domain/` and call `adapters/` through ports
- **UI** must NOT import from `domain/` or `adapters/` directly
- **UI** imports from `application/` only (hooks, stores)

**How to check**: Grep import statements in each layer and flag violations.

### 2. API Dependency Direction

In `src-api/src/`:

- **Domain** must NOT import from `routes/`, `db/`, `auth/`, or `acp/`
- **Domain** must NOT import Axum, sqlx, or HTTP types
- **Routes** may import domain types and call domain logic
- **DB** implements domain traits — may import domain types and sqlx

**How to check**: Grep `use` statements in domain modules for framework imports.

### 3. Namespace Isolation

- No cross-feature imports at the UI layer (features don't reach into each other)
- Shared types belong in `src/shared/domain/` if needed
- API domain types don't leak HTTP types (`axum::*`) or DB types (`sqlx::*`)

### 4. Design Token Compliance

- No raw CSS values in `.tsx` or `.css` files under `src/`
- All colors, spacing, radii, shadows must use `--grove-*` tokens
- Search for patterns like `color: #`, `padding: [0-9]`, `background: rgb`

### 5. Multi-Tenant Isolation

- Every SQL query in `src-api/src/db/` must include `workspace_id` in WHERE
- Every API route must extract org_id/workspace_id from auth context
- No queries that return data across workspace boundaries

### 6. Build Verification

```bash
pnpm check     # TypeScript + ESLint
pnpm test      # Vitest
cargo build    # Rust workspace
cargo test     # Rust tests
```

## Output

Report as a checklist:

```
✅ Frontend dependency direction — PASS
❌ API dependency direction — FAIL
   domain/persona.rs imports axum::Json at line 3
✅ Namespace isolation — PASS
❌ Design token compliance — FAIL
   src/features/home/ui/Dashboard.tsx:42 — raw color value #333
✅ Multi-tenant isolation — PASS
✅ Build verification — PASS

RESULT: FAIL (2 violations)
```
