---
name: audit-docs
description: Validate documentation sync between CLAUDE.md, skills, agents, and disk
user_invocable: true
---

# /audit-docs

Validate that documentation references in `CLAUDE.md` are consistent with what
exists on disk, and that generated AI configs are up to date.

## Workflow

### 1. Check: Skills table completeness (HIGH)

- List every directory in `.claude/skills/`
- Read the Skills table in `CLAUDE.md`
- Every skill directory must have a matching row in the table
- Every row in the table must have a matching skill directory
- Report any mismatches (missing from table, or table references nonexistent skill)

### 2. Check: Agents referenced (MEDIUM)

- List every `.md` file in `.claude/agents/` (if the directory exists)
- Read the Agents table in `CLAUDE.md`
- Every agent file must have a matching row in the table
- Every row in the table must have a matching agent file
- Report any mismatches
- If `.claude/agents/` does not exist, SKIP this check

### 3. Check: Rules referenced (HIGH)

- List every `.md` file in `.claude/rules/`
- Verify each rule file is active (referenced in `.claude/settings.json` or
  loaded by convention)
- Report any orphaned or missing rule files

### 4. Check: Generated AI configs fresh (MEDIUM)

- Run `pnpm ai:check` (if the script exists in `package.json`)
- If the command exits non-zero, configs are stale — report as finding
- If the script does not exist, SKIP this check

### 5. Check: Shell extension table (MEDIUM)

- Read the Shell Extensions table in `CLAUDE.md`
- List directories in `src/features/`
- Each extension in the table should have a matching feature directory
- Report any mismatches between table and disk

## Output Format

```
=== DOCUMENTATION AUDIT ===

Sync Status
| Source              | On Disk | Referenced | Missing | Stale |
|---------------------|---------|------------|---------|-------|
| Skills              | 8       | 7          | 1       | 0     |
| Agents              | 5       | 5          | 0       | 0     |
| Rules               | 3       | 3          | 0       | 0     |
| AI Configs          | —       | —          | 0       | 1     |
| Shell Extensions    | 4       | 4          | 0       | 0     |

Findings
| Severity | Check              | Item                    | Issue                       |
|----------|--------------------|-------------------------|-----------------------------|
| HIGH     | Skills table       | /audit-tokens           | On disk but missing from CLAUDE.md |
| MEDIUM   | AI configs         | AGENTS.md               | Stale — run pnpm ai:generate      |
| MEDIUM   | Shell extensions   | Snapshots               | In table but no src/features/snapshots/ |

=== RESULT: PASS/FAIL ===
```

## Rules

- FAIL if any HIGH findings exist
- PASS if only MEDIUM or LOW findings
- SKIP checks gracefully when directories or scripts don't exist
- Report exact item names so fixes are actionable
