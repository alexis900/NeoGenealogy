-- NeoGenealogy initial schema
-- analysis_runs snapshots semantics, foreign keys ON, WAL friendly

CREATE TABLE trees (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source_filename TEXT,
    gedcom_version TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE persons (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tree_id INTEGER NOT NULL REFERENCES trees(id) ON DELETE CASCADE,
    gedcom_id TEXT NOT NULL,
    given_name TEXT,
    surname TEXT,
    display_name TEXT,
    sex TEXT,
    raw_name TEXT,
    birth_date_original TEXT,
    birth_date_precision TEXT,
    birth_date_year INTEGER,
    birth_date_start INTEGER,
    birth_date_end INTEGER,
    birth_place TEXT,
    death_date_original TEXT,
    death_date_precision TEXT,
    death_date_year INTEGER,
    death_place TEXT,
    occupation TEXT,
    raw_tags TEXT,
    UNIQUE(tree_id, gedcom_id)
);

CREATE TABLE families (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tree_id INTEGER NOT NULL REFERENCES trees(id) ON DELETE CASCADE,
    gedcom_id TEXT NOT NULL,
    raw_tags TEXT,
    UNIQUE(tree_id, gedcom_id)
);

CREATE TABLE family_members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    family_id INTEGER NOT NULL REFERENCES families(id) ON DELETE CASCADE,
    person_id INTEGER NOT NULL REFERENCES persons(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK(role IN ('husband','wife','child','other')),
    UNIQUE(family_id, person_id, role)
);

CREATE TABLE places (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tree_id INTEGER NOT NULL REFERENCES trees(id) ON DELETE CASCADE,
    raw_name TEXT NOT NULL,
    normalized_name TEXT,
    UNIQUE(tree_id, raw_name)
);

CREATE TABLE events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tree_id INTEGER NOT NULL REFERENCES trees(id) ON DELETE CASCADE,
    person_id INTEGER REFERENCES persons(id) ON DELETE SET NULL,
    family_id INTEGER REFERENCES families(id) ON DELETE SET NULL,
    event_type TEXT NOT NULL,
    date_original TEXT,
    date_precision TEXT,
    date_start INTEGER,
    date_end INTEGER,
    date_year INTEGER,
    place_id INTEGER REFERENCES places(id) ON DELETE SET NULL,
    place_raw TEXT,
    raw_value TEXT
);

CREATE TABLE sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tree_id INTEGER NOT NULL REFERENCES trees(id) ON DELETE CASCADE,
    gedcom_id TEXT NOT NULL,
    title TEXT,
    author TEXT,
    publication TEXT,
    text TEXT,
    repository TEXT,
    url TEXT,
    UNIQUE(tree_id, gedcom_id)
);

CREATE TABLE citations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tree_id INTEGER NOT NULL REFERENCES trees(id) ON DELETE CASCADE,
    source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    person_id INTEGER REFERENCES persons(id) ON DELETE SET NULL,
    family_id INTEGER REFERENCES families(id) ON DELETE SET NULL,
    event_id INTEGER REFERENCES events(id) ON DELETE SET NULL,
    page TEXT,
    text TEXT
);

CREATE TABLE analysis_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tree_id INTEGER NOT NULL REFERENCES trees(id) ON DELETE CASCADE,
    started_at TEXT NOT NULL,
    completed_at TEXT,
    engine_version TEXT,
    status TEXT NOT NULL CHECK(status IN ('running','completed','failed')),
    error_message TEXT
);

CREATE TABLE findings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tree_id INTEGER NOT NULL REFERENCES trees(id) ON DELETE CASCADE,
    analysis_run_id INTEGER REFERENCES analysis_runs(id) ON DELETE SET NULL,
    person_id INTEGER REFERENCES persons(id) ON DELETE SET NULL,
    family_id INTEGER REFERENCES families(id) ON DELETE SET NULL,
    related_person_id INTEGER REFERENCES persons(id) ON DELETE SET NULL,
    finding_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    confidence REAL,
    message TEXT,
    evidence TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE research_opportunities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tree_id INTEGER NOT NULL REFERENCES trees(id) ON DELETE CASCADE,
    analysis_run_id INTEGER REFERENCES analysis_runs(id) ON DELETE SET NULL,
    person_id INTEGER NOT NULL REFERENCES persons(id) ON DELETE CASCADE,
    finding_id INTEGER REFERENCES findings(id) ON DELETE SET NULL,
    priority TEXT,
    score INTEGER,
    confidence REAL,
    researchability TEXT,
    why TEXT,
    what TEXT,
    potential_sources TEXT,
    breakdown TEXT,
    missing_information TEXT,
    reasons TEXT
);

CREATE TABLE branch_analyses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tree_id INTEGER NOT NULL REFERENCES trees(id) ON DELETE CASCADE,
    analysis_run_id INTEGER NOT NULL REFERENCES analysis_runs(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    score INTEGER,
    opportunity_count INTEGER,
    high_priority_count INTEGER,
    deepest_generation INTEGER,
    source_coverage REAL
);

CREATE TABLE source_coverages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tree_id INTEGER NOT NULL REFERENCES trees(id) ON DELETE CASCADE,
    analysis_run_id INTEGER NOT NULL REFERENCES analysis_runs(id) ON DELETE CASCADE,
    birth REAL,
    marriage REAL,
    death REAL,
    other_events REAL,
    overall REAL
);

-- Indices
CREATE INDEX idx_persons_tree ON persons(tree_id);
CREATE INDEX idx_persons_tree_gedcom ON persons(tree_id, gedcom_id);
CREATE INDEX idx_families_tree_gedcom ON families(tree_id, gedcom_id);
CREATE INDEX idx_events_tree ON events(tree_id);
CREATE INDEX idx_events_person ON events(person_id);
CREATE INDEX idx_events_family ON events(family_id);
CREATE INDEX idx_sources_tree ON sources(tree_id);
CREATE INDEX idx_findings_tree ON findings(tree_id);
CREATE INDEX idx_findings_person ON findings(person_id);
CREATE INDEX idx_findings_severity ON findings(severity);
CREATE INDEX idx_opps_tree ON research_opportunities(tree_id);
CREATE INDEX idx_opps_score ON research_opportunities(score);
CREATE INDEX idx_opps_priority ON research_opportunities(priority);
CREATE INDEX idx_branch_tree_run ON branch_analyses(tree_id, analysis_run_id);
CREATE INDEX idx_coverage_tree_run ON source_coverages(tree_id, analysis_run_id);
