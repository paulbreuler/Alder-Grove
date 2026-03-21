---
name: add-component
description: Scaffold a React component with Vitest tests (TDD-first)
user_invocable: true
---

# /add-component

Scaffold a new React component inside a feature's `ui/` layer, test-first.

## Pre-flight: Discovery Before Creation (NON-NEGOTIABLE)

Before creating anything, search for an existing solution:

1. **Existing components** -- scan `src/features/*/ui/` for a component that
   already covers the need. If one exists, extend or compose -- do not duplicate.
2. **Base UI primitives** -- check `@base-ui/react` for built-in primitives:
   Dialog, Menu, Tooltip, Popover, Switch, Tabs, Select, Checkbox, Slider,
   Accordion, Collapsible, Progress, Toggle, AlertDialog, NumberField, etc.
   Wrap and style these rather than building from scratch.
3. **Data tables** -- check `@tanstack/react-table` v8 for headless table/grid
   needs. Compose with Grove styling rather than hand-rolling table logic.
4. **Motion Codex** -- before writing any animation code, search the Motion
   Codex MCP server for the canonical pattern. Never guess at Motion 12 /
   MotionPlus APIs.

If an existing primitive or component covers the need, STOP and recommend
composition instead of creation. Only proceed if nothing fits.

**Prerequisites:** This skill references packages that may need to be installed:
- `class-variance-authority` — for style variants (install: `pnpm add class-variance-authority`)
- `@base-ui/react` — for UI primitives (install: `pnpm add @base-ui/react`)
- `motion` + `motion-plus` — for animations (see `.claude/rules/motion.md` for setup)

Skip checks for packages not yet installed.

## Step 1: Gather Requirements

Collect these before writing any code:

| Field              | Description                                             |
| ------------------ | ------------------------------------------------------- |
| Component name     | PascalCase (e.g., `PersonaCard`, `JourneyTimeline`)     |
| Parent feature     | Feature directory name (e.g., `personas`, `journeys`)   |
| Purpose            | One sentence describing what it renders and why          |
| Props interface    | TypeScript props with JSDoc comments                     |
| Variants           | CVA variants (size, intent, state) if applicable         |
| Interactive behavior | Click, hover, keyboard, drag-and-drop                 |
| Data source        | Which application-layer hook or store provides data      |
| Accessibility      | ARIA roles, keyboard navigation, focus management        |

## Step 2: Write Test FIRST

Create `src/features/<feature>/ui/<Name>.test.tsx`.

```tsx
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { ComponentName } from "./<Name>";

describe("<Name>", () => {
  it("renders with required props", () => {
    render(<ComponentName label="Test" />);
    expect(screen.getByTestId("<name>")).toBeInTheDocument();
  });

  it("handles user interaction", async () => {
    const user = userEvent.setup();
    const onAction = vi.fn();
    render(<ComponentName label="Test" onAction={onAction} />);

    await user.click(screen.getByTestId("<name>-action"));
    expect(onAction).toHaveBeenCalledOnce();
  });

  it("renders variant correctly", () => {
    render(<ComponentName label="Test" variant="primary" />);
    const el = screen.getByTestId("<name>");
    expect(el.className).toContain("primary");
  });

  it("handles empty/loading state", () => {
    render(<ComponentName label="Test" loading />);
    expect(screen.getByTestId("<name>-loading")).toBeInTheDocument();
  });
});
```

### Test conventions

- Use `data-testid` for selectors (Testing Library default)
- Test behavior, not implementation details
- Test all variants and edge cases (empty, loading, error, overflow)
- Use `userEvent` over `fireEvent` for realistic interactions
- Use `waitFor` for async state updates
- Mock application-layer hooks, never adapters directly

## Step 3: Verify RED

```bash
pnpm test -- <Name>
```

All tests must FAIL. If any pass, the test is not testing new behavior -- fix it.

## Step 4: Create Component

Create `src/features/<feature>/ui/<Name>.tsx`.

### Component template

```tsx
import { cva, type VariantProps } from "class-variance-authority";
import { type ComponentProps } from "react";

/* ------------------------------------------------------------------ */
/* Styles (CVA)                                                       */
/* ------------------------------------------------------------------ */

const nameStyles = cva(
  [
    // Base styles -- design tokens only
    "grove-component-base",
  ],
  {
    variants: {
      variant: {
        default: "grove-component-default",
        primary: "grove-component-primary",
      },
      size: {
        sm: "grove-component-sm",
        md: "grove-component-md",
        lg: "grove-component-lg",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "md",
    },
  },
);

/* ------------------------------------------------------------------ */
/* Types                                                              */
/* ------------------------------------------------------------------ */

interface NameProps
  extends VariantProps<typeof nameStyles>,
    Omit<ComponentProps<"div">, "className"> {
  /** Visible label text */
  label: string;
  /** Optional loading state */
  loading?: boolean;
  /** Callback on primary action */
  onAction?: () => void;
}

/* ------------------------------------------------------------------ */
/* Component                                                          */
/* ------------------------------------------------------------------ */

export function Name({
  label,
  variant,
  size,
  loading = false,
  onAction,
  ref,
  ...rest
}: NameProps) {
  if (loading) {
    return (
      <div data-testid="<name>-loading" aria-busy="true">
        Loading...
      </div>
    );
  }

  return (
    <div
      ref={ref}
      data-testid="<name>"
      className={nameStyles({ variant, size })}
      {...rest}
    >
      <span>{label}</span>
      {onAction && (
        <button
          type="button"
          data-testid="<name>-action"
          onClick={onAction}
        >
          Action
        </button>
      )}
    </div>
  );
}
```

### Component rules

| Rule                      | Detail                                                       |
| ------------------------- | ------------------------------------------------------------ |
| Design tokens only        | `--grove-*` custom properties. Never raw hex, px, or colors  |
| CVA for variants          | All visual variants through `class-variance-authority`       |
| `data-testid`             | Root element and every interactive child                     |
| `ref` as prop             | Pass `ref` as a regular prop -- never use `forwardRef`       |
| No manual memoization     | React Compiler handles it -- no `useMemo`, `useCallback`, `React.memo` |
| `useReducedMotion()` gate | Wrap every animation with reduced-motion check               |
| Base UI primitives        | Use `@base-ui/react` for dialogs, menus, tooltips, etc.     |
| Hexagonal imports         | Import from application layer only, never from domain or adapters |
| No cross-feature imports  | Components may not import from other features                |

### Animation pattern

```tsx
import { motion, useReducedMotion } from "motion/react";

function AnimatedComponent({ children }: { children: React.ReactNode }) {
  const prefersReduced = useReducedMotion();

  return (
    <motion.div
      initial={prefersReduced ? false : { opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      exit={prefersReduced ? undefined : { opacity: 0, y: -8 }}
      transition={{ duration: 0.2, ease: "easeOut" }}
    >
      {children}
    </motion.div>
  );
}
```

### CSS with design tokens

```css
.grove-component-base {
  padding: var(--grove-space-3);
  border-radius: var(--grove-radius-md);
  font-family: var(--grove-font-sans);
  font-size: var(--grove-font-size-sm);
  color: var(--grove-text-primary);
  background: var(--grove-surface-base);
  border: 1px solid var(--grove-border-default);
  box-shadow: var(--grove-shadow-sm);
  transition: box-shadow 0.15s ease;
}

.grove-component-base:focus-visible {
  outline: 2px solid var(--grove-accent);
  outline-offset: 2px;
}
```

## Step 5: Verify GREEN

```bash
pnpm test
```

All tests must pass. If any fail, fix the component -- not the tests.

## Step 6: Export if Reusable

If the component will be consumed outside its feature (rare -- most are
feature-internal), add a barrel export:

```ts
// src/features/<feature>/index.ts
export { Name } from "./ui/<Name>";
```

## Reference Files

| File                                    | Purpose                                |
| --------------------------------------- | -------------------------------------- |
| `src/features/<feature>/ui/`            | Component location                     |
| `src/features/<feature>/application/`   | Hooks and stores the component consumes |
| `src/features/<feature>/domain/`        | Types the hooks depend on              |
| `src/features/shared/`                  | Shared utilities and types             |
| `package.json`                          | Check available dependencies           |
| `src/app.css`                           | Global `--grove-*` token definitions   |

## Checklist

Before declaring done:

- [ ] Pre-flight discovery completed -- no existing component covers this need
- [ ] Test file written FIRST at `src/features/<feature>/ui/<Name>.test.tsx`
- [ ] RED confirmed -- all tests fail before implementation
- [ ] Component created at `src/features/<feature>/ui/<Name>.tsx`
- [ ] GREEN confirmed -- `pnpm test` passes
- [ ] `data-testid` on root and interactive elements
- [ ] CVA used for all style variants
- [ ] Only `--grove-*` design tokens in CSS (no raw values)
- [ ] `ref` passed as prop (no `forwardRef`)
- [ ] No manual `useMemo` / `useCallback` / `React.memo`
- [ ] `useReducedMotion()` gates all animations
- [ ] `@base-ui/react` used for interactive primitives
- [ ] Imports from application layer only (hexagonal compliance)
- [ ] No cross-feature imports
- [ ] Barrel export added if reusable
- [ ] `pnpm check` passes (TypeScript + ESLint)
