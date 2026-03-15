-- Migration 001: Foundation + v1 Content Tables (11 tables)
-- PostgreSQL 18 — uuidv7(), STORED generated columns, RLS from day one.
-- See docs/authentication-authorization.md for multi-tenancy architecture.

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
-- Tenant context helper (SECURITY DEFINER, fail-safe NULL)
-- ============================================================
CREATE OR REPLACE FUNCTION current_workspace_id()
RETURNS uuid LANGUAGE sql STABLE SECURITY DEFINER
SET search_path = pg_catalog, public
AS $$ SELECT NULLIF(current_setting('app.current_workspace_id', true), '')::uuid $$;

REVOKE ALL ON FUNCTION current_workspace_id() FROM PUBLIC;

-- ============================================================
-- Application role: subject to RLS (no BYPASSRLS)
-- ============================================================
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'grove_app') THEN
        CREATE ROLE grove_app NOLOGIN;
    END IF;
END $$;

GRANT USAGE ON SCHEMA public TO grove_app;
GRANT EXECUTE ON FUNCTION current_workspace_id() TO grove_app;

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

CREATE TRIGGER workspaces_updated_at BEFORE UPDATE ON workspaces
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_workspaces_org ON workspaces (org_id);

ALTER TABLE workspaces ENABLE ROW LEVEL SECURITY;
ALTER TABLE workspaces FORCE ROW LEVEL SECURITY;
CREATE POLICY workspace_isolation ON workspaces FOR ALL
    USING (id = current_workspace_id())
    WITH CHECK (id = current_workspace_id());

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

CREATE TRIGGER repositories_updated_at BEFORE UPDATE ON repositories
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_repositories_ws ON repositories (workspace_id);
CREATE UNIQUE INDEX idx_repositories_ws_id ON repositories (workspace_id, id);

ALTER TABLE repositories ENABLE ROW LEVEL SECURITY;
ALTER TABLE repositories FORCE ROW LEVEL SECURITY;
CREATE POLICY workspace_isolation ON repositories FOR ALL
    USING (workspace_id = current_workspace_id())
    WITH CHECK (workspace_id = current_workspace_id());

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

CREATE TRIGGER personas_updated_at BEFORE UPDATE ON personas
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_personas_ws ON personas (workspace_id);
CREATE UNIQUE INDEX idx_personas_ws_id ON personas (workspace_id, id);

ALTER TABLE personas ENABLE ROW LEVEL SECURITY;
ALTER TABLE personas FORCE ROW LEVEL SECURITY;
CREATE POLICY workspace_isolation ON personas FOR ALL
    USING (workspace_id = current_workspace_id())
    WITH CHECK (workspace_id = current_workspace_id());

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

CREATE TRIGGER journeys_updated_at BEFORE UPDATE ON journeys
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_journeys_ws ON journeys (workspace_id);
CREATE UNIQUE INDEX idx_journeys_ws_id ON journeys (workspace_id, id);

ALTER TABLE journeys ENABLE ROW LEVEL SECURITY;
ALTER TABLE journeys FORCE ROW LEVEL SECURITY;
CREATE POLICY workspace_isolation ON journeys FOR ALL
    USING (workspace_id = current_workspace_id())
    WITH CHECK (workspace_id = current_workspace_id());

-- ============================================================
-- steps (denormalized workspace_id + composite FK to journeys)
-- ============================================================
CREATE TABLE steps (
    id                UUID        PRIMARY KEY DEFAULT uuidv7(),
    journey_id        UUID        NOT NULL,
    workspace_id      UUID        NOT NULL,
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
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (workspace_id, journey_id) REFERENCES journeys(workspace_id, id) ON DELETE CASCADE
);

CREATE TRIGGER steps_updated_at BEFORE UPDATE ON steps
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_steps_journey_order ON steps (journey_id, sort_order);
CREATE INDEX idx_steps_ws ON steps (workspace_id);
CREATE UNIQUE INDEX idx_steps_ws_id ON steps (workspace_id, id);

ALTER TABLE steps ENABLE ROW LEVEL SECURITY;
ALTER TABLE steps FORCE ROW LEVEL SECURITY;
CREATE POLICY workspace_isolation ON steps FOR ALL
    USING (workspace_id = current_workspace_id())
    WITH CHECK (workspace_id = current_workspace_id());

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

CREATE TRIGGER specifications_updated_at BEFORE UPDATE ON specifications
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_specifications_ws ON specifications (workspace_id);
CREATE UNIQUE INDEX idx_specifications_ws_id ON specifications (workspace_id, id);
CREATE INDEX idx_specifications_requirements ON specifications USING GIN (requirements);

ALTER TABLE specifications ENABLE ROW LEVEL SECURITY;
ALTER TABLE specifications FORCE ROW LEVEL SECURITY;
CREATE POLICY workspace_isolation ON specifications FOR ALL
    USING (workspace_id = current_workspace_id())
    WITH CHECK (workspace_id = current_workspace_id());

-- ============================================================
-- step_specifications (M:N join — composite FKs enforce same workspace)
-- ============================================================
CREATE TABLE step_specifications (
    step_id          UUID    NOT NULL,
    specification_id UUID    NOT NULL,
    workspace_id     UUID    NOT NULL,
    sort_order       INTEGER,
    PRIMARY KEY (step_id, specification_id),
    FOREIGN KEY (workspace_id, step_id) REFERENCES steps(workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, specification_id) REFERENCES specifications(workspace_id, id) ON DELETE CASCADE
);

CREATE INDEX idx_step_specs_ws ON step_specifications (workspace_id);

ALTER TABLE step_specifications ENABLE ROW LEVEL SECURITY;
ALTER TABLE step_specifications FORCE ROW LEVEL SECURITY;
CREATE POLICY workspace_isolation ON step_specifications FOR ALL
    USING (workspace_id = current_workspace_id())
    WITH CHECK (workspace_id = current_workspace_id());

-- ============================================================
-- tasks (denormalized workspace_id + composite FK to specifications)
-- ============================================================
CREATE TABLE tasks (
    id                UUID        PRIMARY KEY DEFAULT uuidv7(),
    specification_id  UUID        NOT NULL,
    workspace_id      UUID        NOT NULL,
    title             TEXT        NOT NULL,
    description       TEXT,
    sort_order        INTEGER     NOT NULL,
    status            TEXT        NOT NULL DEFAULT 'pending'
                      CHECK (status IN ('pending', 'in_progress', 'completed', 'blocked')),
    ai_authored       BOOLEAN     NOT NULL DEFAULT false,
    ai_confidence     REAL        CHECK (ai_confidence BETWEEN 0.0 AND 1.0),
    ai_rationale      TEXT,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (workspace_id, specification_id) REFERENCES specifications(workspace_id, id) ON DELETE CASCADE
);

CREATE TRIGGER tasks_updated_at BEFORE UPDATE ON tasks
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_tasks_spec_order ON tasks (specification_id, sort_order);
CREATE INDEX idx_tasks_ws ON tasks (workspace_id);
CREATE UNIQUE INDEX idx_tasks_ws_id ON tasks (workspace_id, id);

ALTER TABLE tasks ENABLE ROW LEVEL SECURITY;
ALTER TABLE tasks FORCE ROW LEVEL SECURITY;
CREATE POLICY workspace_isolation ON tasks FOR ALL
    USING (workspace_id = current_workspace_id())
    WITH CHECK (workspace_id = current_workspace_id());

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

CREATE TRIGGER notes_updated_at BEFORE UPDATE ON notes
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_notes_ws ON notes (workspace_id);
CREATE UNIQUE INDEX idx_notes_ws_id ON notes (workspace_id, id);

ALTER TABLE notes ENABLE ROW LEVEL SECURITY;
ALTER TABLE notes FORCE ROW LEVEL SECURITY;
CREATE POLICY workspace_isolation ON notes FOR ALL
    USING (workspace_id = current_workspace_id())
    WITH CHECK (workspace_id = current_workspace_id());

-- ============================================================
-- note_links (polymorphic — composite FK to notes)
-- ============================================================
CREATE TABLE note_links (
    id          UUID    PRIMARY KEY DEFAULT uuidv7(),
    note_id     UUID    NOT NULL,
    workspace_id UUID   NOT NULL,
    entity_type TEXT    NOT NULL
                CHECK (entity_type IN (
                    'journey', 'step', 'specification', 'task',
                    'persona', 'repository',
                    'session', 'agent', 'gate', 'guardrail'
                )),
    entity_id   UUID    NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (note_id, entity_type, entity_id),
    FOREIGN KEY (workspace_id, note_id) REFERENCES notes(workspace_id, id) ON DELETE CASCADE
);

CREATE INDEX idx_note_links_note ON note_links (note_id);
CREATE INDEX idx_note_links_entity ON note_links (entity_type, entity_id);
CREATE INDEX idx_note_links_ws ON note_links (workspace_id);

ALTER TABLE note_links ENABLE ROW LEVEL SECURITY;
ALTER TABLE note_links FORCE ROW LEVEL SECURITY;
CREATE POLICY workspace_isolation ON note_links FOR ALL
    USING (workspace_id = current_workspace_id())
    WITH CHECK (workspace_id = current_workspace_id());

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

CREATE TRIGGER snapshots_updated_at BEFORE UPDATE ON snapshots
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_snapshots_ws ON snapshots (workspace_id);
CREATE UNIQUE INDEX idx_snapshots_ws_id ON snapshots (workspace_id, id);

ALTER TABLE snapshots ENABLE ROW LEVEL SECURITY;
ALTER TABLE snapshots FORCE ROW LEVEL SECURITY;
CREATE POLICY workspace_isolation ON snapshots FOR ALL
    USING (workspace_id = current_workspace_id())
    WITH CHECK (workspace_id = current_workspace_id());

-- ============================================================
-- Grant grove_app access to all v1 tables
-- ============================================================
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO grove_app;
