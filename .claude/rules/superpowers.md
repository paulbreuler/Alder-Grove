---
paths:
  - ".claude/skills/*/SKILL.md"
---

# Superpowers Integration

Alder Grove uses [superpowers](https://github.com/anthropics/superpowers) (v5.0.0+) for
AI-assisted planning and development workflows.

## Artifact Paths (project override)

| Artifact             | Path                                                | Naming                          |
| -------------------- | --------------------------------------------------- | ------------------------------- |
| Design specs         | `.docs/superpowers/specs/YYYY-MM-DD-topic-design.md`| Date-prefixed, `-design` suffix |
| Implementation plans | `.docs/superpowers/plans/YYYY-MM-DD-topic.md`       | Date-prefixed                   |

Both directories are **gitignored**. Never commit specs or plans.

## Workflow

```
Brainstorm -> Plan -> Execute -> Review
```

1. `superpowers:brainstorming` — explore requirements, save design spec to `.docs/superpowers/specs/`
2. `superpowers:writing-plans` — break spec into TDD tasks, save plan to `.docs/superpowers/plans/`
3. `superpowers:executing-plans` — implement with test-driven development
4. `superpowers:code-reviewer` — validate against plan and standards

## Skill Dependencies

Some Alder Grove skills depend on superpowers:

| Alder Skill    | Superpowers Dependency                       | Type     |
| -------------- | -------------------------------------------- | -------- |
| `/code-review` | `superpowers:code-reviewer` agent            | **HARD** |
| `/pr`          | `superpowers:verification-before-completion` | Soft     |

- **HARD dependency:** STOP and tell the user to install superpowers
- **Soft dependency:** WARN the user, then continue with reduced capability

## Rules

- Design specs and plans are ephemeral working documents — never commit them
- Specs drive plans; plans drive execution. Skip steps only if the user explicitly says to
