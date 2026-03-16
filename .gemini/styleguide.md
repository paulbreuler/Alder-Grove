<!-- GENERATED FROM .claude/ — DO NOT EDIT BY HAND -->

# Alder Grove Coding Standards

Extracted from the canonical `CLAUDE.md`.

## Overview

Alder Grove is a multi-tenant CMS for product and project management, tightly
integrated with AI agents. Desktop Tauri v2 client + cloud-hosted Axum API.
Alder Shell extension system provides the UI framework.

See `docs/prfaq.md` for product vision. See `docs/architecture-reference.md` for technical reference.

---

## Architecture

**Hexagonal** — every feature follows domain → application → adapters → UI:

- **Domain**: Pure types, entities, business rules. Zero external dependencies.
- **Application**: Use cases, hooks, orchestration. Depends only on domain.
- **Adapters**: API clients, persistence, external integrations.
- **UI**: React components. Depends on application layer only.

Dependencies flow inward. Domain never imports from other layers.

---

## Development Practices

1. **TDD mandatory** — RED → GREEN → REFACTOR, no exceptions
2. **SOLID + DRY** — loosely coupled, highly cohesive, no duplicated code (see `.claude/rules/design-principles.md`)
3. **Hexagonal architecture** — enforced via `/check-architecture`
4. **Design tokens only** — `--grove-*`, never raw CSS values
5. **Shell extension model** — features as extensions
6. **Conventional commits** — `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`
7. **Quality gates** — `pnpm check` + `pnpm test` before any PR
8. **Conventional comments** — use [conventionalcomments.org](https://conventionalcomments.org/) labels on PR reviews (`suggestion:`, `issue:`, `nitpick:`, etc.)
9. **Supervised AI** — gates and guardrails are product, not overhead

---

## Commands

```bash
# Frontend
pnpm install              # Install dependencies
pnpm dev                  # Start Vite dev server
pnpm build                # Production build
pnpm test                 # Run Vitest
pnpm check                # TypeScript + ESLint
pnpm e2e                  # Playwright tests

# Rust crates
cargo build               # Build all crates
cargo test                # Test all crates
cargo build -p grove-domain  # Build domain crate only
cargo test -p grove-domain   # Test domain crate only
cargo build -p grove-api     # Build API crate
cargo test -p grove-api      # Test API crate
cargo build -p grove-sync    # Build CRDT sync crate
cargo test -p grove-sync     # Test CRDT sync crate
cargo run -p grove-api       # Run API server

# API E2E (Hurl)
./scripts/e2e.sh             # Run all API e2e tests (starts/stops server)
./scripts/e2e.sh health.hurl # Run a single e2e test file

# Desktop
cargo tauri dev           # Run Tauri dev mode
cargo tauri build         # Build desktop app

# Database
docker compose up -d      # Start PostgreSQL
```

---

## Documentation Structure

```
docs/                              # Public technical documentation
  ├── prfaq.md                     # Product vision, press release, competitor FAQ
  ├── architecture-reference.md    # Tech stack, entity model, hex layers, ACP
  ├── architecture-flows.md        # Request flow, multi-tenant, ACP diagrams
  └── research/                    # Research and reference docs
      └── YYYY-MM-DD-<topic>-research.md

.docs/                             # Internal documentation (gitignored)
  └── superpowers/
      ├── specs/                   # Design specs from brainstorming skill
      │   └── YYYY-MM-DD-<topic>-design.md
      └── plans/                   # Implementation plans from writing-plans skill
          └── YYYY-MM-DD-<topic>.md
```

- **Research** docs → `docs/research/YYYY-MM-DD-<topic>-research.md`
- `.docs/` is **gitignored** — local working documents only
- **Brainstorming** produces specs → `.docs/superpowers/specs/`
- **Writing-plans** produces plans → `.docs/superpowers/plans/`

---

## What NOT to Build (v1)

- Third-party extension marketplace
- Mobile app
- Self-hosted API option
- Billing / subscription management (use Clerk billing or defer)
