---
name: code-review
description: Dispatch domain-specific code reviewers based on what changed
user_invocable: true
dependencies:
  - skill: superpowers:code-reviewer
    type: hard
---

# /code-review

Dispatch the superpowers code-reviewer agent with domain-specific context.
Routes to the correct specialist based on what files changed.

**Hard dependency**: Requires `superpowers:code-reviewer` agent. Stop if unavailable.

## Workflow

1. **Check availability**: Verify superpowers:code-reviewer is available
2. **Gather context**:
   - `git log main..HEAD --oneline` — commits on this branch
   - `git diff main...HEAD --stat` — files changed
   - `git diff main...HEAD` — full diff
3. **Determine scope**: Classify changed files into backend / frontend / both
4. **Find design spec**: Search `.docs/superpowers/specs/` for matching spec
5. **Dispatch**: Send one or two reviewers based on scope (see below)

## Scope Detection

Classify the diff:

| Files changed | Scope | Reviewer |
|---------------|-------|----------|
| Only `crates/**/*.rs` | Backend | Backend review criteria |
| Only `src/**/*.{ts,tsx,css}` | Frontend | Frontend review criteria |
| Both | Full-stack | Dispatch **two** parallel reviews |

## Backend Review Criteria

Provide when `crates/` files changed:

```
Rust backend review criteria:
1. Hexagonal architecture — domain has no framework imports, correct layer deps
2. Port abstraction — handlers consume repos via Arc<dyn PortTrait> from state,
   never import concrete adapter types. Composition roots are main.rs and
   tests/common/mod.rs only.
3. TDD compliance — tests exist for new code
4. Multi-tenant isolation — entity queries use TenantTx, workspace queries
   include org_id WHERE clauses (SECURITY GAP pre-auth — see api rules)
5. Error handling — ApiError implements RFC 9457 with application/problem+json
6. Pool safety — after_release resets tenant context
7. Clippy clean — cargo clippy -p <crate> -- -D warnings
8. Documented architecture rules in .claude/rules/api.md are HARD requirements,
   not suggestions. Violations are FAIL, not "acceptable for v1."
```

## Frontend Review Criteria

Provide when `src/` files changed:

```
React/TypeScript frontend review criteria:
1. Hexagonal architecture — domain → application → adapters → UI layer deps
2. Shell extension model — features as extensions, not ad-hoc routes
3. TDD compliance — tests exist for new code
4. Design token compliance — no raw CSS values, only --grove-* tokens
5. Zustand stores — per-feature, no cross-store imports in domain
6. React 19.2 patterns — proper use of hooks, no deprecated patterns
7. Error boundaries — proper error handling in UI
8. Documented architecture rules in .claude/rules/frontend.md are HARD
   requirements. Violations are FAIL.
```

## Shared Criteria (always include)

```
Cross-cutting review criteria:
1. Conventional commits — commit messages follow feat:/fix:/refactor: format
2. Security — no secrets in code, input sanitized at boundaries
3. No over-engineering — only changes directly requested
```

## Dispatch Context Template

```
What was implemented: $ARGUMENTS (or summary of branch changes)

Plan/requirements: [design spec path or relevant CLAUDE.md section]

[Backend/Frontend/Both review criteria from above]

[Shared criteria]

IMPORTANT: Documented architecture rules (.claude/rules/) are HARD requirements.
If code violates a documented rule, it is a FAIL — not "acceptable for v1."
Only the user can grant exceptions.

Commit range: main..HEAD
```

## Rules

- Never perform the review yourself — always dispatch to the agent
- Always include the correct domain-specific criteria based on scope
- For full-stack changes, dispatch **two** parallel reviews (backend + frontend)
- Always search for a design spec before dispatching
- If superpowers:code-reviewer is not available, STOP and inform the user
