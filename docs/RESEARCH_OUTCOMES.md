# Research Outcomes — Fase 3.0

## Modelo conceptual

NeoGenealogy distingue explícitamente tres capas:

```
WHAT THE SYSTEM FOUND
        ↓
Research Opportunity

WHAT THE USER DECIDED TO INVESTIGATE
        ↓
Research Task

WHAT THE USER DISCOVERED
        ↓
Research Outcome
```

| Entidad | Origen | Cardinalidad | Propósito |
|---------|--------|--------------|-----------|
| **Research Opportunity** | Generada automáticamente por `analyzer` + `scoring` | N por árbol (persistida en `research_opportunities`) | Algo que merece investigación: explainable (`why`, `what`, `potential_sources`, `breakdown`) |
| **Research Task** | Creada por el usuario (`POST /research-tasks` o `POST /research-opportunities/:id/tasks`) | 0..N por oportunidad/persona | Investigación que el usuario ha decidido realizar. Estado `OPEN → IN_PROGRESS → RESOLVED/REJECTED/INCONCLUSIVE` |
| **Research Outcome** | Creado por el usuario (`POST /research-tasks/:task_id/outcome`) | 0..1 por task (`UNIQUE(task_id)`) | Resultado estructurado de esa investigación. Registro, no edición del árbol |

El `Outcome` **no modifica personas, relaciones, eventos ni GEDCOM**. Es un registro de lo descubierto para las siguientes fases (`Evidence & Sources` lo referenciará, no lo sustituye).

## Storage

Migración: `crates/storage/migrations/003_research_outcomes.sql`

```sql
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
```

* `FK` a `trees` y `research_tasks` con `ON DELETE CASCADE` — borrar una Task borra su Outcome.
* `UNIQUE(task_id)` — una Task solo puede tener un Outcome. Segundo `POST` → `409 RESEARCH_OUTCOME_ALREADY_EXISTS`.
* Aislamiento por `tree_id` — un Outcome de `tree=1` no es visible desde `tree=2` (`404`).
* Validación en `Storage::create_research_outcome` / `update_research_outcome`: `type` permitido y `summary` no vacío. También validación en API (`400`).

## Tipos (`type`)

| Tipo | Significado | Ejemplo `summary` |
|------|-------------|-------------------|
| `CONFIRMED` | La hipótesis se confirmó con evidencia | "Birth record found at parish X, 12 Mar 1880" |
| `FALSE_LEAD` | La pista era incorrecta (persona distinta, fecha errónea) | "Not the same Juan García — different parents" |
| `INCONCLUSIVE` | Investigación sin conclusión clara | "Parish books missing 1879-1881, cannot confirm" |
| `NEW_LEAD` | Se encontró una nueva pista que requiere seguir | "Found sibling baptism that mentions mother's maiden name" |
| `NO_EVIDENCE` | Búsqueda exhaustiva sin evidencia | "Checked civil registry 1880-1885, no match" |

Los cinco tipos se cubren en tests de storage, API y web.

## Campos

```json
{
  "id": 1,
  "tree_id": 1,
  "task_id": 42,
  "type": "CONFIRMED",
  "summary": "Baptism record located",
  "details": "Parish book p.12, line 4 — witnesses: ...",
  "created_at": "2026-02-09T02:00:00Z",
  "updated_at": "2026-02-09T02:00:00Z"
}
```

* `summary` — obligatorio, no vacío (trim). Resumen corto (1 línea).
* `details` — opcional, texto libre largo (notas, transcripción, referencias informales).
* `type` — enum validado (`400 INVALID_RESEARCH_OUTCOME_TYPE` si inválido).
* `created_at`/`updated_at` — ISO 8601 UTC.

## API REST

```
POST   /api/v1/trees/:tree_id/research-tasks/:task_id/outcome
GET    /api/v1/trees/:tree_id/research-outcomes
GET    /api/v1/trees/:tree_id/research-outcomes/:outcome_id
PATCH  /api/v1/trees/:tree_id/research-outcomes/:outcome_id
DELETE /api/v1/trees/:tree_id/research-outcomes/:outcome_id
GET    /api/v1/trees/:tree_id/research-tasks/:task_id   (outcome embebido)
```

### Crear

```http
POST /api/v1/trees/1/research-tasks/42/outcome
Content-Type: application/json

{ "type": "CONFIRMED", "summary": "Found baptism", "details": "parish book p.12" }

→ 201
{ "id": 1, "tree_id": 1, "task_id": 42, "type": "CONFIRMED", "summary": "Found baptism", ... }
```

Errores:

| Caso | Status | Code |
|------|--------|------|
| `tree_id`/`task_id` inexistente o cross-tree | 404 | `TREE_NOT_FOUND` / `RESEARCH_TASK_NOT_FOUND` |
| `type` inválido | 400 | `INVALID_RESEARCH_OUTCOME_TYPE` |
| `summary` vacío | 400 | `INVALID_SUMMARY` |
| segundo Outcome para misma Task | 409 | `RESEARCH_OUTCOME_ALREADY_EXISTS` |
| `limit`/`offset` inválidos en listado | 400 | `INVALID_LIMIT` / `INVALID_OFFSET` |

### Listar

```
GET /api/v1/trees/1/research-outcomes?type=CONFIRMED&task_id=42&person_id=5&limit=20&offset=0
→ 200 { "items": [...], "pagination": { "limit": 20, "offset": 0, "total": 5 } }
```

* `type` — filtro exacto (case-insensitive input, almacenado upper).
* `task_id` — filtra por Task.
* `person_id` — filtra vía `JOIN research_tasks.person_id` (`list_research_outcomes_with_person`).
* Paginado `ORDER BY updated_at DESC`.

### Obtener / Actualizar / Eliminar

```
GET    /api/v1/trees/1/research-outcomes/10 → 200 { ... } | 404
PATCH  /api/v1/trees/1/research-outcomes/10 { "type": "NEW_LEAD", "summary": "...", "details": "..." } → 200
DELETE /api/v1/trees/1/research-outcomes/10 → 204
```

`PATCH` valida igual que `POST`; permite actualizar `type`, `summary`, `details` de forma parcial. Cross-tree → `404`.

### Task con Outcome embebido

```
GET /api/v1/trees/1/research-tasks/42
→ 200 {
  "id": 42, "tree_id": 1, "title": "...", "status": "RESOLVED",
  "outcome": { "id": 1, "type": "CONFIRMED", ... } | null
}
```

Si la Task no tiene Outcome, `outcome: null`. Si tiene, objeto completo. El listado `GET /research-tasks` no embebe outcomes (solo `GET` individual).

## Workflow completo

```
complex.ged
    ↓
GEDCOM import → SQLite (trees, persons, findings…)
    ↓
Analyzer + Scoring → Research Opportunity (auto, score, why/what/sources/breakdown)
    ↓
User: Start Research → POST /research-opportunities/:id/tasks
    ↓
Research Task OPEN
    ↓
User: PATCH status IN_PROGRESS (started_at auto)
    ↓
User: PATCH status RESOLVED / REJECTED / INCONCLUSIVE (completed_at auto, resolution opcional)
    ↓
User: Record Outcome → POST /research-tasks/:id/outcome {type, summary}
    ↓
GET /research-tasks/:id → outcome embebido, visible en Web
    ↓
User: PATCH /research-outcomes/:id (editar) | DELETE (eliminar → vuelve a Record Outcome)
    ↓
DELETE /research-tasks/:id → CASCADE elimina Outcome
```

**Validación de estado:** la implementación actual **permite crear Outcome en cualquier estado** (`OPEN`, `IN_PROGRESS`, `RESOLVED`, `REJECTED`, `INCONCLUSIVE`). No hay máquina de estados estricta. El workflow recomendado es registrar el Outcome después de llevar la Task a un estado terminal (`RESOLVED`/`REJECTED`/`INCONCLUSIVE`), pero no se bloquea crear/editar antes. Esto se decidió para no introducir complejidad innecesaria en Fase 3.0; se documenta explícitamente y se cubre con tests.

```
OPEN
  ↓
IN_PROGRESS
  ↓
RESOLVED / REJECTED / INCONCLUSIVE
  ↓
Outcome (recomendado, no obligatorio por código)
```

## Web

* **Routing:** `/trees/:treeId/research/tasks/:taskId` (ResearchTaskDetail)
* **Sin Outcome:** muestra `No research outcome recorded yet.` + form con select `type`, input `summary` (required), textarea `details` + botón `Record Outcome` (disabled si `summary` vacío). Previene requests vacíos.
* **Con Outcome:** muestra badge `type` (Confirmed/False lead/...), `summary`, `details`, `created_at` + sección `Edit Outcome` (type select, summary, details, `Update Outcome` / `Delete Outcome`). No muestra segundo formulario de creación.
* **Original Opportunity:** siempre visible (si `task.opportunity_id` existe) con `PriorityBadge`, `ScoreBadge`, `why/what/sources`, `ScoreBreakdown`. Persiste tras crear/editar/eliminar Outcome.
* **Estados:** `Loading task…`, `Error` con mensaje, `Empty` no outcome. Tras `DELETE` Outcome vuelve a estado `Record Outcome`.
* **Cliente:** `web/src/api/client.ts` maneja `204` → `undefined` (no intenta `JSON.parse` body vacío).
* **Tipos:** `web/src/api/types.ts` define `OutcomeType` + `ResearchTask.outcome?: ResearchOutcome | null`.

## Tests

* `crates/storage/tests/research_outcomes.rs` — CRUD, 5 tipos, validaciones (summary vacío, type inválido, task/outcome inexistente), unique, tree isolation, list/paginación/filtros (`type`, `task_id`, `list_with_person`), cascade, rollback.
* `crates/api/tests/research_outcomes.rs` — `POST 201`, `GET 200`, `PATCH 200`, `DELETE 204`, `GET task` embebido (`null` y objeto), errores `404/400/409`, cross-tree, listado/filtros/paginación, 5 tipos, JSON fields.
* `web/src/pages/__tests__/ResearchTaskDetail.test.tsx` — loading, sin outcome → Record, crear (type/summary/details, mostrar después), editar, eliminar, 5 tipos, existing outcome no duplica, Original Opportunity persiste, error, button disabled.
* `web/src/pages/__tests__/ResearchTasks.test.tsx` — empty, loading.

## Verificación

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
npm --prefix web run build
npm --prefix web run test
docker compose build && docker compose up -d && curl http://127.0.0.1:3000/health
```

Migrations se aplican desde DB limpia (`003_research_outcomes.sql`) vía `sqlx::migrate!`.

## Separación para siguientes fases

El Outcome permanece como **registro del resultado**, no como mecanismo de edición del árbol ni de ingestión de evidencia. `Evidence & Sources` (fase futura) referenciará `research_outcomes` pero no los sustituye. No se introducen en Fase 3.0: `attachments`, `OCR`, `FamilySearch`, `IA/LLM`, `Postgres`, `WebSockets`, etc.
