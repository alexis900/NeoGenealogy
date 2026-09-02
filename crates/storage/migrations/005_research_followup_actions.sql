-- Research Follow-up Actions — Fase 4.4
CREATE TABLE research_followup_actions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tree_id INTEGER NOT NULL REFERENCES trees(id) ON DELETE CASCADE,
    task_id INTEGER NOT NULL REFERENCES research_tasks(id) ON DELETE CASCADE,
    outcome_id INTEGER NOT NULL REFERENCES research_outcomes(id) ON DELETE CASCADE,
    followup_code TEXT NOT NULL CHECK(followup_code IN ('ADD_SUPPORTING_EVIDENCE','ADD_CITATION','REVIEW_CONTRADICTION','ADD_SECOND_SUPPORTING_EVIDENCE','REVIEW_SOURCE_COVERAGE')),
    status TEXT NOT NULL CHECK(status IN ('OPEN','COMPLETED','SKIPPED')),
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    completed_at TEXT
);

CREATE INDEX idx_followup_actions_tree ON research_followup_actions(tree_id);
CREATE INDEX idx_followup_actions_task ON research_followup_actions(task_id);
CREATE INDEX idx_followup_actions_outcome ON research_followup_actions(outcome_id);
CREATE INDEX idx_followup_actions_status ON research_followup_actions(status);
CREATE INDEX idx_followup_actions_code ON research_followup_actions(followup_code);
CREATE INDEX idx_followup_actions_updated ON research_followup_actions(updated_at);
