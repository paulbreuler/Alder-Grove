---
paths:
  - "src/**/*.tsx"
  - "src/**/*.ts"
---

# Motion / MotionPlus — Animation Standards

## What We Use

| Package           | Version    | Purpose                                                  |
| ----------------- | ---------- | -------------------------------------------------------- |
| `motion`          | `^12.26.2` | Core animation library — import from `motion/react`      |
| `motion-plus`     | latest     | MotionPlus premium features — **required, not optional** |
| Motion Studio MCP | via `.mcp.json` | AI tooling: codex search, spring generator, visualizer |

**Import rule:** Always `from 'motion/react'` or `from 'motion-plus'`. **Never** `from 'framer-motion'`.

```tsx
// Correct
import { motion, AnimatePresence, useSpring, useTransform } from "motion/react";
import {} from /* premium APIs */ "motion-plus";

// Wrong — do not use
import { motion } from "framer-motion"; // ❌
```

## MCP Tools (Use These First — Mandatory)

The Motion Studio MCP server is configured in `.mcp.json`. These tools are available in every Claude Code session:

| Tool                                      | When to use                                                                      |
| ----------------------------------------- | -------------------------------------------------------------------------------- |
| `mcp__motion__search-motion-codex`        | **REQUIRED before implementing any animation** — search for the official pattern |
| `mcp__motion__generate-css-spring`        | Design a spring transition; returns production-ready CSS                         |
| `mcp__motion__generate-css-bounce-easing` | Create a bounce easing curve                                                     |
| `mcp__motion__visualise-spring`           | Preview spring behavior (stiffness/damping/mass) before committing               |
| `mcp__motion__visualise-cubic-bezier`     | Preview a cubic-bezier easing before committing                                  |

**Workflow:** Before writing any animation code, call `search-motion-codex` with the UI pattern (e.g. `"accordion"`, `"sidebar collapse"`, `"drag reorder"`, `"shared layout"`). Build from the returned examples — don't guess the API shape.

## Core API — Quick Reference

### `motion` components

```tsx
<motion.div
  initial={{ opacity: 0, y: 8 }}
  animate={{ opacity: 1, y: 0 }}
  exit={{ opacity: 0 }}
  transition={{ type: "spring", stiffness: 300, damping: 30 }}
/>
```

### `AnimatePresence` — enter/exit animations

```tsx
<AnimatePresence mode="wait">
  {isOpen && <motion.div key="panel" exit={{ opacity: 0, y: 4 }} />}
</AnimatePresence>
```

### `layout` — animate layout changes (FLIP)

```tsx
// No manual measurements — Motion handles the FLIP
<motion.div layout />
<motion.div layoutId="shared-element" /> // shared transition between views
<LayoutGroup> {/* coordinate layout animations across siblings */} </LayoutGroup>
```

### `useSpring` — smooth motion values

```tsx
const width = useSpring(rawWidth, { stiffness: 300, damping: 30, mass: 0.8 });
```

### `useTransform` — derive from motion values

```tsx
const opacity = useTransform(scrollYProgress, [0, 1], [1, 0]);
```

### `useScroll` — scroll-linked animations

```tsx
const { scrollY, scrollYProgress } = useScroll({ container: ref });
```

### `MotionConfig` — global defaults

```tsx
// Set reduced-motion globally at app root
<MotionConfig reducedMotion="user">
  <App />
</MotionConfig>
```

## Spring Presets (Project-Standard)

Use `mcp__motion__visualise-spring` to validate before adding new presets.

| Name            | Applied to                                   | Config                                        |
| --------------- | -------------------------------------------- | --------------------------------------------- |
| `sidebarSpring` | Sidebar width, collapse/expand               | `{ stiffness: 300, damping: 30, mass: 0.8 }`  |
| Panel fade      | Canvas/inspector panel mount                 | `{ duration: 0.15 }`                          |
| Reduced motion  | All of the above when `prefersReducedMotion` | `{ duration: 0 }`                             |

**Always respect `useReducedMotion()`:**

```tsx
const prefersReducedMotion = useReducedMotion() === true;
const transition = prefersReducedMotion ? { duration: 0 } : sidebarSpring;
```

## MotionPlus Setup

`motion-plus` requires a MotionPlus license token. To install:

1. **Get the token** — retrieve from shared secrets store or [plus.motion.dev](https://plus.motion.dev)
2. **Install:**
   ```bash
   pnpm add "https://api.motion.dev/registry?package=motion-plus&version=latest&token=YOUR_TOKEN"
   ```
3. **Add to `.env.local`:**
   ```
   MOTION_TOKEN=your_token_here
   ```
4. **`.mcp.json` is already configured** — the `"motion"` server entry handles the MCP side.

## Rules

- **Search Codex first** — `mcp__motion__search-motion-codex` before any new animation
- **Visualise springs** — `mcp__motion__visualise-spring` before hardcoding stiffness/damping
- **motion-plus is required** — if a pattern uses `motion-plus` imports, do not work around it with the OSS package
- **Never import from `framer-motion`** — use `motion/react` only
- **Always handle reduced motion** — `useReducedMotion()` or `<MotionConfig reducedMotion="user">`
- **Don't use `useEffect` + `setState` for animations** — use motion values and `animate()`
- **Don't apply `layout` globally** — only on elements that genuinely change size/position on interaction
- **Use `--grove-transition-*` tokens** for duration/easing in CSS transitions; use Motion for JS-driven animation
