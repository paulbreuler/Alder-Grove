---
name: audit-security
description: OWASP Top 10 pattern scan for common vulnerabilities
user_invocable: true
---

# /audit-security

Static pattern scan for common security vulnerabilities, aligned with OWASP
Top 10 categories relevant to this stack.

## Workflow

### 1. Check: SQL injection (CRITICAL)

Search `crates/**/*.rs` for `format!` macro calls near `sqlx::query`,
`sqlx::query_as`, or `sqlx::query_scalar` invocations.

Parameterized queries using `$1`, `$2` bind parameters are safe. Flag any
query string built with `format!`, string concatenation, or interpolation.

### 2. Check: Auth bypass (CRITICAL)

Inspect route handlers in `crates/grove-api/src/routes/**/*.rs`:
- Each route handler should use Clerk auth middleware or extract authenticated
  claims
- Flag any public endpoint that is not on the allowlist

**Allowlist** (endpoints that may skip auth):
- Health check endpoint (`/health`, `/healthz`, `/readyz`)

### 3. Check: Tenant isolation (CRITICAL)

Search `crates/grove-api/src/**/*.rs` for patterns where `workspace_id` or
`org_id` is taken directly from path parameters or request body without
validation against the authenticated user's tenant context.

Safe pattern: extracting tenant from auth claims or `TenantTx`.
Unsafe pattern: trusting client-provided IDs without cross-referencing auth.

### 4. Check: Hardcoded secrets (CRITICAL)

Search all source files (excluding the items below) for patterns matching:
- `api_key`, `secret`, `password`, `token` as string literal values
  (not type names or field names)
- `DATABASE_URL`, `CLERK_SECRET_KEY` with inline values
- Any string that looks like a credential (base64-encoded keys, `sk_live_*`,
  `pk_live_*`, etc.)

**Exclude** from scan:
- `.env.example` files
- Test files (`*_test.rs`, `*.test.tsx`, `*.spec.tsx`)
- Lock files (`pnpm-lock.yaml`, `Cargo.lock`)
- `target/` and `node_modules/` directories
- Type definitions and documentation (field names like `api_key: String` are OK)

### 5. Check: Input validation (HIGH)

Search route handlers in `crates/grove-api/src/routes/**/*.rs` for `Json<T>`
extractors. Verify that the extracted type `T` either:
- Implements validation (e.g., `validator::Validate`)
- Has explicit validation logic in the handler
- Uses constrained types (newtypes with invariants)

Flag handlers that accept `Json<T>` without apparent validation.

### 6. Check: ACP WebSocket session ownership (HIGH)

If WebSocket handlers exist in `crates/grove-api/src/`:
- Verify that session ownership is validated (the connecting user owns or has
  access to the session)
- Flag any WebSocket upgrade that does not check session ownership

If no WebSocket handlers exist, SKIP this check.

### 7. Check: Dependency audit (MEDIUM)

Run `cargo audit` if the tool is installed:
- Report any known vulnerabilities
- If `cargo audit` is not installed, report as SKIP (not FAIL)

## Output Format

```
=== SECURITY AUDIT ===

Summary
| Metric              | Value  |
|---------------------|--------|
| Status              | PASS/FAIL |
| Critical findings   | N      |
| High findings       | N      |
| Medium findings     | N      |
| Checks run          | N      |
| Checks skipped      | N      |

Findings by Category
| Category            | Status | Findings |
|---------------------|--------|----------|
| SQL Injection       | PASS   | 0        |
| Auth Bypass         | PASS   | 0        |
| Tenant Isolation    | PASS   | 0        |
| Hardcoded Secrets   | PASS   | 0        |
| Input Validation    | WARN   | 2        |
| ACP WebSocket       | SKIP   | —        |
| Dependency Audit    | SKIP   | —        |

Detailed Findings
| Severity | Category          | Location                          | Issue                          |
|----------|-------------------|-----------------------------------|--------------------------------|
| HIGH     | Input Validation  | crates/grove-api/src/routes/foo.rs:30 | Json<CreateFoo> without validation |

=== RESULT: PASS/FAIL ===
```

## Rules

- FAIL if any CRITICAL or HIGH findings exist
- PASS if only MEDIUM findings or all clear
- SKIP checks gracefully (do not FAIL for missing optional tools)
- Report exact file paths and line numbers
- False positives: when uncertain, report as finding with a note to verify manually
- Health check endpoints are explicitly allowed to skip auth
