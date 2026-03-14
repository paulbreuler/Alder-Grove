---
paths:
  - "src/**/*.ts"
  - "src/**/*.tsx"
---

# Frontend Rules (React 19.2 + TypeScript)

**React 19.2** — leverage the full modern API. No legacy patterns.

## Hexagonal Layers

Every feature under `src/features/` follows this structure:

```
feature/
  ├─ domain/        # Pure types, entities, business rules
  ├─ application/   # Hooks, use cases, orchestration
  ├─ adapters/      # API clients, Tauri commands
  └─ ui/            # React components
```

### Domain Layer
- Pure TypeScript — no React, no Zustand, no fetch, no framework imports
- Export types, interfaces, constants, pure functions only
- This is the source of truth for the feature's data shapes

### Application Layer
- Custom hooks, Zustand stores, use cases
- May import from domain (types) and call adapters through ports
- Never import React components

### Adapters Layer
- API client functions (fetch wrappers)
- Tauri invoke wrappers
- WebSocket connections
- Implement ports defined in application layer

### UI Layer
- React components only
- Import from application layer (hooks, stores)
- Never import from domain or adapters directly

## React 19.2 Patterns

### New APIs (use these — they replace older patterns)

- **`use()`** — read promises and context directly in render. Replaces
  `useEffect` + `useState` for data fetching when combined with Suspense:
  ```tsx
  const data = use(fetchPersonas(workspaceId)); // suspends until resolved
  ```
- **`useActionState`** — manages form action state (result + pending). Replaces
  manual `useState` + `useTransition` for form submissions:
  ```tsx
  const [state, submitAction, isPending] = useActionState(createPersona, null);
  ```
- **`useOptimistic`** — show expected outcome immediately during async mutations:
  ```tsx
  const [optimisticPersonas, addOptimistic] = useOptimistic(personas);
  ```
- **`useFormStatus`** — read parent `<form>` pending state from child components
- **`useEffectEvent`** — stable callbacks that always read fresh props/state
  without being in the `useEffect` dependency array. Use for event handlers
  referenced inside effects.

### Ref as Prop
- Pass `ref` directly as a prop — **do not use `forwardRef`** (deprecated pattern):
  ```tsx
  // Correct — React 19
  function Input({ ref, ...props }: { ref?: React.Ref<HTMLInputElement> }) {
    return <input ref={ref} {...props} />;
  }
  ```

### React Compiler
- The React Compiler handles memoization automatically
- **Do not manually use `useMemo`, `useCallback`, or `memo`** unless profiling
  shows a specific need — the compiler does this better
- Enable compiler lint rules via `eslint-plugin-react-hooks`

### Suspense & Transitions
- Wrap async data loading in `<Suspense>` boundaries with fallback UI
- Use `useTransition` for non-urgent state updates (keeps UI responsive)
- Use Actions (async functions in transitions) for form handling — they
  automatically manage pending states, errors, and optimistic updates

### Patterns to Avoid
- No `useEffect` for data fetching — use `use()` + Suspense or Zustand
- No `forwardRef` — pass `ref` as a prop
- No manual `useMemo`/`useCallback` — trust the React Compiler
- No `useEffect` + `setState` for derived state — compute during render

## State Management

- Zustand for feature-level state (one store per feature in application/)
- React state for component-local UI state
- No prop drilling past 2 levels — use Zustand or context

## Shell Extensions

- Each feature exports an `extension.tsx` that registers with `bootstrapShell()`
- Extensions are self-contained — no cross-feature imports at the UI layer
- Shared types belong in a `shared/` domain if needed
