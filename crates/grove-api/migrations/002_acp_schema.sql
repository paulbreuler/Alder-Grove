-- Migration 002: ACP Schema (Agent Communication Protocol)
-- 7 tables for agent sessions, gates, events, and guardrails.
-- RLS + composite FKs baked in from the start.

-- ============================================================
-- agents
-- ============================================================
CREATE TABLE agents (
    id              UUID        PRIMARY KEY DEFAULT uuidv7(),
    workspace_id    UUID        NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name            TEXT        NOT NULL,
    provider        TEXT        NOT NULL,
    model           TEXT,
    description     TEXT,
    capabilities    JSONB       NOT NULL DEFAULT '[]',
    config          JSONB       NOT NULL DEFAULT '{}',
    status          TEXT        NOT NULL DEFAULT 'active'
                    CHECK (status IN ('active', 'disabled')),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER agents_updated_at BEFORE UPDATE ON agents
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_agents_ws ON agents (workspace_id);
CREATE UNIQUE INDEX idx_agents_ws_id ON agents (workspace_id, id);

ALTER TABLE agents ENABLE ROW LEVEL SECURITY;
ALTER TABLE agents FORCE ROW LEVEL SECURITY;
CREATE POLICY workspace_isolation ON agents FOR ALL
    USING (workspace_id = current_workspace_id())
    WITH CHECK (workspace_id = current_workspace_id());

-- ============================================================
-- sessions
-- ============================================================
CREATE TABLE sessions (
    id              UUID        PRIMARY KEY DEFAULT uuidv7(),
    workspace_id    UUID        NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    agent_id        UUID        NOT NULL REFERENCES agents(id) ON DELETE RESTRICT,
    intent          TEXT        NOT NULL
                    CHECK (intent IN ('implement', 'review', 'assess', 'analyze', 'author', 'execute')),
    target_type     TEXT
                    CHECK (target_type IN ('specification', 'task', 'journey', 'step', 'snapshot', 'repository')),
    target_id       UUID,
    title           TEXT        NOT NULL,
    status          TEXT        NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending', 'active', 'gated', 'completed', 'failed', 'cancelled', 'timed_out')),
    context         JSONB       NOT NULL DEFAULT '{}',
    result          JSONB,
    started_at      TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    initiated_by    TEXT        NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((target_type IS NULL) = (target_id IS NULL))
);

CREATE TRIGGER sessions_updated_at BEFORE UPDATE ON sessions
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_sessions_ws ON sessions (workspace_id);
CREATE UNIQUE INDEX idx_sessions_ws_id ON sessions (workspace_id, id);
CREATE INDEX idx_sessions_ws_status ON sessions (workspace_id, status);
CREATE INDEX idx_sessions_agent ON sessions (agent_id);
CREATE INDEX idx_sessions_target ON sessions (target_type, target_id);

ALTER TABLE sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE sessions FORCE ROW LEVEL SECURITY;
CREATE POLICY workspace_isolation ON sessions FOR ALL
    USING (workspace_id = current_workspace_id())
    WITH CHECK (workspace_id = current_workspace_id());

-- ============================================================
-- gate_definitions
-- ============================================================
CREATE TABLE gate_definitions (
    id              UUID        PRIMARY KEY DEFAULT uuidv7(),
    workspace_id    UUID        NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name            TEXT        NOT NULL,
    description     TEXT,
    trigger_type    TEXT        NOT NULL
                    CHECK (trigger_type IN ('automatic', 'manual', 'threshold')),
    trigger_config  JSONB       NOT NULL DEFAULT '{}',
    approval_type   TEXT        NOT NULL DEFAULT 'single'
                    CHECK (approval_type IN ('single', 'any_of', 'all_of')),
    timeout_minutes INTEGER     DEFAULT 60,
    timeout_action  TEXT        NOT NULL DEFAULT 'cancel'
                    CHECK (timeout_action IN ('cancel', 'approve', 'escalate')),
    enabled         BOOLEAN     NOT NULL DEFAULT true,
    sort_order      INTEGER     NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER gate_definitions_updated_at BEFORE UPDATE ON gate_definitions
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_gate_defs_ws_enabled ON gate_definitions (workspace_id, enabled);
CREATE UNIQUE INDEX idx_gate_defs_ws_id ON gate_definitions (workspace_id, id);

ALTER TABLE gate_definitions ENABLE ROW LEVEL SECURITY;
ALTER TABLE gate_definitions FORCE ROW LEVEL SECURITY;
CREATE POLICY workspace_isolation ON gate_definitions FOR ALL
    USING (workspace_id = current_workspace_id())
    WITH CHECK (workspace_id = current_workspace_id());

-- ============================================================
-- gates (composite FK to sessions)
-- ============================================================
CREATE TABLE gates (
    id                  UUID        PRIMARY KEY DEFAULT uuidv7(),
    session_id          UUID        NOT NULL,
    workspace_id        UUID        NOT NULL,
    gate_definition_id  UUID        REFERENCES gate_definitions(id) ON DELETE SET NULL,
    status              TEXT        NOT NULL DEFAULT 'pending'
                        CHECK (status IN ('pending', 'approved', 'denied', 'timed_out')),
    reason              TEXT        NOT NULL,
    context             JSONB       NOT NULL DEFAULT '{}',
    decided_by          TEXT,
    decided_at          TIMESTAMPTZ,
    decision_rationale  TEXT,
    expires_at          TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (workspace_id, session_id) REFERENCES sessions(workspace_id, id) ON DELETE CASCADE
);

CREATE TRIGGER gates_updated_at BEFORE UPDATE ON gates
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_gates_session_status ON gates (session_id, status);
CREATE INDEX idx_gates_pending_expiry ON gates (expires_at) WHERE status = 'pending';
CREATE INDEX idx_gates_ws ON gates (workspace_id);

ALTER TABLE gates ENABLE ROW LEVEL SECURITY;
ALTER TABLE gates FORCE ROW LEVEL SECURITY;
CREATE POLICY workspace_isolation ON gates FOR ALL
    USING (workspace_id = current_workspace_id())
    WITH CHECK (workspace_id = current_workspace_id());

-- ============================================================
-- events (append-only — composite FK to sessions, no updated_at)
-- ============================================================
CREATE TABLE events (
    id          UUID        PRIMARY KEY DEFAULT uuidv7(),
    session_id  UUID        NOT NULL,
    workspace_id UUID       NOT NULL,
    event_type  TEXT        NOT NULL,
    category    TEXT        NOT NULL
                CHECK (category IN ('lifecycle', 'action', 'gate', 'content', 'error', 'metric')),
    summary     TEXT        NOT NULL,
    data        JSONB       NOT NULL DEFAULT '{}',
    emitted_by  TEXT        NOT NULL DEFAULT 'agent'
                CHECK (emitted_by IN ('agent', 'system', 'human')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (workspace_id, session_id) REFERENCES sessions(workspace_id, id) ON DELETE CASCADE
);

CREATE INDEX idx_events_session_time ON events (session_id, created_at);
CREATE INDEX idx_events_session_type ON events (session_id, event_type);
CREATE INDEX idx_events_ws ON events (workspace_id);

ALTER TABLE events ENABLE ROW LEVEL SECURITY;
ALTER TABLE events FORCE ROW LEVEL SECURITY;
CREATE POLICY workspace_isolation ON events FOR ALL
    USING (workspace_id = current_workspace_id())
    WITH CHECK (workspace_id = current_workspace_id());
CREATE POLICY events_no_update ON events AS RESTRICTIVE FOR UPDATE USING (false);
CREATE POLICY events_no_delete ON events AS RESTRICTIVE FOR DELETE USING (false);

-- ============================================================
-- guardrails
-- ============================================================
CREATE TABLE guardrails (
    id          UUID        PRIMARY KEY DEFAULT uuidv7(),
    workspace_id UUID       NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name        TEXT        NOT NULL,
    description TEXT,
    category    TEXT        NOT NULL
                CHECK (category IN ('prohibition', 'requirement', 'preference', 'boundary')),
    scope       TEXT        NOT NULL DEFAULT 'workspace'
                CHECK (scope IN ('workspace', 'session')),
    enforcement TEXT        NOT NULL DEFAULT 'enforced'
                CHECK (enforcement IN ('enforced', 'advisory')),
    rule        JSONB       NOT NULL,
    enabled     BOOLEAN     NOT NULL DEFAULT true,
    version     INTEGER     NOT NULL DEFAULT 1,
    sort_order  INTEGER     NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER guardrails_updated_at BEFORE UPDATE ON guardrails
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_guardrails_ws_scope ON guardrails (workspace_id, scope, enabled);
CREATE UNIQUE INDEX idx_guardrails_ws_id ON guardrails (workspace_id, id);

ALTER TABLE guardrails ENABLE ROW LEVEL SECURITY;
ALTER TABLE guardrails FORCE ROW LEVEL SECURITY;
CREATE POLICY workspace_isolation ON guardrails FOR ALL
    USING (workspace_id = current_workspace_id())
    WITH CHECK (workspace_id = current_workspace_id());

-- ============================================================
-- session_guardrails (M:N join — composite FKs enforce same workspace)
-- ============================================================
CREATE TABLE session_guardrails (
    session_id   UUID NOT NULL,
    guardrail_id UUID NOT NULL,
    workspace_id UUID NOT NULL,
    PRIMARY KEY (session_id, guardrail_id),
    FOREIGN KEY (workspace_id, session_id) REFERENCES sessions(workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, guardrail_id) REFERENCES guardrails(workspace_id, id) ON DELETE CASCADE
);

CREATE INDEX idx_session_guardrails_ws ON session_guardrails (workspace_id);

ALTER TABLE session_guardrails ENABLE ROW LEVEL SECURITY;
ALTER TABLE session_guardrails FORCE ROW LEVEL SECURITY;
CREATE POLICY workspace_isolation ON session_guardrails FOR ALL
    USING (workspace_id = current_workspace_id())
    WITH CHECK (workspace_id = current_workspace_id());

-- ============================================================
-- Grant grove_app access to ACP tables
-- ============================================================
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO grove_app;
