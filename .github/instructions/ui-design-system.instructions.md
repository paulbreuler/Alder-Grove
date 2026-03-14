---
applyTo: "src/**/*.css,src/**/*.tsx,src/styles/**"
---

<!-- GENERATED FROM .claude/ — DO NOT EDIT BY HAND -->

# UI Design System

## Design Tokens

All visual values come from `--grove-*` CSS custom properties. Never use raw values.

```css
/* CORRECT */
color: var(--grove-text-primary);
background: var(--grove-surface-elevated);
border-radius: var(--grove-radius-md);
padding: var(--grove-space-4);

/* WRONG */
color: #1a1a2e;
background: white;
border-radius: 8px;
padding: 16px;
```

## Token Categories

- `--grove-text-*` — Text colors (primary, secondary, muted, inverse, accent)
- `--grove-surface-*` — Backgrounds (base, elevated, sunken, overlay)
- `--grove-border-*` — Border colors (default, subtle, strong)
- `--grove-accent-*` — Brand/action colors
- `--grove-space-*` — Spacing scale (1–12, based on 4px grid)
- `--grove-radius-*` — Border radius (sm, md, lg, xl, full)
- `--grove-shadow-*` — Elevation shadows (sm, md, lg)
- `--grove-font-*` — Font families, sizes, weights
- `--grove-transition-*` — Animation durations and easings

## Elevation

Use elevation tokens to create visual hierarchy:

1. **Base** (`--grove-surface-base`) — default background
2. **Elevated** (`--grove-surface-elevated` + `--grove-shadow-sm`) — cards, panels
3. **Overlay** (`--grove-surface-overlay` + `--grove-shadow-lg`) — dropdowns, popovers

## Typography

- Use `--grove-font-*` tokens for all text styling
- Headings, body, and caption sizes defined in token scale
- No inline font-size, font-weight, or line-height values

## Interaction Patterns

- **No modals for CRUD** — use inline editing, panels, or dedicated views
- **Tycoon paradigm** — direct manipulation, immediate feedback
- Focus states must be visible (accessibility)
- Loading states use skeleton placeholders, not spinners

## Motion

- CSS transitions use `--grove-transition-*` tokens for duration and easing
- JS-driven animations use Motion 12 (`motion/react`) + MotionPlus — see `motion.md` rule
- Prefer `transform` and `opacity` for animations (GPU-accelerated)
- Respect `prefers-reduced-motion` — `useReducedMotion()` or `<MotionConfig reducedMotion="user">`
