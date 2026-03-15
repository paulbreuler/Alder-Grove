---
name: audit
description: Full quality gate — architecture, tests, docs, and build verification
user_invocable: true
---

# /audit

Comprehensive quality gate for Alder Grove. Runs three passes and reports overall
PASS/FAIL.

## Pass 1: Architecture Compliance

Run the specialist architecture checks:

- `/check-frontend-architecture`
- `/check-backend-architecture`
- `/check-architecture` may be used as the aggregate wrapper when a combined report is needed

## Pass 2: Build & Test

```bash
# Frontend
pnpm check          # TypeScript type checking + ESLint
pnpm test           # Vitest unit tests

# Rust
cargo build --workspace    # Build all crates
cargo test --workspace     # Run all Rust tests
cargo clippy --workspace   # Lint Rust code
```

All must pass. Any failure = overall FAIL.

## Pass 3: Documentation Consistency

- `CLAUDE.md` and `AGENTS.md` reflect the current project guidance
- `.agents/skills/` bridges point to the current `.claude/skills/` source
- `docs/architecture-reference.md` tech stack matches actual dependencies
- Shell extension table matches registered extensions
- Design specs in `.docs/superpowers/specs/` reference existing features

## Output

```
=== ALDER GROVE AUDIT ===

Pass 1: Architecture Compliance
  ✅ Frontend architecture
  ✅ Backend architecture
  RESULT: PASS

Pass 2: Build & Test
  ✅ pnpm check
  ✅ pnpm test (42 tests passed)
  ✅ cargo build
  ✅ cargo test (18 tests passed)
  ✅ cargo clippy
  RESULT: PASS

Pass 3: Documentation Consistency
  ✅ CLAUDE.md and AGENTS.md current
  ⚠️  docs/architecture-reference.md tech stack — missing Biome from table
  ✅ Extension table matches code
  RESULT: PASS (1 warning)

=== OVERALL: PASS ===
```

## Rules

- Run ALL passes — do not skip any
- Architecture and Build passes are required (FAIL = overall FAIL)
- Documentation pass warnings don't block, but should be reported
- Run this before creating PRs or claiming work is complete
