---
name: add-feature
description: Add a sub-feature within an existing Shell extension
user_invocable: true
---

# /add-feature

Add a sub-feature within an existing Shell extension, placing code in the
correct hexagonal layer with test-driven development.

## Workflow

### Step 1: Gather Requirements

Ask the user (or extract from `$ARGUMENTS`) for:

| Field             | Required | Description                                           |
| ----------------- | -------- | ----------------------------------------------------- |
| parent extension  | yes      | Existing extension under `src/features/<name>/`       |
| feature name      | yes      | Name for the sub-feature (e.g., `createPersonaForm`)  |
| layers needed     | yes      | Which layers: domain, application, adapters, ui       |

Verify the parent extension exists by checking `src/features/<parent>/`.

### Step 2: Determine Layer Placement

Route the feature to the correct hexagonal layer:

| What you are building                  | Layer           | Path                                          |
| -------------------------------------- | --------------- | --------------------------------------------- |
| Pure types, interfaces, business rules | `domain/`       | `src/features/<parent>/domain/`               |
| Hooks, Zustand stores, orchestration   | `application/`  | `src/features/<parent>/application/`           |
| API clients, Tauri invoke wrappers     | `adapters/`     | `src/features/<parent>/adapters/`              |
| React components, forms, views         | `ui/`           | `src/features/<parent>/ui/`                    |

**Dependency rules** (inward only):
- `ui/` may import from `application/` only (never `domain/` or `adapters/` directly)
- `application/` may import from `domain/`
- `adapters/` may import from `domain/`
- `domain/` may not import from any other layer
- No layer may import from another feature's internals

### Step 3: Write Test FIRST (TDD Red)

Create the test file in the appropriate layer:

| Layer         | Test file pattern                                          |
| ------------- | ---------------------------------------------------------- |
| domain        | `src/features/<parent>/domain/<feature>.test.ts`           |
| application   | `src/features/<parent>/application/<feature>.test.ts`      |
| adapters      | `src/features/<parent>/adapters/<feature>.test.ts`         |
| ui            | `src/features/<parent>/ui/<Feature>.test.tsx`              |

The test must:
- Import from the correct layer
- Assert on observable behavior, not internal state
- Include at least one positive and one negative test case
- FAIL (RED) before the implementation exists

### Step 4: Implement (TDD Green)

Create the implementation file in the correct layer:

| Layer         | File pattern                                               |
| ------------- | ---------------------------------------------------------- |
| domain        | `src/features/<parent>/domain/<feature>.ts`                |
| application   | `src/features/<parent>/application/<feature>.ts`           |
| adapters      | `src/features/<parent>/adapters/<feature>.ts`              |
| ui            | `src/features/<parent>/ui/<Feature>.tsx`                   |

Implementation rules:
- Domain: pure TypeScript, no framework imports
- Application: Zustand stores use per-feature convention, hooks use `use` prefix
- Adapters: API clients return domain types, handle errors at boundary
- UI: React components, `--grove-*` tokens only, no raw CSS values

### Step 5: Wire into Parent Extension

If the feature needs to be registered with the Shell (new route, command, panel):

1. Update `src/features/<parent>/extension.tsx` to register the contribution
2. Export from the appropriate layer's index if needed

### Step 6: Verify

```bash
pnpm check    # TypeScript + ESLint — must pass
pnpm test     # All tests — must pass
```

## Checklist

Before marking complete, verify:

- [ ] Feature is in the correct hexagonal layer
- [ ] Test file exists and was written BEFORE implementation
- [ ] All tests pass (`pnpm test`)
- [ ] No imports from other features (no cross-feature coupling)
- [ ] Dependencies flow inward only (domain has no outer-layer imports)
- [ ] All CSS uses `--grove-*` design tokens only
- [ ] `pnpm check` passes
- [ ] Parent extension updated if new contribution was added

## Rules

- TDD is mandatory — RED then GREEN then REFACTOR
- Correct layer placement is non-negotiable — do not put React components in domain
- No cross-feature imports — if shared logic is needed, it goes in `src/features/shared/`
- Zustand stores are per-feature, in `application/` — never in `domain/`
- Adapters return domain types — UI should not know about API response shapes
- All styling via `--grove-*` design tokens, never raw CSS values
