# Research Sessions — Fase 5.2

`Opportunity → Planning → Session → Task → Outcome`

## Concepto

Una `Research Session` es un **bloque de trabajo consciente** del usuario: “qué voy a investigar ahora”. No es Opportunity, Task, Outcome ni scoring. Es el contenedor de contexto de trabajo.

Session: “Estoy trabajando en esta investigación ahora.”
Task: “Esto es algo concreto que necesito hacer.”

Ejemplo:
```
Session: Find the parents of Josep García
Tasks:
  ✓ Review birth record
  ○ Search parish register
  ○ Verify father's identity
```

Una Session puede tener `0..N` Tasks. Una Task puede existir sin Session (compatibilidad).

## Persistencia

Tabla `research_sessions` (migración `006_research_sessions.sql`):

```sql
research_sessions (
  id, tree_id REFERENCES trees ON DELETE CASCADE,
  title NOT NULL, description,
  status CHECK IN ('PLANNED','ACTIVE','COMPLETED','ABANDONED'),
  person_id REFERENCES persons ON DELETE SET NULL,
  opportunity_id REFERENCES research_opportunities ON DELETE SET NULL,
  created_at, updated_at, started_at, completed_at
)
```

`research_tasks.session_id INTEGER REFERENCES research_sessions(id) ON DELETE SET NULL`

Tree isolation obligatorio: toda Session pertenece a un `tree_id`; Tasks no pueden cruzar árboles.

## Lifecycle

```
PLANNED → ACTIVE (started_at = now)
ACTIVE → COMPLETED | ABANDONED (completed_at = now)
COMPLETED/ABANDONED → ACTIVE (completed_at = NULL, started_at updated)
PLANNED → ABANDONED
```

No es state machine rígida; API valida solo valores permitidos. No se modifican Tasks/Outcomes/Evidence al completar/abandonar/reabrir Session.

## Repository

`Storage` métodos:
- `create_research_session(tree_id, title, description, person_id, opportunity_id)` → `PLANNED`
- `get_research_session(id)`
- `list_research_sessions(tree_id, status?, person_id?, opportunity_id?, limit, offset)` — orden `ACTIVE > PLANNED > COMPLETED > ABANDONED`, `updated_at DESC`, paginado, sin N+1, filtrado por tree
- `update_research_session(id, title?, description??, status?, person_id??, opportunity_id??)` — timestamps `started_at`/`completed_at`/`updated_at`
- `delete_research_session(id)` — `ON DELETE SET NULL` para tasks
- `assign_task_to_session(task_id, session_id)` y `remove_task_from_session(task_id)` — valida mismo `tree_id`
- `list_tasks_for_session(session_id)` — `updated_at DESC`
- `get_session_summary(session_id)` — `total_tasks, open_tasks, in_progress_tasks, terminal_tasks, outcomes_count` (1 query batch, no N+1)
- `get_session_detail(session_id)` — `session + person + opportunity + tasks + summary` (3-4 queries, no N+1)
- `get_tasks_session_map(task_ids)` y `get_active_sessions_by_opportunity(tree_id)` — batch loading

## API

```
GET    /api/v1/trees/:tree_id/research-sessions?status=&person_id=&opportunity_id=&limit=&offset=
POST   /api/v1/trees/:tree_id/research-sessions {title, description?, person_id?, opportunity_id?}
GET    /api/v1/trees/:tree_id/research-sessions/:session_id → {session, person, opportunity, tasks, summary}
PATCH  /api/v1/trees/:tree_id/research-sessions/:session_id {title?, description?, status?, person_id?, opportunity_id?}
DELETE /api/v1/trees/:tree_id/research-sessions/:session_id

POST   /api/v1/trees/:tree_id/research-tasks/:task_id/session {session_id}
DELETE /api/v1/trees/:tree_id/research-tasks/:task_id/session
GET    /api/v1/trees/:tree_id/research-sessions/:session_id/tasks
GET    /api/v1/research-sessions (generic, tree_id in body)
```

`GET /trees/:tree_id/research-tasks` y `GET /trees/:tree_id/research-tasks/:task_id` ahora incluyen `session_id` y `session {id,title,status}` via `get_tasks_session_map` (batch).

OpenAPI: `ResearchSession`, `ResearchSessionStatus`, `ResearchSessionSummary`.

## Frontend

- **Nav**: `Research → Overview / Planning / Opportunities / Sessions / Tasks / History / Sources / Evidence / Coverage`
- **ResearchSessions list** (`/trees/:treeId/research/sessions`): filtros Status/Person/Opportunity, orden `ACTIVE first`, cards compactas `status badge + title + person/opportunity + description`
- **ResearchSessionDetail** (`/trees/:treeId/research/sessions/:sessionId`): `Objective, Person, Opportunity, Tasks (✓/○), Progress (terminal/total), Outcomes count, Actions: Start/Complete (con warning “2 tasks are still open” pero permite continuar)/Abandon/Reopen/Delete`; `Remove` por task.
- **Create Session** desde Planning: `Start Research` abre modal pre-rellenado `title = Opportunity.title, person_id, opportunity_id`, `Create Session` → `POST` y refresh; Planning muestra `Active Session` + `View Session` si `opportunity_id` tiene Session `ACTIVE` (fetch `getSessions(limit=100)` y map).
- **TaskDetail**: bloque `Research Session` → `View Session / Remove` si tiene, `Not assigned + Add to Session (select) / Create new session` si no.
- **Overview**: bloque `Research Sessions` (`Active / Planned` counts + `Current Session` con `terminal/total` y `Continue`).
- **Planning progress**: derivado `terminal_tasks / total_tasks`, `No tasks yet` si 0.
- **Session no crea Outcome** ni modifica Tasks al cerrar/abandonar; Outcome via `View Case Summary` existente.

## Performance y No-objetivos

Sin calendar/deadlines/notifications/collab/Jira; sin IA; sin scoring de sesión; `1 query sessions + 1 query tasks + 1 query outcomes` para detail, batch para list; `getSessions` no hace N+1.

## Verificación

`cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --workspace`, `npm run build`, `npm run test` (88+ tests), Docker `compose build && up`.

