---
name: audit-tests
description: Audit test suite health, coverage gaps, and TDD compliance
user_invocable: true
---

# /audit-tests

Audit the test suites across Rust and Frontend, check for production `unwrap()`
calls, verify test selector conventions, and assess TDD commit ordering.

## Workflow

### 1. Check: Rust test suite (CRITICAL)

Run `cargo test --workspace` and parse the output:
- Total tests, passed, failed, ignored
- Any compilation errors count as FAIL

### 2. Check: Frontend test suite (CRITICAL)

Run `pnpm test` and parse the output:
- Total tests, passed, failed, skipped
- Any compilation or runtime errors count as FAIL

### 3. Check: Production unwrap() (HIGH)

Search the following crate source directories for `.unwrap()` calls:
- `crates/grove-domain/src/**/*.rs`
- `crates/grove-api/src/**/*.rs`
- `crates/grove-tauri/src/**/*.rs`

**Exclude** from findings:
- Lines inside `#[cfg(test)]` modules
- Files in `tests/` directories
- Files ending in `_test.rs`

Each remaining `.unwrap()` is a finding — production code should use `?`,
`.expect("reason")`, or proper error handling.

### 4. Check: Test selector convention (MEDIUM)

Sample components in `src/features/**/*.tsx` (exclude test files). Check whether
interactive elements (buttons, inputs, links, forms) include `data-testid`
attributes for Testing Library selectors.

Report components that lack `data-testid` on interactive elements.

### 5. Check: TDD commit ordering (LOW)

Run `git log --oneline -20` and perform heuristic analysis:
- Look for `test:` commits followed by `feat:` or `fix:` commits (good TDD)
- Look for `feat:` or `fix:` commits without preceding or accompanying test
  commits (potential test-after pattern)
- This check is **informational only** — never causes FAIL

## Output Format

```
=== TEST SUITE AUDIT ===

Summary
| Suite       | Status | Tests | Passed | Failed | Skipped/Ignored |
|-------------|--------|-------|--------|--------|-----------------|
| Rust        | PASS   | 18    | 18     | 0      | 0               |
| Frontend    | PASS   | 42    | 42     | 0      | 0               |

Coverage
| Check              | Status | Count | Details                  |
|--------------------|--------|-------|--------------------------|
| Production unwrap  | PASS   | 0     | No unwrap in prod code   |
| Test selectors     | WARN   | 3     | 3 components lack testid |
| TDD ordering       | INFO   | —     | 60% test-first pattern   |

Findings
| Severity | Check           | Location                        | Issue              |
|----------|-----------------|---------------------------------|--------------------|
| HIGH     | Prod unwrap     | crates/grove-api/src/routes.rs:42 | .unwrap() in handler |
| MEDIUM   | Test selectors  | src/features/Home/Panel.tsx     | No data-testid     |

=== RESULT: PASS/FAIL ===
```

## Rules

- FAIL if any CRITICAL or HIGH findings exist
- PASS if only MEDIUM or LOW findings
- Rust or Frontend suite failure is always CRITICAL
- TDD ordering is always informational — never blocks
- Report exact file paths and line numbers for unwrap findings
