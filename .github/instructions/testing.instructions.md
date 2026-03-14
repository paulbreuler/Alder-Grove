---
applyTo: "**/*.test.*,**/*.spec.*,src/**/__tests__/**,src-api/**/tests/**"
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
- Integration tests for routes using test helpers
- Database tests against a real PostgreSQL instance (no mocks)
- Test multi-tenant isolation: verify workspace A cannot access workspace B data

## E2E Testing (Playwright)

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
