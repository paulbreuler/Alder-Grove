---
name: audit-tokens
description: Validate design token coverage and consistency (--grove-* CSS variables)
user_invocable: true
---

# /audit-tokens

Validate that design tokens (`--grove-*` CSS custom properties) are used
consistently and that components avoid hardcoded styling values.

## Workflow

### 1. Inventory declared tokens

Read `src/app.css` and collect every `--grove-*` custom property declared in
`:root`. Record the full list as the **declared set**.

### 2. Check: Orphaned tokens (MEDIUM)

For each token in the declared set, search `src/**/*.tsx` files for any
reference to that token name. A token is **orphaned** if it appears in
`src/app.css` but is never referenced in any `.tsx` file.

Report each orphaned token with its declaration location.

### 3. Check: Hardcoded hex in components (HIGH)

Search `src/**/*.tsx` for hex color patterns matching `#[0-9a-fA-F]{3,8}`.

**Exclude** from findings:
- Files ending in `.test.tsx` or `.spec.tsx`
- Matches inside comments (`//` or `/* */`)
- SVG `d=` attribute values
- `url(#...)` references (SVG filter/gradient refs)

Each remaining match is a finding — components must use `var(--grove-*)` tokens
instead of raw hex values.

### 4. Check: Token naming convention (LOW)

Search all `src/**/*.tsx` and `src/**/*.css` for CSS custom property
declarations or references that do NOT use the `--grove-` prefix.

Valid `--grove-` semantic categories: `text`, `surface`, `border`, `accent`,
`space`, `radius`, `shadow`, `font`.

Flag any `--grove-` token that does not fit these categories, or any non-grove
custom property used in component styles.

## Output Format

```
=== DESIGN TOKEN AUDIT ===

Summary
| Metric          | Value  |
|-----------------|--------|
| Status          | PASS/FAIL |
| Total tokens    | N      |
| Referenced      | N      |
| Orphaned        | N      |
| Hardcoded hex   | N      |
| Naming issues   | N      |

Findings
| Severity | Check           | Location              | Issue                    |
|----------|-----------------|-----------------------|--------------------------|
| HIGH     | Hardcoded hex   | src/features/Foo.tsx:42 | #ff0000 — use token    |
| MEDIUM   | Orphaned token  | src/app.css:15        | --grove-unused never ref |
| LOW      | Token naming    | src/bar.css:8         | --custom-bg non-grove    |

=== RESULT: PASS/FAIL ===
```

## Rules

- FAIL if any HIGH findings exist
- PASS if only MEDIUM or LOW findings
- Always scan the full `src/` tree — do not sample
- Exclude test files from the hardcoded hex check only
