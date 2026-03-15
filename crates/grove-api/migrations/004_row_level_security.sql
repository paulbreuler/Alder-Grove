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

GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO grove_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA public
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO grove_app;
