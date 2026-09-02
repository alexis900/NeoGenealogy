# Research Case Summary — Phase 4.5

Resumen de cierre/auditoría para una investigación. No añade estado persistente; reutiliza `Research Task.status` como fuente de verdad.

```
Research Opportunity
        ↓
Research Task (OPEN → IN_PROGRESS → RESOLVED/REJECTED/INCONCLUSIVE)
        ↓
Research Outcome (CONFIRMED/…)
        ↓
Evidence (SUPPORTS/CONTRADICTS) → Source + Citation
        ↓
Evidence Assessment (score/status/reasons)
        ↓
Evidence Gaps (CRITICAL/WARNING/INFO)
        ↓
Research Follow-ups (HIGH/MEDIUM/LOW)
        ↓
Follow-up Actions (OPEN/COMPLETED/SKIPPED)
        ↓
Research Case Summary (vista derivada)
```

## Qué es un Research Case

Un caso es una `Research Task` y todo lo derivado de ella. No existe tabla `cases`. El ciclo se considera cerrado cuando `task.status ∈ {RESOLVED, REJECTED, INCONCLUSIVE}` pero sigue siendo modificable — no hay lock.

## Por qué no existe CaseStatus

La fase 4.5 evita duplicar estado. `Task.status` ya representa `OPEN/IN_PROGRESS/RESOLVED/REJECTED/INCONCLUSIVE`. El summary es una vista derivada, no un segundo sistema de estados (`CaseStatus`, `ClosureStatus`, etc. no existen).

## Relación Task.status ↔ Case Summary

El summary reúne:

- `task {id,title,description,status,resolution,created_at,started_at,completed_at,updated_at}`
- `person {person_id, person_name}` si `person_id` existe (resuelto por join, no duplicado)
- `opportunity {opportunity_id,score,priority,researchability,confidence,title}` si `opportunity_id` existe (title = `why` o fallback)
- `outcome {outcome_id,type,summary,details,created_at,updated_at}` si existe
- `evidence_assessment {score,status,evidence_total,supporting,contradicting,sources,cited,uncited,cited_supporting,reasons}` — reutiliza exactamente Phase 4.1
- `evidence_gaps [{code,severity,title,description}]` — Phase 4.2
- `research_followups [{code,priority,title,description,gap_code}]` — Phase 4.3
- `followup_actions [{id,followup_code,status,notes,created_at,updated_at,completed_at}]` — Phase 4.4
- `timeline [{event_type,timestamp,label}]` derivado de timestamps existentes, orden `timestamp ASC` (empate determinista)
- `closure_warnings [{code,severity,title,description}]` — evaluación pura

Todos los objetos opcionales pueden ser `null`; arrays nunca `null`.

## Timeline

Derivada, sin tabla nueva, eventos:

- `TASK_CREATED` ← `task.created_at`
- `TASK_STARTED` ← `task.started_at`
- `OUTCOME_CREATED` ← `outcome.created_at`
- `OUTCOME_UPDATED` ← `outcome.updated_at` si difiere
- `FOLLOWUP_ACTION_CREATED` ← `action.created_at`
- `FOLLOWUP_ACTION_COMPLETED` ← `action.completed_at`
- `TASK_COMPLETED` ← `task.completed_at`

Ordenada por timestamp y rank determinista.

## Closure Warnings

No son errores, no bloquean, no modifican estado. `calculate_closure_warnings(task_status, outcome_type, assessment_status, gaps)` pura/determinista.

| Código | Severidad | Condición |
|--------|-----------|-----------|
| `RESOLVED_WITHOUT_OUTCOME` | WARNING | `task.status==RESOLVED && outcome==None` |
| `CONFIRMED_WITHOUT_SUPPORT` | CRITICAL | `outcome.type==CONFIRMED && assessment.status==NO_EVIDENCE` |
| `RESOLVED_WITH_EVIDENCE_GAPS` | INFO | `task.status==RESOLVED && outcome exists && gaps non-empty` |
| `REJECTED_WITH_CONFIRMED_OUTCOME` | WARNING | `task.status==REJECTED && outcome.type==CONFIRMED` |
| `INCONCLUSIVE_WITH_CONFIRMED_OUTCOME` | WARNING | `task.status==INCONCLUSIVE && outcome.type==CONFIRMED` |

INFO no implica error; solo indica que quedan gaps.

## API

`GET /api/v1/trees/:tree_id/research-tasks/:task_id/case-summary` — 200 con `ResearchCaseSummary` siempre que la task exista; 404 solo `TASK_NOT_FOUND` si `task` no existe o no pertenece al tree. Funciona para tasks sin outcome/evidence/gaps/etc.

Evita N+1: `Storage::get_research_case_summary(tree_id, task_id)` usa queries agrupadas (task, person, opportunity, outcome, stats+assessment, gaps, followups, actions) — no cascada N queries.

## Web

`ResearchTaskDetail` integra `Research Case Summary` (estado loading/error/retry, valores para task parcial). Para estado terminal muestra `Case Closure` con Status/Resolution/Completed at/Outcome/Assessment/Warnings. Warnings usan mismos tokens/colores que Gaps (`CRITICAL` rojo, `WARNING` ámbar, `INFO` azul). `ResearchHistory` añade `View Case Summary` por outcome.

El summary es informativo; no cambia `Task.status`, no crea `Outcome`, no modifica gaps/followups, etc.

## Storage

`crates/storage/src/case_summary.rs` — DTOs `ResearchCaseSummary`, `ResearchCaseTimelineEvent`, `ResearchCaseClosureWarning`, `calculate_closure_warnings`, `build_timeline`. `crates/storage/src/repositories.rs::get_research_case_summary`.

Tree isolation garantizado: `task.tree_id != tree_id → NotFound`.

## No objetivos

Sin AI/LLM/scraping, sin locking, sin workflow obligatorio, sin notificaciones, sin fuentes nuevas.
