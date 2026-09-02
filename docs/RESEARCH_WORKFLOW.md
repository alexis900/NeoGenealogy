# Research Workflow

`Research Opportunity` (sistema) → `Research Task` (usuario) → `OPEN → IN_PROGRESS → RESOLVED/REJECTED/INCONCLUSIVE`

## Opportunity

Recomendación generada automáticamente por `Analyzer` + `Scoring`. Contiene `score`, `confidence`, `priority`, `researchability`, `why`, `what`, `potential_sources`, `breakdown`. No se edita, no se recalcula. Persiste en `research_opportunities` con `analysis_run_id`.

## Task

Investigación que el usuario decide hacer. Creada desde una oportunidad (`POST /research-opportunities/:id/tasks`) o manualmente (`POST /research-tasks` con `title`, `description`, `person_id`, `opportunity_id` opcionales). Pertenece a un `tree_id`; `opportunity_id` y `person_id` deben pertenecer al mismo árbol (validado en `storage`). `opportunity_id` con `ON DELETE SET NULL` — borrar la oportunidad no borra la task.

## Estados

- `OPEN` — creada, no iniciada (`created_at`, `updated_at`)
- `IN_PROGRESS` — activa, establece `started_at` si es NULL al pasar a este estado
- `RESOLVED` / `REJECTED` / `INCONCLUSIVE` — finales, establecen `completed_at` si es NULL y requieren `resolution` (texto libre)

Transiciones validadas en API (`validate_status`). Volver de final a `IN_PROGRESS` no borra `started_at`/`completed_at`; se mantiene historial simple.

## SQLite

Migración `002_research_workflow.sql`:

```sql
CREATE TABLE research_tasks (
  id INTEGER PRIMARY KEY,
  tree_id INTEGER NOT NULL REFERENCES trees ON DELETE CASCADE,
  opportunity_id INTEGER REFERENCES research_opportunities ON DELETE SET NULL,
  person_id INTEGER REFERENCES persons ON DELETE SET NULL,
  title TEXT NOT NULL,
  description TEXT,
  status TEXT CHECK(status IN ('OPEN','IN_PROGRESS','RESOLVED','REJECTED','INCONCLUSIVE')),
  created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
  started_at TEXT, completed_at TEXT, resolution TEXT
);
CREATE INDEX idx_research_tasks_tree ...;
CREATE UNIQUE INDEX idx_research_tasks_unique_active ON research_tasks(opportunity_id, status) WHERE opportunity_id IS NOT NULL AND status IN ('OPEN','IN_PROGRESS');
```

El índice único evita duplicar `OPEN`/`IN_PROGRESS` para la misma oportunidad (reutiliza la existente, permite nueva tras `RESOLVED`).

## Repository

`Storage` en `crates/storage/src/repositories.rs`:

- `create_research_task(tree_id, opportunity_id, person_id, title, description)` — valida aislamiento, reutiliza activa, inserta `OPEN`
- `get_research_task(id)`, `list_research_tasks(tree_id, status, person_id, opportunity_id, limit, offset)` — paginado, filtros
- `update_research_task(id, title, description, status, resolution)` — actualiza `updated_at`, `started_at`/`completed_at` según transición
- `delete_research_task(id)`

Todas parametrizadas, sin SQL en handlers.

## API

- `GET /api/v1/trees/:tree_id/research-tasks?status=&person_id=&opportunity_id=&limit=&offset=`
- `POST /api/v1/trees/:tree_id/research-tasks` `{title, description, person_id, opportunity_id}` → 201
- `GET /api/v1/trees/:tree_id/research-tasks/:task_id`
- `PATCH /api/v1/trees/:tree_id/research-tasks/:task_id` `{title, description, status, resolution}`
- `DELETE /api/v1/trees/:tree_id/research-tasks/:task_id` → 204
- `POST /api/v1/trees/:tree_id/research-opportunities/:opportunity_id/tasks` `{title, description}` → crea/reutiliza task asociada a esa oportunidad y su `person_id`

Errores: `RESEARCH_TASK_NOT_FOUND 404`, `INVALID_RESEARCH_TASK_STATUS 400`, `TREE_NOT_FOUND 404`, `PERSON_NOT_FOUND 404`, `RESEARCH_OPPORTUNITY_NOT_FOUND 404`, `INVALID_LIMIT 400`.

## Web

- `Research Queue` (`/trees/:treeId/research`) → `ResearchOpportunityCard` con `Start Research` (POST from opportunity) o `View Research Task` si ya existe.
- `Research Tasks` (`/trees/:treeId/research/tasks`) — lista con filtros `All/Open/In Progress/Resolved/Rejected/Inconclusive`, paginación.
- `ResearchTaskDetail` (`/trees/:treeId/research/tasks/:taskId`) — edita `title/description/status/resolution`, muestra `Original Research Opportunity` con `ScoreBreakdown`.
- `Dashboard` — resumen `Open/In Progress/Resolved` y link a `Research Tasks`.
- `PersonDetail` — sección `Research Tasks` para esa persona.

`Research Queue` = "What should I investigate?" (automático) vs `Research Tasks` = "What am I actually investigating?" (humano).

No se modifica scoring; no se crean nuevas reglas de análisis.
