# Storage — SQLite Persistence

## Arquitectura

```
GEDCOM → Parser → Model → Analyzer → Scoring → Storage (SQLite)
```

`crates/storage` es una capa de almacenamiento aislada. El dominio (`core`, `gedcom`, `analyzer`, `scoring`) no conoce SQLite; `cli` depende de `storage` solo para comandos con `--db`.

Traits `PersonRepository`, `FamilyRepository` existen como `Storage` concreto (`repositories.rs`) para desacoplar y permitir sustituir SQLite por PostgreSQL sin reescribir el dominio (se cambian `sqlx` pool options).

## SQLite

- **Runtime:** `sqlx 0.8` + `tokio`
- **Archivo:** por defecto `neogenealogy.db` en cwd, configurable vía `--db /path.db` (no hardcodeado)
- **URL:** `sqlite://path` con `create_if_missing(true)`
- **Pragmas (`db.rs`):**
  - `journal_mode = WAL` — concurrencia lectura/escritura, recomendado para app local
  - `busy_timeout = 5s`
  - `foreign_keys = ON` en cada conexión (obligatorio para integridad)
- **Migraciones:** `crates/storage/migrations/001_initial.sql` versionadas con `sqlx::migrate!`
  ```bash
  sqlx migrate run   # o automáticamente en CLI: run_migrations(&pool).await
  ```
  Para base vacía:
  ```bash
  rm -f neogenealogy.db
  cargo run -p neogenealogy -- import test-data/complex.ged --db neogenealogy.db
  ```

## Esquema

Tablas principales (con `tree_id` como límite de aislamiento):

- `trees(id, name, source_filename, gedcom_version, created_at, updated_at)`
- `persons(id, tree_id, gedcom_id, given_name, surname, display_name, sex, raw_name, birth_date_original, birth_date_precision, birth_date_year, birth_date_start/end, birth_place, death_..., raw_tags JSON, UNIQUE(tree_id, gedcom_id))`
- `families(id, tree_id, gedcom_id, raw_tags, UNIQUE(tree_id, gedcom_id))`
- `family_members(id, family_id, person_id, role CHECK('husband','wife','child','other'), UNIQUE(family_id, person_id, role))`
- `places(id, tree_id, raw_name, normalized_name, UNIQUE(tree_id, raw_name))`
- `events(id, tree_id, person_id, family_id, event_type, date_original, date_precision, date_start/end/year, place_id, place_raw, raw_value)`
- `sources(id, tree_id, gedcom_id, title, author, publication, text, repository, url, UNIQUE(tree_id, gedcom_id))`
- `citations(id, tree_id, source_id, person_id, family_id, event_id, page, text)` — **cita ≠ fuente**
- `analysis_runs(id, tree_id, started_at, completed_at, engine_version, status CHECK('running','completed','failed'), error_message)`
- `findings(id, tree_id, analysis_run_id, person_id, family_id, related_person_id, finding_type, severity, confidence, message, evidence JSON, created_at)`
- `research_opportunities(id, tree_id, analysis_run_id, person_id, priority, score, confidence, researchability, why, what JSON, potential_sources JSON, breakdown JSON, missing_information JSON, reasons JSON)`
- `branch_analyses(id, tree_id, analysis_run_id, name, score, opportunity_count, high_priority_count, deepest_generation, source_coverage)`
- `source_coverages(id, tree_id, analysis_run_id, birth, marriage, death, other_events, overall)`
- `research_tasks(id, tree_id, opportunity_id FK SET NULL, person_id FK SET NULL, title, description, status CHECK, created_at, updated_at, started_at, completed_at, resolution, UNIQUE(active opportunity))`
- `research_outcomes(id, tree_id FK CASCADE, task_id UNIQUE FK CASCADE, type CHECK(CONFIRMED…), summary, details, created_at, updated_at)` — ver `docs/RESEARCH_OUTCOMES.md`
- `research_sources(id, tree_id FK CASCADE, title, author, publication, date, type CHECK(BOOK…OTHER), created_at, updated_at)` — ver `docs/EVIDENCE_SOURCES.md` (tabla `sources` GEDCOM permanece en `gedcom_sources`)
- `research_citations(id, source_id FK CASCADE, locator, text, created_at, updated_at)`
- `evidence(id, tree_id FK CASCADE, source_id FK CASCADE, citation_id FK SET NULL, statement, notes, created_at, updated_at)`
- `outcome_evidence(outcome_id FK CASCADE, evidence_id FK CASCADE, relationship CHECK(SUPPORTS|CONTRADICTS), PK(outcome_id,evidence_id))`

### Foreign keys

`ON DELETE CASCADE` para `trees → persons/families/events/sources/findings/opportunities/branch_analyses/source_coverages/research_tasks/research_outcomes/research_sources/evidence/outcome_evidence` — borrar árbol borra todo.  
`ON DELETE CASCADE` para `research_outcomes.task_id` y `research_outcomes.tree_id` — borrar Task borra su Outcome.  
`ON DELETE CASCADE` para `research_sources.tree_id`, `evidence.tree_id/source_id`, `research_citations.source_id`, `outcome_evidence` ambos FK.
`ON DELETE SET NULL` para `evidence.citation_id`, `research_tasks.opportunity_id/person_id`, `events.person_id`, `findings.person_id` etc. — permite conservar tasks/evidence aunque se borre cita/oportunidad/persona.  
`ON DELETE CASCADE` para `citations.source_id` (GEDCOM), `family_members` — consistencia.

Activadas con `PRAGMA foreign_keys = ON`.

### Índices

```sql
persons(tree_id)
persons(tree_id, gedcom_id)
families(tree_id, gedcom_id)
events(tree_id) events(person_id) events(family_id)
sources(tree_id) -- GEDCOM
findings(tree_id) findings(person_id) findings(severity)
research_opportunities(tree_id) (score) (priority)
branch_analyses(tree_id, analysis_run_id)
source_coverages(tree_id, analysis_run_id)
research_tasks(tree_id) (tree_id,status) (person_id) (opportunity_id) unique_active(opportunity_id,status) WHERE status IN ('OPEN','IN_PROGRESS')
research_outcomes(tree_id) (task_id UNIQUE) (type) (created_at)
research_sources(tree_id) (type)
research_citations(source_id)
evidence(tree_id) (source_id) (citation_id)
outcome_evidence(outcome_id) (evidence_id)
```

### Raw tags

`persons.raw_tags` y `families.raw_tags` guardan `RawTag` como JSON (`[{"level":1,"tag":"_CUSTOM","value":"..."}]`) para no perder información desconocida (principio de no destrucción).

## Analysis Run / Snapshot

Importación genera un `analysis_runs` con `engine_version = CARGO_PKG_VERSION`:

```
running → completed|failed
```

Todos los `findings`, `research_opportunities`, `branch_analyses`, `source_coverages` apuntan a `analysis_run_id`. Permite comparar snapshots futuros.

## Importación atómica

`import::import_gedcom_file` (`import.rs`):

1. Parse GEDCOM (sin DB)
2. `analyze` + `opportunities` + `branch_analyses` + `source_coverage` (en memoria)
3. `BEGIN`
4. Insert `trees` → `analysis_runs('running')` → `places` → `persons` → `families` → `family_members` → `events` → `sources` → `citations` → `findings` → `research_opportunities` → `branch_analyses` → `source_coverages` → `UPDATE analysis_runs SET status='completed'`
5. `COMMIT` — si falla, `ROLLBACK`, no queda medio árbol.

Idempotencia: `UNIQUE(tree_id, gedcom_id)` para `persons`/`families`/`sources`; cada importación crea nuevo `tree`, no duplica dentro del mismo árbol.

## Repositories

`Storage { pool: SqlitePool }` expone:

```
get_tree, list_trees, get_person, list_persons(limit/offset), get_family, list_families,
get_findings, get_research_opportunities, get_top_research_opportunities(limit, priority>=High),
get_branches, get_source_coverage, get_analysis_runs, count(tree_id),
create_research_task, get_research_task, list_research_tasks, update_research_task, delete_research_task,
create_research_outcome, get_research_outcome, get_research_outcome_by_task, list_research_outcomes, list_research_outcomes_with_person, update_research_outcome, delete_research_outcome,
create_research_source, get_research_source, list_research_sources, update_research_source, delete_research_source,
create_research_citation, get_research_citation, list_research_citations, update_research_citation, delete_research_citation,
create_evidence, get_evidence, list_evidence, update_evidence, delete_evidence,
 attach_evidence_to_outcome, detach_evidence_from_outcome, list_outcome_evidence, list_outcome_evidence_detailed,
 get_outcome_evidence_stats, get_outcomes_evidence_stats, get_outcome_assessment, get_outcomes_assessments (batch sin N+1, `EvidenceStats` → `calculate_evidence_assessment`),
 research_summary (con sources/evidence counts + assessment {no_evidence,weak,mixed,supported,strongly_supported})
```

Paginación: `limit/offset` en listados para preparar futura API HTTP sin rediseñar acceso a datos.

CLI no conoce SQL.

## Consultas mínimas / Research Queue

```sql
SELECT * FROM research_opportunities WHERE tree_id=?1 ORDER BY score DESC LIMIT ?2;
-- con filtro priority>=High:
SELECT * FROM research_opportunities WHERE tree_id=?1 AND priority IN ('high','critical') ORDER BY score DESC
-- con confidence sort en Rust (o SQL ORDER BY confidence)
```

## Error handling

`StorageError::{Database, Migration, Import, NotFound, Io}` sin `unwrap/expect` en paths de runtime (`import.rs`, `db.rs`, `repositories.rs`).

## Cómo abrir la DB

```bash
sqlite3 neogenealogy.db "SELECT * FROM trees;"
sqlite3 neogenealogy.db ".schema"
```

## Benchmark persistence

Baseline con generador `benchmarks/generate.rs` (ver `benchmarks/README.md`):

| n | parse | analysis | sqlite persist | total |
|---|---|---|---|---|
| 1 000 | ~0.02s | ~0.06s | ~0.08s | ~0.16s |
| 5 000 | ~0.08s | ~0.24s | ~0.38s | ~0.70s |
|10 000 | ~0.15s | ~0.46s | ~0.72s | ~1.33s |

Medido en `import_gedcom_file` con `WAL`; persistencia domina por inserts + índices.

## Limitaciones conocidas

- `citations.page` aún placeholder (extracción de `2 PAGE` bajo `1 SOUR` pendiente refinamiento)
- `events` no distingue aún `description` vs `raw_value` para etiquetas desconocidas
- `analysis_runs` single-per-tree; comparar dos runs requiere query adicional
- Sin pool PostgreSQL aún (cambio requiere solo `SqlitePool` → `PgPool`)

Preparado para Fase 3 (API Axum) sin cambios en dominio.
