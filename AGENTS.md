<!-- GENERATED FROM .claude/ — DO NOT EDIT BY HAND -->

# Codex Repository Instructions

This repository keeps canonical assistant playbooks in `.claude/`.
Use the generated compatibility files in `.agents/skills/` to bridge those playbooks into Codex.

Project-wide guidance lives in `CLAUDE.md`.
Path-specific guidance lives in `.claude/rules/*.md`.

## Command Playbooks

When a user invokes a slash-style command, load the matching file from `.claude/commands/`.

- `/audit` -> `.claude/commands/audit.md`
- `/check-architecture` -> `.claude/commands/check-architecture.md`
- `/check-backend-architecture` -> `.claude/commands/check-backend-architecture.md`
- `/check-frontend-architecture` -> `.claude/commands/check-frontend-architecture.md`
- `/code-review` -> `.claude/commands/code-review.md`
- `/commit` -> `.claude/commands/commit.md`
- `/pr` -> `.claude/commands/pr.md`

## Skills

Codex discovers skills from `.agents/skills/*/SKILL.md`.
Each generated skill file is a minimal wrapper around the canonical `.claude/skills/*/SKILL.md` source.

- `audit` -> `.claude/skills/audit/SKILL.md`
- `check-architecture` -> `.claude/skills/check-architecture/SKILL.md`
- `check-backend-architecture` -> `.claude/skills/check-backend-architecture/SKILL.md`
- `check-frontend-architecture` -> `.claude/skills/check-frontend-architecture/SKILL.md`
- `code-review` -> `.claude/skills/code-review/SKILL.md`
- `commit` -> `.claude/skills/commit/SKILL.md`
- `pr` -> `.claude/skills/pr/SKILL.md`

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
