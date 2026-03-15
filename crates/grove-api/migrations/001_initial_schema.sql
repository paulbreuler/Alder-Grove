-- Migration 001: Initial Schema (v1 content tables)
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
CREATE INDEX idx_workspaces_org_id_id ON workspaces (org_id, id);

-- ============================================================
-- repositories
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

CREATE INDEX idx_repositories_workspace ON repositories (workspace_id);
CREATE INDEX idx_repositories_workspace_id ON repositories (workspace_id, id);

-- ============================================================
-- personas
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

CREATE INDEX idx_personas_workspace ON personas (workspace_id);
CREATE INDEX idx_personas_workspace_id ON personas (workspace_id, id);

-- ============================================================
-- journeys
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

CREATE INDEX idx_journeys_workspace ON journeys (workspace_id);
CREATE INDEX idx_journeys_workspace_id ON journeys (workspace_id, id);

-- ============================================================
-- steps
-- ============================================================
CREATE TABLE steps (
    id                UUID        PRIMARY KEY DEFAULT uuidv7(),
    journey_id        UUID        NOT NULL REFERENCES journeys(id) ON DELETE CASCADE,
    name              TEXT        NOT NULL,
    description       TEXT,
    sort_order        INTEGER     NOT NULL,
    status            TEXT        NOT NULL DEFAULT 'pending'
                      CHECK (status IN ('pending', 'in_progress', 'completed', 'skipped')),
    persona_id        UUID        REFERENCES personas(id) ON DELETE SET NULL,
    percent_complete  REAL        DEFAULT 0.0 CHECK (percent_complete BETWEEN 0.0 AND 1.0),
    ai_authored       BOOLEAN     NOT NULL DEFAULT false,
    ai_confidence     REAL        CHECK (ai_confidence BETWEEN 0.0 AND 1.0),
    ai_rationale      TEXT,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER steps_updated_at
    BEFORE UPDATE ON steps
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_steps_journey_order ON steps (journey_id, sort_order);

-- ============================================================
-- specifications
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
    acceptance_count     INTEGER     GENERATED ALWAYS AS (jsonb_array_length(requirements->'acceptance')) STORED,
    functional_count     INTEGER     GENERATED ALWAYS AS (jsonb_array_length(requirements->'functional')) STORED,
    non_functional_count INTEGER     GENERATED ALWAYS AS (jsonb_array_length(requirements->'non_functional')) STORED,
    ai_authored          BOOLEAN     NOT NULL DEFAULT false,
    ai_confidence        REAL        CHECK (ai_confidence BETWEEN 0.0 AND 1.0),
    ai_rationale         TEXT,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER specifications_updated_at
    BEFORE UPDATE ON specifications
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_specifications_workspace ON specifications (workspace_id);
CREATE INDEX idx_specifications_workspace_id ON specifications (workspace_id, id);
CREATE INDEX idx_specifications_requirements ON specifications USING GIN (requirements);

-- ============================================================
-- step_specifications (M:N join)
-- ============================================================
CREATE TABLE step_specifications (
    step_id          UUID    NOT NULL REFERENCES steps(id) ON DELETE CASCADE,
    specification_id UUID    NOT NULL REFERENCES specifications(id) ON DELETE CASCADE,
    sort_order       INTEGER,
    PRIMARY KEY (step_id, specification_id)
);

-- ============================================================
-- tasks
-- ============================================================
CREATE TABLE tasks (
    id                UUID        PRIMARY KEY DEFAULT uuidv7(),
    specification_id  UUID        NOT NULL REFERENCES specifications(id) ON DELETE CASCADE,
    title             TEXT        NOT NULL,
    description       TEXT,
    sort_order        INTEGER     NOT NULL,
    status            TEXT        NOT NULL DEFAULT 'pending'
                      CHECK (status IN ('pending', 'in_progress', 'completed', 'blocked')),
    ai_authored       BOOLEAN     NOT NULL DEFAULT false,
    ai_confidence     REAL        CHECK (ai_confidence BETWEEN 0.0 AND 1.0),
    ai_rationale      TEXT,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER tasks_updated_at
    BEFORE UPDATE ON tasks
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_tasks_spec_order ON tasks (specification_id, sort_order);

-- ============================================================
-- notes
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

CREATE INDEX idx_notes_workspace ON notes (workspace_id);
CREATE INDEX idx_notes_workspace_id ON notes (workspace_id, id);

-- ============================================================
-- note_links (polymorphic)
-- ============================================================
CREATE TABLE note_links (
    id          UUID    PRIMARY KEY DEFAULT uuidv7(),
    note_id     UUID    NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    entity_type TEXT    NOT NULL
                CHECK (entity_type IN (
                    'journey', 'step', 'specification', 'task',
                    'persona', 'repository',
                    'session', 'agent', 'gate', 'guardrail'
                )),
    entity_id   UUID    NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (note_id, entity_type, entity_id)
);

CREATE INDEX idx_note_links_note ON note_links (note_id);
CREATE INDEX idx_note_links_entity ON note_links (entity_type, entity_id);

-- ============================================================
-- snapshots (v2 placeholder)
-- ============================================================
CREATE TABLE snapshots (
    id              UUID        PRIMARY KEY DEFAULT uuidv7(),
    workspace_id    UUID        NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    repository_id   UUID        NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    status          TEXT        NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending', 'analyzing', 'completed', 'failed')),
    summary         TEXT,
    analysis        JSONB       NOT NULL DEFAULT '{}',
    ai_authored     BOOLEAN     NOT NULL DEFAULT true,
    ai_confidence   REAL        CHECK (ai_confidence BETWEEN 0.0 AND 1.0),
    ai_rationale    TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER snapshots_updated_at
    BEFORE UPDATE ON snapshots
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_snapshots_workspace ON snapshots (workspace_id);
CREATE INDEX idx_snapshots_workspace_id ON snapshots (workspace_id, id);
