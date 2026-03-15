-- Migration 003: Collaborative Documents (CRDT state storage)
-- Stores Yrs binary state for reconnect/resume during real-time co-editing.

CREATE TABLE collaborative_documents (
    id          UUID        PRIMARY KEY DEFAULT uuidv7(),
    workspace_id UUID       NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    entity_type TEXT        NOT NULL
                CHECK (entity_type IN ('specification', 'task', 'note', 'journey', 'step', 'persona')),
    entity_id   UUID        NOT NULL,
    field_name  TEXT        NOT NULL,
    crdt_state  BYTEA       NOT NULL,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (entity_type, entity_id, field_name)
);

CREATE TRIGGER collaborative_documents_updated_at
    BEFORE UPDATE ON collaborative_documents
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_collab_docs_entity ON collaborative_documents (entity_type, entity_id);
CREATE INDEX idx_collab_docs_workspace ON collaborative_documents (workspace_id);
