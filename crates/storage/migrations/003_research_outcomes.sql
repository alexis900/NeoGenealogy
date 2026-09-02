-- Research Outcomes: resultado estructurado de una Research Task
CREATE TABLE research_outcomes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tree_id INTEGER NOT NULL REFERENCES trees(id) ON DELETE CASCADE,
    task_id INTEGER NOT NULL REFERENCES research_tasks(id) ON DELETE CASCADE,
    type TEXT NOT NULL CHECK(type IN ('CONFIRMED','FALSE_LEAD','INCONCLUSIVE','NEW_LEAD','NO_EVIDENCE')),
    summary TEXT NOT NULL,
    details TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(task_id)
);

CREATE INDEX idx_research_outcomes_tree ON research_outcomes(tree_id);
CREATE INDEX idx_research_outcomes_task ON research_outcomes(task_id);
CREATE INDEX idx_research_outcomes_type ON research_outcomes(type);
CREATE INDEX idx_research_outcomes_created ON research_outcomes(created_at);
