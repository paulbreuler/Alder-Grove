# Alder Grove

> Desktop OS for AI-assisted software development.
> *Your applications grow in the Grove.*

## Quick Start

```bash
# Prerequisites: Rust 1.93+, Node 22+, pnpm 10+, Docker

# Start database
docker compose up -d

# Build Rust crates
cargo build

# Install frontend dependencies
pnpm install

# Run tests
cargo test
pnpm test
```

## Architecture

Tauri v2 desktop client + cloud-hosted Axum API, backed by PostgreSQL 18.

### Rust Crates (`crates/`)

| Crate | Purpose |
|-------|---------|
| `grove-domain` | Pure types, port traits, business rules (zero framework deps) |
| `grove-sync` | CRDT sync layer — Yrs (Rust port of Yjs) |
| `grove-api` | Axum 0.8 cloud API server |
| `grove-tauri` | Tauri v2 desktop app (IPC commands, API proxy) |
| `grove-ts-gen` | Build-time TypeScript type generation (ts-rs) |

### Frontend (`src/`)

React + TypeScript in the Tauri webview (dependencies added as features are built). Hexagonal architecture per feature:

```
src/features/<feature>/
  domain/       Pure types, business rules
  application/  Hooks, stores, use cases
  adapters/     API clients, Tauri invoke wrappers
  ui/           React components
```

### Database

PostgreSQL 18. Schema design targets 19 tables (11 content + 7 ACP + 1 CRDT sync) — migrations are added incrementally.

See `docs/architecture-reference.md` for the full technical reference.

## Documentation

- `docs/prfaq.md` — Product vision
- `docs/architecture-reference.md` — Tech stack, entity model, schema
- `docs/architecture-flows.md` — Sequence diagrams, state machines (Mermaid)

## AI Assistant Configs

Assistant compatibility files for Codex, Gemini, and GitHub Copilot are
generated from `CLAUDE.md` and `.claude/`.

```bash
pnpm ai:generate   # Regenerate assistant config files
pnpm ai:check      # Fail if generated files are stale
pnpm prepare       # Install Husky hooks if needed
```

Do not hand-edit `AGENTS.md`, `.agents/skills/*`, `GEMINI.md`, `.gemini/*`, or
`.github/copilot-instructions.md`.
