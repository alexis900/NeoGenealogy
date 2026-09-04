-- FamilySearch OAuth — Phase 6.1 interactive login
-- Single-tenant connection (id=1) + CSRF state store

CREATE TABLE familysearch_connections (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    access_token TEXT NOT NULL,
    token_type TEXT,
    expires_at TEXT,
    scope TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE familysearch_oauth_states (
    state TEXT PRIMARY KEY,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_familysearch_oauth_states_created ON familysearch_oauth_states(created_at);
