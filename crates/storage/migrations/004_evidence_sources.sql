-- Evidence & Sources — Fase 4.0
-- Nota: tablas `sources` y `citations` ya existen para GEDCOM (001_initial).
-- Para no colisionar, las nuevas tablas de investigación se crean con prefijo `research_`
-- y `evidence`/`outcome_evidence` mantienen los nombres del spec.
-- Mapping: spec `sources` -> `research_sources`, spec `citations` -> `research_citations`

CREATE TABLE research_sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tree_id INTEGER NOT NULL REFERENCES trees(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    author TEXT,
    publication TEXT,
    date TEXT,
    type TEXT NOT NULL CHECK(type IN ('BOOK','REGISTER','CENSUS','CIVIL_RECORD','PARISH_RECORD','NEWSPAPER','WEBSITE','OTHER')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_research_sources_tree ON research_sources(tree_id);
CREATE INDEX idx_research_sources_type ON research_sources(type);

CREATE TABLE research_citations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id INTEGER NOT NULL REFERENCES research_sources(id) ON DELETE CASCADE,
    locator TEXT,
    text TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_research_citations_source ON research_citations(source_id);

CREATE TABLE evidence (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tree_id INTEGER NOT NULL REFERENCES trees(id) ON DELETE CASCADE,
    source_id INTEGER NOT NULL REFERENCES research_sources(id) ON DELETE CASCADE,
    citation_id INTEGER REFERENCES research_citations(id) ON DELETE SET NULL,
    statement TEXT NOT NULL,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_evidence_tree ON evidence(tree_id);
CREATE INDEX idx_evidence_source ON evidence(source_id);
CREATE INDEX idx_evidence_citation ON evidence(citation_id);

CREATE TABLE outcome_evidence (
    outcome_id INTEGER NOT NULL REFERENCES research_outcomes(id) ON DELETE CASCADE,
    evidence_id INTEGER NOT NULL REFERENCES evidence(id) ON DELETE CASCADE,
    relationship TEXT NOT NULL CHECK(relationship IN ('SUPPORTS','CONTRADICTS')),
    PRIMARY KEY (outcome_id, evidence_id)
);

CREATE INDEX idx_outcome_evidence_outcome ON outcome_evidence(outcome_id);
CREATE INDEX idx_outcome_evidence_evidence ON outcome_evidence(evidence_id);
