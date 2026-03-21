---
name: add-extension
description: Scaffold a new Shell extension with hexagonal layers and TDD
user_invocable: true
---

# /add-extension

Scaffold a new Alder Shell extension with hexagonal architecture layers and
test-driven development.

## Workflow

### Step 1: Gather Requirements

Ask the user (or extract from `$ARGUMENTS`) for:

| Field          | Required | Default | Description                                  |
| -------------- | -------- | ------- | -------------------------------------------- |
| name           | yes      | —       | Extension name (kebab-case, e.g. `settings`) |
| scope          | no       | v1      | Release scope: `v1` or `v2`                  |
| description    | yes      | —       | One-line description                         |
| contributions  | no       | —       | ActivityBar icon, panels, routes, commands    |

### Step 2: Create Feature Directory

Create the hexagonal layer structure:

```
src/features/<name>/
  ├── domain/types.ts          # Pure types — no React, no framework deps
  ├── domain/types.test.ts     # Domain type tests (written FIRST)
  ├── application/             # Hooks, Zustand stores, orchestration
  ├── adapters/                # API clients, Tauri invoke wrappers
  ├── ui/                      # React components
  └── extension.tsx            # Shell extension registration
```

Create empty directories for `application/`, `adapters/`, and `ui/` with
`.gitkeep` files so they are tracked.

### Step 3: Write Domain Types Test FIRST (TDD Red)

Create `src/features/<name>/domain/types.test.ts`:

```typescript
import { describe, it, expect } from 'vitest';
// Import types once they exist — test file is written first

describe('<Name> domain types', () => {
  it('should define <Name> type with required fields', () => {
    // Assert the type shape — fields, optionality, constraints
  });
});
```

The test must compile and FAIL (RED) before proceeding.

### Step 4: Create Domain Types (TDD Green)

Create `src/features/<name>/domain/types.ts`:

- Pure TypeScript types and interfaces only
- No React imports, no framework dependencies
- No imports from other features
- Export all public types

### Step 5: Verify GREEN

```bash
pnpm test
```

All tests must pass before proceeding.

### Step 6: Create Extension Registration

Create `src/features/<name>/extension.tsx`:

```tsx
import type { Extension } from '@paulbreuler/shell';

export const <camelName>Extension: Extension = {
  id: 'grove.<name>',
  activate(ctx) {
    // Register contributions: ActivityBar, panels, routes, commands
  },
  deactivate() {
    // Cleanup subscriptions, stores, side effects
  },
};
```

Follow the naming convention:
- Extension variable: `<camelCase>Extension` (e.g., `settingsExtension`)
- Extension ID: `grove.<kebab-case>` (e.g., `grove.settings`)

### Step 7: Wire into App.tsx

Add the extension to the bootstrap array in `src/App.tsx`:

1. Import the extension: `import { <camelName>Extension } from './features/<name>/extension';`
2. Add to the extensions array passed to Shell

### Step 8: Verify

```bash
pnpm check    # TypeScript + ESLint — must pass
pnpm test     # All tests — must pass
```

## Checklist

Before marking complete, verify:

- [ ] `src/features/<name>/domain/types.ts` exists with pure types
- [ ] `src/features/<name>/domain/types.test.ts` exists and passes
- [ ] `src/features/<name>/extension.tsx` exists with correct id and structure
- [ ] Extension is wired into `src/App.tsx` bootstrap array
- [ ] No imports from other features (no cross-feature coupling)
- [ ] All CSS uses `--grove-*` design tokens only
- [ ] `application/`, `adapters/`, `ui/` directories exist (with `.gitkeep`)
- [ ] `pnpm check` passes
- [ ] `pnpm test` passes

## Rules

- TDD is mandatory — write the test file BEFORE the implementation
- Domain layer has zero framework dependencies (no React, no Zustand, no Axios)
- No cross-feature imports — features are isolated modules
- All styling via `--grove-*` design tokens, never raw CSS values
- Extension ID must be `grove.<name>` to avoid collisions
- If the extension needs a Zustand store, it goes in `application/`, not `domain/`
- This skill establishes the convention for `extension.tsx` — no prior examples exist in the codebase
