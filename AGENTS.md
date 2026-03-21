<!-- GENERATED FROM .claude/ — DO NOT EDIT BY HAND -->

# Codex Repository Instructions

This repository keeps canonical assistant playbooks in `.claude/`.
Use the generated compatibility files in `.agents/skills/` to bridge those playbooks into Codex.

Project-wide guidance lives in `CLAUDE.md`.
Path-specific guidance lives in `.claude/rules/*.md`.

## Command Playbooks

When a user invokes a slash-style command, load the matching file from `.claude/commands/`.

- `/add-api-endpoint` -> `.claude/commands/add-api-endpoint.md`
- `/add-component` -> `.claude/commands/add-component.md`
- `/add-extension` -> `.claude/commands/add-extension.md`
- `/add-feature` -> `.claude/commands/add-feature.md`
- `/add-migration` -> `.claude/commands/add-migration.md`
- `/add-tauri-command` -> `.claude/commands/add-tauri-command.md`
- `/audit` -> `.claude/commands/audit.md`
- `/audit-docs` -> `.claude/commands/audit-docs.md`
- `/audit-security` -> `.claude/commands/audit-security.md`
- `/audit-tests` -> `.claude/commands/audit-tests.md`
- `/audit-tokens` -> `.claude/commands/audit-tokens.md`
- `/check-architecture` -> `.claude/commands/check-architecture.md`
- `/check-backend-architecture` -> `.claude/commands/check-backend-architecture.md`
- `/check-frontend-architecture` -> `.claude/commands/check-frontend-architecture.md`
- `/code-review` -> `.claude/commands/code-review.md`
- `/commit` -> `.claude/commands/commit.md`
- `/pr` -> `.claude/commands/pr.md`
- `/red-team` -> `.claude/commands/red-team.md`
- `/scaffold-entity` -> `.claude/commands/scaffold-entity.md`

## Skills

Codex discovers skills from `.agents/skills/*/SKILL.md`.
Each generated skill file is a minimal wrapper around the canonical `.claude/skills/*/SKILL.md` source.

- `add-api-endpoint` -> `.claude/skills/add-api-endpoint/SKILL.md`
- `add-component` -> `.claude/skills/add-component/SKILL.md`
- `add-extension` -> `.claude/skills/add-extension/SKILL.md`
- `add-feature` -> `.claude/skills/add-feature/SKILL.md`
- `add-migration` -> `.claude/skills/add-migration/SKILL.md`
- `add-tauri-command` -> `.claude/skills/add-tauri-command/SKILL.md`
- `audit` -> `.claude/skills/audit/SKILL.md`
- `audit-docs` -> `.claude/skills/audit-docs/SKILL.md`
- `audit-security` -> `.claude/skills/audit-security/SKILL.md`
- `audit-tests` -> `.claude/skills/audit-tests/SKILL.md`
- `audit-tokens` -> `.claude/skills/audit-tokens/SKILL.md`
- `check-architecture` -> `.claude/skills/check-architecture/SKILL.md`
- `check-backend-architecture` -> `.claude/skills/check-backend-architecture/SKILL.md`
- `check-frontend-architecture` -> `.claude/skills/check-frontend-architecture/SKILL.md`
- `code-review` -> `.claude/skills/code-review/SKILL.md`
- `commit` -> `.claude/skills/commit/SKILL.md`
- `pr` -> `.claude/skills/pr/SKILL.md`
- `red-team` -> `.claude/skills/red-team/SKILL.md`
- `scaffold-entity` -> `.claude/skills/scaffold-entity/SKILL.md`

## Agent Roles

- `api-developer` -> `.claude/agents/api-developer.md`
- `backend-architect` -> `.claude/agents/backend-architect.md`
- `debugger` -> `.claude/agents/debugger.md`
- `domain-expert` -> `.claude/agents/domain-expert.md`
- `frontend-architect` -> `.claude/agents/frontend-architect.md`
- `frontend-developer` -> `.claude/agents/frontend-developer.md`
- `researcher` -> `.claude/agents/researcher.md`
- `security-reviewer` -> `.claude/agents/security-reviewer.md`
- `test-runner` -> `.claude/agents/test-runner.md`

## Generated File Policy

- `AGENTS.md`, `.agents/skills/*`, `GEMINI.md`, `.gemini/*`, and `.github/copilot-instructions.md` are generated from `.claude/`.
- Do not edit generated files by hand. Update `CLAUDE.md` or `.claude/**`, then rerun `pnpm ai:generate`.
