---
name: check-backend-architecture
description: Verify Rust hexagonal architecture boundaries across domain and adapters
user_invocable: true
---

# /check-backend-architecture

Evaluate the Rust codebase for adherence to Hexagonal Architecture (Ports and
Adapters). Focus on separation of Domain Logic from Infrastructure and the use
of Traits as architectural boundaries.

## Checks

### 1. Dependency Analysis

#### The Inward Rule

- The domain or core crate must not import from infrastructure, persistence,
  transport, web, or adapter modules
- Domain modules must not depend on backend framework types

#### External Crates

- Domain logic must not depend on infrastructure-heavy crates such as `sqlx`,
  `diesel`, `reqwest`, or `axum`
- Toolbelt crates like `serde`, `uuid`, and `chrono` are acceptable when used
  as value-object support rather than infrastructure coupling

### 2. Port Definitions (Traits)

#### Trait Ownership

- Driven ports such as repositories and external services must be defined inside
  the domain layer

#### Abstraction Level

- Trait methods must not leak implementation details
- Repository methods should return domain-friendly types and errors, not
  transport/database/framework errors

### 3. Adapter Implementation

#### The Outward Rule

- Adapters in infrastructure or API crates should implement traits defined in
  the domain layer

#### Mapping

- Adapters must map database rows, transport DTOs, and external payloads into
  domain entities before returning them to the core

### 4. Dependency Injection & Wiring

#### Composition Root

- Concrete adapter wiring belongs in `main.rs`, `lib.rs`, or a dedicated
  composition module at the application boundary

#### Generics vs. Dynamics

- Review whether the code uses generics or trait objects appropriately
- If the runtime is async or multi-threaded, injected ports must handle
  `Send + Sync` requirements correctly

### 5. Scoring Rubric

Provide a score from 1-5 for:

- **Isolation** — how well business logic is shielded from the outside world
- **Symmetry** — whether inbound and outbound edges follow port/adapter rigor
- **Testability** — whether domain logic can be tested with mock adapters and
  no environment/database requirements

## Output

Report as a backend-only checklist plus scorecard:

```text
✅ Dependency analysis — PASS
✅ Port definitions — PASS
❌ Adapter implementation — FAIL
   crates/grove-api/src/db/persona_repo.rs:14 — returns sqlx::Error from port method
⚠️ Dependency injection & wiring — N/A
   crates/grove-api/src/main.rs — composition root not implemented yet

Scores
  Isolation: 4/5
  Symmetry: 3/5
  Testability: 5/5

RESULT: FAIL (1 violation, 1 N/A)
```

## Rules

- Review only Rust backend architecture in `crates/`
- Separate implemented violations from not-yet-implemented areas
- Cite file paths and exact evidence for failures
