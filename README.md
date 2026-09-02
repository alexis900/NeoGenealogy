# NeoGenealogy v0.3.0 — Research Outcomes

Herramienta local de análisis genealógico. Release v0.3.0 cierra Research Outcomes (`Opportunity → Task → Outcome`).

```
WHAT THE SYSTEM FOUND        → Research Opportunity (auto)
WHAT THE USER DECIDED TO INVESTIGATE → Research Task (OPEN → RESOLVED/…)
WHAT THE USER DISCOVERED     → Research Outcome (CONFIRMED/…)
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
- `storage` — SQLite, WAL, `analysis_runs` + `research_tasks` + `research_outcomes` (003, UNIQUE task, CASCADE)
- `api` — Axum REST `/api/v1` (paginación, filtros, Research Tasks, Research Outcomes, OpenAPI)
- `cli` — `analyze / import / stats / report / serve` (`--db`, `--host`, `--port`)
- `web/` — React 19 + Vite + Tailwind + React Router (Research Queue + Research Tasks + Outcomes)

Véase `docs/RESEARCH_OUTCOMES.md`, `docs/RESEARCH_WORKFLOW.md`, `docs/API.md`, `docs/STORAGE.md`, `docs/WEB.md`, `docs/RESEARCH_ENGINE.md`, `docs/SCORING.md`, `docs/SOURCE_COVERAGE.md`, `docs/ANALYSIS_RULES.md`.

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

Rutas: `/`, `/trees`, `/trees/:treeId`, `/trees/:treeId/research` (Queue), `/trees/:treeId/research/tasks` (Tasks), `/trees/:treeId/research/tasks/:taskId` (Outcome: Record/Edit/Delete) , `/trees/:treeId/persons/:personId`, `/trees/:treeId/findings`, `/trees/:treeId/branches`, `/trees/:treeId/sources`.
Workflow: `Opportunity → Start Research → Task OPEN → IN_PROGRESS → RESOLVED/REJECTED/INCONCLUSIVE → Outcome (CONFIRMED/FALSE_LEAD/INCONCLUSIVE/NEW_LEAD/NO_EVIDENCE)` — ver `docs/RESEARCH_OUTCOMES.md`.

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
