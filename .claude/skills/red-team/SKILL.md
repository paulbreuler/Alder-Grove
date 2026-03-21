---
name: red-team
description: Adversarial security sweep for Alder Grove — TenantTx bypass, cross-workspace IDOR, Clerk JWT tampering, ACP session hijacking, gate bypass, CRDT tampering
user_invocable: true
---

# /red-team

Adversarial security sweep targeting Alder Grove's multi-tenant isolation,
authentication, and authorization boundaries.

Accepts optional scope via `$ARGUMENTS`:

| Invocation        | Scope                                                  |
| ----------------- | ------------------------------------------------------ |
| `/red-team`       | Full sweep — all 8 vectors                             |
| `/red-team rls`   | Row-level security only (vectors 1-3)                  |
| `/red-team acp`   | Agent Communication Protocol only (vectors 5-7)        |
| `/red-team auth`  | Clerk JWT + route auth only (vectors 4, 8)             |

## Phase 1 — Recon

### 1.1 RLS Inventory

Inspect the database schema for row-level security posture:

- Search `crates/grove-api/src/db/` for `CREATE POLICY`, `ALTER TABLE ... ENABLE ROW LEVEL SECURITY`, and `SET LOCAL` GUC statements
- Search migration files (if present) for RLS policy definitions
- Catalog which tables have RLS enabled and which do not
- Note the GUC variable used for tenant context (`app.current_workspace_id`)

### 1.2 API Endpoint Inventory

Enumerate all route handlers and their auth middleware:

- Search `crates/grove-api/src/routes/` for all route definitions (`.route()`, `.get()`, `.post()`, `.put()`, `.patch()`, `.delete()`)
- For each route, verify presence of Clerk auth middleware/extractor
- Flag any route missing auth middleware

### 1.3 JWT Claim Inventory

Audit how tenant identity flows from JWT to query:

- Search `crates/grove-api/src/extract.rs` for JWT claim extraction
- Trace how `org_id` and `workspace_id` are sourced (path params, JWT claims, headers)
- Verify claims are validated, not just parsed
- Check `resolve_workspace()` for ownership verification

## Phase 2 — Adversarial Tests

For each vector: define negative control (expected rejection), exploit attempt,
and proof of success/failure. Skip vectors outside the requested scope.

### Vector 1: TenantTx Bypass

**Target:** `TenantTx` wrapper in `crates/grove-api/src/db/tenant.rs`

- **Negative control:** Confirm that queries through `TenantTx` only return rows for the set workspace
- **Exploit:** Can a handler reset `app.current_workspace_id` GUC mid-transaction? Can a query bypass TenantTx by using the raw pool directly?
- **Proof:** Search for any handler that acquires a connection from the pool without going through `TenantTx`. Search for any `SET LOCAL` or `RESET` of the GUC outside the TenantTx setup path

### Vector 2: Cross-Workspace IDOR

**Target:** All entity CRUD routes

- **Negative control:** Request entity UUID belonging to workspace B using workspace A credentials — expect 404 or 403
- **Exploit:** Do repo queries filter by workspace_id, or only by entity UUID? Can a user enumerate entity UUIDs across workspaces?
- **Proof:** Search all repo `SELECT` queries for missing `workspace_id` WHERE clauses. Check if UUID-based lookups include workspace scoping

### Vector 3: GUC Injection

**Target:** `app.current_workspace_id` GUC lifecycle

- **Negative control:** GUC is set only via TenantTx, not from user-controlled input
- **Exploit:** Can the GUC value be injected via HTTP header, query parameter, request body field, or nested/savepoint transaction?
- **Proof:** Search for any code path that constructs a `SET LOCAL` statement from user input. Check `after_release` pool hook resets GUC

### Vector 4: Clerk JWT Tampering

**Target:** JWT validation in auth middleware

- **Negative control:** Valid Clerk JWT accepted, invalid rejected
- **Exploit:** Does the validator reject `alg: none`? Does it validate the issuer (`iss`) and audience (`aud`)? Does it check token expiration? Is the JWKS endpoint pinned or could a self-signed key be injected?
- **Proof:** Search auth middleware for algorithm allowlist, issuer validation, expiration checks. Check if Clerk SDK handles these or if custom validation is used

### Vector 5: ACP Session Hijacking

**Target:** WebSocket ACP connections

- **Negative control:** WebSocket upgrade requires valid auth, session belongs to requesting user
- **Exploit:** Can user A connect to user B's session by providing B's session_id? Is session ownership verified at WebSocket upgrade time?
- **Proof:** Search ACP handler for session ownership check. Verify session_id is scoped to workspace and user

### Vector 6: Gate Approval Bypass

**Target:** Gate approval endpoints

- **Negative control:** Gate approval requires correct permissions and valid session state
- **Exploit:** Can a gate be approved without the required permission? Can a gate be approved when the session is not in a state that expects gate approval? Can gate_id be guessed?
- **Proof:** Search gate approval handler for permission checks, session state validation, workspace scoping

### Vector 7: CRDT Document Tampering

**Target:** Collaborative document sync via Yrs

- **Negative control:** CRDT updates require valid session and document ownership
- **Exploit:** Can a client send Yrs updates for a document in another workspace? Is the document ID validated against the session's workspace?
- **Proof:** Search CRDT sync handlers for workspace scoping on document access

### Vector 8: Missing Auth on Routes

**Target:** All API routes

- **Negative control:** Every non-health route requires Clerk JWT
- **Exploit:** Call each endpoint without an Authorization header — expect 401
- **Proof:** Compare route inventory (Phase 1.2) against auth middleware coverage. Flag any route (excluding `/health`) without auth

## Phase 3 — Chain Rules

Map trigger vulnerabilities to derived attacks:

| Trigger (if exploited)          | Derived Attack                                    |
| ------------------------------- | ------------------------------------------------- |
| TenantTx bypass (V1)           | Cross-workspace data exfiltration                 |
| Cross-workspace IDOR (V2)      | Entity enumeration, data theft                    |
| GUC injection (V3)             | Arbitrary workspace impersonation                 |
| JWT tampering (V4)             | Full account takeover, org escalation             |
| ACP session hijack (V5)        | Agent impersonation, unauthorized code execution  |
| Gate bypass (V6)               | Skip approval checkpoints, unsafe deployments     |
| CRDT tampering (V7)            | Document corruption, injected content             |
| Missing auth (V8)              | Unauthenticated access to any unprotected route   |
| V3 + V1                        | Full tenant isolation bypass                      |
| V4 + V5                        | Authenticated agent session hijacking             |
| V8 + V2                        | Unauthenticated cross-workspace IDOR              |

## Output Format

```
=== ALDER GROVE RED TEAM REPORT ===
Scope: [full | rls | acp | auth]
Date: YYYY-MM-DD

--- Phase 1: Recon Summary ---
Tables with RLS: [count] / [total]
Routes with auth: [count] / [total]
JWT validation: [Clerk SDK | custom | mixed]

--- Phase 2: Findings ---

[CONFIRMED | LIKELY | INFORMATIONAL] V<N>: <Title>
  Confidence: <0-100>%
  Evidence: <file:line — what was found>
  Impact: <what an attacker could do>
  Negative control: [PASSED | FAILED — explanation]
  Remediation: <specific fix>

... (repeat for each vector tested) ...

--- Phase 3: Chain Paths ---
<trigger> -> <derived> : [CONFIRMED | LIKELY | INFORMATIONAL]

--- Summary ---
Confirmed (>=80%): [count]
Likely (>=50%):     [count]
Informational:      [count]
Negative controls passed: [count] / [total]
Residual risks: [list of accepted risks or areas needing manual review]

=== END REPORT ===
```

## Rules

- Every vector requires a negative control test — if the control passes, note it
- Never fabricate findings — if a code path does not exist yet, mark as INFORMATIONAL with note "not yet implemented"
- All file references must be actual paths in the codebase
- Scope argument filters which vectors to run; unscoped runs all 8
- No destructive actions — this is a read-only audit
- Report must include both vulnerabilities found AND controls that are working correctly
