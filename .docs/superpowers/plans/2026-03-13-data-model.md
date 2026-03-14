# Data Model Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the v1 PostgreSQL schema, Rust domain types, and TypeScript domain types for all 11 core entities.

**Architecture:** Normalized Core + Strategic JSONB. SQL migration creates tables with uuidv7() PKs, virtual generated columns, and AI provenance fields. Rust structs mirror the schema for compile-time-checked queries (sqlx). TypeScript interfaces mirror Rust types for frontend consumption.

**Tech Stack:** PostgreSQL 18, sqlx 0.8, serde, uuid, chrono, TypeScript, Vitest

**Spec:** `.docs/superpowers/specs/2026-03-13-data-model-design.md`

---

## File Structure

```
# Database
docker-compose.yml                          # PostgreSQL 18 dev instance
src-api/migrations/001_initial_schema.sql   # Complete DDL for 11 tables

# Rust domain types (grove-api crate)
Cargo.toml                                  # Workspace root
src-api/Cargo.toml                          # grove-api crate
src-api/src/lib.rs                          # Crate root, re-exports domain
src-api/src/domain/mod.rs                   # Domain module, declares submodules
src-api/src/domain/workspace.rs             # Workspace entity
src-api/src/domain/repository.rs            # Repository entity
src-api/src/domain/persona.rs               # Persona entity
src-api/src/domain/journey.rs               # Journey entity
src-api/src/domain/step.rs                  # Step entity
src-api/src/domain/specification.rs         # Specification + JSONB types
src-api/src/domain/task.rs                  # Task entity
src-api/src/domain/note.rs                  # Note + NoteLink entities
src-api/src/domain/snapshot.rs              # Snapshot entity
src-api/src/domain/common.rs                # Shared types (AiProvenance)

# TypeScript domain types (per-feature)
package.json                                # Workspace root
pnpm-workspace.yaml                         # pnpm workspace config
tsconfig.json                               # Root TypeScript config
vitest.config.ts                            # Vitest config
src/features/workspaces/domain/types.ts     # Workspace types
src/features/repositories/domain/types.ts   # Repository types
src/features/personas/domain/types.ts       # Persona types
src/features/journeys/domain/types.ts       # Journey + Step types
src/features/specs/domain/types.ts          # Specification + Task types
src/features/notes/domain/types.ts          # Note + NoteLink types
src/features/snapshots/domain/types.ts      # Snapshot types
src/features/shared/domain/types.ts         # Shared types (AiProvenance)
src/features/shared/domain/types.test.ts    # Shared type tests
src/features/workspaces/domain/types.test.ts
src/features/repositories/domain/types.test.ts
src/features/personas/domain/types.test.ts
src/features/journeys/domain/types.test.ts
src/features/specs/domain/types.test.ts
src/features/notes/domain/types.test.ts
src/features/snapshots/domain/types.test.ts
```

---

## Chunk 1: Database Infrastructure + SQL Migration

### Task 1: Docker Compose for PostgreSQL 18

**Files:**
- Create: `docker-compose.yml`

- [ ] **Step 1: Create docker-compose.yml**

```yaml
services:
  db:
    image: postgres:18
    ports:
      - "5432:5432"
    environment:
      POSTGRES_USER: grove
      POSTGRES_PASSWORD: grove_dev
      POSTGRES_DB: grove_dev
    volumes:
      - grove_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U grove"]
      interval: 5s
      timeout: 5s
      retries: 5

volumes:
  grove_data:
```

- [ ] **Step 2: Start PostgreSQL**

Run: `docker compose up -d`
Expected: Container starts, healthcheck passes

- [ ] **Step 3: Verify connection**

Run: `docker compose exec db psql -U grove -d grove_dev -c "SELECT version();"`
Expected: Output contains "PostgreSQL 18"

- [ ] **Step 4: Commit**

```bash
git add docker-compose.yml
git commit -m "chore: add docker-compose for PostgreSQL 18 dev instance"
```

---

### Task 2: SQL Migration — Trigger Function + Workspaces

**Files:**
- Create: `src-api/migrations/001_initial_schema.sql`

- [ ] **Step 1: Create migration directory**

Run: `mkdir -p src-api/migrations`

- [ ] **Step 2: Write trigger function and workspaces table**

```sql
-- Migration 001: Initial Schema
-- PostgreSQL 18 — uses uuidv7(), virtual generated columns

-- ============================================================
-- Trigger function: auto-update updated_at on row modification
-- ============================================================
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ============================================================
-- workspaces
-- Tenant-scoped container. Organization managed by Clerk.
-- ============================================================
CREATE TABLE workspaces (
    id          UUID        PRIMARY KEY DEFAULT uuidv7(),
    org_id      TEXT        NOT NULL,
    name        TEXT        NOT NULL,
    description TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (org_id, name)
);

CREATE TRIGGER workspaces_updated_at
    BEFORE UPDATE ON workspaces
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_workspaces_org_id ON workspaces (org_id);
CREATE INDEX idx_workspaces_id_org ON workspaces (org_id, id);
```

- [ ] **Step 3: Run migration against PostgreSQL**

Run: `docker compose exec db psql -U grove -d grove_dev -f /dev/stdin < src-api/migrations/001_initial_schema.sql`
Expected: CREATE FUNCTION, CREATE TABLE, CREATE TRIGGER, CREATE INDEX — no errors

- [ ] **Step 4: Verify table exists**

Run: `docker compose exec db psql -U grove -d grove_dev -c "\d workspaces"`
Expected: Shows columns id, org_id, name, description, created_at, updated_at

- [ ] **Step 5: Test uuidv7() and updated_at trigger**

Run:
```bash
docker compose exec db psql -U grove -d grove_dev -c "
INSERT INTO workspaces (org_id, name) VALUES ('org_test', 'Test Workspace') RETURNING id, created_at;
UPDATE workspaces SET name = 'Updated' WHERE org_id = 'org_test' RETURNING updated_at;
SELECT id, org_id, name, created_at, updated_at FROM workspaces;
"
```
Expected: UUID v7 format id, updated_at > created_at after update

- [ ] **Step 6: Test unique constraint**

Run:
```bash
docker compose exec db psql -U grove -d grove_dev -c "
INSERT INTO workspaces (org_id, name) VALUES ('org_test', 'Updated');
"
```
Expected: ERROR — duplicate key value violates unique constraint

- [ ] **Step 7: Clean test data**

Run: `docker compose exec db psql -U grove -d grove_dev -c "DELETE FROM workspaces;"`

- [ ] **Step 8: Commit**

```bash
git add src-api/migrations/001_initial_schema.sql
git commit -m "feat(db): add initial migration with workspaces table"
```

---

### Task 3: SQL Migration — Repositories + Personas

**Files:**
- Modify: `src-api/migrations/001_initial_schema.sql`

- [ ] **Step 1: Append repositories table**

```sql
-- ============================================================
-- repositories
-- Reference to an external codebase.
-- ============================================================
CREATE TABLE repositories (
    id              UUID        PRIMARY KEY DEFAULT uuidv7(),
    workspace_id    UUID        NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name            TEXT        NOT NULL,
    url             TEXT,
    default_branch  TEXT        NOT NULL DEFAULT 'main',
    description     TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER repositories_updated_at
    BEFORE UPDATE ON repositories
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_repositories_workspace_id ON repositories (workspace_id);
CREATE INDEX idx_repositories_ws_id ON repositories (workspace_id, id);
```

- [ ] **Step 2: Append personas table**

```sql
-- ============================================================
-- personas
-- Design archetype representing a user type.
-- ============================================================
CREATE TABLE personas (
    id              UUID        PRIMARY KEY DEFAULT uuidv7(),
    workspace_id    UUID        NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name            TEXT        NOT NULL,
    description     TEXT,
    goals           TEXT,
    pain_points     TEXT,
    ai_authored     BOOLEAN     NOT NULL DEFAULT false,
    ai_confidence   REAL        CHECK (ai_confidence BETWEEN 0.0 AND 1.0),
    ai_rationale    TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER personas_updated_at
    BEFORE UPDATE ON personas
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_personas_workspace_id ON personas (workspace_id);
CREATE INDEX idx_personas_ws_id ON personas (workspace_id, id);
```

- [ ] **Step 3: Reset database and run full migration**

Run:
```bash
docker compose exec db psql -U grove -d grove_dev -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
docker compose exec db psql -U grove -d grove_dev -f /dev/stdin < src-api/migrations/001_initial_schema.sql
```
Expected: All CREATE statements succeed

- [ ] **Step 4: Verify CASCADE delete**

Run:
```bash
docker compose exec db psql -U grove -d grove_dev -c "
INSERT INTO workspaces (org_id, name) VALUES ('org1', 'WS1') RETURNING id;
"
```
Then use the returned id:
```bash
docker compose exec db psql -U grove -d grove_dev -c "
INSERT INTO personas (workspace_id, name) VALUES ('<workspace_id>', 'Dev');
INSERT INTO repositories (workspace_id, name, url) VALUES ('<workspace_id>', 'repo1', 'https://github.com/test/repo');
DELETE FROM workspaces WHERE name = 'WS1';
SELECT count(*) FROM personas;
SELECT count(*) FROM repositories;
"
```
Expected: Both counts return 0 (CASCADE deleted)

- [ ] **Step 5: Commit**

```bash
git add src-api/migrations/001_initial_schema.sql
git commit -m "feat(db): add repositories and personas tables"
```

---

### Task 4: SQL Migration — Journeys + Steps + Step-Specifications

**Files:**
- Modify: `src-api/migrations/001_initial_schema.sql`

- [ ] **Step 1: Append journeys table**

```sql
-- ============================================================
-- journeys
-- A user flow composed of ordered steps.
-- ============================================================
CREATE TABLE journeys (
    id              UUID        PRIMARY KEY DEFAULT uuidv7(),
    workspace_id    UUID        NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name            TEXT        NOT NULL,
    description     TEXT,
    persona_id      UUID        REFERENCES personas(id) ON DELETE SET NULL,
    status          TEXT        NOT NULL DEFAULT 'draft'
                    CHECK (status IN ('draft', 'active', 'completed', 'archived')),
    ai_authored     BOOLEAN     NOT NULL DEFAULT false,
    ai_confidence   REAL        CHECK (ai_confidence BETWEEN 0.0 AND 1.0),
    ai_rationale    TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER journeys_updated_at
    BEFORE UPDATE ON journeys
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_journeys_workspace_id ON journeys (workspace_id);
CREATE INDEX idx_journeys_ws_id ON journeys (workspace_id, id);
```

- [ ] **Step 2: Append steps table**

```sql
-- ============================================================
-- steps
-- First-class entity within a journey. AI-assessed completion.
-- ============================================================
CREATE TABLE steps (
    id               UUID        PRIMARY KEY DEFAULT uuidv7(),
    journey_id       UUID        NOT NULL REFERENCES journeys(id) ON DELETE CASCADE,
    name             TEXT        NOT NULL,
    description      TEXT,
    sort_order       INTEGER     NOT NULL,
    status           TEXT        NOT NULL DEFAULT 'pending'
                     CHECK (status IN ('pending', 'in_progress', 'completed', 'skipped')),
    persona_id       UUID        REFERENCES personas(id) ON DELETE SET NULL,
    percent_complete REAL        NOT NULL DEFAULT 0.0
                     CHECK (percent_complete BETWEEN 0.0 AND 1.0),
    ai_authored      BOOLEAN     NOT NULL DEFAULT false,
    ai_confidence    REAL        CHECK (ai_confidence BETWEEN 0.0 AND 1.0),
    ai_rationale     TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER steps_updated_at
    BEFORE UPDATE ON steps
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_steps_journey_sort ON steps (journey_id, sort_order);
```

- [ ] **Step 3: Commit (do NOT run migration yet — specifications table needed for step_specifications)**

```bash
git add src-api/migrations/001_initial_schema.sql
git commit -m "feat(db): add journeys and steps tables"
```

---

### Task 5: SQL Migration — Specifications + Tasks

**Files:**
- Modify: `src-api/migrations/001_initial_schema.sql`

- [ ] **Step 1: Append specifications table**

```sql
-- ============================================================
-- specifications
-- Detailed requirements with JSONB for criteria and metadata.
-- ============================================================
CREATE TABLE specifications (
    id                   UUID        PRIMARY KEY DEFAULT uuidv7(),
    workspace_id         UUID        NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    title                TEXT        NOT NULL,
    description          TEXT,
    scope                TEXT,
    status               TEXT        NOT NULL DEFAULT 'draft'
                         CHECK (status IN ('draft', 'ready', 'in_progress', 'done', 'archived')),
    requirements         JSONB       NOT NULL DEFAULT '{"functional":[],"non_functional":[],"acceptance":[]}',
    dependencies         JSONB       NOT NULL DEFAULT '[]',
    error_handling       JSONB       NOT NULL DEFAULT '[]',
    testing_strategy     JSONB,
    components           JSONB       NOT NULL DEFAULT '[]',
    acceptance_count     INTEGER     GENERATED ALWAYS AS (jsonb_array_length(requirements->'acceptance')) VIRTUAL,
    functional_count     INTEGER     GENERATED ALWAYS AS (jsonb_array_length(requirements->'functional')) VIRTUAL,
    non_functional_count INTEGER     GENERATED ALWAYS AS (jsonb_array_length(requirements->'non_functional')) VIRTUAL,
    ai_authored          BOOLEAN     NOT NULL DEFAULT false,
    ai_confidence        REAL        CHECK (ai_confidence BETWEEN 0.0 AND 1.0),
    ai_rationale         TEXT,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER specifications_updated_at
    BEFORE UPDATE ON specifications
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_specifications_workspace_id ON specifications (workspace_id);
CREATE INDEX idx_specifications_ws_id ON specifications (workspace_id, id);
CREATE INDEX idx_specifications_requirements ON specifications USING GIN (requirements);
```

- [ ] **Step 2: Append step_specifications join table (after specifications)**

```sql
-- ============================================================
-- step_specifications
-- Many-to-many: Step ↔ Specification traceability link.
-- ============================================================
CREATE TABLE step_specifications (
    step_id          UUID    NOT NULL REFERENCES steps(id) ON DELETE CASCADE,
    specification_id UUID    NOT NULL REFERENCES specifications(id) ON DELETE CASCADE,
    sort_order       INTEGER,
    PRIMARY KEY (step_id, specification_id)
);
```

- [ ] **Step 3: Append tasks table**

```sql
-- ============================================================
-- tasks
-- Actionable work items under a specification.
-- ============================================================
CREATE TABLE tasks (
    id               UUID        PRIMARY KEY DEFAULT uuidv7(),
    specification_id UUID        NOT NULL REFERENCES specifications(id) ON DELETE CASCADE,
    title            TEXT        NOT NULL,
    description      TEXT,
    sort_order       INTEGER     NOT NULL,
    status           TEXT        NOT NULL DEFAULT 'pending'
                     CHECK (status IN ('pending', 'in_progress', 'completed', 'blocked')),
    ai_authored      BOOLEAN     NOT NULL DEFAULT false,
    ai_confidence    REAL        CHECK (ai_confidence BETWEEN 0.0 AND 1.0),
    ai_rationale     TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER tasks_updated_at
    BEFORE UPDATE ON tasks
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_tasks_spec_sort ON tasks (specification_id, sort_order);
```

- [ ] **Step 4: Reset database and run full migration**

Run:
```bash
docker compose exec db psql -U grove -d grove_dev -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
docker compose exec db psql -U grove -d grove_dev -f /dev/stdin < src-api/migrations/001_initial_schema.sql
```
Expected: All CREATE statements succeed including step_specifications (now after specifications)

- [ ] **Step 5: Test virtual generated columns**

Run:
```bash
docker compose exec db psql -U grove -d grove_dev -c "
INSERT INTO workspaces (org_id, name) VALUES ('org1', 'WS1') RETURNING id;
"
```
Then:
```bash
docker compose exec db psql -U grove -d grove_dev -c "
INSERT INTO specifications (workspace_id, title, requirements) VALUES (
    '<workspace_id>',
    'Test Spec',
    '{\"functional\": [{\"description\": \"req1\", \"met\": false}, {\"description\": \"req2\", \"met\": true}], \"non_functional\": [{\"description\": \"perf\", \"category\": \"performance\", \"met\": false}], \"acceptance\": [{\"description\": \"ac1\", \"met\": false}]}'
) RETURNING title, functional_count, non_functional_count, acceptance_count;
"
```
Expected: functional_count=2, non_functional_count=1, acceptance_count=1

- [ ] **Step 6: Test GIN index with JSONB query**

Run:
```bash
docker compose exec db psql -U grove -d grove_dev -c "
SELECT title FROM specifications WHERE requirements @> '{\"functional\": [{\"met\": false}]}';
"
```
Expected: Returns 'Test Spec'

- [ ] **Step 7: Clean test data and commit**

Run: `docker compose exec db psql -U grove -d grove_dev -c "DELETE FROM workspaces;"`

```bash
git add src-api/migrations/001_initial_schema.sql
git commit -m "feat(db): add specifications, step_specifications, and tasks tables"
```

---

### Task 6: SQL Migration — Notes + Note Links

**Files:**
- Modify: `src-api/migrations/001_initial_schema.sql`

- [ ] **Step 1: Append notes table**

```sql
-- ============================================================
-- notes
-- Knowledge capture: decisions, learnings, gotchas.
-- ============================================================
CREATE TABLE notes (
    id              UUID        PRIMARY KEY DEFAULT uuidv7(),
    workspace_id    UUID        NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    title           TEXT        NOT NULL,
    content         TEXT        NOT NULL,
    category        TEXT        NOT NULL DEFAULT 'general'
                    CHECK (category IN ('decision', 'learning', 'gotcha', 'general')),
    ai_authored     BOOLEAN     NOT NULL DEFAULT false,
    ai_confidence   REAL        CHECK (ai_confidence BETWEEN 0.0 AND 1.0),
    ai_rationale    TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER notes_updated_at
    BEFORE UPDATE ON notes
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_notes_workspace_id ON notes (workspace_id);
CREATE INDEX idx_notes_ws_id ON notes (workspace_id, id);
```

- [ ] **Step 2: Append note_links table**

```sql
-- ============================================================
-- note_links
-- Polymorphic linking: note → any entity type.
-- No DB-level FK on entity_id (polymorphic trade-off).
-- Tenant isolation inherited through notes.workspace_id.
-- ============================================================
CREATE TABLE note_links (
    id          UUID        PRIMARY KEY DEFAULT uuidv7(),
    note_id     UUID        NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    entity_type TEXT        NOT NULL
                CHECK (entity_type IN ('journey', 'step', 'specification', 'task', 'persona', 'repository')),
    entity_id   UUID        NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (note_id, entity_type, entity_id)
);

CREATE INDEX idx_note_links_note_id ON note_links (note_id);
CREATE INDEX idx_note_links_entity ON note_links (entity_type, entity_id);
```

- [ ] **Step 3: Commit**

```bash
git add src-api/migrations/001_initial_schema.sql
git commit -m "feat(db): add notes and note_links tables"
```

---

### Task 7: SQL Migration — Snapshots + Full Verification

**Files:**
- Modify: `src-api/migrations/001_initial_schema.sql`

- [ ] **Step 1: Append snapshots table**

```sql
-- ============================================================
-- snapshots (v2 placeholder)
-- Structured analysis of a linked codebase.
-- ============================================================
CREATE TABLE snapshots (
    id              UUID        PRIMARY KEY DEFAULT uuidv7(),
    workspace_id    UUID        NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    repository_id   UUID        NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    status          TEXT        NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending', 'analyzing', 'completed', 'failed')),
    summary         TEXT,
    analysis        JSONB       DEFAULT '{}',
    ai_authored     BOOLEAN     NOT NULL DEFAULT true,
    ai_confidence   REAL        CHECK (ai_confidence BETWEEN 0.0 AND 1.0),
    ai_rationale    TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER snapshots_updated_at
    BEFORE UPDATE ON snapshots
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_snapshots_workspace_id ON snapshots (workspace_id);
CREATE INDEX idx_snapshots_ws_id ON snapshots (workspace_id, id);
CREATE INDEX idx_snapshots_repository_id ON snapshots (repository_id);
```

- [ ] **Step 2: Reset database and run complete migration**

Run:
```bash
docker compose exec db psql -U grove -d grove_dev -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
docker compose exec db psql -U grove -d grove_dev -f /dev/stdin < src-api/migrations/001_initial_schema.sql
```
Expected: All 11 tables created, all triggers created, all indexes created — zero errors

- [ ] **Step 3: Verify all tables exist**

Run:
```bash
docker compose exec db psql -U grove -d grove_dev -c "\dt"
```
Expected: 11 tables listed: workspaces, repositories, personas, journeys, steps, step_specifications, specifications, tasks, notes, note_links, snapshots

- [ ] **Step 4: Full integration test — insert data through all tables**

Run:
```bash
docker compose exec db psql -U grove -d grove_dev -c "
-- Create workspace
INSERT INTO workspaces (org_id, name) VALUES ('org1', 'Test WS') RETURNING id;
"
```
Then use returned workspace_id for all subsequent inserts:
```bash
docker compose exec db psql -U grove -d grove_dev -c "
-- Create entities
INSERT INTO repositories (workspace_id, name, url) VALUES ('<ws_id>', 'my-repo', 'https://github.com/test/repo');
INSERT INTO personas (workspace_id, name, goals) VALUES ('<ws_id>', 'Developer', 'Ship features fast');
INSERT INTO journeys (workspace_id, name, persona_id, status) VALUES ('<ws_id>', 'Onboarding', (SELECT id FROM personas LIMIT 1), 'draft') RETURNING id;
"
```
Then:
```bash
docker compose exec db psql -U grove -d grove_dev -c "
INSERT INTO steps (journey_id, name, sort_order) VALUES ((SELECT id FROM journeys LIMIT 1), 'Clone repo', 10);
INSERT INTO specifications (workspace_id, title) VALUES ('<ws_id>', 'Auth Spec');
INSERT INTO step_specifications (step_id, specification_id) VALUES ((SELECT id FROM steps LIMIT 1), (SELECT id FROM specifications LIMIT 1));
INSERT INTO tasks (specification_id, title, sort_order) VALUES ((SELECT id FROM specifications LIMIT 1), 'Implement login', 10);
INSERT INTO notes (workspace_id, title, content, category) VALUES ('<ws_id>', 'Decision: Use JWT', 'Chose JWT for stateless auth', 'decision');
INSERT INTO note_links (note_id, entity_type, entity_id) VALUES ((SELECT id FROM notes LIMIT 1), 'specification', (SELECT id FROM specifications LIMIT 1));
INSERT INTO snapshots (workspace_id, repository_id) VALUES ('<ws_id>', (SELECT id FROM repositories LIMIT 1));
"
```
Expected: All inserts succeed

- [ ] **Step 5: Test CASCADE delete — workspace deletion cleans everything**

Run:
```bash
docker compose exec db psql -U grove -d grove_dev -c "
DELETE FROM workspaces WHERE name = 'Test WS';
SELECT (SELECT count(*) FROM repositories) AS repos,
       (SELECT count(*) FROM personas) AS personas,
       (SELECT count(*) FROM journeys) AS journeys,
       (SELECT count(*) FROM steps) AS steps,
       (SELECT count(*) FROM specifications) AS specs,
       (SELECT count(*) FROM tasks) AS tasks,
       (SELECT count(*) FROM notes) AS notes,
       (SELECT count(*) FROM note_links) AS note_links,
       (SELECT count(*) FROM snapshots) AS snapshots;
"
```
Expected: All counts are 0

- [ ] **Step 6: Test CHECK constraints reject invalid data**

Run:
```bash
docker compose exec db psql -U grove -d grove_dev -c "
INSERT INTO workspaces (org_id, name) VALUES ('org1', 'WS') RETURNING id;
"
```
Then test invalid values:
```bash
docker compose exec db psql -U grove -d grove_dev -c "
-- Invalid status
INSERT INTO journeys (workspace_id, name, status) VALUES ('<ws_id>', 'J1', 'invalid');
"
```
Expected: ERROR — new row violates check constraint

```bash
docker compose exec db psql -U grove -d grove_dev -c "
-- Invalid ai_confidence (> 1.0)
INSERT INTO personas (workspace_id, name, ai_confidence) VALUES ('<ws_id>', 'P1', 1.5);
"
```
Expected: ERROR — new row violates check constraint

```bash
docker compose exec db psql -U grove -d grove_dev -c "
-- Invalid percent_complete (< 0)
INSERT INTO journeys (workspace_id, name, status) VALUES ('<ws_id>', 'J1', 'draft') RETURNING id;
"
```
Then:
```bash
docker compose exec db psql -U grove -d grove_dev -c "
INSERT INTO steps (journey_id, name, sort_order, percent_complete) VALUES ((SELECT id FROM journeys LIMIT 1), 'S1', 10, -0.5);
"
```
Expected: ERROR — new row violates check constraint

- [ ] **Step 7: Clean up and commit**

Run: `docker compose exec db psql -U grove -d grove_dev -c "DELETE FROM workspaces;"`

```bash
git add src-api/migrations/001_initial_schema.sql
git commit -m "feat(db): complete initial schema with all 11 tables

Includes: workspaces, repositories, personas, journeys, steps,
step_specifications, specifications, tasks, notes, note_links,
snapshots. Features: uuidv7() PKs, virtual generated columns,
updated_at triggers, CHECK constraints, CASCADE deletes."
```

---

## Chunk 2: Rust Domain Types

### Task 8: Cargo Workspace + grove-api Crate Scaffolding

**Files:**
- Create: `Cargo.toml`
- Create: `src-api/Cargo.toml`
- Create: `src-api/src/lib.rs`

- [ ] **Step 1: Create workspace Cargo.toml**

```toml
[workspace]
resolver = "3"
members = ["src-api"]
```

- [ ] **Step 2: Create grove-api crate Cargo.toml**

```toml
[package]
name = "grove-api"
version = "0.1.0"
edition = "2024"

[dependencies]
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["serde", "v7"] }

[dev-dependencies]
```

- [ ] **Step 3: Create lib.rs**

```rust
pub mod domain;
```

- [ ] **Step 4: Create domain/mod.rs**

```rust
pub mod common;
pub mod journey;
pub mod note;
pub mod persona;
pub mod repository;
pub mod snapshot;
pub mod specification;
pub mod step;
pub mod task;
pub mod workspace;
```

- [ ] **Step 5: Create placeholder files**

Create empty files for each domain module so the crate compiles:

Run: `mkdir -p src-api/src/domain`

Create each file with just a comment:
```rust
// TODO: implement
```

- [ ] **Step 6: Verify crate compiles**

Run: `cargo build -p grove-api`
Expected: Compiles with no errors (just placeholder modules)

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml src-api/Cargo.toml src-api/src/lib.rs src-api/src/domain/mod.rs src-api/src/domain/*.rs
git commit -m "chore: scaffold Cargo workspace and grove-api crate"
```

---

### Task 9: Common Types (AiProvenance)

**Files:**
- Create: `src-api/src/domain/common.rs`

- [ ] **Step 1: Write failing test**

```rust
// src-api/src/domain/common.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ai_provenance_defaults() {
        let prov = AiProvenance::default();
        assert!(!prov.ai_authored);
        assert!(prov.ai_confidence.is_none());
        assert!(prov.ai_rationale.is_none());
    }

    #[test]
    fn ai_provenance_with_values() {
        let prov = AiProvenance {
            ai_authored: true,
            ai_confidence: Some(0.95),
            ai_rationale: Some("High confidence based on repo analysis".to_string()),
        };
        assert!(prov.ai_authored);
        assert_eq!(prov.ai_confidence, Some(0.95));
    }

    #[test]
    fn ai_provenance_serializes() {
        let prov = AiProvenance {
            ai_authored: true,
            ai_confidence: Some(0.8),
            ai_rationale: None,
        };
        let json = serde_json::to_string(&prov).unwrap();
        let deser: AiProvenance = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.ai_authored, prov.ai_authored);
        assert_eq!(deser.ai_confidence, prov.ai_confidence);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p grove-api -- common`
Expected: FAIL — cannot find type `AiProvenance`

- [ ] **Step 3: Write implementation**

Add above the tests in `src-api/src/domain/common.rs`:

```rust
use serde::{Deserialize, Serialize};

/// AI provenance metadata. Baked into content entities to track
/// whether content was AI-authored and how confident the AI was.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AiProvenance {
    pub ai_authored: bool,
    pub ai_confidence: Option<f32>,
    pub ai_rationale: Option<String>,
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p grove-api -- common`
Expected: 3 tests pass

- [ ] **Step 5: Commit**

```bash
git add src-api/src/domain/common.rs
git commit -m "feat(domain): add AiProvenance shared type"
```

---

### Task 10: Workspace Entity

**Files:**
- Create: `src-api/src/domain/workspace.rs`

- [ ] **Step 1: Write failing test**

```rust
// src-api/src/domain/workspace.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_serialization_roundtrip() {
        let ws = Workspace {
            id: Uuid::now_v7(),
            org_id: "org_clerk_123".to_string(),
            name: "My Workspace".to_string(),
            description: Some("A test workspace".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&ws).unwrap();
        let deser: Workspace = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.id, ws.id);
        assert_eq!(deser.org_id, ws.org_id);
        assert_eq!(deser.name, ws.name);
        assert_eq!(deser.description, ws.description);
    }

    #[test]
    fn workspace_description_optional() {
        let ws = Workspace {
            id: Uuid::now_v7(),
            org_id: "org_1".to_string(),
            name: "Minimal".to_string(),
            description: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert!(ws.description.is_none());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p grove-api -- workspace`
Expected: FAIL — cannot find type `Workspace`

- [ ] **Step 3: Write implementation**

Add above tests:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Workspace {
    pub id: Uuid,
    pub org_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p grove-api -- workspace`
Expected: 2 tests pass

- [ ] **Step 5: Commit**

```bash
git add src-api/src/domain/workspace.rs
git commit -m "feat(domain): add Workspace entity"
```

---

### Task 11: Repository Entity

**Files:**
- Create: `src-api/src/domain/repository.rs`

- [ ] **Step 1: Write failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_serialization_roundtrip() {
        let repo = Repository {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            name: "alder-grove".to_string(),
            url: Some("https://github.com/paulbreuler/alder-grove".to_string()),
            default_branch: "main".to_string(),
            description: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&repo).unwrap();
        let deser: Repository = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.name, repo.name);
        assert_eq!(deser.url, repo.url);
        assert_eq!(deser.default_branch, "main");
    }
}
```

- [ ] **Step 2: Run test — expect fail**

Run: `cargo test -p grove-api -- repository`

- [ ] **Step 3: Write implementation**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Repository {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub url: Option<String>,
    pub default_branch: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

- [ ] **Step 4: Run tests — expect pass**

Run: `cargo test -p grove-api -- repository`

- [ ] **Step 5: Commit**

```bash
git add src-api/src/domain/repository.rs
git commit -m "feat(domain): add Repository entity"
```

---

### Task 12: Persona Entity

**Files:**
- Create: `src-api/src/domain/persona.rs`

- [ ] **Step 1: Write failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::common::AiProvenance;

    #[test]
    fn persona_with_ai_provenance() {
        let persona = Persona {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            name: "Mobile Developer".to_string(),
            description: Some("Builds mobile apps".to_string()),
            goals: Some("Ship features fast".to_string()),
            pain_points: Some("Slow build times".to_string()),
            ai: AiProvenance {
                ai_authored: true,
                ai_confidence: Some(0.85),
                ai_rationale: Some("Generated from repo analysis".to_string()),
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert!(persona.ai.ai_authored);
        assert_eq!(persona.name, "Mobile Developer");
    }

    #[test]
    fn persona_serialization_roundtrip() {
        let persona = Persona {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            name: "PM".to_string(),
            description: None,
            goals: None,
            pain_points: None,
            ai: AiProvenance::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&persona).unwrap();
        let deser: Persona = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.name, "PM");
        assert!(!deser.ai.ai_authored);
    }
}
```

- [ ] **Step 2: Run test — expect fail**

Run: `cargo test -p grove-api -- persona`

- [ ] **Step 3: Write implementation**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::AiProvenance;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Persona {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub goals: Option<String>,
    pub pain_points: Option<String>,
    #[serde(flatten)]
    pub ai: AiProvenance,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

- [ ] **Step 4: Run tests — expect pass**

Run: `cargo test -p grove-api -- persona`

- [ ] **Step 5: Commit**

```bash
git add src-api/src/domain/persona.rs
git commit -m "feat(domain): add Persona entity with AiProvenance"
```

---

### Task 13: Journey + Step Entities

**Files:**
- Create: `src-api/src/domain/journey.rs`
- Create: `src-api/src/domain/step.rs`

- [ ] **Step 1: Write failing test for Journey**

```rust
// src-api/src/domain/journey.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::common::AiProvenance;

    #[test]
    fn journey_status_values() {
        let statuses = ["draft", "active", "completed", "archived"];
        for s in statuses {
            let status: JourneyStatus = serde_json::from_str(&format!("\"{}\"", s)).unwrap();
            let json = serde_json::to_string(&status).unwrap();
            assert_eq!(json, format!("\"{}\"", s));
        }
    }

    #[test]
    fn journey_defaults_to_draft() {
        assert_eq!(JourneyStatus::default(), JourneyStatus::Draft);
    }

    #[test]
    fn journey_serialization_roundtrip() {
        let journey = Journey {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            name: "Onboarding".to_string(),
            description: Some("New user onboarding flow".to_string()),
            persona_id: Some(Uuid::now_v7()),
            status: JourneyStatus::Active,
            ai: AiProvenance::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&journey).unwrap();
        let deser: Journey = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.name, "Onboarding");
        assert_eq!(deser.status, JourneyStatus::Active);
        assert!(deser.persona_id.is_some());
    }
}
```

- [ ] **Step 2: Write Journey implementation**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::AiProvenance;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JourneyStatus {
    #[default]
    Draft,
    Active,
    Completed,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Journey {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub persona_id: Option<Uuid>,
    pub status: JourneyStatus,
    #[serde(flatten)]
    pub ai: AiProvenance,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

- [ ] **Step 3: Run Journey tests — expect pass**

Run: `cargo test -p grove-api -- journey`

- [ ] **Step 4: Write failing test for Step**

```rust
// src-api/src/domain/step.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::common::AiProvenance;

    #[test]
    fn step_status_values() {
        let statuses = ["pending", "in_progress", "completed", "skipped"];
        for s in statuses {
            let status: StepStatus = serde_json::from_str(&format!("\"{}\"", s)).unwrap();
            let json = serde_json::to_string(&status).unwrap();
            assert_eq!(json, format!("\"{}\"", s));
        }
    }

    #[test]
    fn step_percent_complete_bounds() {
        let step = Step {
            id: Uuid::now_v7(),
            journey_id: Uuid::now_v7(),
            name: "Clone repo".to_string(),
            description: None,
            sort_order: 10,
            status: StepStatus::InProgress,
            persona_id: None,
            percent_complete: 0.75,
            ai: AiProvenance {
                ai_authored: true,
                ai_confidence: Some(0.9),
                ai_rationale: Some("Based on code review".to_string()),
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert!(step.percent_complete >= 0.0 && step.percent_complete <= 1.0);
        assert_eq!(step.sort_order, 10);
    }

    #[test]
    fn step_serialization_roundtrip() {
        let step = Step {
            id: Uuid::now_v7(),
            journey_id: Uuid::now_v7(),
            name: "Setup env".to_string(),
            description: Some("Install dependencies".to_string()),
            sort_order: 20,
            status: StepStatus::Pending,
            persona_id: Some(Uuid::now_v7()),
            percent_complete: 0.0,
            ai: AiProvenance::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&step).unwrap();
        let deser: Step = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.name, "Setup env");
        assert_eq!(deser.sort_order, 20);
    }
}
```

- [ ] **Step 5: Write Step implementation**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::AiProvenance;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    #[default]
    Pending,
    InProgress,
    Completed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Step {
    pub id: Uuid,
    pub journey_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub status: StepStatus,
    pub persona_id: Option<Uuid>,
    pub percent_complete: f32,
    #[serde(flatten)]
    pub ai: AiProvenance,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

- [ ] **Step 6: Run Step tests — expect pass**

Run: `cargo test -p grove-api -- step`

- [ ] **Step 7: Add StepSpecification join type to step.rs**

Add to the bottom of the implementation section (above tests) in `src-api/src/domain/step.rs`:

```rust
/// Join record for the step_specifications M:N table.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepSpecification {
    pub step_id: Uuid,
    pub specification_id: Uuid,
    pub sort_order: Option<i32>,
}
```

Add a test:

```rust
    #[test]
    fn step_specification_roundtrip() {
        let ss = StepSpecification {
            step_id: Uuid::now_v7(),
            specification_id: Uuid::now_v7(),
            sort_order: Some(1),
        };
        let json = serde_json::to_string(&ss).unwrap();
        let deser: StepSpecification = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.step_id, ss.step_id);
        assert_eq!(deser.sort_order, Some(1));
    }
```

- [ ] **Step 8: Run all tests — expect pass**

Run: `cargo test -p grove-api -- step`

- [ ] **Step 9: Commit**

```bash
git add src-api/src/domain/journey.rs src-api/src/domain/step.rs
git commit -m "feat(domain): add Journey, Step, and StepSpecification entities"
```

---

### Task 14: Specification Entity + JSONB Types

**Files:**
- Create: `src-api/src/domain/specification.rs`

- [ ] **Step 1: Write failing tests**

```rust
// src-api/src/domain/specification.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::common::AiProvenance;

    #[test]
    fn requirement_item_serialization() {
        let item = RequirementItem {
            description: "Must handle 1000 RPS".to_string(),
            met: false,
            category: Some("performance".to_string()),
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("performance"));
        let deser: RequirementItem = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.description, item.description);
    }

    #[test]
    fn requirements_default_empty() {
        let req = Requirements::default();
        assert!(req.functional.is_empty());
        assert!(req.non_functional.is_empty());
        assert!(req.acceptance.is_empty());
    }

    #[test]
    fn spec_dependency_relationships() {
        let dep = SpecDependency {
            specification_id: Uuid::now_v7(),
            relationship: DependencyRelationship::DependsOn,
        };
        let json = serde_json::to_string(&dep).unwrap();
        assert!(json.contains("depends_on"));
    }

    #[test]
    fn error_scenario_serialization() {
        let err = ErrorScenario {
            scenario: "Workspace not found".to_string(),
            response: "Return 404".to_string(),
        };
        let json = serde_json::to_string(&err).unwrap();
        let deser: ErrorScenario = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.scenario, err.scenario);
    }

    #[test]
    fn testing_strategy_serialization() {
        let strat = TestingStrategy {
            unit: Some("Test validation rules".to_string()),
            integration: Some("Test against PG".to_string()),
            e2e: None,
        };
        let json = serde_json::to_string(&strat).unwrap();
        let deser: TestingStrategy = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.unit, strat.unit);
        assert!(deser.e2e.is_none());
    }

    #[test]
    fn component_action_values() {
        let actions = ["create", "modify", "delete"];
        for a in actions {
            let action: ComponentAction = serde_json::from_str(&format!("\"{}\"", a)).unwrap();
            let json = serde_json::to_string(&action).unwrap();
            assert_eq!(json, format!("\"{}\"", a));
        }
    }

    #[test]
    fn specification_roundtrip() {
        let spec = Specification {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            title: "Auth System".to_string(),
            description: Some("JWT-based auth".to_string()),
            scope: Some("src-api/src/auth/".to_string()),
            status: SpecificationStatus::Ready,
            requirements: Requirements {
                functional: vec![RequirementItem {
                    description: "Login endpoint".to_string(),
                    met: false,
                    category: None,
                }],
                non_functional: vec![],
                acceptance: vec![RequirementItem {
                    description: "User can log in".to_string(),
                    met: false,
                    category: None,
                }],
            },
            dependencies: vec![],
            error_handling: vec![],
            testing_strategy: None,
            components: vec![],
            ai: AiProvenance::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&spec).unwrap();
        let deser: Specification = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.title, "Auth System");
        assert_eq!(deser.requirements.functional.len(), 1);
        assert_eq!(deser.requirements.acceptance.len(), 1);
    }
}
```

- [ ] **Step 2: Run tests — expect fail**

Run: `cargo test -p grove-api -- specification`

- [ ] **Step 3: Write implementation**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::AiProvenance;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RequirementItem {
    pub description: String,
    pub met: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Requirements {
    pub functional: Vec<RequirementItem>,
    pub non_functional: Vec<RequirementItem>,
    pub acceptance: Vec<RequirementItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyRelationship {
    DependsOn,
    RelatedTo,
    Supersedes,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpecDependency {
    pub specification_id: Uuid,
    pub relationship: DependencyRelationship,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorScenario {
    pub scenario: String,
    pub response: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestingStrategy {
    pub unit: Option<String>,
    pub integration: Option<String>,
    pub e2e: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentAction {
    Create,
    Modify,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Component {
    pub path: String,
    pub action: ComponentAction,
    pub description: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecificationStatus {
    #[default]
    Draft,
    Ready,
    InProgress,
    Done,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Specification {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub scope: Option<String>,
    pub status: SpecificationStatus,
    pub requirements: Requirements,
    pub dependencies: Vec<SpecDependency>,
    pub error_handling: Vec<ErrorScenario>,
    pub testing_strategy: Option<TestingStrategy>,
    pub components: Vec<Component>,
    #[serde(flatten)]
    pub ai: AiProvenance,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

- [ ] **Step 4: Run tests — expect pass**

Run: `cargo test -p grove-api -- specification`
Expected: 7 tests pass

- [ ] **Step 5: Commit**

```bash
git add src-api/src/domain/specification.rs
git commit -m "feat(domain): add Specification entity with JSONB types

Includes RequirementItem, Requirements, SpecDependency, ErrorScenario,
TestingStrategy, Component, and their enum types."
```

---

### Task 15: Task Entity

**Files:**
- Create: `src-api/src/domain/task.rs`

- [ ] **Step 1: Write failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::common::AiProvenance;

    #[test]
    fn task_status_values() {
        let statuses = ["pending", "in_progress", "completed", "blocked"];
        for s in statuses {
            let status: TaskStatus = serde_json::from_str(&format!("\"{}\"", s)).unwrap();
            let json = serde_json::to_string(&status).unwrap();
            assert_eq!(json, format!("\"{}\"", s));
        }
    }

    #[test]
    fn task_serialization_roundtrip() {
        let task = Task {
            id: Uuid::now_v7(),
            specification_id: Uuid::now_v7(),
            title: "Implement login".to_string(),
            description: Some("JWT-based login endpoint".to_string()),
            sort_order: 10,
            status: TaskStatus::Pending,
            ai: AiProvenance::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&task).unwrap();
        let deser: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.title, "Implement login");
        assert_eq!(deser.sort_order, 10);
    }
}
```

- [ ] **Step 2: Run test — expect fail**

Run: `cargo test -p grove-api -- task`

- [ ] **Step 3: Write implementation**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::AiProvenance;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    #[default]
    Pending,
    InProgress,
    Completed,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub id: Uuid,
    pub specification_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub status: TaskStatus,
    #[serde(flatten)]
    pub ai: AiProvenance,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

- [ ] **Step 4: Run tests — expect pass**

Run: `cargo test -p grove-api -- task`

- [ ] **Step 5: Commit**

```bash
git add src-api/src/domain/task.rs
git commit -m "feat(domain): add Task entity"
```

---

### Task 16: Note + NoteLink Entities

**Files:**
- Create: `src-api/src/domain/note.rs`

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::common::AiProvenance;

    #[test]
    fn note_category_values() {
        let categories = ["decision", "learning", "gotcha", "general"];
        for c in categories {
            let cat: NoteCategory = serde_json::from_str(&format!("\"{}\"", c)).unwrap();
            let json = serde_json::to_string(&cat).unwrap();
            assert_eq!(json, format!("\"{}\"", c));
        }
    }

    #[test]
    fn note_link_entity_types() {
        let types = ["journey", "step", "specification", "task", "persona", "repository"];
        for t in types {
            let et: LinkableEntityType = serde_json::from_str(&format!("\"{}\"", t)).unwrap();
            let json = serde_json::to_string(&et).unwrap();
            assert_eq!(json, format!("\"{}\"", t));
        }
    }

    #[test]
    fn note_serialization_roundtrip() {
        let note = Note {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            title: "Use JWT".to_string(),
            content: "Decided to use JWT for stateless auth".to_string(),
            category: NoteCategory::Decision,
            ai: AiProvenance::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&note).unwrap();
        let deser: Note = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.title, "Use JWT");
        assert_eq!(deser.category, NoteCategory::Decision);
    }

    #[test]
    fn note_link_serialization() {
        let link = NoteLink {
            id: Uuid::now_v7(),
            note_id: Uuid::now_v7(),
            entity_type: LinkableEntityType::Specification,
            entity_id: Uuid::now_v7(),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&link).unwrap();
        assert!(json.contains("specification"));
    }
}
```

- [ ] **Step 2: Run tests — expect fail**

Run: `cargo test -p grove-api -- note`

- [ ] **Step 3: Write implementation**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::AiProvenance;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoteCategory {
    Decision,
    Learning,
    Gotcha,
    #[default]
    General,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkableEntityType {
    Journey,
    Step,
    Specification,
    Task,
    Persona,
    Repository,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Note {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub title: String,
    pub content: String,
    pub category: NoteCategory,
    #[serde(flatten)]
    pub ai: AiProvenance,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NoteLink {
    pub id: Uuid,
    pub note_id: Uuid,
    pub entity_type: LinkableEntityType,
    pub entity_id: Uuid,
    pub created_at: DateTime<Utc>,
}
```

- [ ] **Step 4: Run tests — expect pass**

Run: `cargo test -p grove-api -- note`

- [ ] **Step 5: Commit**

```bash
git add src-api/src/domain/note.rs
git commit -m "feat(domain): add Note, NoteLink, and LinkableEntityType"
```

---

### Task 17: Snapshot Entity

**Files:**
- Create: `src-api/src/domain/snapshot.rs`

- [ ] **Step 1: Write failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::common::AiProvenance;

    #[test]
    fn snapshot_status_values() {
        let statuses = ["pending", "analyzing", "completed", "failed"];
        for s in statuses {
            let status: SnapshotStatus = serde_json::from_str(&format!("\"{}\"", s)).unwrap();
            let json = serde_json::to_string(&status).unwrap();
            assert_eq!(json, format!("\"{}\"", s));
        }
    }

    #[test]
    fn snapshot_analysis_default() {
        let analysis = SnapshotAnalysis::default();
        assert!(analysis.languages.is_empty());
        assert!(analysis.frameworks.is_empty());
        assert!(analysis.entry_points.is_empty());
        assert!(analysis.dependencies.is_empty());
    }

    #[test]
    fn snapshot_serialization_roundtrip() {
        let snapshot = Snapshot {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            repository_id: Uuid::now_v7(),
            status: SnapshotStatus::Completed,
            summary: Some("A Rust web API project".to_string()),
            analysis: SnapshotAnalysis {
                languages: vec!["Rust".to_string(), "TypeScript".to_string()],
                frameworks: vec!["Axum".to_string()],
                entry_points: vec!["src-api/src/main.rs".to_string()],
                dependencies: vec!["axum".to_string(), "sqlx".to_string()],
            },
            ai: AiProvenance {
                ai_authored: true,
                ai_confidence: Some(0.92),
                ai_rationale: Some("Static analysis complete".to_string()),
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&snapshot).unwrap();
        let deser: Snapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.status, SnapshotStatus::Completed);
        assert_eq!(deser.analysis.languages.len(), 2);
        assert!(deser.ai.ai_authored);
    }
}
```

- [ ] **Step 2: Run test — expect fail**

Run: `cargo test -p grove-api -- snapshot`

- [ ] **Step 3: Write implementation**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::AiProvenance;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotStatus {
    #[default]
    Pending,
    Analyzing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SnapshotAnalysis {
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub entry_points: Vec<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Snapshot {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub repository_id: Uuid,
    pub status: SnapshotStatus,
    pub summary: Option<String>,
    pub analysis: SnapshotAnalysis,
    #[serde(flatten)]
    pub ai: AiProvenance,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

- [ ] **Step 4: Run tests — expect pass**

Run: `cargo test -p grove-api -- snapshot`

- [ ] **Step 5: Run ALL Rust tests**

Run: `cargo test -p grove-api`
Expected: All tests pass (across all domain modules)

- [ ] **Step 6: Commit**

```bash
git add src-api/src/domain/snapshot.rs
git commit -m "feat(domain): add Snapshot entity with SnapshotAnalysis type"
```

---

## Chunk 3: TypeScript Domain Types

### Task 18: pnpm Workspace + Vitest Scaffolding

**Files:**
- Create: `package.json`
- Create: `pnpm-workspace.yaml`
- Create: `tsconfig.json`
- Create: `vitest.config.ts`

- [ ] **Step 1: Create workspace package.json**

```json
{
  "name": "alder-grove",
  "private": true,
  "type": "module",
  "scripts": {
    "test": "vitest run",
    "test:watch": "vitest",
    "check": "tsc --noEmit"
  },
  "devDependencies": {
    "typescript": "^5.8",
    "vitest": "^3"
  }
}
```

- [ ] **Step 2: Create pnpm-workspace.yaml**

```yaml
packages:
  - "."
```

- [ ] **Step 3: Create tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "noEmit": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"]
    }
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules"]
}
```

- [ ] **Step 4: Create vitest.config.ts**

```typescript
import { defineConfig } from "vitest/config";
import { resolve } from "path";

export default defineConfig({
  resolve: {
    alias: {
      "@": resolve(__dirname, "src"),
    },
  },
  test: {
    globals: true,
  },
});
```

- [ ] **Step 5: Install dependencies**

Run: `pnpm install`
Expected: Dependencies installed, lockfile created

- [ ] **Step 6: Verify TypeScript compiles**

Run: `pnpm check`
Expected: No errors (no source files yet)

- [ ] **Step 7: Commit**

```bash
git add package.json pnpm-workspace.yaml tsconfig.json vitest.config.ts pnpm-lock.yaml
git commit -m "chore: scaffold pnpm workspace with TypeScript and Vitest"
```

---

### Task 19: Shared Types (AiProvenance)

**Files:**
- Create: `src/features/shared/domain/types.ts`
- Create: `src/features/shared/domain/types.test.ts`

- [ ] **Step 1: Create directories**

Run: `mkdir -p src/features/shared/domain`

- [ ] **Step 2: Write failing test**

```typescript
// src/features/shared/domain/types.test.ts
import { describe, it, expect } from "vitest";
import type { AiProvenance } from "./types";
import { createDefaultAiProvenance } from "./types";

describe("AiProvenance", () => {
  it("creates default with ai_authored false", () => {
    const prov = createDefaultAiProvenance();
    expect(prov.aiAuthored).toBe(false);
    expect(prov.aiConfidence).toBeNull();
    expect(prov.aiRationale).toBeNull();
  });

  it("satisfies the type contract", () => {
    const prov: AiProvenance = {
      aiAuthored: true,
      aiConfidence: 0.95,
      aiRationale: "High confidence",
    };
    expect(prov.aiAuthored).toBe(true);
    expect(prov.aiConfidence).toBe(0.95);
  });
});
```

- [ ] **Step 3: Run test — expect fail**

Run: `pnpm test -- shared`
Expected: FAIL — module not found

- [ ] **Step 4: Write implementation**

```typescript
// src/features/shared/domain/types.ts

/** AI provenance metadata. Tracks whether content was AI-authored. */
export interface AiProvenance {
  aiAuthored: boolean;
  aiConfidence: number | null;
  aiRationale: string | null;
}

export function createDefaultAiProvenance(): AiProvenance {
  return {
    aiAuthored: false,
    aiConfidence: null,
    aiRationale: null,
  };
}

/** Base timestamp fields present on all entities. */
export interface Timestamps {
  createdAt: string; // ISO 8601
  updatedAt: string;
}
```

- [ ] **Step 5: Run test — expect pass**

Run: `pnpm test -- shared`
Expected: 2 tests pass

- [ ] **Step 6: Commit**

```bash
git add src/features/shared/domain/types.ts src/features/shared/domain/types.test.ts
git commit -m "feat(domain): add shared TypeScript types (AiProvenance, Timestamps)"
```

---

### Task 20: Workspace + Repository Types

**Files:**
- Create: `src/features/workspaces/domain/types.ts`
- Create: `src/features/workspaces/domain/types.test.ts`
- Create: `src/features/repositories/domain/types.ts`
- Create: `src/features/repositories/domain/types.test.ts`

- [ ] **Step 1: Create directories**

Run: `mkdir -p src/features/workspaces/domain src/features/repositories/domain`

- [ ] **Step 2: Write Workspace test**

```typescript
// src/features/workspaces/domain/types.test.ts
import { describe, it, expect } from "vitest";
import type { Workspace } from "./types";

describe("Workspace", () => {
  it("satisfies the type contract", () => {
    const ws: Workspace = {
      id: "019505a1-0000-7000-8000-000000000000",
      orgId: "org_clerk_123",
      name: "My Workspace",
      description: "A test workspace",
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
    expect(ws.id).toBeDefined();
    expect(ws.orgId).toBe("org_clerk_123");
  });

  it("allows null description", () => {
    const ws: Workspace = {
      id: "019505a1-0000-7000-8000-000000000000",
      orgId: "org_1",
      name: "Minimal",
      description: null,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
    expect(ws.description).toBeNull();
  });
});
```

- [ ] **Step 3: Write Workspace implementation**

```typescript
// src/features/workspaces/domain/types.ts
import type { Timestamps } from "@/features/shared/domain/types";

export interface Workspace extends Timestamps {
  id: string;
  orgId: string;
  name: string;
  description: string | null;
}
```

- [ ] **Step 4: Write Repository test**

```typescript
// src/features/repositories/domain/types.test.ts
import { describe, it, expect } from "vitest";
import type { Repository } from "./types";

describe("Repository", () => {
  it("satisfies the type contract", () => {
    const repo: Repository = {
      id: "019505a1-0000-7000-8000-000000000000",
      workspaceId: "019505a1-0000-7000-8000-000000000001",
      name: "alder-grove",
      url: "https://github.com/paulbreuler/alder-grove",
      defaultBranch: "main",
      description: null,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
    expect(repo.defaultBranch).toBe("main");
  });
});
```

- [ ] **Step 5: Write Repository implementation**

```typescript
// src/features/repositories/domain/types.ts
import type { Timestamps } from "@/features/shared/domain/types";

export interface Repository extends Timestamps {
  id: string;
  workspaceId: string;
  name: string;
  url: string | null;
  defaultBranch: string;
  description: string | null;
}
```

- [ ] **Step 6: Run tests**

Run: `pnpm test -- workspaces repositories`
Expected: 3 tests pass

- [ ] **Step 7: Commit**

```bash
git add src/features/workspaces/domain/ src/features/repositories/domain/
git commit -m "feat(domain): add Workspace and Repository TypeScript types"
```

---

### Task 21: Persona Types

**Files:**
- Create: `src/features/personas/domain/types.ts`
- Create: `src/features/personas/domain/types.test.ts`

- [ ] **Step 1: Create directory**

Run: `mkdir -p src/features/personas/domain`

- [ ] **Step 2: Write test**

```typescript
// src/features/personas/domain/types.test.ts
import { describe, it, expect } from "vitest";
import type { Persona } from "./types";

describe("Persona", () => {
  it("satisfies the type contract with AI provenance", () => {
    const persona: Persona = {
      id: "019505a1-0000-7000-8000-000000000000",
      workspaceId: "019505a1-0000-7000-8000-000000000001",
      name: "Mobile Developer",
      description: "Builds mobile apps",
      goals: "Ship features fast",
      painPoints: "Slow build times",
      aiAuthored: true,
      aiConfidence: 0.85,
      aiRationale: "Generated from repo analysis",
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
    expect(persona.aiAuthored).toBe(true);
    expect(persona.goals).toBe("Ship features fast");
  });
});
```

- [ ] **Step 3: Write implementation**

```typescript
// src/features/personas/domain/types.ts
import type { AiProvenance, Timestamps } from "@/features/shared/domain/types";

export interface Persona extends AiProvenance, Timestamps {
  id: string;
  workspaceId: string;
  name: string;
  description: string | null;
  goals: string | null;
  painPoints: string | null;
}
```

- [ ] **Step 4: Run test — expect pass**

Run: `pnpm test -- personas`

- [ ] **Step 5: Commit**

```bash
git add src/features/personas/domain/
git commit -m "feat(domain): add Persona TypeScript type"
```

---

### Task 22: Journey + Step Types

**Files:**
- Create: `src/features/journeys/domain/types.ts`
- Create: `src/features/journeys/domain/types.test.ts`

- [ ] **Step 1: Create directory**

Run: `mkdir -p src/features/journeys/domain`

- [ ] **Step 2: Write test**

```typescript
// src/features/journeys/domain/types.test.ts
import { describe, it, expect } from "vitest";
import { JOURNEY_STATUSES, STEP_STATUSES } from "./types";
import type { Journey, Step } from "./types";

describe("Journey", () => {
  it("has valid status values", () => {
    expect(JOURNEY_STATUSES).toEqual(["draft", "active", "completed", "archived"]);
  });

  it("satisfies the type contract", () => {
    const journey: Journey = {
      id: "019505a1-0000-7000-8000-000000000000",
      workspaceId: "019505a1-0000-7000-8000-000000000001",
      name: "Onboarding",
      description: "New user flow",
      personaId: "019505a1-0000-7000-8000-000000000002",
      status: "active",
      aiAuthored: false,
      aiConfidence: null,
      aiRationale: null,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
    expect(journey.status).toBe("active");
  });
});

describe("Step", () => {
  it("has valid status values", () => {
    expect(STEP_STATUSES).toEqual(["pending", "in_progress", "completed", "skipped"]);
  });

  it("satisfies the type contract with AI-assessed completion", () => {
    const step: Step = {
      id: "019505a1-0000-7000-8000-000000000000",
      journeyId: "019505a1-0000-7000-8000-000000000001",
      name: "Clone repo",
      description: null,
      sortOrder: 10,
      status: "in_progress",
      personaId: null,
      percentComplete: 0.75,
      aiAuthored: true,
      aiConfidence: 0.9,
      aiRationale: "Based on code review",
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
    expect(step.percentComplete).toBe(0.75);
    expect(step.sortOrder).toBe(10);
  });
});
```

- [ ] **Step 3: Write implementation**

```typescript
// src/features/journeys/domain/types.ts
import type { AiProvenance, Timestamps } from "@/features/shared/domain/types";

export const JOURNEY_STATUSES = ["draft", "active", "completed", "archived"] as const;
export type JourneyStatus = (typeof JOURNEY_STATUSES)[number];

export const STEP_STATUSES = ["pending", "in_progress", "completed", "skipped"] as const;
export type StepStatus = (typeof STEP_STATUSES)[number];

export interface Journey extends AiProvenance, Timestamps {
  id: string;
  workspaceId: string;
  name: string;
  description: string | null;
  personaId: string | null;
  status: JourneyStatus;
}

export interface Step extends AiProvenance, Timestamps {
  id: string;
  journeyId: string;
  name: string;
  description: string | null;
  sortOrder: number;
  status: StepStatus;
  personaId: string | null;
  percentComplete: number;
}

export interface StepSpecification {
  stepId: string;
  specificationId: string;
  sortOrder: number | null;
}
```

- [ ] **Step 4: Run tests — expect pass**

Run: `pnpm test -- journeys`

- [ ] **Step 5: Commit**

```bash
git add src/features/journeys/domain/
git commit -m "feat(domain): add Journey and Step TypeScript types"
```

---

### Task 23: Specification + Task Types

**Files:**
- Create: `src/features/specs/domain/types.ts`
- Create: `src/features/specs/domain/types.test.ts`

- [ ] **Step 1: Create directory**

Run: `mkdir -p src/features/specs/domain`

- [ ] **Step 2: Write test**

```typescript
// src/features/specs/domain/types.test.ts
import { describe, it, expect } from "vitest";
import {
  SPEC_STATUSES,
  TASK_STATUSES,
  DEPENDENCY_RELATIONSHIPS,
  COMPONENT_ACTIONS,
} from "./types";
import type {
  Specification,
  Task,
  Requirements,
  RequirementItem,
} from "./types";

describe("Specification", () => {
  it("has valid status values", () => {
    expect(SPEC_STATUSES).toEqual(["draft", "ready", "in_progress", "done", "archived"]);
  });

  it("has valid dependency relationships", () => {
    expect(DEPENDENCY_RELATIONSHIPS).toEqual(["depends_on", "related_to", "supersedes"]);
  });

  it("has valid component actions", () => {
    expect(COMPONENT_ACTIONS).toEqual(["create", "modify", "delete"]);
  });

  it("satisfies the type contract with requirements", () => {
    const req: RequirementItem = {
      description: "Must handle 1000 RPS",
      met: false,
      category: "performance",
    };
    const spec: Specification = {
      id: "019505a1-0000-7000-8000-000000000000",
      workspaceId: "019505a1-0000-7000-8000-000000000001",
      title: "Auth System",
      description: "JWT-based auth",
      scope: "src-api/src/auth/",
      status: "ready",
      requirements: {
        functional: [{ description: "Login endpoint", met: false }],
        non_functional: [req],
        acceptance: [{ description: "User can log in", met: false }],
      },
      dependencies: [{ specification_id: "uuid-123", relationship: "depends_on" }],
      errorHandling: [{ scenario: "Not found", response: "Return 404" }],
      testingStrategy: { unit: "Test rules", integration: "Test against PG", e2e: null },
      components: [{ path: "src/auth.rs", action: "create", description: "Auth module" }],
      aiAuthored: false,
      aiConfidence: null,
      aiRationale: null,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
    expect(spec.requirements.functional).toHaveLength(1);
    expect(spec.requirements.non_functional[0].category).toBe("performance");
  });
});

describe("Task", () => {
  it("has valid status values", () => {
    expect(TASK_STATUSES).toEqual(["pending", "in_progress", "completed", "blocked"]);
  });

  it("satisfies the type contract", () => {
    const task: Task = {
      id: "019505a1-0000-7000-8000-000000000000",
      specificationId: "019505a1-0000-7000-8000-000000000001",
      title: "Implement login",
      description: "JWT-based login endpoint",
      sortOrder: 10,
      status: "pending",
      aiAuthored: false,
      aiConfidence: null,
      aiRationale: null,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
    expect(task.sortOrder).toBe(10);
  });
});
```

- [ ] **Step 3: Write implementation**

```typescript
// src/features/specs/domain/types.ts
import type { AiProvenance, Timestamps } from "@/features/shared/domain/types";

// --- Status and enum constants ---

export const SPEC_STATUSES = ["draft", "ready", "in_progress", "done", "archived"] as const;
export type SpecificationStatus = (typeof SPEC_STATUSES)[number];

export const TASK_STATUSES = ["pending", "in_progress", "completed", "blocked"] as const;
export type TaskStatus = (typeof TASK_STATUSES)[number];

export const DEPENDENCY_RELATIONSHIPS = ["depends_on", "related_to", "supersedes"] as const;
export type DependencyRelationship = (typeof DEPENDENCY_RELATIONSHIPS)[number];

export const COMPONENT_ACTIONS = ["create", "modify", "delete"] as const;
export type ComponentAction = (typeof COMPONENT_ACTIONS)[number];

// --- JSONB shapes (snake_case keys to match stored JSON format) ---

export interface RequirementItem {
  description: string;
  met: boolean;
  category?: string;
}

export interface Requirements {
  functional: RequirementItem[];
  non_functional: RequirementItem[];
  acceptance: RequirementItem[];
}

export interface SpecDependency {
  specification_id: string;
  relationship: DependencyRelationship;
}

export interface ErrorScenario {
  scenario: string;
  response: string;
}

export interface TestingStrategy {
  unit: string | null;
  integration: string | null;
  e2e: string | null;
}

export interface SpecComponent {
  path: string;
  action: ComponentAction;
  description: string;
}

// --- Entities ---

export interface Specification extends AiProvenance, Timestamps {
  id: string;
  workspaceId: string;
  title: string;
  description: string | null;
  scope: string | null;
  status: SpecificationStatus;
  requirements: Requirements;
  dependencies: SpecDependency[];
  errorHandling: ErrorScenario[];
  testingStrategy: TestingStrategy | null;
  components: SpecComponent[];
}

export interface Task extends AiProvenance, Timestamps {
  id: string;
  specificationId: string;
  title: string;
  description: string | null;
  sortOrder: number;
  status: TaskStatus;
}
```

- [ ] **Step 4: Run tests — expect pass**

Run: `pnpm test -- specs`

- [ ] **Step 5: Commit**

```bash
git add src/features/specs/domain/
git commit -m "feat(domain): add Specification and Task TypeScript types with JSONB shapes"
```

---

### Task 24: Note Types

**Files:**
- Create: `src/features/notes/domain/types.ts`
- Create: `src/features/notes/domain/types.test.ts`

- [ ] **Step 1: Create directory**

Run: `mkdir -p src/features/notes/domain`

- [ ] **Step 2: Write test**

```typescript
// src/features/notes/domain/types.test.ts
import { describe, it, expect } from "vitest";
import { NOTE_CATEGORIES, LINKABLE_ENTITY_TYPES } from "./types";
import type { Note, NoteLink } from "./types";

describe("Note", () => {
  it("has valid category values", () => {
    expect(NOTE_CATEGORIES).toEqual(["decision", "learning", "gotcha", "general"]);
  });

  it("satisfies the type contract", () => {
    const note: Note = {
      id: "019505a1-0000-7000-8000-000000000000",
      workspaceId: "019505a1-0000-7000-8000-000000000001",
      title: "Use JWT",
      content: "Decided to use JWT for stateless auth",
      category: "decision",
      aiAuthored: false,
      aiConfidence: null,
      aiRationale: null,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
    expect(note.category).toBe("decision");
  });
});

describe("NoteLink", () => {
  it("has valid entity types", () => {
    expect(LINKABLE_ENTITY_TYPES).toEqual([
      "journey", "step", "specification", "task", "persona", "repository",
    ]);
  });

  it("satisfies the type contract", () => {
    const link: NoteLink = {
      id: "019505a1-0000-7000-8000-000000000000",
      noteId: "019505a1-0000-7000-8000-000000000001",
      entityType: "specification",
      entityId: "019505a1-0000-7000-8000-000000000002",
      createdAt: new Date().toISOString(),
    };
    expect(link.entityType).toBe("specification");
  });
});
```

- [ ] **Step 3: Write implementation**

```typescript
// src/features/notes/domain/types.ts
import type { AiProvenance, Timestamps } from "@/features/shared/domain/types";

export const NOTE_CATEGORIES = ["decision", "learning", "gotcha", "general"] as const;
export type NoteCategory = (typeof NOTE_CATEGORIES)[number];

export const LINKABLE_ENTITY_TYPES = [
  "journey", "step", "specification", "task", "persona", "repository",
] as const;
export type LinkableEntityType = (typeof LINKABLE_ENTITY_TYPES)[number];

export interface Note extends AiProvenance, Timestamps {
  id: string;
  workspaceId: string;
  title: string;
  content: string;
  category: NoteCategory;
}

export interface NoteLink {
  id: string;
  noteId: string;
  entityType: LinkableEntityType;
  entityId: string;
  createdAt: string;
}
```

- [ ] **Step 4: Run tests — expect pass**

Run: `pnpm test -- notes`

- [ ] **Step 5: Commit**

```bash
git add src/features/notes/domain/
git commit -m "feat(domain): add Note and NoteLink TypeScript types"
```

---

### Task 25: Snapshot Types

**Files:**
- Create: `src/features/snapshots/domain/types.ts`
- Create: `src/features/snapshots/domain/types.test.ts`

- [ ] **Step 1: Create directory**

Run: `mkdir -p src/features/snapshots/domain`

- [ ] **Step 2: Write test**

```typescript
// src/features/snapshots/domain/types.test.ts
import { describe, it, expect } from "vitest";
import { SNAPSHOT_STATUSES } from "./types";
import type { Snapshot, SnapshotAnalysis } from "./types";

describe("Snapshot", () => {
  it("has valid status values", () => {
    expect(SNAPSHOT_STATUSES).toEqual(["pending", "analyzing", "completed", "failed"]);
  });

  it("satisfies the type contract", () => {
    const analysis: SnapshotAnalysis = {
      languages: ["Rust", "TypeScript"],
      frameworks: ["Axum"],
      entry_points: ["src-api/src/main.rs"],
      dependencies: ["axum", "sqlx"],
    };
    const snapshot: Snapshot = {
      id: "019505a1-0000-7000-8000-000000000000",
      workspaceId: "019505a1-0000-7000-8000-000000000001",
      repositoryId: "019505a1-0000-7000-8000-000000000002",
      status: "completed",
      summary: "A Rust web API",
      analysis,
      aiAuthored: true,
      aiConfidence: 0.92,
      aiRationale: "Static analysis complete",
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
    expect(snapshot.analysis.languages).toHaveLength(2);
    expect(snapshot.aiAuthored).toBe(true);
  });
});
```

- [ ] **Step 3: Write implementation**

```typescript
// src/features/snapshots/domain/types.ts
import type { AiProvenance, Timestamps } from "@/features/shared/domain/types";

export const SNAPSHOT_STATUSES = ["pending", "analyzing", "completed", "failed"] as const;
export type SnapshotStatus = (typeof SNAPSHOT_STATUSES)[number];

export interface SnapshotAnalysis {
  languages: string[];
  frameworks: string[];
  entry_points: string[];
  dependencies: string[];
}

export interface Snapshot extends AiProvenance, Timestamps {
  id: string;
  workspaceId: string;
  repositoryId: string;
  status: SnapshotStatus;
  summary: string | null;
  analysis: SnapshotAnalysis;
}
```

- [ ] **Step 4: Run tests — expect pass**

Run: `pnpm test -- snapshots`

- [ ] **Step 5: Run ALL TypeScript tests**

Run: `pnpm test`
Expected: All tests pass across all feature domains

- [ ] **Step 6: Run TypeScript check**

Run: `pnpm check`
Expected: No type errors

- [ ] **Step 7: Commit**

```bash
git add src/features/snapshots/domain/
git commit -m "feat(domain): add Snapshot TypeScript type

Completes all TypeScript domain types for v1 entities."
```

---

## Verification

After all chunks are complete:

1. `docker compose exec db psql -U grove -d grove_dev -c "\dt"` — 11 tables
2. `cargo test -p grove-api` — all Rust domain tests pass
3. `pnpm test` — all TypeScript domain tests pass
4. `pnpm check` — no TypeScript errors
5. SQL migration runs cleanly from scratch (drop + re-run)
