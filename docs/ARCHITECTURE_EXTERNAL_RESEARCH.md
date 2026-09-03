# Architecture External Research — Phase 5.4 Review

> **Phase 5.4 does not implement External Research.** This document is a formal architecture review that defines the boundary between the current core and a future external-research layer (Phase 6.0). No web search, no FamilySearch/Ancestry, no scraping, no AI, no OAuth, no external APIs are added in this phase.

## 1. Current Architecture

### Workflow (implemented)

```
GEDCOM → Parser (gedcom) → Analyzer (analyzer) → Scoring (scoring) → SQLite (storage) → Axum API (api) → React Web (web) → Docker
                                      │
                              AnalysisRun + Findings + Opportunities + Branch + Coverage
```

```
Research Opportunity
        ↓
Research Planning (planning_score deterministic, no persistence)
        ↓
Research Session (PLANNED/ACTIVE/COMPLETED/ABANDONED, persistent)
        ↓
Research Task (OPEN/IN_PROGRESS/RESOLVED/REJECTED/INCONCLUSIVE, persistent)
        ↓
Research Outcome (CONFIRMED/FALSE_LEAD/INCONCLUSIVE/NEW_LEAD/NO_EVIDENCE, UNIQUE(task_id))
        ↓
Evidence (SUPPORTS/CONTRADICTS via outcome_evidence)
        ↓
Evidence Assessment (score/status/reasons, derived)
        ↓
Evidence Gaps (CRITICAL/WARNING/INFO, derived)
        ↓
Research Follow-ups (HIGH/MEDIUM/LOW, derived)
        ↓
Follow-up Actions (OPEN/COMPLETED/SKIPPED, persistent)
        ↓
Research Case Summary (task+person+opportunity+outcome+assessment+gaps+followups+actions+timeline+warnings, derived)
        ↓
Research Session History + Statistics (COMPLETED/ABANDONED ordered COALESCE(completed_at,updated_at) DESC, derived stats+timeline, no persistence)
```

### Source model (implemented)

```
ResearchSource (bibliographic identity)
        ↓
ResearchCitation (locator inside source)
        ↓
Evidence (factual observation extracted)
        ↓ outcome_evidence (SUPPORTS|CONTRADICTS) ↓
Research Outcome
```

GEDCOM `sources`/`citations` (001_initial) are kept separate from research `research_sources`/`research_citations`/`evidence` (004) to avoid collision. `places`, `events`, `persons`, `families` are genealogy facts, never mutated by research except via explicit user edits (future rule).

### Layers

```
core (model) — no DB, no HTTP
gedcom / analyzer / scoring — pure, deterministic
storage (SQLite, WAL, FK=ON, migrations) — repositories, no HTTP
api (Axum) — DTOs, pagination, validation, maps StorageError → ApiError, no domain mutation
web (React 19, Vite, Tailwind, Router) — consumes GET/POST/PATCH/DELETE, no business logic
cli — import/analyze/report/serve
```

All layers pass tree isolation (`tree_id`), foreign keys, and batch queries (no N+1).

---

## 2. Source / Citation / Evidence Semantics

### ResearchSource — bibliographic/documentary identity

> What *is* the source? The dataset, book, archive, website, register as an entity.

Example:
```
Source:
  title: "Parish Register of Sant Martí d'Albars"
  type: PARISH_RECORD
  author: null
  publication: "Arxiu Parroquial, Book 4 (1879-1891)"
  date: "1879-1891"
```

`ResearchSource` does **not** contain the factual claim. It answers *what* you consulted. Current columns: `id, tree_id (FK CASCADE), title NOT NULL, author, publication, date, type CHECK(...), created_at, updated_at`. `tree_id` guarantees isolation.

### ResearchCitation — location inside the source

> Where *exactly* inside the source did you look?

Must support heterogeneous locators without assuming `page`:

```
page 42
volume 3, folio 12, entry 17
record ID 123456
https://example.org/record/123#section2
database record, chapter 2, etc.
```

Current columns: `id, source_id FK CASCADE, locator TEXT nullable, text TEXT nullable, created_at, updated_at`. `locator` is free-form; no format is required. `text` holds transcription excerpt. This design already satisfies heterogeneity. URL fragments belong here, not in a mandatory `page` field.

### Evidence — factual observation extracted

> What *observation* did you extract from that citation?

Example:
```
Evidence:
  statement: "The baptism register lists Josep García as son of Joan García and Maria Soler on 14 March 1882."
  source_id: 12 (Parish Register Sant Martí)
  citation_id: 45 (Book 4, page 127, entry 32)
  notes: "Handwriting unclear for mother surname"
```

Rules:
- Evidence is **not** truth, not source, not outcome.
- `Source → Citation → Evidence → (SUPPORTS|CONTRADICTS) → Outcome` is the only path.
- `evidence.tree_id` + `source_id` (+ optional `citation_id`) preserve provenance. Deleting a citation sets `evidence.citation_id = NULL` (ON DELETE SET NULL) without deleting the evidence statement; deleting a source cascades evidence (source is identity, cannot orphan evidence).
- Evidence remains understandable without live provider access (see §6).

Audit result: current model cleanly separates the three concepts. No structural change required in 5.4.

---

## 3. Candidate Result Concept

### Definition

```
External Search Result (from provider) ≠ Evidence
Candidate ≔ "I found something that might be relevant, not yet reviewed"
Evidence  ≔ "I extracted this factual observation from a concrete source/citation"
Outcome   ≔ "My research conclusion for this task is ..."
```

Future entity (not persisted yet) — `ResearchCandidate` or `ResearchResult`:

```
provider: "FamilySearch" | "Archive" | "GenericWeb"
external_id: string | null          // provider's stable id if available
title: string
description/snippet: string | null
url: string | null
record_type: string | null          // baptism, census, etc.
date / place: string | null
raw_metadata: JSON | null
```

### Persistence decision

**Candidate should be ephemeral (API response) in 6.0, not persisted**, unless 6.1 discovers a need for "saved searches". Rationale:
- Candidates are live external data, may change/vanish; persisting them as facts would conflate Candidate with Evidence.
- User review is the gate; only reviewed Candidates that become Evidence deserve persistence.
- Keeping Candidates ephemeral avoids snapshot/URL availability problems (§6) and keeps `core` provider-free.

If persistence is later required, create a separate `research_candidates` table **outside** the core genealogy model, with `tree_id`, `task_id` (optional), `provider`, `external_id`, `url`, `raw`, `created_at`, `expires_at`, and never FK to `evidence` automatically. This phase does **not** create it.

### Boundary

```
External Research → Candidate Results (ephemeral) → User Review → (if accepted) → Source/Citation → Evidence → Assessment → Outcome
```

Never:
```
External Search → CONFIRMED
External Search → Evidence (auto)
```

---

## 4. External Provider Boundary

### Isolation

```
Core Domain (core, scoring, analyzer, storage)          ← no provider imports
    ↑
Research Infrastructure (api handlers, DTOs)            ← maps provider errors → ApiError
    ↑
External Providers (future: FamilySearchProvider, …)    ← isolated, behind trait
```

Core must run without any provider. No `FamilySearch` string in `storage` or `scoring`. Providers are added as `crates/external` or `crates/providers/*` behind a trait:

```rust
#[async_trait]
trait ResearchProvider {
    fn name(&self) -> &str;
    async fn search(&self, query: ResearchQuery) -> Result<Vec<ResearchResult>, ProviderError>;
    // no fn create_evidence, no fn confirm_outcome
}
```

Provider output is `ResearchResult`, not `Evidence`/`Outcome`.

### Responsibilities (future)

| Provider | Creates | Does not create |
|----------|---------|-----------------|
| Provider | `ResearchResult {provider, external_id, title, url, snippet, metadata}` | Evidence, Outcome, Person, Family |
| User review | Source, Citation, Evidence (explicit POST) | — |
| Core | Assessment, Gaps, Follow-ups (derived) | — |

Rate limits, retries, timeouts, pagination belong to provider/infrastructure, never leak into `ResearchTask`/`Outcome`.

---

## 5. Evidence Provenance

Every Evidence must be traceable:

```
Evidence --citation_id--> ResearchCitation --source_id--> ResearchSource
   |
   +--tree_id--> Tree
   |
   +--outcome_evidence--> ResearchOutcome --task_id--> ResearchTask
```

And:
```
Source = identity (what dataset)
Citation = location (where inside it: locator + text)
Evidence = observation (what you read: statement)
Outcome = conclusion (what you conclude: CONFIRMED etc.)
```

Current FKs already enable this:
- `evidence(source_id FK CASCADE, citation_id FK SET NULL, tree_id FK CASCADE)`
- `outcome_evidence(outcome_id FK CASCADE, evidence_id FK CASCADE)`
- All tables carry `tree_id` (except `research_citations` via `source_id → tree_id` indirect, but evidence's `tree_id` is authoritative).

No separate provenance system needed. Future external provenance will reuse the same chain: when a Candidate is accepted, create `ResearchSource` (if not existing by `title`+`type`+`url`), then `ResearchCitation` (`locator` = record id / page / URL fragment), then `Evidence` (`statement` = user's transcription/observation). The live URL is **not** the provenance; the persisted `statement` is.

---

## 6. Snapshot vs Live Resources — URL Availability

### Problem

External URL `https://example.org/record/123` today shows "John García", tomorrow "John Garcia", next week 404 or login wall.

### Architecture decision

> **Evidence stores the researcher's observation, not a live view. A URL change or disappearance must not invalidate Evidence/Outcome/Case.**

Therefore:
- Scenario A (preferred for 6.0): persist `statement` (observation) + `source.url` (resource identifier, nullable) + `citation.locator` (record id / fragment) + `citation.text` (transcription). If URL dies, evidence remains.
- No web archiving in 5.4/6.0.
- Stale resources are **not** auto-corrected; user creates new Evidence if re-consulted.

Documented principle: *Current external resource ≠ what the researcher actually observed; the latter is what lives in `evidence.statement`.*

---

## 7. Failure Semantics

| Event | Core meaning | Must not become |
|-------|--------------|-----------------|
| `NO_RESULTS` | Provider returned empty candidate list | `NO_EVIDENCE` (which means "investigation produced no useful evidence") |
| `PROVIDER_UNAVAILABLE` / `RATE_LIMITED` / `AUTH_REQUIRED` / `INVALID_QUERY` / `TIMEOUT` | Technical failure | `NO_EVIDENCE` or `FALSE_LEAD` |

API mapping (future): provider errors → `429 RATE_LIMITED`, `502 PROVIDER_UNAVAILABLE`, `401 AUTH_REQUIRED`, not `200` with `NO_EVIDENCE`.

`NO_EVIDENCE` outcome type remains: "investigation produced no useful evidence" — a **research conclusion**, not a network error.

---

## 8. Authentication Boundary

- OAuth, API keys, sessions, cookies belong **only** to infrastructure (`crates/providers`, `api` config, env `PROVIDER_API_KEY` etc.).
- Never store credentials in `ResearchSource`, `ResearchCitation`, `Evidence`, `ResearchOutcome`, or any domain entity.
- No auth in 5.4; 6.0 will use env/secrets, `NEOGENEALOGY_*` prefix, not DB.

---

## 9. AI Boundary

AI may in the future assist with:

```
candidate ranking
text extraction / transcription
record interpretation
query formulation / normalization
```

AI must **never** be final authority for:

```
Evidence (statement requires user review)
Outcome (type requires user decision)
Person identity / Family relationship (genealogy fact)
Source existence
```

No AI in 5.4. Any future AI layer sits **beside** providers, outputs ranked Candidates, not auto-confirmed Evidence.

---

## 10. Future 6.0 Integration

### Target flow (proposed)

```
Research Opportunity
        ↓
Research Planning (planning_score)
        ↓
Research Session (PLANNED/ACTIVE)
        ↓
Research Task
        ↓
Research Query (from person/date/place/event/relationship/researchability, not auto-generated yet)
        ↓
External Provider(s) — parallel, isolated
        ↓
Merged Candidate Results (deduplicated by provider+external_id, fallback URL, not yet person matching)
        ↓
User selects result
        ↓
Source (create or reuse by title/type/url) → Citation (locator = record id / URL fragment, text = snippet) → Evidence (statement = user's observation, explicit POST)
        ↓
Evidence Assessment (still independent, still derived)
        ↓
Outcome (user chooses CONFIRMED etc., still independent)
```

Multiple providers merged without mixing results with Evidence. `Research Query → Multiple Providers → Merged Candidates` is a future capability, not required for first provider.

### Module placement

```
crates/core, analyzer, scoring, storage — unchanged
crates/api — adds /research/query, /research/candidates (future), maps ProviderError
crates/providers (new, optional) — trait ResearchProvider, familysearch, generic_web
web/src — adds Candidate review UI (future), no changes now except integration points documented in §12
```

---

## 11. Explicit Non-Goals (5.4)

This phase does **not**:

- call external APIs, search internet, scrape, browser-automate
- create providers, OAuth, API keys
- create `research_candidates` persistence (unless 6.0 proves essential)
- create External Research UI
- modify GEDCOM import, Research Score, Planning Score, Evidence Assessment
- add source reliability/trust score, truth score, person matching engine, automatic deduplication
- mutate `persons`/`families`/`events` automatically
- add AI, OCR, remote documents
- create analytics persistence or speculative abstractions (`GenericResearchEngine`, etc.)

---

## 12. Existing Storage Audit (crate `storage`)

### Entities (migrations)

| Table | PK | FK tree_id | Purpose |
|-------|----|------------|---------|
| `trees` | id | — | isolation root |
| `persons` | id | tree_id CASCADE, UNIQUE(tree_id,gedcom_id) | genealogy |
| `families` | id | tree_id CASCADE, UNIQUE(tree_id,gedcom_id) | genealogy |
| `family_members` | id | family_id CASCADE, person_id CASCADE | husband/wife/child/other |
| `places` | id | tree_id CASCADE, UNIQUE(raw_name) | normalized |
| `events` | id | tree_id CASCADE, person_id SET NULL, family_id SET NULL | date/place |
| `sources` (GEDCOM) | id | tree_id CASCADE, UNIQUE(gedcom_id) | raw GEDCOM SOUR |
| `citations` (GEDCOM) | id | tree_id CASCADE, source_id CASCADE | SOUR/PAGE under person/family/event |
| `analysis_runs` | id | tree_id CASCADE | snapshot |
| `findings` | id | tree_id CASCADE, analysis_run_id SET NULL | analyzer output |
| `research_opportunities` | id | tree_id CASCADE, analysis_run_id SET NULL | scoring |
| `branch_analyses` | id | tree_id CASCADE, analysis_run_id CASCADE | branch stats |
| `source_coverages` | id | tree_id CASCADE, analysis_run_id CASCADE | coverage % |
| `research_tasks` | id | tree_id CASCADE, opportunity_id SET NULL, person_id SET NULL, session_id SET NULL, UNIQUE(active opportunity) | human decision |
| `research_sessions` | id | tree_id CASCADE, person_id SET NULL, opportunity_id SET NULL, status CHECK | context |
| `research_outcomes` | id | tree_id CASCADE, task_id UNIQUE CASCADE, type CHECK | conclusion |
| `research_sources` | id | tree_id CASCADE | bibliographic identity |
| `research_citations` | id | source_id CASCADE | locator |
| `evidence` | id | tree_id CASCADE, source_id CASCADE, citation_id SET NULL | observation |
| `outcome_evidence` | (outcome_id,evidence_id) | both CASCADE | SUPPORTS/CONTRADICTS |
| `research_followup_actions` | id | tree_id CASCADE, task_id CASCADE, outcome_id CASCADE | actionable |

### Relations & Isolation

- Every user-generated row carries `tree_id` (except `research_citations` via `source_id`), all repositories filter by it. Tests: `tree_isolation` in `storage/tests/*` and `api/tests/*` assert cross-tree 404.
- `ON DELETE CASCADE` for `trees →` all domain tables; `research_outcomes.task_id` CASCADE; `research_sources.tree_id` CASCADE. `ON DELETE SET NULL` for `evidence.citation_id`, `research_tasks.opportunity_id/person_id/session_id`, `research_sessions.person_id/opportunity_id` — preserves tasks/evidence when linked entities deleted.
- `PRAGMA foreign_keys = ON` per connection (db.rs), `journal_mode=WAL`, `busy_timeout=5s`.

### Indices

```
persons(tree_id), (tree_id,gedcom_id); families(tree_id,gedcom_id); events(tree_id), (person_id), (family_id)
research_opportunities(tree_id), (score), (priority); branch_analyses(tree_id,analysis_run_id)
research_tasks(tree_id), (tree_id,status), (person_id), (opportunity_id), (session_id), unique_active(opportunity_id) WHERE status IN ('OPEN','IN_PROGRESS')
research_sessions(tree_id), (tree_id,status), (person_id), (opportunity_id), (updated_at); research_outcomes(tree_id), (task_id UNIQUE), (type)
research_sources(tree_id), (type); research_citations(source_id); evidence(tree_id),(source_id),(citation_id); outcome_evidence(outcome|evidence)
research_followup_actions(tree_id),(task_id),(outcome_id),(status),(followup_code),(updated_at)
```

Naming: `research_*` prefix avoids collision with GEDCOM `sources`/`citations`. Migration order 001_initial → 006_research_sessions, all versioned with `sqlx::migrate!`. No redundant indices observed.

### Verdict

No structural migration required for 5.4. Future 6.0 will add **one nullable column** `research_sources.url TEXT` (and possibly `provider TEXT`) — additive, no breaking change, indexed if needed. Citation heterogeneity already covered via free-form `locator`.

---

## 13. Existing API Audit (crate `api`)

- Domain objects (`Storage`, `ResearchOpportunity`, `ResearchTask`) do **not** depend on `axum`/`http`. Handlers (`handlers/*`) translate `Path/Query/Json` → `Storage` calls → `Paginated<Json<Value>>` and map `StorageError::NotFound → 404`, `Import → 400`, others → 500. Domain remains provider-free.
- Future providers can integrate by adding new handlers (`/research/query`, `/research/candidates`) without modifying existing handlers. `ProviderError` can map to `ApiError` with codes `PROVIDER_UNAVAILABLE`, `RATE_LIMITED`, etc., without touching `ResearchOutcome`.
- DTOs are plain `serde_json::Value`/`structs` mirroring storage rows plus derived fields (`evidence_assessment`, `timeline`). No storage model leakage of internal SQL.
- Pagination `limit 0..100, offset >=0` and validation (`INVALID_TREE_ID`, `INVALID_SESSION_STATUS` etc.) already isolate tree.

Verdict: API is ready; no refactor needed except adding new routes in 6.0.

---

## 14. Existing Frontend Audit (web/src)

- Centralized `api/client.ts` (`BASE = VITE_API_BASE_URL || ""`, `ApiError{code,status}`) with no scattered `fetch`. Types in `api/types.ts` mirror `docs/API.md`.
- Routing: `/trees/:treeId/research` (Workspace), `/planning`, `/opportunities`, `/sessions`, `/sessions/history`, `/sessions/:sessionId`, `/tasks`, `/tasks/:taskId`, `/history`, `/sources`, `/evidence`, etc. All carry `treeId` via URL, no global assumption.
- Components: `ResearchOpportunityCard`, `TaskStatusBadge`, `SessionStatusBadge`, `ScoreBreakdown`, `Pagination`, `Loading/Error/Empty` — reusable.
- Integration points for 6.0 without breakage:
  - `ResearchWorkspace` → add "External Research" card beside Planning (optional, not in 5.4)
  - `ResearchSessionDetail` → add "Search external" action that opens Candidate review (future), keeps current `Tasks/Outcomes/Evidence` blocks
  - `ResearchTaskDetail` already has `Research Session` block; future `Candidate → Evidence` button would live there
  - `ResearchHistory` vs `Session History` already separated — external history would be third, not colliding

No UI for external research created in 5.4. No frontend architecture rewrite needed.

---

## 15. Source Types — Current vs Proposed

### Current enum (004_evidence_sources.sql, enforced in storage/api/web)

```
BOOK, REGISTER, CENSUS, CIVIL_RECORD, PARISH_RECORD, NEWSPAPER, WEBSITE, OTHER
```

### Proposed minimum (Phase 5.4 spec)

```
BOOK, ARTICLE, REGISTRY, PARISH, CENSUS, CIVIL_REGISTRY, NEWSPAPER, WEBSITE, DATABASE, ARCHIVE, OTHER
```

### Analysis

- Overlap: BOOK, CENSUS, NEWSPAPER, WEBSITE, OTHER are identical.
- Differences: `REGISTER` vs `REGISTRY`, `PARISH_RECORD` vs `PARISH`, `CIVIL_RECORD` vs `CIVIL_REGISTRY`; proposed adds `ARTICLE`, `DATABASE`, `ARCHIVE`.
- Current `REGISTER` is generic; `PARISH_RECORD` and `CIVIL_RECORD` already cover the two most common registry subtypes.

### Decision (documented, no migration in 5.4)

- **Do not change the CHECK constraint arbitrarily in 5.4** — it would require a migration and break existing data/tests for no functional gain (external research not yet implemented).
- For future external sources, map provider concepts onto existing types:
  - `ARCHIVE` → `REGISTER` or `OTHER` with `publication` holding archive name
  - `DATABASE` → `WEBSITE` or `OTHER` with `publication` holding database name
  - `ARTICLE` → `BOOK` or `OTHER`
- In **6.0**, if a provider strictly needs the new literals, extend the CHECK in a single additive migration:

```sql
-- 007_external_sources.sql (future, not in 5.4)
ALTER TABLE research_sources RENAME TO research_sources_old;
CREATE TABLE research_sources (..., type TEXT NOT NULL CHECK(type IN ('BOOK','ARTICLE','REGISTER','REGISTRY','PARISH','PARISH_RECORD','CENSUS','CIVIL_RECORD','CIVIL_REGISTRY','NEWSPAPER','WEBSITE','DATABASE','ARCHIVE','OTHER')), ...);
INSERT INTO research_sources SELECT * FROM research_sources_old; DROP TABLE research_sources_old;
-- or, since SQLite CHECK is not enforced on rename, simply document and extend validation in Rust
```

Prefer **Rust-side validation extension** without DB CHECK change initially, to avoid migration. This is explicitly deferred.

---

## 16. URL Semantics — Decision

**Recommended rule:** *The URL identifies the resource; the Citation identifies the place within the resource.*

- `ResearchSource.url TEXT nullable` (future, 6.0) — base resource: `https://familysearch.org/ark:/61903/1:1:XXXX` or `https://archive.org/details/parish-register-sant-marti`
- `ResearchCitation.locator TEXT nullable` — specific record/page/entry/id/fragment: `page 127 / entry 32` or `recordID=123456` or `URL fragment #page=127`
- `ResearchCitation.text TEXT nullable` — transcription/snippet

If no `research_sources.url` exists yet, external URLs can temporarily live in `ResearchCitation.locator` (as `https://…#entry=32`) without breaking the model. Long term, add `research_sources.url` and keep `locator` for the fragment.

- GEDCOM `sources.url` already exists for raw imports, but research `research_sources` currently has no url column — this is the intentional gap to fill in 6.0 additively.

No URL column added in 5.4.

---

## 17. Citation Model — Heterogeneous Locators

`research_citations.locator` is nullable TEXT, free-form. No required format. Valid examples:

```
page 42
volume 3, folio 12
entry 17
Book 4, page 127, baptism entry 32
record ID 123456
https://example.org/record/123#section=2
database: archive_sant_marti / collection 1879-1891
```

`text` holds excerpt. This already satisfies external needs; no change.

---

## 18. Evidence Provenance — Recap

See §2 and §5. The chain `Evidence → Citation → Source` plus `Evidence → Outcome → Task → Session` gives full traceability. Tree isolation ensures `tree_id` never leaks. Future external evidence will follow the same chain after user review (see §6).

---

## 19. Provider Isolation & Output

Provider trait (future, not in 5.4) returns `ResearchResult`:

```rust
struct ResearchResult {
    provider: String,        // "FamilySearch"
    external_id: Option<String>,
    title: String,
    url: Option<String>,
    snippet: Option<String>,
    date: Option<String>,
    place: Option<String>,
    record_type: Option<String>,
    raw_metadata: Option<serde_json::Value>,
}
```

Never `Evidence`/`Outcome` directly. Deduplication: `provider + external_id` primary, `url` fallback. Not person deduplication.

---

## 20. Search Query Abstraction (Future)

Opportunity → Query would need:

```
person {given_name, surname, birth_date, death_date, birth_place}
finding type / event
relationship
researchability, confidence, priority
potential_sources (already in opportunity)
```

Example:

```
Opportunity: "Find parents of Josep García (b. 1882, Sant Martí)"
Potential query: "\"Josep García\" + baptism + Sant Martí + 1882"
```

No auto-generation in 5.4; documented as future helper that produces `ResearchQuery {terms, date_range, place, record_type}` without calling providers.

---

## 21. Authentication, Rate Limiting, Failure

- Auth (OAuth, API keys) → infrastructure env, never domain.
- Rate limits, retries, timeouts, provider errors → provider layer, never contaminate `ResearchOutcome`/`Evidence`.
- Failure codes: `NO_RESULTS` (empty candidate list) vs `PROVIDER_UNAVAILABLE`/`RATE_LIMITED`/`AUTH_REQUIRED`/`INVALID_QUERY` — mapped to HTTP 502/429/401, never to `NO_EVIDENCE` outcome type.

`NO_EVIDENCE` stays a research conclusion, not a network error.

---

## 22. Architecture Rules — Non-Negotiable

### Rule 1 – External result ≠ Evidence
A candidate becomes Evidence only after explicit user review and `POST /evidence`.

### Rule 2 – Evidence ≠ Outcome
Evidence is observation; Outcome is conclusion. One outcome may cite multiple evidence items with `SUPPORTS`/`CONTRADICTS`.

### Rule 3 – Outcome ≠ truth probability
`CONFIRMED` means "researcher recorded this conclusion", not "external source proved truth".

### Rule 4 – Provider failure ≠ NO_EVIDENCE
Technical failure maps to `PROVIDER_UNAVAILABLE` etc., never to `NO_EVIDENCE` type.

### Rule 5 – External Research never mutates the genealogy automatically
No auto-update of `persons`/`families`/`events`. Only user creates Evidence/Outcome; genealogy edits remain manual.

### Rule 6 – Evidence must remain understandable without live provider
`statement` + `source`/`citation` persists even if URL dies.

### Rule 7 – Core domain must work without external providers
All current workflows function with providers disabled.

### Rule 8 – Credentials never belong in domain entities
Env/secrets only, never in `research_sources` or `evidence`.

### Rule 9 – Research Score and Evidence Assessment remain independent
External Research alters neither `Research Score` (priority/researchability of Opportunity) nor `Evidence Assessment` (structural support for Outcome). Assessment is derived from `Evidence` only.

### Rule 10 – User remains final authority for converting Candidate → Evidence
No auto-confirmation; user selects Candidate → creates Source/Citation/Evidence.

---

## 23. Diagram — Current + Future Boundary

```
Opportunity
     ↓
Planning (planning_score)
     ↓
Session (PLANNED/ACTIVE/COMPLETED/ABANDONED)
     ↓
Task (OPEN/IN_PROGRESS/RESOLVED/REJECTED/INCONCLUSIVE)
     ↓
External Research  ──→  Candidate Results (ephemeral, provider+external_id, url)
     ↓                         │
User Review  ←─────────────────┘
     ↓
Evidence (statement) ← Citation (locator) ← Source (identity, future url)
     ↓
Evidence Assessment (score/status/reasons, derived, independent)
     ↓
Outcome (CONFIRMED/FALSE_LEAD/INCONCLUSIVE/NEW_LEAD/NO_EVIDENCE, user conclusion)
     ↓
Case Summary / Session History + Statistics (derived)
```

```
                 NEO GENEALOGY CORE
┌──────────────────────────────────────────────┐
│ Opportunity → Planning → Session → Task      │
│                                  ↓           │
│                               Outcome        │
│                                  ↑           │
│ Evidence ← Citation ← Source                 │
└───────────────────────┬──────────────────────┘
                        │ USER REVIEW BOUNDARY
┌───────────────────────▼──────────────────────┐
│          EXTERNAL RESEARCH (future)          │
│ Query → Provider → Candidate Results         │
│                         └──→ User Review     │
└──────────────────────────────────┼───────────┘
                                   ↓
                                Evidence
```

> **Finding something on the internet does not create evidence. Finding something creates an opportunity for the researcher to decide whether evidence exists.**

---

## 24. Phase 6.0 Proposal

### 6.0 External Research Core
- Add `research_sources.url TEXT nullable` (and `provider TEXT nullable`) additively, no breaking change
- Define `ResearchQuery` struct and `ResearchProvider` trait (isolated crate, no implementation)
- Add `ResearchResult` normalisation struct (provider, external_id, title, url, snippet, date, place, record_type, raw_metadata)
- Document query generation from Opportunity (person/date/place) without implementing it

### 6.1 Provider Abstraction
- Implement `trait ResearchProvider { fn name, async fn search }`, error type `ProviderError {NO_RESULTS, PROVIDER_UNAVAILABLE, RATE_LIMITED, AUTH_REQUIRED, INVALID_QUERY}`
- Provider isolation: no dependency from `storage`/`scoring`; providers live in `crates/providers`

### 6.2 First Provider
- Implement `GenericWebProvider` (or `ArchiveProvider`) returning `ResearchResult[]`, with rate limiting, retries, timeouts in infrastructure
- No FamilySearch/Ancestry specific logic yet; keep generic

### 6.3 Candidate Review
- API: `POST /trees/:treeId/research/queries` → `GET /trees/:treeId/research/candidates?query=` (ephemeral, merged, deduplicated by provider+external_id → URL fallback)
- Web: Candidate list UI (title, snippet, url, provider) with no auto-Evidence; pagination, empty/loading/error states

### 6.4 Candidate → Evidence
- Web: `Select result → Create Source/Citation → Create Evidence` explicit flow: `POST /research/sources` (reuse by title/url), `POST /research/citations`, `POST /evidence`, `POST /outcome_evidence`
- Evidence provenance preserved; `Research Score` / `Evidence Assessment` unchanged; manual Outcome selection

No AI, no auto-person matching, no reliability score in 6.0.

---

## 25. Verification (5.4)

This phase introduces **no functional change**; verification confirms no regression:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
npm --prefix web run build
npm --prefix web run test
sg docker -c "docker compose build"
sg docker -c "docker compose up -d"
curl http://127.0.0.1:3000/health
curl http://127.0.0.1:3000/ready
curl http://127.0.0.1:3000/api/v1/openapi.json
```

E2E existing (Planning → Session → Task → Outcome → Evidence) must continue to pass; new doc does not add migrations.

---

## 26. References

- `docs/RESEARCH_SESSIONS.md` — Session lifecycle
- `docs/RESEARCH_SESSIONS_HISTORY.md` — History & Statistics (derived, no persistence)
- `docs/EVIDENCE_SOURCES.md` — Source/Citation/Evidence model
- `docs/EVIDENCE_ASSESSMENT.md`, `docs/EVIDENCE_GAPS.md`, `docs/RESEARCH_FOLLOWUPS.md`, `docs/RESEARCH_CASE_SUMMARY.md`, `docs/RESEARCH_PLANNING.md`
- `docs/API.md`, `docs/STORAGE.md`, `docs/WEB.md`
- `crates/storage/migrations/001_initial.sql` … `006_research_sessions.sql`
- `crates/api/src/handlers/*`, `web/src/api/*`

