# Alder Grove

> Desktop OS for AI-assisted software development.
> *Your applications grow in the Grove.*

Grove is a desktop-native application for startups, solo developers, and small teams
who want a single, connected system of record for the entire software lifecycle —
from Personas and Journeys through Executable Specifications to Gated AI Agent
Sessions and Codebase Snapshots.

Built on Tauri v2 (desktop) + Rust Axum 0.8 (cloud API) + React 19.2 (frontend) +
PostgreSQL 18.

## Documentation

| Document | Description |
| --- | --- |
| [`docs/prfaq.md`](docs/prfaq.md) | Product vision, press release, competitor FAQ |
| [`docs/architecture-reference.md`](docs/architecture-reference.md) | Tech stack, entity model, hexagonal layers, ACP |
| [`docs/architecture-flows.md`](docs/architecture-flows.md) | Request flow, multi-tenant, and ACP diagrams |
| [`CLAUDE.md`](CLAUDE.md) | AI collaboration guidelines (Claude Code) |

## Quick Start

```bash
# Frontend
pnpm install
pnpm dev

# API
cargo build -p grove-api
cargo run -p grove-api

# Desktop
cargo tauri dev

# Database
docker compose up -d
```

## Architecture

**Hexagonal** — every feature follows domain → application → adapters → UI.
Dependencies flow inward. See
[`docs/architecture-reference.md`](docs/architecture-reference.md) for full details.

## License

Proprietary. All rights reserved.
