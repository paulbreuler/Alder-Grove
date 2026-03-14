---
name: pr
description: Push branch and create a GitHub pull request
user_invocable: true
---

# /pr

Push the current branch and create a GitHub pull request via `gh` CLI.

## Workflow

1. **Check state**: Run `git status`, `git log`, and `git diff main...HEAD`
2. **Verify clean**: Ensure no uncommitted changes — commit first if needed
3. **Check remote**: Verify branch is pushed and up to date
4. **Search for spec**: Look in `.docs/superpowers/specs/` for a matching design spec
5. **Create PR**: Use `gh pr create` with structured body

```bash
gh pr create --title "feat(personas): add persona CRUD" --body "$(cat <<'EOF'
## Summary
- Added persona creation, listing, and deletion
- Integrated with workspace-scoped API endpoints
- Full TDD coverage with Vitest

## Design Spec
- `.docs/superpowers/specs/2026-03-13-personas-design.md` (if applicable)

## Test Plan
- [ ] Unit tests pass (`pnpm test`)
- [ ] Type check passes (`pnpm check`)
- [ ] API tests pass (`cargo test -p grove-api`)
- [ ] Manual verification in Tauri dev mode
EOF
)"
```

## Rules

- Title under 70 chars
- Always include test plan
- Reference design spec if one exists
- Never force push to main
- User must approve the push (guard-no-push.sh will block)
