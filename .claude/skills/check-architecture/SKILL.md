---
name: check-architecture
description: Coordinate full-stack architecture review using frontend and backend specialist agents
user_invocable: true
---

# /check-architecture

Run the architecture review as a **parallel specialists** workflow using the
agent-team patterns documented in
`docs/research/2026-03-14-agent-teams-research.md`.

## Team Pattern

- **Frontend architect** reviews React/TypeScript architecture in `src/`
- **Backend architect** reviews Rust hexagonal architecture in `crates/`
- **Lead reviewer** aggregates both reports into a single result

Use the agent-team guidance from the research doc:

- Start with read-only research
- Use role-based specialists with minimal overlap
- Prefer parallel review passes
- Aggregate findings after both specialists finish
- Treat architectural changes as high-scrutiny work

## Workflow

1. Run `/check-frontend-architecture`
2. Run `/check-backend-architecture`
3. Merge both results into one report
4. Report overall PASS only if both specialist checks pass

## Output

Report in three sections:

```text
=== ARCHITECTURE CHECK ===

Frontend Architecture
  [frontend specialist report]
  RESULT: PASS|FAIL

Backend Architecture
  [backend specialist report]
  RESULT: PASS|FAIL

=== OVERALL: PASS|FAIL ===
```

## Rules

- Do not collapse frontend and backend criteria into one checklist
- Keep specialist findings grouped by stack
- If one stack is not implemented yet, report `N/A` with evidence
- Cite file paths and concrete violations for every failure
