-- Migration 004: Row Level Security (Multi-Tenant Workspace Isolation)
-- Enforces workspace-scoped data isolation at the database level.
-- See docs/authentication-authorization.md for architecture.

-- ============================================================
-- 1. Helper function
-- ============================================================
CREATE OR REPLACE FUNCTION current_workspace_id()
RETURNS uuid LANGUAGE sql STABLE SECURITY DEFINER
SET search_path = pg_catalog, public
AS $$ SELECT NULLIF(current_setting('app.current_workspace_id', true), '')::uuid $$;

REVOKE ALL ON FUNCTION current_workspace_id() FROM PUBLIC;

-- ============================================================
-- 2. Application role
-- ============================================================
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'grove_app') THEN
        CREATE ROLE grove_app NOLOGIN;
    END IF;
END $$;

GRANT USAGE ON SCHEMA public TO grove_app;
GRANT EXECUTE ON FUNCTION current_workspace_id() TO grove_app;

-- ============================================================
-- 3. Add workspace_id columns to 7 indirect tables
-- ============================================================
ALTER TABLE steps ADD COLUMN workspace_id UUID;
ALTER TABLE tasks ADD COLUMN workspace_id UUID;
ALTER TABLE note_links ADD COLUMN workspace_id UUID;
ALTER TABLE gates ADD COLUMN workspace_id UUID;
ALTER TABLE events ADD COLUMN workspace_id UUID;
ALTER TABLE step_specifications ADD COLUMN workspace_id UUID;
ALTER TABLE session_guardrails ADD COLUMN workspace_id UUID;

-- ============================================================
-- 4. Backfill from parent tables
-- ============================================================
UPDATE steps SET workspace_id = (SELECT workspace_id FROM journeys WHERE journeys.id = steps.journey_id);
UPDATE tasks SET workspace_id = (SELECT workspace_id FROM specifications WHERE specifications.id = tasks.specification_id);
UPDATE note_links SET workspace_id = (SELECT workspace_id FROM notes WHERE notes.id = note_links.note_id);
UPDATE gates SET workspace_id = (SELECT workspace_id FROM sessions WHERE sessions.id = gates.session_id);
UPDATE events SET workspace_id = (SELECT workspace_id FROM sessions WHERE sessions.id = events.session_id);
UPDATE step_specifications SET workspace_id = (SELECT s.workspace_id FROM steps s WHERE s.id = step_specifications.step_id);
UPDATE session_guardrails SET workspace_id = (SELECT workspace_id FROM sessions WHERE sessions.id = session_guardrails.session_id);

-- ============================================================
-- 5. Set NOT NULL after backfill
-- ============================================================
ALTER TABLE steps ALTER COLUMN workspace_id SET NOT NULL;
ALTER TABLE tasks ALTER COLUMN workspace_id SET NOT NULL;
ALTER TABLE note_links ALTER COLUMN workspace_id SET NOT NULL;
ALTER TABLE gates ALTER COLUMN workspace_id SET NOT NULL;
ALTER TABLE events ALTER COLUMN workspace_id SET NOT NULL;
ALTER TABLE step_specifications ALTER COLUMN workspace_id SET NOT NULL;
ALTER TABLE session_guardrails ALTER COLUMN workspace_id SET NOT NULL;

-- ============================================================
-- 6. Drop conflicting indexes, create UNIQUE composite indexes on parents
-- ============================================================
DROP INDEX IF EXISTS idx_journeys_workspace_id;
DROP INDEX IF EXISTS idx_specifications_workspace_id;
DROP INDEX IF EXISTS idx_notes_workspace_id;

CREATE UNIQUE INDEX idx_journeys_ws_id ON journeys (workspace_id, id);
CREATE UNIQUE INDEX idx_specifications_ws_id ON specifications (workspace_id, id);
CREATE UNIQUE INDEX idx_notes_ws_id ON notes (workspace_id, id);
CREATE UNIQUE INDEX idx_sessions_ws_id ON sessions (workspace_id, id);
CREATE UNIQUE INDEX idx_guardrails_ws_id ON guardrails (workspace_id, id);
CREATE UNIQUE INDEX idx_steps_ws_id ON steps (workspace_id, id);
CREATE UNIQUE INDEX idx_tasks_ws_id ON tasks (workspace_id, id);

-- ============================================================
-- 7. Composite FKs — enforce same-workspace integrity at DB level
-- ============================================================
ALTER TABLE steps ADD CONSTRAINT fk_steps_ws FOREIGN KEY (workspace_id, journey_id) REFERENCES journeys(workspace_id, id) ON DELETE CASCADE;
ALTER TABLE tasks ADD CONSTRAINT fk_tasks_ws FOREIGN KEY (workspace_id, specification_id) REFERENCES specifications(workspace_id, id) ON DELETE CASCADE;
ALTER TABLE note_links ADD CONSTRAINT fk_note_links_ws FOREIGN KEY (workspace_id, note_id) REFERENCES notes(workspace_id, id) ON DELETE CASCADE;
ALTER TABLE gates ADD CONSTRAINT fk_gates_ws FOREIGN KEY (workspace_id, session_id) REFERENCES sessions(workspace_id, id) ON DELETE CASCADE;
ALTER TABLE events ADD CONSTRAINT fk_events_ws FOREIGN KEY (workspace_id, session_id) REFERENCES sessions(workspace_id, id) ON DELETE CASCADE;
ALTER TABLE step_specifications ADD CONSTRAINT fk_step_specs_ws_step FOREIGN KEY (workspace_id, step_id) REFERENCES steps(workspace_id, id) ON DELETE CASCADE;
ALTER TABLE step_specifications ADD CONSTRAINT fk_step_specs_ws_spec FOREIGN KEY (workspace_id, specification_id) REFERENCES specifications(workspace_id, id) ON DELETE CASCADE;
ALTER TABLE session_guardrails ADD CONSTRAINT fk_sg_ws_session FOREIGN KEY (workspace_id, session_id) REFERENCES sessions(workspace_id, id) ON DELETE CASCADE;
ALTER TABLE session_guardrails ADD CONSTRAINT fk_sg_ws_guardrail FOREIGN KEY (workspace_id, guardrail_id) REFERENCES guardrails(workspace_id, id) ON DELETE CASCADE;

-- ============================================================
-- 8. Indexes on denormalized columns
-- ============================================================
CREATE INDEX idx_steps_workspace ON steps (workspace_id);
CREATE INDEX idx_tasks_workspace ON tasks (workspace_id);
CREATE INDEX idx_note_links_workspace ON note_links (workspace_id);
CREATE INDEX idx_gates_workspace ON gates (workspace_id);
CREATE INDEX idx_gates_ws_id ON gates (workspace_id, id);
CREATE INDEX idx_events_workspace ON events (workspace_id);
CREATE INDEX idx_events_ws_id ON events (workspace_id, id);
CREATE INDEX idx_step_specs_workspace ON step_specifications (workspace_id);
CREATE INDEX idx_session_guardrails_workspace ON session_guardrails (workspace_id);

-- ============================================================
-- 9. Grant grove_app access
-- ============================================================
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO grove_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO grove_app;

-- ============================================================
-- 10. Enable RLS on all 19 tables with FORCE
-- ============================================================
ALTER TABLE workspaces ENABLE ROW LEVEL SECURITY;
ALTER TABLE workspaces FORCE ROW LEVEL SECURITY;
CREATE POLICY workspace_isolation ON workspaces FOR ALL
    USING (id = current_workspace_id())
    WITH CHECK (id = current_workspace_id());

DO $$ DECLARE tbl TEXT; BEGIN
    FOR tbl IN SELECT unnest(ARRAY[
        'repositories', 'personas', 'journeys', 'specifications',
        'notes', 'snapshots', 'agents', 'sessions',
        'gate_definitions', 'guardrails', 'collaborative_documents',
        'steps', 'tasks', 'note_links', 'gates', 'events',
        'step_specifications', 'session_guardrails'
    ]) LOOP
        EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', tbl);
        EXECUTE format('ALTER TABLE %I FORCE ROW LEVEL SECURITY', tbl);
        EXECUTE format('CREATE POLICY workspace_isolation ON %I FOR ALL USING (workspace_id = current_workspace_id()) WITH CHECK (workspace_id = current_workspace_id())', tbl);
    END LOOP;
END $$;

-- ============================================================
-- 11. Append-only events (RESTRICTIVE = AND with permissive policies)
-- ============================================================
CREATE POLICY events_no_update ON events AS RESTRICTIVE FOR UPDATE USING (false);
CREATE POLICY events_no_delete ON events AS RESTRICTIVE FOR DELETE USING (false);
