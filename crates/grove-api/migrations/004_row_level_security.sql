-- Migration 004: Row Level Security (Multi-Tenant Workspace Isolation)
-- Enforces workspace-scoped data isolation at the database level.
-- See docs/authentication-authorization.md for architecture.

-- ============================================================
-- Helper: resolve current workspace from session variable
-- ============================================================
-- SECURITY DEFINER prevents search_path injection attacks.
-- NULLIF handles empty string after RESET (PostgreSQL 18 behavior).
-- current_setting(..., true) returns NULL instead of error when unset.
CREATE OR REPLACE FUNCTION current_workspace_id()
RETURNS uuid
LANGUAGE sql
STABLE
SECURITY DEFINER
SET search_path = pg_catalog, public
AS $$
    SELECT NULLIF(current_setting('app.current_workspace_id', true), '')::uuid
$$;

-- ============================================================
-- Application role: cannot bypass RLS
-- ============================================================
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'grove_app') THEN
        CREATE ROLE grove_app NOLOGIN;
    END IF;
END
$$;

GRANT USAGE ON SCHEMA public TO grove_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO grove_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA public
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO grove_app;

-- ============================================================
-- Denormalize workspace_id into indirectly-scoped tables
-- Enables O(1) RLS policy evaluation (no subqueries needed)
-- ============================================================

-- steps (scoped through journeys)
ALTER TABLE steps
    ADD COLUMN workspace_id UUID REFERENCES workspaces(id) ON DELETE CASCADE;
UPDATE steps SET workspace_id = (SELECT workspace_id FROM journeys WHERE journeys.id = steps.journey_id);
ALTER TABLE steps ALTER COLUMN workspace_id SET NOT NULL;
CREATE INDEX idx_steps_workspace ON steps (workspace_id);

-- tasks (scoped through specifications)
ALTER TABLE tasks
    ADD COLUMN workspace_id UUID REFERENCES workspaces(id) ON DELETE CASCADE;
UPDATE tasks SET workspace_id = (SELECT workspace_id FROM specifications WHERE specifications.id = tasks.specification_id);
ALTER TABLE tasks ALTER COLUMN workspace_id SET NOT NULL;
CREATE INDEX idx_tasks_workspace ON tasks (workspace_id);

-- note_links (scoped through notes)
ALTER TABLE note_links
    ADD COLUMN workspace_id UUID REFERENCES workspaces(id) ON DELETE CASCADE;
UPDATE note_links SET workspace_id = (SELECT workspace_id FROM notes WHERE notes.id = note_links.note_id);
ALTER TABLE note_links ALTER COLUMN workspace_id SET NOT NULL;
CREATE INDEX idx_note_links_workspace ON note_links (workspace_id);

-- gates (scoped through sessions)
ALTER TABLE gates
    ADD COLUMN workspace_id UUID REFERENCES workspaces(id) ON DELETE CASCADE;
UPDATE gates SET workspace_id = (SELECT workspace_id FROM sessions WHERE sessions.id = gates.session_id);
ALTER TABLE gates ALTER COLUMN workspace_id SET NOT NULL;
CREATE INDEX idx_gates_workspace ON gates (workspace_id);

-- events (scoped through sessions)
ALTER TABLE events
    ADD COLUMN workspace_id UUID REFERENCES workspaces(id) ON DELETE CASCADE;
UPDATE events SET workspace_id = (SELECT workspace_id FROM sessions WHERE sessions.id = events.session_id);
ALTER TABLE events ALTER COLUMN workspace_id SET NOT NULL;
CREATE INDEX idx_events_workspace ON events (workspace_id);

-- step_specifications (join table)
ALTER TABLE step_specifications
    ADD COLUMN workspace_id UUID REFERENCES workspaces(id) ON DELETE CASCADE;
UPDATE step_specifications SET workspace_id = (
    SELECT j.workspace_id FROM steps s
    JOIN journeys j ON j.id = s.journey_id
    WHERE s.id = step_specifications.step_id
);
ALTER TABLE step_specifications ALTER COLUMN workspace_id SET NOT NULL;
CREATE INDEX idx_step_specs_workspace ON step_specifications (workspace_id);

-- session_guardrails (join table)
ALTER TABLE session_guardrails
    ADD COLUMN workspace_id UUID REFERENCES workspaces(id) ON DELETE CASCADE;
UPDATE session_guardrails SET workspace_id = (
    SELECT workspace_id FROM sessions WHERE sessions.id = session_guardrails.session_id
);
ALTER TABLE session_guardrails ALTER COLUMN workspace_id SET NOT NULL;
CREATE INDEX idx_session_guardrails_workspace ON session_guardrails (workspace_id);

-- ============================================================
-- Re-grant after denormalization (ensures grove_app can access all tables)
-- ============================================================
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO grove_app;

-- ============================================================
-- Enable Row Level Security on all tables
-- FORCE ensures RLS applies even to table owners
-- ============================================================

-- workspaces (root — policy on id, not workspace_id)
ALTER TABLE workspaces ENABLE ROW LEVEL SECURITY;
ALTER TABLE workspaces FORCE ROW LEVEL SECURITY;
CREATE POLICY workspace_isolation ON workspaces
    FOR ALL
    USING (id = current_workspace_id())
    WITH CHECK (id = current_workspace_id());

-- All workspace_id-scoped tables (18 tables)
DO $$
DECLARE
    tbl TEXT;
BEGIN
    FOR tbl IN
        SELECT unnest(ARRAY[
            'repositories', 'personas', 'journeys', 'specifications',
            'notes', 'snapshots', 'agents', 'sessions',
            'gate_definitions', 'guardrails', 'collaborative_documents',
            'steps', 'tasks', 'note_links', 'gates', 'events',
            'step_specifications', 'session_guardrails'
        ])
    LOOP
        EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', tbl);
        EXECUTE format('ALTER TABLE %I FORCE ROW LEVEL SECURITY', tbl);
        EXECUTE format(
            'CREATE POLICY workspace_isolation ON %I FOR ALL USING (workspace_id = current_workspace_id()) WITH CHECK (workspace_id = current_workspace_id())',
            tbl
        );
    END LOOP;
END
$$;

-- ============================================================
-- Append-only enforcement for events table
-- RESTRICTIVE policies use AND logic — must pass alongside
-- the permissive workspace_isolation policy. USING (false)
-- unconditionally blocks the operation for the grove_app role.
-- Superuser (grove) bypasses RLS for CASCADE deletes.
-- ============================================================
CREATE POLICY events_no_update ON events AS RESTRICTIVE FOR UPDATE USING (false);
CREATE POLICY events_no_delete ON events AS RESTRICTIVE FOR DELETE USING (false);
