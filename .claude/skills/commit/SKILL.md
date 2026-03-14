---
name: commit
description: Stage changes and create a conventional commit
user_invocable: true
---

# /commit

Stage changes and create a conventional commit with pre-commit verification.

## Workflow

1. **Check state**: Run `git status` and `git diff` to understand what changed
2. **Verify quality**: Run `pnpm check` and `pnpm test` (frontend) and/or
   `cargo test` (Rust) depending on what changed
3. **Stage files**: Add files by name — never use `git add -A` or `git add .`
   - Do not stage `.env`, credentials, or `node_modules`
4. **Draft message**: Create a conventional commit message
   - `feat:` new feature
   - `fix:` bug fix
   - `refactor:` code restructuring
   - `test:` test additions/changes
   - `docs:` documentation
   - `chore:` build, CI, dependencies
   - Include scope when clear: `feat(personas): add create persona form`
   - Body explains WHY, not WHAT
5. **Commit**: Use HEREDOC format with Co-Authored-By line

```bash
git commit -m "$(cat <<'EOF'
feat(personas): add create persona form

Introduces the persona creation form with name, role, and description
fields. Validates required fields before submission.
EOF
)"
```

## Rules

- Never amend a previous commit — always create a new one
- Never force push
- Never skip pre-commit hooks (--no-verify)
- Never commit secrets (guard-secrets.sh will block)
- If a hook fails, fix the issue and create a NEW commit
- Stage files by name, not with -A or .
- If tests fail, do not commit — fix first
- Add a `Co-Authored-By` trailer only if the user or environment explicitly requires it
