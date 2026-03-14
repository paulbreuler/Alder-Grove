# Alder Grove

> Desktop OS for AI-assisted software development.
> *Your applications grow in the Grove.*

## Overview

Alder Grove is a multi-tenant CMS for product and project management, tightly
integrated with AI agents. Desktop Tauri v2 client + cloud-hosted Axum API.
Alder Shell extension system provides the UI framework.

See `docs/prfaq.md` for product vision. See `docs/architecture-reference.md` for technical reference.

## Tech Stack

| Layer       | Technology                         |
| ----------- | ---------------------------------- |
| Desktop     | Tauri v2                           |
| Frontend    | React 19.2, Vite, TypeScript       |
| State       | Zustand (per-feature stores)       |
| UI Base     | Alder Shell (`@paulbreuler/shell`) |
| Animation   | Motion 12 + MotionPlus             |
| Design      | `--grove-*` tokens only            |
| API         | Rust (Axum 0.8)                    |
| Database    | PostgreSQL 18                      |
| Auth        | Clerk (JWT)                        |
| Unit Tests  | Vitest                             |
| E2E Tests   | Playwright                         |
| Package Mgr | pnpm workspaces                    |

## Architecture

**Hexagonal** — every feature follows domain → application → adapters → UI:

- **Domain**: Pure types, entities, business rules. Zero external dependencies.
- **Application**: Use cases, hooks, orchestration. Depends only on domain.
- **Adapters**: API clients, persistence, external integrations.
- **UI**: React components. Depends on application layer only.

Dependencies flow inward. Domain never imports from other layers.

## Entity Model

```
Organization (Clerk-managed)
  └─ Workspace
       ├─ Repository        # linked codebases
       ├─ Persona            # design archetype
       ├─ Journey            # flow → steps → spec links
       │    └─ Step           # ordered, AI-assessed completion
       ├─ Specification      # requirements (JSONB), tasks
       │    └─ Task           # actionable work item
       ├─ Note               # decision, learning, gotcha
       │    └─ note_links     # polymorphic entity linking
       ├─ Session            # AI agent instance [deferred]
       │    ├─ Gate           # approval checkpoint
       │    └─ Event          # activity log
       └─ Snapshot           # codebase analysis [v2]
```

All content is workspace-scoped. API routes: `/orgs/{org_id}/workspaces/{ws_id}/...`

Database: Normalized Core + Strategic JSONB. uuidv7() PKs. AI provenance on content entities. See `.docs/superpowers/specs/2026-03-13-data-model-design.md` for full schema.

## Shell Extensions

| Extension | Scope | Description                              |
| --------- | ----- | ---------------------------------------- |
| Home      | v1    | Dashboard / landing                      |
| Workspace | v1    | Workspace management, switching          |
| Personas  | v1    | Persona CRUD + AI assist                 |
| Journeys  | v1    | Journey mapping, steps, spec links       |
| Specs     | v1    | Specification management, criteria       |
| ACP       | v1    | Agent sessions, gates, guardrails        |
| Settings  | v1    | App and workspace settings               |
| Snapshots | v2    | Codebase intelligence                    |

## Development Practices

1. **TDD mandatory** — RED → GREEN → REFACTOR, no exceptions
2. **Hexagonal architecture** — enforced via `/check-architecture`
3. **Design tokens only** — `--grove-*`, never raw CSS values
4. **Shell extension model** — features as extensions
5. **Conventional commits** — `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`
6. **Quality gates** — `pnpm check` + `pnpm test` before any PR
7. **Supervised AI** — gates and guardrails are product, not overhead

## Commands

```bash
# Frontend
pnpm install              # Install dependencies
pnpm dev                  # Start Vite dev server
pnpm build                # Production build
pnpm test                 # Run Vitest
pnpm check                # TypeScript + ESLint
pnpm e2e                  # Playwright tests

# API (Rust)
cargo build -p grove-api  # Build API
cargo test -p grove-api   # Test API
cargo run -p grove-api    # Run API server

# Desktop
cargo tauri dev           # Run Tauri dev mode
cargo tauri build         # Build desktop app

# Database
docker compose up -d      # Start PostgreSQL
```

## Skills

| Skill                | Description                                     |
| -------------------- | ----------------------------------------------- |
| `/commit`            | Stage and create conventional commit            |
| `/pr`                | Push branch and create GitHub PR                |
| `/check-architecture`| Verify hexagonal constraints                    |
| `/code-review`       | Dispatch superpowers code reviewer              |
| `/audit`             | Full quality gate (arch + docs + tests)         |

## Agents

| Agent              | Trigger                              |
| ------------------ | ------------------------------------ |
| security-reviewer  | Auth, HTTP, config, input changes    |

## Key Design Decisions

1. **Desktop-first** — Tauri v2 for performance. Cloud API for collaboration.
2. **Workspace isolation** — all data scoped to workspace, enforced at API layer.
3. **ACP over direct integration** — agents communicate through protocol, not hardcoded.
4. **Gates are product** — approval checkpoints are a feature, not dev tooling overhead.
5. **Guardrails as entities** — managed in Grove, not scattered across repo files.
6. **Extensions, not routes** — Shell extensions keep features modular and independently loadable.
7. **Tokens, not styles** — `--grove-*` design tokens enforce visual consistency.
8. **TDD, not test-after** — tests drive design, not verify implementation.

## Vocabulary

| Term          | Meaning                                                    |
| ------------- | ---------------------------------------------------------- |
| Workspace     | Tenant-scoped container for all content                    |
| Persona       | A design archetype representing a user type                |
| Journey       | A user flow composed of ordered steps                      |
| Step          | An ordered action within a journey, with AI-assessed completion |
| Specification | Detailed requirements (functional, non-functional, acceptance) with tasks |
| Task          | An actionable work item under a specification              |
| Note          | A knowledge artifact: decision, learning, gotcha, or general |
| Note Link     | Polymorphic association from a note to any entity          |
| Session       | An AI agent execution instance (deferred)                  |
| Gate          | An approval checkpoint within a session                    |
| Guardrail     | A rule or constraint governing agent behavior              |
| Snapshot      | A structured analysis of a linked codebase (v2)            |
| ACP           | Agent Communication Protocol (WebSocket-based)             |
| Shell         | Alder Shell — the extension framework                      |
| Extension     | A feature module registered with the Shell                 |

## Documentation Structure

```
docs/                              # Public technical documentation
  ├── prfaq.md                     # Product vision, press release, competitor FAQ
  ├── architecture-reference.md    # Tech stack, entity model, hex layers, ACP
  └── architecture-flows.md        # Request flow, multi-tenant, ACP diagrams

.docs/                             # Internal documentation (gitignored)
  └── superpowers/
      ├── specs/                   # Design specs from brainstorming skill
      │   └── YYYY-MM-DD-<topic>-design.md
      └── plans/                   # Implementation plans from writing-plans skill
          └── YYYY-MM-DD-<topic>.md
```

- `.docs/` is **gitignored** — local working documents only
- **Brainstorming** produces specs → `.docs/superpowers/specs/`
- **Writing-plans** produces plans → `.docs/superpowers/plans/`
- Research and reference docs go at the `.docs/` root level

## What NOT to Build (v1)

- Real-time collaboration / multiplayer editing
- Third-party extension marketplace
- Mobile app
- Self-hosted API option
- Billing / subscription management (use Clerk billing or defer)
