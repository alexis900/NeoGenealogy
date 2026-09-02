-- Research Workflow: ResearchTask persistent, independent from ResearchOpportunity
CREATE TABLE research_tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tree_id INTEGER NOT NULL REFERENCES trees(id) ON DELETE CASCADE,
    opportunity_id INTEGER REFERENCES research_opportunities(id) ON DELETE SET NULL,
    person_id INTEGER REFERENCES persons(id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL CHECK(status IN ('OPEN','IN_PROGRESS','RESOLVED','REJECTED','INCONCLUSIVE')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    started_at TEXT,
    completed_at TEXT,
    resolution TEXT
);

CREATE INDEX idx_research_tasks_tree ON research_tasks(tree_id);
CREATE INDEX idx_research_tasks_tree_status ON research_tasks(tree_id, status);
CREATE INDEX idx_research_tasks_person ON research_tasks(person_id);
CREATE INDEX idx_research_tasks_opportunity ON research_tasks(opportunity_id);
-- Prevent duplicate active tasks for same opportunity (optional, allow reopen after resolved)
CREATE UNIQUE INDEX idx_research_tasks_unique_active ON research_tasks(opportunity_id, status) WHERE opportunity_id IS NOT NULL AND status IN ('OPEN','IN_PROGRESS');
