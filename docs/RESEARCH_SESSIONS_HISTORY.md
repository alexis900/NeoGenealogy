# Research Sessions History & Statistics — Fase 5.3

`Statistics 100% derived — No persistence, no scoring, no truth claim`

## Concepto

Fase 5.3 responde a **“¿Qué trabajo de investigación hemos realizado y qué resultados estamos obteniendo?”**.

No introduce un nuevo motor de negocio; es una **capa de observabilidad** sobre:

```
research_sessions
research_tasks
research_outcomes
evidence + outcome_evidence (SUPPORTS/CONTRADICTS)
research_followup_actions
```

No existe `research_statistics`, ni contadores persistidos, ni snapshots, ni tablas de eventos o analytics. Todo se calcula en el momento de la consulta.

```
Research Planning         → Qué debería investigar
Research Session          → Qué estoy investigando ahora (PLANNED/ACTIVE/COMPLETED/ABANDONED)
Research Task(s)          → Qué tengo que hacer
Research Outcome          → Qué encontré (CONFIRMED/FALSE_LEAD/INCONCLUSIVE/NEW_LEAD/NO_EVIDENCE)
Evidence + Sources        → En qué me baso
Research Session History  → Qué trabajo he realizado (5.3)
Research Statistics       → Qué actividad y resultados describen ese trabajo (5.3)
```

## Diferencia Session History / Research History

| Concepto | Contenido | Ruta UI | Ruta API |
|----------|-----------|---------|----------|
| **Research History** | Historial de **Outcomes** (qué se descubrió). Orden `created_at DESC`. | `/trees/:treeId/research/history` | `GET /trees/:treeId/research-outcomes` |
| **Research Session History** | Historial de **Sesiones** terminadas (`COMPLETED`, `ABANDONED`). Orden `COALESCE(completed_at, updated_at) DESC`. | `/trees/:treeId/research/sessions/history` | `GET /trees/:treeId/research-sessions/history` y `/api/v1/research-sessions/history?tree_id=` |

No fusionados: un Outcome puede existir sin Session; una Session puede tener 0 Outcomes. Session History describe **contenedores de trabajo**; Research History describe **conclusiones**.

## Session History

### Ruta

```
/trees/:treeId/research/sessions/history
```

Muestra solo `COMPLETED` y `ABANDONED`. Si `completed_at` es `NULL` (datos antiguos/inconsistentes) se ordena por `updated_at DESC`.

Cada card:

```
Find the parents of Josep García
COMPLETED — Completed 2 Sep 2026
3 tasks — 2 outcomes — 3 evidence (2 supporting, 1 contradicting)
[View Session]
```

Incluye **stats derivados** ya calculados (sin N+1): se hace `1 query sessions + batch aggregates`.

### Filtros

Solo:

- `status = COMPLETED | ABANDONED`
- `person_id`

Con paginación existente (`limit 0..100`, `offset`, `page`). No búsqueda textual, no rangos de fechas, no múltiples personas.

Estados vacíos diferenciados:

- Sin sesiones terminadas: `No completed research sessions yet. Completed and abandoned sessions will appear here.`
- Con filtros sin resultados: `No sessions match the selected filters.`

### API

```
GET /api/v1/trees/:treeId/research-sessions/history?status=COMPLETED&person_id=5&limit=20&page=2
GET /api/v1/research-sessions/history?tree_id=1&status=ABANDONED&person_id=5&limit=20&offset=0
GET /api/v1/trees/:treeId/research-sessions?history=true   // alias sin ambigüedad si conviene
```

Respuesta paginada con `stats` embebido por sesión (batch):

```json
{
  "items": [{
    "id": 1,
    "title": "Find the parents of Josep García",
    "status": "COMPLETED",
    "completed_at": "2026-09-02T00:00:00Z",
    "stats": {
      "total_tasks": 3,
      "completed_tasks": 1,
      "open_tasks": 1,
      "in_progress_tasks": 1,
      "total_outcomes": 2,
      "confirmed_outcomes": 1,
      "total_evidence": 3,
      "supporting_evidence": 2,
      "contradicting_evidence": 1,
      "open_followups": 1,
      "completed_followup_actions": 1,
      "skipped_followup_actions": 0
    }
  }],
  "pagination": { "limit": 20, "offset": 0, "total": 18 }
}
```

### Performance

```
N sessions → 1 query sessions + 4 batch aggregates
  1) tasks GROUP BY session_id,status
  2) outcomes GROUP BY session_id,type
  3) evidence GROUP BY session_id,relationship
  4) followup_actions GROUP BY session_id,status
```

Nunca `1+N`.

### Tree isolation

 Toda agregación filtra por `tree_id`. Tests explícitos verifican que `Tree 1` nunca ve datos de `Tree 2`.

## ResearchSessionStats — modelo derivado

No persistido. Calculado por `Storage::get_session_stats` / `get_sessions_stats`.

```rust
ResearchSessionStats {
  total_tasks, completed_tasks (RESOLVED), open_tasks, in_progress_tasks, inconclusive_tasks, rejected_tasks,
  total_outcomes, confirmed_outcomes, false_lead_outcomes, inconclusive_outcomes, new_lead_outcomes, no_evidence_outcomes,
  total_evidence, supporting_evidence (SUPPORTS), contradicting_evidence (CONTRADICTS),
  open_followups (OPEN actions), completed_followup_actions, skipped_followup_actions
}
```

- `total_tasks` = `COUNT(*) WHERE session_id = ?`
- `completed_tasks` = `WHERE status='RESOLVED'` (no confundir con `terminal` = RESOLVED+REJECTED+INCONCLUSIVE de `ResearchSessionSummary`)
- `total_outcomes` = `JOIN research_tasks → research_outcomes WHERE session_id`
- `supporting_evidence` = `JOIN outcome_evidence WHERE relationship='SUPPORTS'`
- `open_followups` = `JOIN research_followup_actions WHERE status='OPEN'`

Se expone también en `GET /trees/:treeId/research-sessions/:session_id` como `stats` junto a `summary`, `tasks`, `timeline`.

### Session Detail — métricas visibles

```
Session Summary
Tasks        3 total · 1 completed · 2 open
Outcomes     2 (Confirmed 1, No evidence 1)
Evidence     3 (2 supporting, 1 contradicting)
Follow-ups   2 open · 1 completed

Task progress 1 / 3
Progress 1 / 3 (terminal/total — mantenido de 5.2)
```

Se ocultan secciones vacías excepto `0 outcomes` que se muestra explícitamente. Nunca se usan Outcomes para calcular progress.

## Research Activity — Overview

`GET /api/v1/trees/:treeId/research/summary` ampliado (compatible) incluye:

```json
{
  "sessions": { "total": 27, "active": 2, "planned": 3, "completed": 18, "abandoned": 4 },
  "research_activity": {
    "tasks": { "open": 12, "in_progress": 3, "resolved": 21, "rejected": 2, "inconclusive": 1, "total": 39 },
    "outcomes": { "total": 33, "confirmed": 14, "false_lead": 5, "inconclusive": 3, "new_lead": 7, "no_evidence": 4 },
    "evidence": { "total": 42, "supporting": 31, "contradicting": 11 },
    "followups": { "open": 8, "completed": 12, "skipped": 3, "total": 23 }
  },
  "opportunities": { "high": 5, "medium": 10, "low": 12 },
  "tasks": { "open": 12, "in_progress": 3, "resolved": 21, "rejected": 2, "inconclusive": 1 },
  "outcomes": { "total": 33, "confirmed": 14, "false_lead": 5, "inconclusive": 3, "new_lead": 7, "no_evidence": 4 },
  "evidence": { "total": 42, "supporting": 31, "contradicting": 11 },
  "followup_actions": { "open": 8, "completed": 12, "skipped": 3 },
  "sessions": { ... }
}
```

Campos antiguos (`opportunities`, `tasks`, `outcomes.total`, `evidence`, `followup_actions`) se mantienen para no romper consumidores; los nuevos (`sessions`, `research_activity`, `outcomes.confirmed`…) son aditivos.

En la UI: `/trees/:treeId/research` muestra bloque **Research Activity**:

```
Sessions  Active 2  Planned 3  Completed 18  Abandoned 4
Tasks     Open 12  In Progress 3  Resolved 21
Outcomes  Confirmed 14  False leads 5  Inconclusive 3  New leads 7  No evidence 4
Evidence  Total 42  Supporting 31  Contradicting 11
Follow-ups Open 8  Completed 12  Skipped 3
```

Usa metric cards / distribution lists, sin librería de charts.

La API usa **1 endpoint** (`research/summary`) en vez de `N` endpoints `/stats/*`.

## Timeline derivada

No existe tabla `events`. Se construye a partir de timestamps existentes:

- `session.created_at`, `session.started_at`, `session.completed_at`
- `task.created_at`, `task.started_at`, `task.completed_at`, `task.updated_at`
- `outcome.created_at`, `outcome.updated_at`
- `evidence.created_at`
- `research_followup_actions.created_at`, `completed_at`

Ejemplo:

```
Sep 3  Outcome created
Sep 2  Task completed
Sep 2  Evidence added
Sep 1  Task started
Sep 1  Session started
```

- Orden `DESC` (más reciente primero), desempate por `created_at/updated_at` según entidad.
- Limitada a 20 eventos (`Storage::get_session_timeline(session_id, 20)`), con posibilidad futura de `Load more` pero sin paginación compleja.
- Si una entidad no tiene timestamp suficiente, simplemente no genera evento; no se inventan eventos.
- Etiqueta claramente como **Activity timeline** — no afirma ser exhaustive.

Se expone en `GET /research-sessions/:id` como `timeline: [{event_type,timestamp,label}]`.

## Qué no hace 5.3

- No modifica `Research Score`, `Planning Score`, `Confidence`, `Researchability`, `Priority`.
- No crea `Session Score`, `Productivity Score`, `Success Rate` (CONFIRMED ≠ verdad absoluta).
- No calcula “truth score”.
- No añade analytics avanzado, gráficos complejos, tendencias predictivas, benchmarking, gamificación, objetivos, productividad por usuario, calendario, deadlines, reminders, notifications.
- No persiste estadísticas; todo es `GROUP BY` en el momento.

## Por qué sin persistencia

- Evita duplicación y desincronización (`research_tasks` es fuente de verdad).
- Mantiene el modelo simple: 6 tablas + derivadas puras.
- Permite `tree isolation` trivial (añadir `WHERE tree_id` a cada `GROUP BY`).
- Facilita tests (counts deterministas) y evita migraciones de snapshots.
- Si en el futuro se necesita cache, se añadiría como capa opcional sin cambiar el contrato.

## Verificación

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
npm --prefix web run build
npm --prefix web run test
sg docker -c "docker compose build"
sg docker -c "docker compose up -d"
curl http://127.0.0.1:3000/api/v1/health
curl http://127.0.0.1:3000/api/v1/openapi.json
```

## Flujo completo tras 5.3

```
WHAT SHOULD I RESEARCH?        → Planning
WHAT AM I RESEARCHING NOW?     → Session (PLANNED/ACTIVE)
WHAT DID I DO?                 → Tasks / Evidence / Follow-ups
WHAT DID I FIND?               → Outcome
WHAT HAVE I DONE OVERALL?      → Research History + Session History + Statistics (5.3)
```

5.3 **describe** el trabajo realizado; no lo juzga.
