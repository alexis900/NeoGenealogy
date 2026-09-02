# API v1

Base: `http://127.0.0.1:3000`

```
GET /health
GET /ready
GET /api/v1/openapi.json
GET /api/v1/docs
GET /api/v1/trees
GET /api/v1/trees/:tree_id
GET /api/v1/trees/:tree_id/persons
GET /api/v1/trees/:tree_id/persons/:person_id
GET /api/v1/trees/:tree_id/families
GET /api/v1/trees/:tree_id/families/:family_id
GET /api/v1/trees/:tree_id/findings
GET /api/v1/trees/:tree_id/research-opportunities
GET /api/v1/trees/:tree_id/research-opportunities/top
GET /api/v1/trees/:tree_id/branches
GET /api/v1/trees/:tree_id/source-coverage
GET /api/v1/trees/:tree_id/analysis-runs
GET /api/v1/trees/:tree_id/analysis-runs/:run_id
```

## Health

```
GET /health → 200 { "status": "ok" }
```

## Paginación

Todos los listados:

```
?limit=50&offset=0   # limit 0..100, offset >=0
```

Respuesta:

```json
{
  "items": [],
  "pagination": { "limit": 50, "offset": 0, "total": 2431 }
}
```

## Trees

```
GET /api/v1/trees
GET /api/v1/trees/1 → { id, name, source_filename, gedcom_version, created_at, updated_at, persons, families, findings, research_opportunities }
```

`persons/families/findings/research_opportunities` se obtienen con `Storage::count` en una sola transacción por árbol (sin N+1).

## Persons / Families

```
GET /api/v1/trees/1/persons?limit=50&offset=100
GET /api/v1/trees/1/persons/5
GET /api/v1/trees/1/families
GET /api/v1/trees/1/families/2  # incluye members { husband, wife, children }
```

## Findings

```
GET /api/v1/trees/1/findings?severity=high&type=chronology&person_id=3
```

- `severity` ∈ low,info,medium,warning,high,critical (validado, 400 si inválido)
- `type` es `finding_type` libre
- `person_id` es id entero de `persons.id`

## Research Opportunities

```
GET /api/v1/trees/1/research-opportunities?priority=high&min_score=70&sort=score&limit=20
GET /api/v1/trees/1/research-opportunities/top?priority=high&limit=10
```

- `priority` ∈ low,info,medium,warning,high,critical
- `min_score` 0..100
- `sort` ∈ score,priority,confidence (mapea a ORDER BY score / CASE priority / confidence)

Respuesta incluye `score, confidence, priority, researchability, why, what, potential_sources, breakdown` (array `[{name,points,reason}]`) — no recalculado.

## Branches / Coverage / Runs

```
GET /api/v1/trees/1/branches → { items: [{ name, branch_score, opportunity_count, high_priority_count, deepest_generation, source_coverage }] }
GET /api/v1/trees/1/source-coverage → { birth, marriage, death, other_events, overall }
GET /api/v1/trees/1/analysis-runs → { items: [{ id, started_at, completed_at, engine_version, status }] }
```

## Research Tasks

```
GET /api/v1/trees/1/research-tasks?status=OPEN&person_id=5&opportunity_id=2&limit=20
POST /api/v1/trees/1/research-tasks {title, description, person_id, opportunity_id} → 201 {id, status:OPEN}
GET /api/v1/trees/1/research-tasks/5 → 200 { ..., outcome: {id,type,summary,...}|null }
PATCH /api/v1/trees/1/research-tasks/5 {title, description, status, resolution} → updated_at, started_at/completed_at auto
DELETE /api/v1/trees/1/research-tasks/5 → 204 (CASCADE outcome)
POST /api/v1/trees/1/research-opportunities/42/tasks {title, description} → 201 (reutiliza si OPEN/IN_PROGRESS existe)
```

- `status` ∈ OPEN,IN_PROGRESS,RESOLVED,REJECTED,INCONCLUSIVE (validado)
- `person_id`/`opportunity_id` deben pertenecer al mismo `tree_id` (aislamiento)
- `POST .../tasks` desde oportunidad valida oportunidad y asocia `person_id` automáticamente
- `GET /research-tasks/:task_id` embebe `outcome` (`null` si no existe).

## Research Outcomes — Fase 3.0

```
POST   /api/v1/trees/1/research-tasks/5/outcome {type, summary, details} → 201 {id,tree_id,task_id,type,summary,details,created_at,updated_at}
GET    /api/v1/trees/1/research-outcomes?type=CONFIRMED&task_id=5&person_id=3&limit=20&offset=0 → 200 {items, pagination} (ORDER BY updated_at DESC)
GET    /api/v1/trees/1/research-outcomes/10 → 200 | 404
PATCH  /api/v1/trees/1/research-outcomes/10 {type, summary, details} → 200 (parcial)
DELETE /api/v1/trees/1/research-outcomes/10 → 204
```

- `type` ∈ CONFIRMED,FALSE_LEAD,INCONCLUSIVE,NEW_LEAD,NO_EVIDENCE (400 `INVALID_RESEARCH_OUTCOME_TYPE` si inválido)
- `summary` obligatorio no vacío (400 `INVALID_SUMMARY`)
- `UNIQUE(task_id)` — segundo `POST` para misma Task → 409 `RESEARCH_OUTCOME_ALREADY_EXISTS`
- `person_id` en listado filtra vía `JOIN research_tasks.person_id`
- Aislamiento por `tree_id`; cross-tree → 404
- `GET /research-tasks/:id` refleja `POST/PATCH/DELETE` outcome inmediatamente; `DELETE task` → `CASCADE` borra outcome

Ver `docs/RESEARCH_OUTCOMES.md` para modelo completo, tipos con ejemplos y workflow `Opportunity → Task → Outcome`.

## Evidence & Sources — Fase 4.0

```
GET    /api/v1/trees/1/sources?type=PARISH_RECORD&limit=20 -> {items, pagination}
GET    /api/v1/trees/1/sources/4 -> {id,tree_id,title,author,publication,date,type}
POST   /api/v1/trees/1/sources {title,author,publication,date,type} -> 201
PATCH  /api/v1/trees/1/sources/4 {title,type} -> 200
DELETE /api/v1/trees/1/sources/4 -> 204 (CASCADE citations/evidence)

GET    /api/v1/trees/1/sources/4/citations -> {items}
GET    /api/v1/trees/1/citations/12 -> {id,source_id,locator,text}
POST   /api/v1/trees/1/sources/4/citations {locator,text} -> 201
PATCH  /api/v1/trees/1/citations/12 {locator,text} -> 200
DELETE /api/v1/trees/1/citations/12 -> 204 (SET NULL en evidence)

GET    /api/v1/trees/1/evidence?limit=20 -> {items, pagination} (con source/citation embebidos)
GET    /api/v1/trees/1/evidence/10 -> {id,tree_id,source_id,citation_id,statement,notes,source{title,type},citation{locator}}
POST   /api/v1/trees/1/evidence {source_id,citation_id,statement,notes} -> 201
PATCH  /api/v1/trees/1/evidence/10 {statement,notes,citation_id} -> 200
DELETE /api/v1/trees/1/evidence/10 -> 204 (CASCADE outcome_evidence)

GET    /api/v1/trees/1/research-outcomes/5/evidence -> {items: [{id,relationship,statement,source{...},citation{...}}]}
POST   /api/v1/trees/1/research-outcomes/5/evidence/10 {relationship: SUPPORTS|CONTRADICTS} -> 201
DELETE /api/v1/trees/1/research-outcomes/5/evidence/10 -> 204

GET    /api/v1/trees/1/research-outcomes/5 -> 200 {id,type,summary,details,evidence: [{id,relationship,statement,source{citation}}], ...}
```

- `Source.type` ∈ BOOK,REGISTER,CENSUS,CIVIL_RECORD,PARISH_RECORD,NEWSPAPER,WEBSITE,OTHER (400 `INVALID_SOURCE_TYPE`)
- `Evidence.statement` obligatorio (400 `INVALID_STATEMENT`), `citation` debe pertenecer al `source`
- `relationship` ∈ SUPPORTS,CONTRADICTS (400 `INVALID_EVIDENCE_RELATIONSHIP`, duplicado 409 `EVIDENCE_ALREADY_ATTACHED`)
- Tree isolation en todos los niveles; cross-tree → 404
- `GET outcome` incluye `evidence:[]` sin N+1 (batch), `DELETE outcome` mantiene `Evidence` reutilizable

Ver `docs/EVIDENCE_SOURCES.md` para modelo `Source → Citation → Evidence → Outcome`.

## Evidence Assessment — Fase 4.1

```
GET /api/v1/trees/1/research-outcomes/5 → {evidence:[], evidence_assessment:{score,status,evidence_total,supporting_count,contradicting_count,sources_count,cited_count,uncited_count,reasons:[{code,points,message}]}}
GET /api/v1/trees/1/research-outcomes?assessment_status=NO_EVIDENCE&limit=20 → {items:[{evidence,evidence_assessment}], pagination}
GET /api/v1/trees/1/research/summary → {opportunities,tasks,outcomes,sources,evidence,assessment:{no_evidence,weak,mixed,supported,strongly_supported}}
```

- `EvidenceAssessment.status` ∈ NO_EVIDENCE,WEAK,MIXED,SUPPORTED,STRONGLY_SUPPORTED — ver `docs/EVIDENCE_ASSESSMENT.md` para fórmula y reglas.
- `score` 0..100 con `reasons` explicables (bonuses +30/+20/+15/+10/+10/+5, penalties -30/-15/-10, clamp 0..100).
- `assessment_status` filtra server-side sin N+1 (batch `GROUP BY`, 400 `INVALID_ASSESSMENT_STATUS` si inválido).
- `GET /research-outcomes` y `GET /research-outcomes/:id` coinciden con JSON real (`evidence_assessment` incluido).
- OpenAPI documenta `EvidenceAssessment`, `EvidenceAssessmentReason`, `EvidenceStats`, `assessment_status`.

Ver `docs/EVIDENCE_ASSESSMENT.md` para propósito ("¿Qué tan respaldada está esta conclusión?" vs "¿Es verdadera?") y diferencia Research Score vs Evidence Score.

## Evidence Gaps — Fase 4.2

```
GET /api/v1/trees/1/research-outcomes/5 → {evidence:[], evidence_assessment:{...}, evidence_gaps:[{code,severity,title,description}]}
GET /api/v1/trees/1/research-outcomes?gap=CONTRADICTORY_EVIDENCE&limit=20 → {items:[{evidence,evidence_assessment,evidence_gaps}], pagination}
GET /api/v1/trees/1/research-outcomes?assessment_status=MIXED&gap=SINGLE_SOURCE → combinado
GET /api/v1/trees/1/research/summary → {opportunities,tasks,outcomes,sources,evidence,assessment,evidence_gaps:{critical,warning,info}}
```

- `EvidenceGap.code` ∈ NO_SUPPORTING_EVIDENCE,NO_CITATION,SINGLE_SUPPORTING_EVIDENCE,CONTRADICTORY_EVIDENCE,SINGLE_SOURCE,CONFIRMED_WITHOUT_SUPPORT — ver `docs/EVIDENCE_GAPS.md`.
- `severity` ∈ INFO,WARNING,CRITICAL (CRITICAL: sin supporting, WARNING: contradicción / sin citation / single supporting, INFO: single source).
- `calculate_evidence_gaps(outcome_type, stats)` puro, sin persistencia; `CONFIRMED_WITHOUT_SUPPORT` reemplaza `NO_SUPPORTING_EVIDENCE` si CONFIRMED.
- `gap` filtra server-side sin N+1 (batch `GROUP BY` + gaps, 400 `INVALID_GAP_CODE` si inválido); combinable con `assessment_status`.
- `GET /research-outcomes` y `GET /research-outcomes/:id` coinciden con JSON real (`evidence_gaps` incluido, `[]` si ninguno).
- OpenAPI documenta `EvidenceGap`, `EvidenceGapSeverity`, `EvidenceGapCode`, `gap`.

Ver `docs/EVIDENCE_GAPS.md` para `Assessment vs Gaps` y ejemplos.

## Research Follow-ups — Fase 4.3

```
GET /api/v1/trees/1/research-outcomes/5 → {evidence:[], evidence_assessment:{...}, evidence_gaps:[...], research_followups:[{code,priority,title,description,gap_code}]}
GET /api/v1/trees/1/research-outcomes?limit=20 → {items:[{evidence,evidence_assessment,evidence_gaps,research_followups}], pagination}
GET /api/v1/trees/1/research/summary → {opportunities,tasks,outcomes,sources,evidence,assessment,evidence_gaps,research_followups:{high,medium,low}}
```

- `ResearchFollowUp.code` ∈ ADD_SUPPORTING_EVIDENCE,ADD_CITATION,REVIEW_CONTRADICTION,ADD_SECOND_SUPPORTING_EVIDENCE,REVIEW_SOURCE_COVERAGE — ver `docs/RESEARCH_FOLLOWUPS.md`.
- `priority` ∈ HIGH,MEDIUM,LOW (HIGH: `ADD_SUPPORTING_EVIDENCE`/`REVIEW_CONTRADICTION`, MEDIUM: `ADD_CITATION`/`ADD_SECOND_SUPPORTING_EVIDENCE`, LOW: `REVIEW_SOURCE_COVERAGE`).
- `calculate_research_followups(outcome_type, stats, gaps)` puro, determinista, sin persistencia; deduplica por `code` y ordena `HIGH→MEDIUM→LOW` estable.
- `GET /research-outcomes` y `GET /research-outcomes/:id` coinciden con JSON real (`research_followups` incluido, `[]` si ninguno, batch sin N+1).
- OpenAPI documenta `ResearchFollowUp`, `ResearchFollowUpCode`, `ResearchFollowUpPriority`, `research_followups`.

Ver `docs/RESEARCH_FOLLOWUPS.md` para `Gap → Follow-up`, prioridades y qué NO es un Follow-up.

## Research Follow-up Actions — Fase 4.4

```
POST /api/v1/trees/1/research-outcomes/5/followup-actions {followup_code, notes} → 201 {id,tree_id,task_id,outcome_id,followup_code,status:OPEN,notes,created_at,updated_at,completed_at:null} | 422 FOLLOWUP_NOT_ACTIVE
GET  /api/v1/trees/1/research-followup-actions?task_id=3&outcome_id=5&status=OPEN&followup_code=REVIEW_CONTRADICTION&limit=20 → Paginated
GET  /api/v1/trees/1/research-followup-actions/42 → {id,...,status,notes,completed_at}
PATCH /api/v1/trees/1/research-followup-actions/42 {status, notes} → 200 | 400 INVALID_FOLLOWUP_ACTION_STATUS
DELETE /api/v1/trees/1/research-followup-actions/42 → 204
GET  /api/v1/trees/1/research-tasks/3/followup-actions → Paginated
GET  /api/v1/trees/1/research-outcomes/5/followup-actions → {items:[...]}
GET  /api/v1/trees/1/research-outcomes/5 → {evidence,evidence_assessment,evidence_gaps,research_followups,followup_actions:[...]}
GET  /api/v1/trees/1/research-outcomes?limit=20 → items[].followup_actions_count (GROUP BY sin N+1)
GET  /api/v1/trees/1/research/summary → {followup_actions:{open,completed,skipped}}
```

- `followup_code` ∈ 5, `status` ∈ OPEN/COMPLETED/SKIPPED, `FOLLOWUP_NOT_ACTIVE` si el código no está activo para el outcome actual.
- `completed_at` se establece en `COMPLETED`/`SKIPPED` y se limpia en `OPEN`.
- `GET /research-outcomes/:id` incluye `followup_actions` históricas ordenadas `updated_at DESC`; list incluye `followup_actions_count`.
- Tree isolation: `task.tree_id==outcome.tree_id==action.tree_id`, cross-tree → 404.

Ver `docs/RESEARCH_FOLLOWUP_ACTIONS.md` para `Gap → Follow-up → Action` y `Action completed ≠ Gap resolved`.

## Research Case Summary — Fase 4.5

```
GET /api/v1/trees/:tree_id/research-tasks/:task_id/case-summary → 200 {task, person, opportunity, outcome, evidence_assessment, evidence_gaps, research_followups, followup_actions, timeline, closure_warnings}
```

- `task {id,title,description,status,resolution,created_at,started_at,completed_at,updated_at}` siempre presente; `person/opportunity/outcome/evidence_assessment` `null` si no existe; arrays nunca `null`.
- `timeline [{event_type,timestamp,label}]` derivado (`TASK_CREATED`, `TASK_STARTED`, `OUTCOME_CREATED`, `OUTCOME_UPDATED`, `FOLLOWUP_ACTION_CREATED`, `FOLLOWUP_ACTION_COMPLETED`, `TASK_COMPLETED`), orden `timestamp ASC`.
- `closure_warnings [{code,severity,title,description}]` códigos `RESOLVED_WITHOUT_OUTCOME(WARNING)`, `CONFIRMED_WITHOUT_SUPPORT(CRITICAL)`, `RESOLVED_WITH_EVIDENCE_GAPS(INFO)`, `REJECTED_WITH_CONFIRMED_OUTCOME(WARNING)`, `INCONCLUSIVE_WITH_CONFIRMED_OUTCOME(WARNING)` — puros, no bloquean.
- 404 solo `TASK_NOT_FOUND` (tree isolation); funciona para task sin outcome/evidence/gaps.
- Sin N+1, sin nuevo `CaseStatus`; `Task.status` sigue fuente de verdad.

Ver `docs/RESEARCH_CASE_SUMMARY.md`.

## Research Planning — Fase 5.0

```
GET /api/v1/trees/:tree_id/research/plan?limit=10&min_score=70&priority=high&researchability=high → 200 {generated_at, total_candidates, summary:{total_candidates,recommended_count,deferred_count,active_count,inconclusive_count,high_priority_count,critical_gap_count}, recommended:[{opportunity_id,person_id,title,priority,research_score,planning_score,researchability,confidence,active_task,task_status,reasons:[{code,label,description}]}], deferred:[...]}
```

- `limit` 1..100 default 10 (recomended size, `DEFAULT_PLAN_SIZE`), `min_score` 0..100 filtra `planning_score`, `priority` ∈ low/info/medium/warning/high/critical, `researchability` ∈ low/medium/high (400 `INVALID_*` si inválido)
- `planning_score = research_score*0.55 + researchability*0.20 + confidence*0.10 + evidence_gap*0.10 + task_state*0.05`, clamp 0..100, puro en `storage::planning` (sin SQL, sin persistencia)
- `researchability HIGH→100 MEDIUM→60 LOW→20`, `confidence*100`, `gap CRITICAL→100 WARNING→60 INFO→20 / max`, `task_state NO TASK→100 OPEN→80 IN_PROGRESS→40 INCONCLUSIVE→20 RESOLVED/REJECTED→0 (excluidas)`
- Ranking `planning_score DESC, research_score DESC, confidence DESC, opportunity_id ASC` determinista; `RESOLVED/REJECTED` excluidas, `INCONCLUSIVE` penalizada con `PREVIOUSLY_INCONCLUSIVE`, `OPEN/IN_PROGRESS` penalizadas con `ACTIVE_TASK`
- `reasons` códigos `HIGH_RESEARCH_SCORE,HIGH_RESEARCHABILITY,HIGH_CONFIDENCE,CRITICAL_EVIDENCE_GAP,WARNING_EVIDENCE_GAP,NO_ACTIVE_TASK,ACTIVE_TASK,PREVIOUSLY_INCONCLUSIVE` (+ `INFO_EVIDENCE_GAP`)
- Sin tabla `research_plans`; `GET /trees/:tree_id/research/plan` usa `get_research_planning_candidates(tree_id)` O(1) queries (opportunities + tasks + outcomes + gaps batch), tree isolation obligatoria
- OpenAPI `ResearchPlan`, `ResearchPlanItem`, `ResearchPlanSummary`, `ResearchPlanningReason`, `ResearchPlanningReasonCode`

Ver `docs/RESEARCH_PLANNING.md`.

## Errores

```json
{ "error": { "code": "TREE_NOT_FOUND", "message": "Tree 42 was not found" } }
```

Códigos: `400 Bad Request` (validación), `404 Not Found`, `500 Internal` (sin stack trace).

Validación cubre `tree_id/person_id/family_id >0`, `limit 0..100`, `offset>=0`, enums.

## Seguridad

- Queries parametrizadas (sqlx bind), sin concatenación.
- Sin CORS `*` por defecto; `NEOGENEALOGY_CORS_ORIGIN` permite configurar.
- No se exponen rutas internas ni secretos.
- Solo `GET` (read-only).

## OpenAPI

```
GET /api/v1/openapi.json
GET /api/v1/docs (Swagger placeholder)
```

## Ejemplos curl

```bash
curl http://127.0.0.1:3000/health
curl http://127.0.0.1:3000/api/v1/trees
curl http://127.0.0.1:3000/api/v1/trees/1/persons?limit=5
curl 'http://127.0.0.1:3000/api/v1/trees/1/findings?severity=high'
curl 'http://127.0.0.1:3000/api/v1/trees/1/research-opportunities?priority=high&sort=score&limit=20'
curl http://127.0.0.1:3000/api/v1/trees/1/research-opportunities/top
```

## Serve

```bash
neogenealogy serve --db neogenealogy.db --host 127.0.0.1 --port 3000
# env: NEOGENEALOGY_HOST, NEOGENEALOGY_PORT, NEOGENEALOGY_DATABASE_URL=sqlite://...
```
