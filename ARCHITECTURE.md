# Alder Grove — Architecture Reference

> Extracted from base-plan.md, modernized for a fresh start.
> Old plan is superseded — this is the living reference.

---

## Identity

- **Product**: Desktop OS for AI-assisted software development
- **Tagline**: "Your applications grow in the Grove."
- **Target**: Startups, solo devs, small teams
- **Deployment**: Desktop Tauri v2 client + cloud-hosted Axum API server

---

## Tech Stack

| Layer       | Technology                          | Notes                            |
| ----------- | ----------------------------------- | -------------------------------- |
| Desktop     | Tauri v2                            | Native shell, local filesystem   |
| Frontend    | React 19.2, Vite, TypeScript        | Inside Tauri webview             |
| State       | Zustand                             | Per-feature stores               |
| UI Base     | Alder Shell (`@paulbreuler/shell`)  | Extension system, Workbench      |
| Animation   | Motion 12 + MotionPlus              | `motion/react`, never framer-motion |
| Design      | `--grove-*` tokens                  | Token-only, no raw CSS values    |
| API         | Rust (Axum 0.8)                     | Cloud-deployed, independent      |
| Database    | PostgreSQL 18                       | Multi-tenant, workspace-scoped   |
| Auth        | Clerk                               | JWT on API, ClerkProvider on FE  |
| Testing     | Vitest (unit), Playwright (e2e)     | TDD mandatory                    |
| Package Mgr | pnpm workspaces                     | Monorepo                         |
| Rust WS     | Cargo workspace                     | src-tauri + src-api              |
| Registry    | GitHub Packages                     | `@paulbreuler` scope             |

---

## Entity Model

Everything is workspace-scoped. Organization is managed by Clerk.

```
Organization (Clerk-managed)
  └─ Workspace
       ├─ Repository (linked codebases)
       ├─ Persona (user or agent)
       ├─ Journey (flow → steps → spec links)
       ├─ Specification (criteria, tasks)
       ├─ Note (decision, learning, gotcha)
       ├─ Session (AI agent instance)
       │    ├─ Gate (approval checkpoint)
       │    └─ Event (activity log)
       └─ Snapshot (codebase analysis)
```

- Dedicated tables per entity type
- Foreign keys for relationships
- Workspace is the tenant-scoped container
- All API routes scoped by `org_id` / `workspace_id`

---

## Hexagonal Architecture

Each feature follows **domain → application → adapters → UI** layering:

```
feature/
  ├─ domain/        # Pure types, entities, business rules — no imports from other layers
  ├─ application/   # Use cases, hooks, orchestration — depends only on domain
  ├─ adapters/      # API clients, persistence, external integrations
  └─ ui/            # React components — depends on application layer
```

**Rules**:
- Domain has zero external dependencies (no React, no API, no framework)
- Application orchestrates domain logic and calls adapters through ports (interfaces)
- UI calls application hooks/services, never domain or adapters directly
- Dependencies flow inward: UI → Application → Domain ← Adapters

---

## Shell Extension Model

Each major feature registers as an Alder Shell extension:

| Extension   | Scope | Description                                  |
| ----------- | ----- | -------------------------------------------- |
| Home        | v1    | Dashboard / landing                          |
| Workspace   | v1    | Workspace management, switching              |
| Personas    | v1    | Persona CRUD + AI assist                     |
| Journeys    | v1    | Journey mapping, steps, spec links           |
| Specs       | v1    | Specification management, criteria, tasks    |
| ACP         | v1    | Agent sessions, gates, guardrails            |
| Settings    | v1    | App and workspace settings                   |
| Snapshots   | v2    | Codebase intelligence, reverse-engineering   |

Each extension has an `extension.tsx` entry point that registers with `bootstrapShell()`.

---

## Multi-Tenancy

- **Clerk** manages Organizations and user membership
- Frontend: `<ClerkProvider>` wraps the app, hooks provide org/workspace context
- API: Clerk JWT in `Authorization` header → middleware extracts `org_id`
- Every API route is scoped: `/orgs/{org_id}/workspaces/{ws_id}/...`
- Workspace is the isolation boundary for all content entities

---

## Agent Communication Protocol (ACP)

- WebSocket-based protocol between Grove and AI agents
- Handles: session lifecycle, message routing, gate enforcement
- Frontend: Zustand store for session state, hooks for WebSocket management
- API: Axum WebSocket server in `src-api/acp/`
- **Gates**: Configurable approval checkpoints — pause agent, surface for human review
- **Guardrails**: First-class managed entities (not scattered repo files)

---

## Core Principles

Carried forward and refined:

1. **TDD mandatory** — RED → GREEN → REFACTOR, no exceptions
2. **Hexagonal architecture** — domain → application → adapters → UI
3. **Design tokens only** — `--grove-*` tokens, never raw values
4. **Shell extension model** — features as extensions, not monolithic routes
5. **Conventional commits** — `feat:`, `fix:`, `refactor:`, etc.
6. **Quality gates** — `pnpm check` + `pnpm test` before any PR
7. **Supervised AI** — gates and guardrails are product, not overhead

---

## API Design

- **Error format**: RFC 9457 Problem Details
- **Documentation**: utoipa + Swagger UI (OpenAPI)
- **Migrations**: SQL files, versioned, run at startup
- **Auth middleware**: JWKS fetch, JWT decode, org/workspace extraction
