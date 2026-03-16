---
applyTo: "**/*.test.*,**/*.spec.*,src/**/__tests__/**,crates/**/tests/**,tests/e2e/**/*.hurl"
---

<!-- GENERATED FROM .claude/ — DO NOT EDIT BY HAND -->

# Testing Rules

## TDD Mandatory

RED → GREEN → REFACTOR. No exceptions.

1. **RED**: Write a failing test that describes the desired behavior
2. **GREEN**: Write the minimum code to make it pass
3. **REFACTOR**: Clean up while keeping tests green

If you write implementation before tests, delete the implementation and start over.

## Frontend Testing (Vitest)

### Unit Tests

- Test domain logic (pure functions, business rules) — no mocking needed
- Test application hooks with `renderHook` from `@testing-library/react`
- Test Zustand stores by calling actions and asserting state

### Component Tests

- Use `@testing-library/react` — test behavior, not implementation
- Query by role, label, or text — never by class name or test ID unless necessary
- Use `userEvent` over `fireEvent`
- Wrap async operations in `waitFor`

### What to Test

- Domain: all business rules, type guards, transformations
- Application: hook behavior, store state transitions
- UI: user interactions, conditional rendering, error states
- Adapters: API client response mapping (mock fetch)

### What NOT to Test

- Implementation details (internal state, private methods)
- Third-party library behavior
- Styling / CSS classes

## API Testing (Rust)

- Unit tests for domain logic (pure functions, business rules)
- Integration tests for routes using `tower::ServiceExt::oneshot` (in-process, no TCP)
- Database tests against a real PostgreSQL instance (no mocks)
- Test multi-tenant isolation: verify workspace A cannot access workspace B data
- Test RLS enforcement: `TenantTx` scoped queries should filter by workspace_id
- Test cross-workspace rejection: inserts with wrong workspace_id should fail

### Integration Test Patterns

```rust
// Test helpers in tests/common/mod.rs
// #![allow(dead_code)] required — each test binary compiles its own copy
// of the common module, so helpers used by some tests but not all
// trigger false dead_code warnings.

let state = common::test_state().await;  // Pool + migrations
let app = grove_api::create_app(state);
let response = app.oneshot(request).await.unwrap();
```

- Use `unique_org_id()` for test isolation — avoids collisions across parallel tests
- Always `cleanup_org()` at end of tests to remove test data
- Test RLS with `TenantTx::begin(&pool, workspace_id)` + raw SQL queries

## API E2E Testing (Hurl)

- `.hurl` files in `tests/e2e/` — declarative HTTP request/response tests
- Run with `./scripts/e2e.sh` (builds API, starts server, runs tests, cleans up)
- Run single file: `./scripts/e2e.sh health.hurl`
- Tests run against a real running server over HTTP (not in-process)
- Use `[Captures]` to chain responses (e.g., capture `ws_id` from create, use in get)
- Use `[Asserts]` for JSONPath assertions, status codes, headers

### When to Use Hurl vs Integration Tests

| Use Case                           | Tool                                            |
| ---------------------------------- | ----------------------------------------------- |
| Domain logic, port implementations | Rust unit/integration tests                     |
| Route behavior, error mapping      | `tower::ServiceExt::oneshot` integration tests  |
| Full HTTP lifecycle (real TCP)     | Hurl e2e tests                                  |
| Multi-tenant RLS enforcement       | Rust integration tests (need `TenantTx` access) |
| API contract validation            | Hurl e2e tests                                  |

## UI E2E Testing (Playwright)

- Test critical user flows end-to-end
- Test in the Tauri desktop context when possible
- Keep E2E tests focused — prefer unit/integration tests for logic

## Naming Convention

```
// Vitest
describe('PersonaStore', () => {
  it('adds a persona to the workspace', () => { ... })
  it('rejects persona with missing name', () => { ... })
})

// Rust
#[test]
fn create_persona_with_valid_name() { ... }

#[test]
fn reject_persona_without_workspace_id() { ... }
```
