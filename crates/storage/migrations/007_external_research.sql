-- External Research Core — Phase 6.0
-- Research Query → Execution → Results (candidate boundary, never Evidence)

CREATE TABLE research_queries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tree_id INTEGER NOT NULL REFERENCES trees(id) ON DELETE CASCADE,
    task_id INTEGER NOT NULL REFERENCES research_tasks(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    query TEXT NOT NULL CHECK(length(trim(query)) > 0),
    status TEXT NOT NULL CHECK(status IN ('PENDING','RUNNING','COMPLETED','FAILED')),
    created_at TEXT NOT NULL,
    started_at TEXT,
    completed_at TEXT,
    error_code TEXT,
    error_message TEXT
);

CREATE TABLE research_query_executions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    query_id INTEGER NOT NULL REFERENCES research_queries(id) ON DELETE CASCADE,
    status TEXT NOT NULL CHECK(status IN ('PENDING','RUNNING','COMPLETED','FAILED')),
    started_at TEXT,
    completed_at TEXT,
    error_code TEXT,
    error_message TEXT,
    provider_request_id TEXT,
    provider_metadata TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE research_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    execution_id INTEGER NOT NULL REFERENCES research_query_executions(id) ON DELETE CASCADE,
    query_id INTEGER NOT NULL REFERENCES research_queries(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    external_id TEXT,
    title TEXT NOT NULL,
    description TEXT,
    url TEXT,
    record_type TEXT,
    date TEXT,
    place TEXT,
    metadata TEXT,
    position INTEGER NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_research_queries_tree ON research_queries(tree_id);
CREATE INDEX idx_research_queries_task ON research_queries(task_id);
CREATE INDEX idx_research_queries_provider ON research_queries(provider);
CREATE INDEX idx_research_queries_status ON research_queries(status);

CREATE INDEX idx_research_query_executions_query ON research_query_executions(query_id);
CREATE INDEX idx_research_query_executions_status ON research_query_executions(status);
CREATE INDEX idx_research_query_executions_created ON research_query_executions(created_at);

CREATE INDEX idx_research_results_execution ON research_results(execution_id);
CREATE INDEX idx_research_results_query ON research_results(query_id);
CREATE INDEX idx_research_results_provider ON research_results(provider);
CREATE INDEX idx_research_results_position ON research_results(execution_id, position);
