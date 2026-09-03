# NeoGenealogy v0.5.3 — Research Sessions History & Statistics

Herramienta local de análisis genealógico. Release v0.5.3 añade **Research Sessions History & Statistics** (100% derivado, sin persistencia): `/research/sessions/history` con filtros `status/person_id` y stats batch (`tasks/outcomes/evidence/followups`), `Session Detail` con `Session Summary` + `Research Activity` + `timeline` derivada 20 DESC, `Overview` con bloque `Research Activity` y `GET /research/summary` ampliado con `sessions` + `research_activity`. 0.5.2 aportó Sessions `PLANNED/ACTIVE/COMPLETED/ABANDONED` (`Opportunity → Planning → Session → Task(s) → Outcome`); 5.1 Planning UI `What should I research next?`, 5.0 motor determinista.

```
WHAT THE SYSTEM FOUND        → Research Opportunity (auto)
WHAT TO RESEARCH NEXT        → Research Planning (planning_score + reasons, Recommended/Deferred)
WHAT THE USER DECIDED TO INVESTIGATE → Research Task (OPEN → RESOLVED/…)
WHAT THE USER DISCOVERED     → Research Outcome (CONFIRMED/…)
WHAT SUPPORTS THAT CONCLUSION → Evidence (SUPPORTS/CONTRADICTS) → Source + Citation
HOW WELL IT IS SUPPORTED     → Evidence Assessment (score/status/reasons)
WHAT IS MISSING              → Evidence Gaps (CRITICAL/WARNING/INFO)
WHAT TO DO NEXT (GENERIC)    → Research Follow-ups (HIGH/MEDIUM/LOW)
WHAT WAS DONE                → Follow-up Actions (OPEN/COMPLETED/SKIPPED)
HOW IT ENDED                 → Research Case Summary (timeline + closure warnings)
```

```
GEDCOM → Parser → Analyzer → Scoring → SQLite → Axum API → React Web → Docker
```

## Uso (sin DB — análisis directo)

```bash
cargo run -p neogenealogy -- analyze test-data/simple.ged --output analysis.json
cargo run -p neogenealogy -- analyze test-data/complex.ged --explain-score
cargo run -p neogenealogy -- analyze test-data/complex.ged --severity high
cargo run -p neogenealogy -- analyze test-data/complex.ged --severity high --sort confidence
cargo run -p neogenealogy -- stats test-data/simple.ged
cargo run -p neogenealogy -- report test-data/simple.ged --output report.html
```

## Uso (con SQLite)

```bash
# Importar y persistir (transacción atómica, WAL, foreign_keys=ON)
cargo run -p neogenealogy -- import test-data/complex.ged --db neogenealogy.db
cargo run -p neogenealogy -- import test-data/complex.ged --db /tmp/neogenealogy.db

# Consultar desde DB
cargo run -p neogenealogy -- stats --db neogenealogy.db
cargo run -p neogenealogy -- report --db neogenealogy.db --output report.html

# Migraciones (automáticas en CLI) o manual:
# sqlx migrate run  # requiere DATABASE_URL=sqlite://neogenealogy.db
sqlite3 neogenealogy.db ".tables"
```

El importador mantiene etiquetas no reconocidas en `Person.raw`/`Family.raw` → `persons.raw_tags` JSON; `SOUR` vs `citación` se distingue (`sources` vs `citations`). `GedcomParser` es reemplazable.

## Capas

- `core` — modelo genealógico
- `gedcom` — parser conservador
- `analyzer` — reglas (cronología, ciclos, duplicados, gaps)
- `scoring` — Research Score 0–100 explicable
 - `storage` — SQLite, WAL, `analysis_runs` + `research_tasks` + `research_outcomes` (003) + `research_sources`/`research_citations`/`evidence`/`outcome_evidence` (004) + `research_followup_actions` (005) + `Research Sessions` (006, `PLANNED/ACTIVE/COMPLETED/ABANDONED`, `session_id`) + `Research Case Summary` / `Planning` / `Session History & Stats` (todo derivado, sin nuevas tablas, `GROUP BY` batch)
 - `api` — Axum REST `/api/v1` (paginación, filtros, Tasks/Outcomes, Sources/Citations/Evidence, Follow-ups/Actions, Sessions, Session History, Case Summary, Planning, Research Summary con `sessions`+`research_activity`, OpenAPI)
 - `cli` — `analyze / import / stats / report / serve` (`--db`, `--host`, `--port`)
 - `web/` — React 19 + Vite + Tailwind + React Router (Workspace + Planning + Sessions + Session History + Tasks + Outcome + Evidence + Case Summary)

 Véase `docs/RESEARCH_SESSIONS_HISTORY.md`, `docs/RESEARCH_PLANNING.md`, `docs/RESEARCH_CASE_SUMMARY.md`, `docs/RESEARCH_FOLLOWUP_ACTIONS.md`, `docs/RESEARCH_FOLLOWUPS.md`, `docs/EVIDENCE_GAPS.md`, `docs/EVIDENCE_ASSESSMENT.md`, `docs/EVIDENCE_SOURCES.md`, `docs/RESEARCH_OUTCOMES.md`, `docs/RESEARCH_WORKFLOW.md`, `docs/API.md`, `docs/STORAGE.md`, `docs/WEB.md`.

## Web UI

```bash
cargo run -p neogenealogy -- import test-data/complex.ged --db /tmp/neogenealogy.db
cargo run -p neogenealogy -- serve --db /tmp/neogenealogy.db --host 127.0.0.1 --port 3000
cd web
npm install
echo "VITE_API_BASE_URL=http://127.0.0.1:3000" > .env
npm run dev    # http://localhost:5173
npm run build  # tsc -b && vite build
npm run test   # vitest run
```

Rutas: `/`, `/trees`, `/trees/:treeId`, `/trees/:treeId/research` (Workspace: **Sessions Active/Planned + Current Session + Research Activity** + Planning preview + Opportunities/Active Tasks/Recent Outcomes), `/trees/:treeId/research/planning` (Planning → **Start Research** abre **Create Session** modal + muestra `Active Session`/`View Session`), `/trees/:treeId/research/sessions` (Sessions list con tabs Active & Planned / History), `/trees/:treeId/research/sessions/history` (Session History: COMPLETED/ABANDONED, filtros status/person, stats batch), `/trees/:treeId/research/sessions/:sessionId` (Session Detail: Objective/Person/Opportunity/Tasks + Session Summary + Research Activity + Progress + Task progress + timeline + Complete/Abandon/Reopen), `/trees/:treeId/research/tasks` (Tasks con `session`), `/trees/:treeId/research/tasks/:taskId` (Task Detail con bloque **Research Session** Add/Remove/View), `/trees/:treeId/research/history` (History outcomes), `/trees/:treeId/sources` (Sources), `/trees/:treeId/evidence` (Evidence).
Workflow: `Opportunity → Planning → Session (PLANNED→ACTIVE→COMPLETED) → Task(s) → Outcome → Evidence → Assessment → Gaps → Follow-ups → Case Summary → Session History + Statistics` — ver `docs/RESEARCH_SESSIONS_HISTORY.md`.

## API REST

```bash
cargo run -p neogenealogy -- import test-data/complex.ged --db /tmp/neogenealogy.db
cargo run -p neogenealogy -- serve --db /tmp/neogenealogy.db --host 127.0.0.1 --port 3000
curl http://127.0.0.1:3000/health
curl http://127.0.0.1:3000/api/v1/trees
curl http://127.0.0.1:3000/api/v1/trees/1/persons?limit=5
curl 'http://127.0.0.1:3000/api/v1/trees/1/research-opportunities/top'
curl http://127.0.0.1:3000/api/v1/openapi.json
```

Env: `NEOGENEALOGY_HOST`, `NEOGENEALOGY_PORT`, `NEOGENEALOGY_DATABASE_URL`, `NEOGENEALOGY_CORS_ORIGIN`.

## Docker

```bash
cargo build -p neogenealogy
npm --prefix web run build
docker compose build
docker compose up
# en otra terminal
docker compose exec neogenealogy neogenealogy import test-data/complex.ged --db /data/neogenealogy.db
# abre http://localhost:3000/  (web servida por API, mismo origen)

# Tests en Docker (Rust + web)
docker compose -f docker-compose.test.yml build
docker compose -f docker-compose.test.yml run --rm test
```

## Benchmark

```bash
rustc benchmarks/generate.rs -o /tmp/gen
/tmp/gen 1000  > /tmp/bench-1000.ged
/tmp/gen 5000  > /tmp/bench-5000.ged
/tmp/gen 10000 > /tmp/bench-10000.ged
/usr/bin/time -f "%e sec %M KB" cargo run --release -p neogenealogy -- import /tmp/bench-1000.ged --db /tmp/bench.db
```

Véase `benchmarks/README.md` para baseline (parse/analysis/persist).

## Verificación

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features
cargo test --workspace
```
