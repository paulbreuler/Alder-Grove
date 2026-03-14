# Alder Grove Data Model — Design

**Date:** 2026-03-13
**Status:** Approved
**Scope:** Full v1 PostgreSQL schema for all core entities

---

## Overview

Complete field-level data model for Alder Grove's v1 entities. Uses PostgreSQL 18 features (uuidv7, virtual generated columns, JSON_TABLE) with a "Normalized Core + Strategic JSONB" approach: core entities and relationships are proper tables with foreign keys; embedded data uses typed JSONB with virtual columns for queryability.

## Problem

The entity model is conceptually defined in `docs/architecture-reference.md` but has no field-level details, no database schema, and no type definitions. Implementation cannot begin without a concrete schema design that addresses AI provenance, multi-tenancy, and the relationships between entities.

## Design

### Approach: Normalized Core + Strategic JSONB

- Core entities get dedicated tables with proper FKs and constraints
- Embedded data (requirements, error handling, testing strategy) uses typed JSONB
- PG18 virtual generated columns expose JSONB aggregates at zero storage cost
- AI provenance fields (`ai_authored`, `ai_confidence`, `ai_rationale`) baked directly into entities
- Polymorphic note linking for cross-entity knowledge capture
- Hard delete for v1 (no soft delete complexity)

### Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Persona | Design archetype, not agent identity | Agents come later via repo .md files |
| Steps | First-class entity with AI-assessed completion | percent_complete is AI-judged, not spec-count-derived |
| Spec requirements | JSONB: functional / non-functional / acceptance | PG18 JSON_TABLE + virtual columns for queryability |
| Tasks | First-class entity under Specification | Actionable work items agents will eventually execute |
| Notes | Flat with category + polymorphic linking | Future-friendly for knowledge graph |
| AI provenance | Fields on entities | AI-native, not a separate annotation layer |
| Workspace membership | Clerk org-level for v1 | Defer granular permissions |
| ACP entities | Deferred | Core entities are AI-ready; protocol designed separately |
| IDs | uuidv7() | PG18 native, time-sortable, globally unique |
| Soft delete | No | Hard delete for v1; add audit trail later if needed |

### PostgreSQL 18 Features Leveraged

- **`uuidv7()`**: Native time-sortable UUID generation for all primary keys
- **Virtual generated columns**: Expose JSONB array lengths (requirement counts) without storage
- **`JSON_TABLE()`**: Query JSONB arrays as relational rows (requirements, components)
- **`RETURNING OLD/NEW`**: Capture before/after state on AI-driven updates
- **Parallel GIN index builds**: Fast indexing on JSONB columns

---

## Schema (11 tables)

### Cross-Cutting Conventions

Every content entity carries:

```sql
id UUID PRIMARY KEY DEFAULT uuidv7(),
created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
```

Every top-level workspace-scoped entity carries:

```sql
workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE
```

Child entities (steps, tasks, step_specifications, note_links) inherit workspace scope through their parent FK. They do not carry a redundant `workspace_id` — tenant isolation is enforced via CASCADE from the parent and JOINs in queries. This avoids data duplication and update anomalies.

AI-authorable entities carry (where relevant):

```sql
ai_authored BOOLEAN NOT NULL DEFAULT false,
ai_confidence REAL CHECK (ai_confidence BETWEEN 0.0 AND 1.0),
ai_rationale TEXT
```

Table names are plural, snake_case. All SQL identifiers are snake_case.

**`updated_at` trigger:** The migration includes a reusable trigger function `set_updated_at()` that automatically sets `updated_at = now()` on row update. Applied to all tables with `updated_at`.

---

### workspaces

Tenant-scoped container for all content. Organization is managed by Clerk.

| Column | Type | Constraints | Notes |
|--------|------|-------------|-------|
| id | UUID | PK, DEFAULT uuidv7() | |
| org_id | TEXT | NOT NULL | Clerk organization ID (external string) |
| name | TEXT | NOT NULL | |
| description | TEXT | | |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | |

UNIQUE constraint on `(org_id, name)` — no duplicate workspace names within an org.

No membership table for v1. Clerk org membership = workspace access.

---

### repositories

Reference to an external codebase. Tauri client handles local cloning/access.

| Column | Type | Constraints | Notes |
|--------|------|-------------|-------|
| id | UUID | PK | |
| workspace_id | UUID | FK → workspaces, ON DELETE CASCADE | |
| name | TEXT | NOT NULL | |
| url | TEXT | | Git remote URL (HTTPS or SSH) |
| default_branch | TEXT | NOT NULL, DEFAULT 'main' | For snapshot analysis |
| description | TEXT | | |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | |

---

### personas

Design archetype representing a user type. Not agent identity (agents come later).

| Column | Type | Constraints | Notes |
|--------|------|-------------|-------|
| id | UUID | PK | |
| workspace_id | UUID | FK → workspaces, ON DELETE CASCADE | |
| name | TEXT | NOT NULL | e.g., "Mobile Developer", "Product Manager" |
| description | TEXT | | |
| goals | TEXT | | What this persona is trying to achieve |
| pain_points | TEXT | | Frustrations and blockers |
| ai_authored | BOOLEAN | NOT NULL, DEFAULT false | |
| ai_confidence | REAL | CHECK 0.0–1.0 | |
| ai_rationale | TEXT | | |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | |

---

### journeys

A user flow composed of ordered steps, designed for a specific persona.

| Column | Type | Constraints | Notes |
|--------|------|-------------|-------|
| id | UUID | PK | |
| workspace_id | UUID | FK → workspaces, ON DELETE CASCADE | |
| name | TEXT | NOT NULL | e.g., "First-time project setup" |
| description | TEXT | | |
| persona_id | UUID | FK → personas, ON DELETE SET NULL, NULLABLE | Which persona this journey is for |
| status | TEXT | NOT NULL, DEFAULT 'draft' | CHECK IN ('draft', 'active', 'completed', 'archived') |
| ai_authored | BOOLEAN | NOT NULL, DEFAULT false | |
| ai_confidence | REAL | CHECK 0.0–1.0 | |
| ai_rationale | TEXT | | |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | |

---

### steps

First-class entity within a journey. AI-assessed completion, not spec-count-derived.

| Column | Type | Constraints | Notes |
|--------|------|-------------|-------|
| id | UUID | PK | |
| journey_id | UUID | FK → journeys, ON DELETE CASCADE | |
| name | TEXT | NOT NULL | e.g., "Clone repository" |
| description | TEXT | | |
| sort_order | INTEGER | NOT NULL | Gaps for reordering (10, 20, 30) |
| status | TEXT | NOT NULL, DEFAULT 'pending' | CHECK IN ('pending', 'in_progress', 'completed', 'skipped') |
| persona_id | UUID | FK → personas, ON DELETE SET NULL, NULLABLE | Override; inherits from journey by default |
| percent_complete | REAL | DEFAULT 0.0, CHECK 0.0–1.0 | AI-assessed completion |
| ai_authored | BOOLEAN | NOT NULL, DEFAULT false | |
| ai_confidence | REAL | CHECK 0.0–1.0 | |
| ai_rationale | TEXT | | AI's explanation of completion assessment |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | |

---

### step_specifications (join table)

Many-to-many link: Journey → Step → Specification. The traceability chain.

| Column | Type | Constraints | Notes |
|--------|------|-------------|-------|
| step_id | UUID | FK → steps, ON DELETE CASCADE | |
| specification_id | UUID | FK → specifications, ON DELETE CASCADE | |
| sort_order | INTEGER | | Order within the step |
| PRIMARY KEY | | (step_id, specification_id) | |

---

### specifications

Detailed requirements with functional, non-functional, and acceptance criteria.

| Column | Type | Constraints | Notes |
|--------|------|-------------|-------|
| id | UUID | PK | |
| workspace_id | UUID | FK → workspaces, ON DELETE CASCADE | |
| title | TEXT | NOT NULL | |
| description | TEXT | | |
| scope | TEXT | | Area/components affected |
| status | TEXT | NOT NULL, DEFAULT 'draft' | CHECK IN ('draft', 'ready', 'in_progress', 'done', 'archived') |
| requirements | JSONB | DEFAULT '{"functional":[],"non_functional":[],"acceptance":[]}' | See shape below |
| dependencies | JSONB | DEFAULT '[]' | See shape below |
| error_handling | JSONB | DEFAULT '[]' | See shape below |
| testing_strategy | JSONB | | See shape below |
| components | JSONB | DEFAULT '[]' | See shape below |
| acceptance_count | INT | VIRTUAL GENERATED | `jsonb_array_length(requirements->'acceptance')` |
| functional_count | INT | VIRTUAL GENERATED | `jsonb_array_length(requirements->'functional')` |
| non_functional_count | INT | VIRTUAL GENERATED | `jsonb_array_length(requirements->'non_functional')` |
| ai_authored | BOOLEAN | NOT NULL, DEFAULT false | |
| ai_confidence | REAL | CHECK 0.0–1.0 | |
| ai_rationale | TEXT | | |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | |

**Requirements JSONB shape:**

```json
{
  "functional": [
    { "description": "System shall allow users to create workspaces", "met": false }
  ],
  "non_functional": [
    { "description": "API response time under 200ms p95", "category": "performance", "met": false }
  ],
  "acceptance": [
    { "description": "User can create a workspace and see it in the list", "met": false }
  ]
}
```

**Dependencies JSONB shape:**

```json
[
  { "specification_id": "uuid", "relationship": "depends_on" },
  { "specification_id": "uuid", "relationship": "related_to" },
  { "specification_id": "uuid", "relationship": "supersedes" }
]
```

**Error Handling JSONB shape:**

```json
[
  { "scenario": "Workspace not found", "response": "Return 404 with RFC 9457 Problem Details" },
  { "scenario": "Duplicate workspace name in org", "response": "Return 409 Conflict" }
]
```

**Testing Strategy JSONB shape:**

```json
{
  "unit": "Test domain validation rules for workspace name length and format",
  "integration": "Test CRUD endpoints against PostgreSQL, verify workspace isolation",
  "e2e": "Test workspace creation through Tauri UI, verify it appears in workspace list"
}
```

**Components JSONB shape:**

```json
[
  { "path": "src/features/workspaces/domain/types.ts", "action": "create", "description": "Workspace domain types" },
  { "path": "src-api/src/routes/workspaces.rs", "action": "modify", "description": "Add CRUD endpoints" }
]
```

Valid `action` values: `create`, `modify`, `delete`.

**Dependency cleanup:** Dependencies reference specification IDs as UUIDs inside JSONB (no DB-level FK). Application-layer validation must check referenced specifications exist on write. When a specification is deleted, a cleanup job or application hook should remove dangling references from other specs' `dependencies` arrays.

---

### tasks

Actionable work items under a specification. What AI agents will eventually execute.

| Column | Type | Constraints | Notes |
|--------|------|-------------|-------|
| id | UUID | PK | |
| specification_id | UUID | FK → specifications, ON DELETE CASCADE | |
| title | TEXT | NOT NULL | |
| description | TEXT | | |
| sort_order | INTEGER | NOT NULL | |
| status | TEXT | NOT NULL, DEFAULT 'pending' | CHECK IN ('pending', 'in_progress', 'completed', 'blocked') |
| ai_authored | BOOLEAN | NOT NULL, DEFAULT false | |
| ai_confidence | REAL | CHECK 0.0–1.0 | |
| ai_rationale | TEXT | | |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | |

---

### notes

Knowledge capture: decisions, learnings, gotchas. Linkable to any entity.

| Column | Type | Constraints | Notes |
|--------|------|-------------|-------|
| id | UUID | PK | |
| workspace_id | UUID | FK → workspaces, ON DELETE CASCADE | |
| title | TEXT | NOT NULL | |
| content | TEXT | NOT NULL | |
| category | TEXT | NOT NULL, DEFAULT 'general' | CHECK IN ('decision', 'learning', 'gotcha', 'general') |
| ai_authored | BOOLEAN | NOT NULL, DEFAULT false | |
| ai_confidence | REAL | CHECK 0.0–1.0 | |
| ai_rationale | TEXT | | |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | |

Design decisions are Notes with category 'decision', linked to the relevant entity via note_links. This keeps specs focused on requirements while notes capture reasoning.

---

### note_links

Polymorphic linking: a note can attach to any entity type.

| Column | Type | Constraints | Notes |
|--------|------|-------------|-------|
| id | UUID | PK | |
| note_id | UUID | FK → notes, ON DELETE CASCADE | |
| entity_type | TEXT | NOT NULL | CHECK IN ('journey', 'step', 'specification', 'task', 'persona', 'repository') |
| entity_id | UUID | NOT NULL | Polymorphic (no DB-level FK) |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | |
| UNIQUE | | (note_id, entity_type, entity_id) | Prevent duplicate links |

Application-layer validation ensures the target entity exists. Adding a new linkable entity type requires only extending the CHECK constraint.

`note_links` has no `workspace_id` — tenant isolation is inherited through the parent `notes` table (which carries `workspace_id`). Queries for "all notes linked to entity X" must JOIN through `notes` to enforce workspace scoping.

---

### snapshots (v2 placeholder)

Structured analysis of a linked codebase. Minimal schema reserved for v2 scoping.

| Column | Type | Constraints | Notes |
|--------|------|-------------|-------|
| id | UUID | PK | |
| workspace_id | UUID | FK → workspaces, ON DELETE CASCADE | |
| repository_id | UUID | FK → repositories, ON DELETE CASCADE | |
| status | TEXT | NOT NULL, DEFAULT 'pending' | CHECK IN ('pending', 'analyzing', 'completed', 'failed') |
| summary | TEXT | | AI-generated codebase summary |
| analysis | JSONB | DEFAULT '{}' | See shape below |
| ai_authored | BOOLEAN | NOT NULL, DEFAULT true | Always AI-generated |
| ai_confidence | REAL | CHECK 0.0–1.0 | |
| ai_rationale | TEXT | | |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | |

Multiple snapshots per repository (point-in-time analyses).

**Analysis JSONB shape (v2 — minimal stub for now):**

```json
{
  "languages": [],
  "frameworks": [],
  "entry_points": [],
  "dependencies": []
}
```

The analysis shape will be expanded during v2 scoping. The stub provides enough structure for Rust/TypeScript type generation while leaving room for evolution.

---

## Indexes

| Table | Index | Purpose |
|-------|-------|---------|
| All content tables | `(workspace_id)` | Tenant-scoped queries |
| All content tables | `(workspace_id, id)` | Single-entity lookup within tenant |
| steps | `(journey_id, sort_order)` | Ordered step retrieval |
| tasks | `(specification_id, sort_order)` | Ordered task retrieval |
| note_links | `(note_id)` | Find all links for a note |
| note_links | `(entity_type, entity_id)` | Find all notes for an entity |
| specifications | GIN on `requirements` | JSONB queries |

---

## Entity Relationship Summary

```
Workspace (1) ──────┬──── (N) Repository
                    ├──── (N) Persona
                    ├──── (N) Journey
                    │           └──── (N) Step ───── (M:N) ──── Specification
                    │                        │                      ├──── (N) Task
                    │                        │                      └──── requirements (JSONB)
                    │                        └──── persona_id (FK, optional override)
                    ├──── (N) Note
                    │           └──── (N) note_links → any entity (polymorphic)
                    └──── (N) Snapshot
                                └──── repository_id (FK)
```

---

## Testing Strategy

- **Unit**: Rust domain type validation, JSONB shape validation
- **Integration**: SQL migration runs cleanly against PostgreSQL 18, CASCADE deletes propagate, CHECK constraints reject invalid data, virtual generated columns compute expected values
- **E2E**: Full CRUD through API endpoints (when implemented)

---

## Files Affected

| File | Action |
|------|--------|
| `src-api/migrations/001_initial_schema.sql` | Create — SQL migration |
| `src-api/src/domain/` | Create — Rust entity structs |
| `src/features/*/domain/` | Create — TypeScript domain types |
| `docs/architecture-reference.md` | Update — field-level entity details, PG18 features |
| `CLAUDE.md` | Update — if conventions change |
