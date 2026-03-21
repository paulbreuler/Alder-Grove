---
name: audit
description: Full quality gate — architecture, tests, docs, and build verification
user_invocable: true
---

# /audit

Comprehensive 5-pass quality gate for Alder Grove. Runs all sub-audits and
reports overall PASS/FAIL.

## Arguments

`$ARGUMENTS` controls which sub-audits to run:

- `/audit` — run all 5 passes
- `/audit architecture` — run only Architecture pass
- `/audit docs tokens` — run only Documentation and Design Tokens passes
- `/audit build security` — run only Build & Test and Security passes

Valid names: `architecture`, `build`, `docs`, `tokens`, `security`

If `$ARGUMENTS` is empty or not provided, run all 5 passes.

## Passes

### Pass 1: Architecture Compliance

Run the `/check-architecture` skill (reads `.claude/skills/check-architecture/SKILL.md`).

This dispatches the frontend and backend architecture specialists and
aggregates results.

### Pass 2: Build & Test

Run these commands sequentially — all must pass:

```bash
# Frontend
pnpm check          # TypeScript type checking + ESLint
pnpm test           # Vitest unit tests

# Rust
cargo build --workspace       # Build all crates
cargo test --workspace        # Run all Rust tests
cargo clippy --workspace -- -D warnings   # Lint Rust code (warnings = errors)
```

Any failure in this pass = overall FAIL.

### Pass 3: Documentation Consistency

Run the `/audit-docs` skill (reads `.claude/skills/audit-docs/SKILL.md`).

If the skill file does not exist, report as SKIP with WARNING.

### Pass 4: Design Tokens

Run the `/audit-tokens` skill (reads `.claude/skills/audit-tokens/SKILL.md`).

If the skill file does not exist, report as SKIP with WARNING.

### Pass 5: Security

Run the `/audit-security` skill (reads `.claude/skills/audit-security/SKILL.md`).

If the skill file does not exist, report as SKIP with WARNING.

## Parallelization

Passes 3, 4, and 5 are independent of each other. When running all passes,
dispatch them via the Agent tool in parallel where possible. Passes 1 and 2
may also run in parallel with each other but their results must be collected
before the final summary.

## Output Format

```
=== ALDER GROVE AUDIT ===

| #  | Section              | Status | Findings | Top Issue                        |
|----|----------------------|--------|----------|----------------------------------|
| 1  | Architecture         | PASS   | 0        | —                                |
| 2  | Build & Test         | PASS   | 0        | —                                |
| 3  | Documentation        | PASS   | 1        | /audit-tokens missing from table |
| 4  | Design Tokens        | FAIL   | 3        | Hardcoded #ff0000 in Panel.tsx   |
| 5  | Security             | SKIP   | —        | Skill not found (WARNING)        |

=== OVERALL: FAIL — 3/4 PASS (1 SKIP) — 4 total findings ===
```

## Rules

- Run ALL requested passes — do not skip any unless the skill file is missing
- A pass is FAIL if it has any HIGH or CRITICAL findings
- A pass is PASS if it has only LOW or MEDIUM findings (or none)
- A pass is SKIP if its skill file does not exist (shown as WARNING)
- Overall result: FAIL if ANY pass is FAIL; PASS if all passes are PASS or SKIP
- Skipped sections count toward neither PASS nor FAIL in the denominator
- Report the total number of passes that were PASS out of those actually run
- When `$ARGUMENTS` limits scope, only count the requested passes
- Run this before creating PRs or claiming work is complete
