---
paths:
  - "**/*.rs"
  - "**/*.ts"
  - "**/*.tsx"
---

# Design Principles

All code must demonstrate PhD-level software engineering rigor. These
principles apply to every agent and every file.

## SOLID

- **Single Responsibility** — every type, module, and function has one reason to change
- **Open/Closed** — extend behavior via new variants, trait implementations, or composition — not by modifying existing code
- **Liskov Substitution** — implementations of a trait must be interchangeable without breaking callers
- **Interface Segregation** — prefer focused traits over broad ones; never force an implementor to provide methods it doesn't need
- **Dependency Inversion** — domain defines interfaces (port traits); adapters implement them; no concrete dependencies crossing layer boundaries

## Coupling & Cohesion

- **Loose coupling** — modules communicate through well-defined interfaces, not internal details
- **High cohesion** — related behavior lives together; a module's parts should all serve a single purpose
- **No cross-feature imports** in frontend; no cross-crate leaks in Rust (domain stays pure)

## DRY & Reusability

- Before writing a new type, function, or trait — search for an existing one that fits
- Extract shared behavior into traits, helper methods, or generic abstractions
- Three similar code blocks = extract immediately
- Port traits sharing the same CRUD shape should use a generic base trait

## Extensibility & Closure

- Design types and enums so new variants or implementations can be added without modifying existing match arms or trait signatures
- Use tagged enums and trait objects to keep behavior open for extension
- State machines should validate transitions via a single method — named convenience methods delegate, not duplicate

## What To Check Before Committing

- Does every type have a single, clear purpose?
- Are there duplicated code blocks that should be extracted?
- Could a new feature be added without modifying existing code?
- Do tests cover behavior, not just shape (serde roundtrips)?
- Are error types specific enough to guide the caller?
