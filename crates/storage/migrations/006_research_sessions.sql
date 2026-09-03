-- Research Sessions — Fase 5.2
CREATE TABLE research_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tree_id INTEGER NOT NULL REFERENCES trees(id) ON DELETE CASCADE,
    title TEXT NOT NULL CHECK(length(trim(title)) > 0),
    description TEXT,
    status TEXT NOT NULL CHECK(status IN ('PLANNED','ACTIVE','COMPLETED','ABANDONED')),
    person_id INTEGER REFERENCES persons(id) ON DELETE SET NULL,
    opportunity_id INTEGER REFERENCES research_opportunities(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    started_at TEXT,
    completed_at TEXT
);

CREATE INDEX idx_research_sessions_tree ON research_sessions(tree_id);
CREATE INDEX idx_research_sessions_tree_status ON research_sessions(tree_id, status);
CREATE INDEX idx_research_sessions_person ON research_sessions(person_id);
CREATE INDEX idx_research_sessions_opportunity ON research_sessions(opportunity_id);
CREATE INDEX idx_research_sessions_updated ON research_sessions(updated_at);

ALTER TABLE research_tasks ADD COLUMN session_id INTEGER REFERENCES research_sessions(id) ON DELETE SET NULL;
CREATE INDEX idx_research_tasks_session ON research_tasks(session_id);
