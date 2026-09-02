# Research Follow-up Actions — Fase 4.4

## Purpose

Responde:

> "¿Qué hice con esta recomendación?"

Si 4.3 responde *qué acción sugerida existe*, 4.4 registra *qué acción decidió ejecutar el investigador*.

```
Evidence Gap
      ↓
Research Follow-up   (sugerencia derivada, no persistida)
      ↓
Research Follow-up Action (actividad registrada por el usuario, persistida)
```

No es un hecho genealógico, ni una resolución automática, ni una Research Task.

## Separation

```
Research Follow-up       → sugerencia del sistema según gaps actuales
Research FollowupAction → acción humana registrada (OPEN/COMPLETED/SKIPPED)
Research Task           → investigación completa (OPEN→RESOLVED)
```

Una Action es más pequeña que un Task. Un Task puede tener varios Follow-ups y cada Follow-up varias Actions históricas.

```
Task: Investigate parents of Juan García
  Outcome: CONFIRMED
    Follow-up: REVIEW_CONTRADICTION (derivado)
      Action 1: OPEN → COMPLETED "Compared baptism and census" (persistida)
      Action 2: OPEN (nueva contradicción posterior)
```

## Model persistido

```sql
research_followup_actions (
  id INTEGER PRIMARY KEY,
  tree_id INTEGER NOT NULL REFERENCES trees ON DELETE CASCADE,
  task_id INTEGER NOT NULL REFERENCES research_tasks ON DELETE CASCADE,
  outcome_id INTEGER NOT NULL REFERENCES research_outcomes ON DELETE CASCADE,
  followup_code TEXT NOT NULL CHECK(IN 5 codes),
  status TEXT NOT NULL CHECK(OPEN/COMPLETED/SKIPPED),
  notes TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  completed_at TEXT
)
```

Índices: `tree_id, task_id, outcome_id, status, followup_code, updated_at`.

No se almacena `title/description/priority` (pertenecen al Follow-up derivado) ni `evidence_id` (la relación Evidence ya existe vía `outcome_evidence`).

`followup_code` se guarda como snapshot para mantener historial aunque las reglas derivadas cambien.

## Status

- `OPEN` — usuario quiere realizar la acción (`completed_at = NULL`)
- `COMPLETED` — realizada (`completed_at = now`)
- `SKIPPED` — decidida no realizar (`completed_at = now`)

`SKIPPED` no es fracaso. Transiciones libres entre los tres estados; `updated_at` siempre se actualiza.

## Idempotencia

Se permiten múltiples acciones históricas para el mismo `followup_code`. No existe `UNIQUE(task_id, followup_code)`.

## API

### Crear

```
POST /api/v1/trees/:tree_id/research-outcomes/:outcome_id/followup-actions
Body: { "followup_code": "REVIEW_CONTRADICTION", "notes": "Reviewed both records." }
→ 201 { id, tree_id, task_id, outcome_id, followup_code, status:OPEN, notes, created_at, updated_at, completed_at:null }
→ 422 FOLLOWUP_NOT_ACTIVE si el código no está entre los followups activos actuales (calculados vía calculate_research_followups)
→ 400 INVALID_FOLLOWUP_CODE si código no es de los 5 permitidos
```

Valida `followup_code ∈ active_followups` mediante `get_outcome_followups`.

### List / Get / Update / Delete

```
GET  /api/v1/trees/:tree_id/research-followup-actions?task_id=&outcome_id=&status=&followup_code=&limit=&offset=   → Paginated
GET  /api/v1/trees/:tree_id/research-followup-actions/:id
PATCH /api/v1/trees/:tree_id/research-followup-actions/:id  Body: { status, notes }  → actualiza status/notes, recalcula completed_at
DELETE /api/v1/trees/:tree_id/research-followup-actions/:id → 204
GET  /api/v1/trees/:tree_id/research-tasks/:task_id/followup-actions
GET  /api/v1/trees/:tree_id/research-outcomes/:outcome_id/followup-actions → { items: [...] }
```

### Outcome enriquecido

```
GET /api/v1/trees/:tree_id/research-outcomes/:outcome_id
→ { evidence_assessment, evidence_gaps, research_followups, followup_actions: [ ... ] }
```

### List con batch

```
GET /api/v1/trees/:tree_id/research-outcomes?limit=20
→ cada item: { ..., research_followups, followup_actions_count }
```

`followup_actions_count` es `GROUP BY outcome_id` sin N+1. `GET /research/summary` añade `followup_actions: {open,completed,skipped}` con `COUNT GROUP BY status`.

## No automation

- `COMPLETED` no hace desaparecer gaps, no mejora assessment, no cambia `Outcome.type` ni `Task.status`.
- No crea Evidence/Citation automáticamente.
- No resuelve contradicciones (`preferred evidence` no existe aún).
- La Action solo registra que el usuario actuó; el sistema solo cambiará si los datos subyacentes cambian.

## UI

### ResearchTaskDetail

```
Research Follow-ups
HIGH Review contradictory evidence [Start follow-up] [Review Evidence]

Research Follow-up Actions
OPEN  Review contradictory evidence  Created Sep 2  [Mark completed] [Skip]
  textarea Notes
COMPLETED ✓ Review contradictory evidence  Completed Sep 2 "Compared ..." [Reopen] [Delete]  → muestra "Action completed, not gap resolved"
```

- Botón `Start follow-up` por cada Follow-up activo.
- Lista ordenada `updated_at DESC`, con `Status`, `Created`, `Completed`, `Notes`.
- `Completed` no oculta gaps/followups derivados.

### Workspace / History

- Workspace: bloque `Follow-up Actions` desde `GET /research/summary`.
- History: por outcome `Follow-up actions: 2` vía `followup_actions_count` sin N+1.

## Storage

Migración `005_research_followup_actions.sql`, `ResearchFollowupActionRow`, métodos:

```
create_followup_action, get_followup_action, list_followup_actions, list_task_followup_actions, list_outcome_followup_actions, get_outcomes_followup_actions_counts, update_followup_action, delete_followup_action, count_followup_actions_by_status
```

Cascade: borrar `Task` u `Outcome` borra sus Actions (`ON DELETE CASCADE`). Tree isolation en cada query.

## Validation

- `followup_code` ∈ 5, `status` ∈ OPEN/COMPLETED/SKIPPED, `tree_id==task.tree_id==outcome.tree_id`, `outcome.task_id==task_id`.
- Tree isolation: cross-tree → 404.

## Testing

Unit: create/open/complete/skip/reopen/timestamps/notes/invalid code/multiple same code.
Storage: CRUD/pagination/filters/tree isolation/cascade/batch counts.
API: POST/GET/PATCH/DELETE/task actions/outcome actions/FOLLOWUP_NOT_ACTIVE/tree isolation.
Frontend: no actions/open/completed/skipped/multiple/start/complete/skip/notes y que completed no oculta gaps.
E2E 8 pasos definidos en spec.

## Qué NO es

No AI/LLM, no búsqueda externa, no FamilySearch, no notificaciones, no creación automática de Task/Opportunity/Evidence/Citation/Outcome.

Ver `docs/RESEARCH_FOLLOWUPS.md`, `docs/EVIDENCE_GAPS.md`, `docs/STORAGE.md`, `docs/API.md`, `docs/WEB.md`.
