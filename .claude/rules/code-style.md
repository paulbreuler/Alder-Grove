---
paths:
  - "**/*.ts"
  - "**/*.tsx"
  - "**/*.rs"
---

# Code Style

## TypeScript

### Naming
- `PascalCase`: types, interfaces, components, enums
- `camelCase`: variables, functions, parameters, hooks (`usePersona`)
- `UPPER_SNAKE_CASE`: constants
- Prefix interfaces with purpose, not `I` (e.g., `PersonaRepository` not `IPersonaRepository`)
- Boolean variables/props: `is*`, `has*`, `should*`, `can*`

### Language
- Strict TypeScript — no `any` without an explanatory comment
- Prefer `type` over `interface` for data shapes (use `interface` for contracts/ports)
- Use `unknown` over `any` for untyped data
- Prefer `const` over `let`, never `var`
- Use optional chaining (`?.`) and nullish coalescing (`??`)
- Prefer early returns over deep nesting

### Modules
- Named exports only — no default exports
- One component/hook/type per file (colocated helpers are fine)
- Barrel exports (`index.ts`) only at feature boundaries

### React
- Functional components only
- Props types colocated with component: `type PersonaCardProps = { ... }`
- Destructure props in function signature
- No inline styles — use `--grove-*` tokens via CSS

## Rust

**Toolchain**: Rust 1.94+ stable, **Edition 2024**, pinned via `rust-toolchain.toml`.

### Naming
- `PascalCase`: types, structs, enums, traits
- `snake_case`: functions, variables, modules, files
- `UPPER_SNAKE_CASE`: constants
- Crate names: `grove-api`, `grove-tauri` (kebab-case)

### Edition 2024 Features (use these)
- **Let chains** in `if` and `while` — prefer over nested `if let`:
  ```rust
  if let Some(session) = sessions.get(id) && session.is_active() { ... }
  ```
- **Async closures** — use `async || {}` with `AsyncFn` / `AsyncFnMut` traits
  instead of workarounds with `Box::pin` or manual futures
- **`unsafe` blocks in `unsafe fn`** — always use an explicit `unsafe {}` block
  inside unsafe functions (edition 2024 warns without it)
- **`unsafe extern`** — declare all extern blocks with `unsafe extern`
- Reserve `gen` keyword — do not use `gen` as an identifier

### Language
- Use `Result<T, E>` and `?` for error propagation — no `.unwrap()` in production code
- `.unwrap()` and `.expect("reason")` are fine in tests
- Use `thiserror` for domain/library error types — derive `Error` on enums
- Use `anyhow` only at the application boundary (main, CLI) — not in libraries
- Implement `From<SourceError>` for error type conversion — keep `?` chains clean
- Prefer `impl Trait` in function args/returns for static dispatch
- Use `dyn Trait` (trait objects) for hexagonal port abstractions stored in `Arc<dyn T>`
- Use `#[derive]` liberally: `Debug`, `Clone`, `Serialize`, `Deserialize`
- Document public items with `///` doc comments
- Respect all `clippy` lints — `#[allow(clippy::...)]` requires a comment explaining why

### Error Handling
- Define a crate-level `AppError` enum with `#[derive(thiserror::Error)]`
- Implement `IntoResponse` for `AppError` to produce RFC 9457 Problem Details
- Use `tracing::error!` for logging errors — structured fields, not string formatting
- Never `.unwrap()` on user input, database results, or network responses
- Use `?` with `From` impls — not `.map_err()` chains when a blanket conversion exists

### Async
- Use `async`/`.await` — all Axum 0.8 handlers are async
- Use `tokio::select!` for cooperative cancellation and timeouts
- Use `tokio_util::sync::CancellationToken` for graceful shutdown propagation
- No blocking operations on the Tokio runtime — use `tokio::task::spawn_blocking`
- Prefer `tokio::time::timeout` over manual timer logic
- Use async closures (`async || {}`) for callbacks that need `.await`

### Tracing
- Use `tracing` crate — not `println!` or `log`
- Structured fields: `tracing::info!(workspace_id = %id, "created persona")`
- Instrument async functions with `#[tracing::instrument(skip(pool))]`
- Use `tracing-subscriber` with `EnvFilter` for runtime log level control

### Key Crate Versions (pin in Cargo.toml)
- `axum` 0.8 — path syntax: `/{param}` not `/:param`
- `sqlx` 0.8 — compile-time checked queries, `PgPool`
- `tokio` 1.x — async runtime
- `serde` / `serde_json` — serialization
- `thiserror` 1.x — error derive macros
- `tracing` / `tracing-subscriber` — structured logging
- `utoipa` — OpenAPI spec generation
- `tower` / `tower-http` — middleware (CORS, compression, tracing)
- `jsonwebtoken` — Clerk JWT verification

## Formatting & Linting

- TypeScript: Prettier (or Biome) — enforced by CI
- Rust: `rustfmt` (edition 2024 style) — enforced by CI
- Rust: `clippy` with `--deny warnings` — enforced by CI
- Markdown: `markdownlint-cli2` — enforced by pre-commit hook
