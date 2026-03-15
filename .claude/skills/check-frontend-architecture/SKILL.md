---
name: check-frontend-architecture
description: Verify React and TypeScript architecture boundaries in the frontend
user_invocable: true
---

# /check-frontend-architecture

Verify that the frontend follows the Alder Grove React architecture.

## Checks

### 1. Feature Dependency Direction

For each feature under `src/features/`:

- **Domain** must NOT import from `application/`, `adapters/`, or `ui/`
- **Domain** must NOT import React, Zustand, fetch, or framework/runtime code
- **Application** must NOT import from `ui/`
- **Application** may import from `domain/` and call adapters through ports
- **UI** must NOT import from `domain/` or `adapters/` directly
- **UI** imports from `application/` only

**How to check**: inspect imports and flag reverse dependencies.

### 2. Namespace Isolation

- No cross-feature imports at the UI layer
- Shared types belong in `src/shared/domain/` or another shared boundary
- Features should not reach into another feature's internals

### 3. Adapter Discipline

- API clients and other side-effecting code live in adapters
- UI components do not call fetchers or external clients directly
- Stores/hooks orchestrate use cases rather than embed transport details

### 4. Design Token Compliance

- No raw CSS values in `.tsx` or `.css` under `src/`
- Colors, spacing, radii, shadows, and typography use `--grove-*` tokens

### 5. Testability

- Application and domain logic should be testable without rendering the full UI
- Adapter boundaries should be mockable in unit tests
- Flag hard-coupling that makes feature logic require browser/runtime setup

## Output

Report as a frontend-only checklist:

```text
✅ Feature dependency direction — PASS
✅ Namespace isolation — PASS
❌ Adapter discipline — FAIL
   src/features/personas/ui/PersonaForm.tsx:42 — component calls fetch directly
✅ Design token compliance — PASS
✅ Testability — PASS

RESULT: FAIL (1 violation)
```

## Rules

- Review only frontend architecture in `src/`
- Do not apply Rust-specific criteria here
- Cite file paths and exact evidence for failures
