---
name: code-review
description: Dispatch superpowers code reviewer with Alder Grove context
user_invocable: true
dependencies:
  - skill: superpowers:code-reviewer
    type: hard
---

# /code-review

Dispatch the superpowers code-reviewer agent with Alder Grove-specific context.

**Hard dependency**: Requires `superpowers:code-reviewer` agent. Stop if unavailable.

## Workflow

1. **Check availability**: Verify superpowers:code-reviewer is available
2. **Gather context**:
   - `git log main..HEAD --oneline` — commits on this branch
   - `git diff main...HEAD --stat` — files changed
   - `git diff main...HEAD` — full diff
3. **Find design spec**: Search `.docs/superpowers/specs/` for matching spec
4. **Dispatch**: Send to superpowers:code-reviewer with context below

## Dispatch Context

Provide to the code-reviewer agent:

```
What was implemented: $ARGUMENTS (or summary of branch changes)

Plan/requirements: [design spec path or relevant repo instructions from CLAUDE.md/AGENTS.md]

Alder Grove review criteria:
1. Hexagonal architecture — domain has no framework imports, correct layer dependencies
2. TDD compliance — tests exist for new code, written before implementation
3. Design token compliance — no raw CSS values, only --grove-* tokens
4. Multi-tenant isolation — all queries scoped by workspace_id
5. Shell extension model — features registered as extensions, not ad-hoc routes
6. Conventional commits — commit messages follow format
7. Error handling — RFC 9457 on API, proper error boundaries on frontend
8. Security — no secrets in code, Clerk auth validated, input sanitized

Commit range: main..HEAD
```

## Rules

- Never perform the review yourself — always dispatch to the agent
- Always include Alder Grove-specific review criteria
- Always search for a design spec before dispatching
- If superpowers:code-reviewer is not available, STOP and inform the user
