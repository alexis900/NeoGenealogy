# Web UI

React + TypeScript + Vite + Tailwind, consume `GET /api/v1/*` y `POST/PATCH/DELETE` para Research Tasks, Outcomes, Sources/Citations/Evidence.

## Stack

- React 19, React Router 7
- Vite 8 (dev HMR, proxy `/api` → 3000, build)
- Tailwind CSS 3, PostCSS, autoprefixer
- Vitest + Testing Library (jsdom)

No Redux/Zustand/TanStack/GraphQL/SSR/auth.

## Instalación / Desarrollo

```bash
# Terminal 1: DB + análisis
cargo run -p neogenealogy -- import test-data/complex.ged --db /tmp/neogenealogy.db

# Terminal 2: API
cargo run -p neogenealogy -- serve --db /tmp/neogenealogy.db --host 127.0.0.1 --port 3000
# env: NEOGENEALOGY_HOST, NEOGENEALOGY_PORT, NEOGENEALOGY_DATABASE_URL, NEOGENEALOGY_CORS_ORIGIN

# Terminal 3: Web
cd web
npm install
# configurar API
echo "VITE_API_BASE_URL=http://127.0.0.1:3000" > .env
npm run dev      # http://localhost:5173 (proxy /api → 3000)
npm run build    # tsc -b && vite build → dist/
npm run test     # vitest run
npm run preview  # vite preview
```

## Configuración API

`web/src/api/client.ts` centraliza `BASE = import.meta.env.VITE_API_BASE_URL || ""` (usa proxy Vite `/api→3000` en dev, mismo origen en prod) y `ApiError{code,status,message}`. Maneja `204` → `undefined` (sin `JSON.parse` body vacío). Todas las páginas usan:

```
getTrees(), getTree(id), getPersons(treeId,{limit,offset}), getPerson(treeId,personId),
getFamilies, getFamily, getFindings({severity,type,person_id}), 
getResearchOpportunities({priority,min_score,sort}), getTopResearchOpportunities,
getBranches, getSourceCoverage, getAnalysisRuns,
getTasks({status,person_id,opportunity_id,has_outcome}), getTask, createTask, createTaskFromOpportunity, updateTask, deleteTask,
getOutcomes({type,task_id,person_id,assessment_status,gap}), getOutcome, createOutcome, updateOutcome, deleteOutcome,
getSources({type}), getSource, createSource, updateSource, deleteSource,
getCitations, getCitation, createCitation, updateCitation, deleteCitation,
getEvidenceList, getEvidence, createEvidence, updateEvidence, deleteEvidence,
  getOutcomeEvidence, attachEvidence, detachEvidence,
getResearchSummary, getCaseSummary
```

No hay `fetch()` disperso.

## Tipos

`web/src/api/types.ts` refleja exactamente DTOs de `docs/API.md`:

`TreeSummary, Person, Family, Finding, ResearchOpportunity {score,confidence,priority,researchability,why,what,potential_sources,breakdown:{total,components[]}}, ResearchTask {id,tree_id,opportunity_id,person_id,title,description,status,created_at,updated_at,started_at,completed_at,resolution,outcome,has_outcome,opportunity}, ResearchOutcome {id,tree_id,task_id,type,summary,details,created_at,updated_at,evidence,evidence_assessment,evidence_gaps,research_followups}, OutcomeType, AssessmentStatus, EvidenceAssessment {score,status,evidence_total,supporting_count,contradicting_count,sources_count,cited_count,uncited_count,cited_supporting_count,reasons}, EvidenceAssessmentReason, EvidenceGap {code,severity,title,description}, GapCode, GapSeverity, ResearchFollowUp {code,priority,title,description,gap_code}, FollowUpCode, FollowUpPriority, ResearchSource, ResearchCitation, Evidence, EvidenceWithRelationship, ResearchCaseSummary {task,person,opportunity,outcome,evidence_assessment,evidence_gaps,research_followups,followup_actions,timeline,closure_warnings}, ResearchCaseTimelineEvent, ResearchCaseClosureWarning, ClosureWarningCode, ClosureWarningSeverity, Branch, SourceCoverage, AnalysisRun, Paginated<T>, ApiError`

No recalcula score/confidence.

## Routing

```
 /              → Trees (selección)
 /trees         → Trees
 /trees/:treeId → Dashboard (overview + top 5 + Research Tasks summary)
 /trees/:treeId/research → Research Workspace (Overview: Opportunities/Active Tasks/Recent Outcomes + evidence/sources metrics)
 /trees/:treeId/research/opportunities → Research Queue (filtros priority/min_score/sort)
 /trees/:treeId/research/tasks → Research Tasks (filtros status/has_outcome/person/opportunity, paginación, has_outcome badge)
 /trees/:treeId/research/tasks/:taskId → Research Task Detail (workflow actions + Outcome + Evidence SUPPORTS/CONTRADICTS)
 /trees/:treeId/research/history → Research History (outcomes + evidence count)
 /trees/:treeId/research/:oppId → Opportunity Detail (ScoreBreakdown + Start Research)
 /trees/:treeId/sources → Research Sources (lista + Create/Edit/Delete)
 /trees/:treeId/sources/:sourceId -> SourceDetail (Citations)
 /trees/:treeId/evidence -> Evidence (lista + Create)
 /trees/:treeId/evidence/:evidenceId -> EvidenceDetail
 /trees/:treeId/persons → Persons list (paginado)
 /trees/:treeId/persons/:personId → Person Detail (findings + opps + tasks + Research this person)
 /trees/:treeId/findings → Findings (severity/type)
 /trees/:treeId/branches → Branches (branch_score)
 /trees/:treeId/coverage -> SourceCoverage (barras Birth/Marriage/Death/Other/Overall) (alias /source-coverage)
 ```

Layout `web/src/components/Layout.tsx` con sidebar. TreeId se propaga por URL, no se asume `1`.

## Páginas clave

- **Dashboard**: `getTree` + `getTop(limit=5)` + `getTasks(limit=100)` → overview + Research Tasks summary (Open/In Progress/Resolved) + `Ver toda la cola →`
- **Research Queue**: `getResearchOpportunities` con filtros `Priority, Sort(score/priority/confidence), min_score` → `ResearchOpportunityCard`; distinción `Research Queue` (automático) vs `Research Tasks` (humano).
- **Opportunity Detail**: `ScoreBreakdown` + `Start Research` (`POST /research-opportunities/:id/tasks`) o `View Research Task` si ya existe.
- **Research Workspace**: `getResearchSummary` + `getTasks(limit 5)` + `getOutcomes(limit 5)` → 3 bloques Opportunities (high/medium/low) + Active Tasks (OPEN/IN_PROGRESS) + Recent Outcomes (con gaps) + métricas `Evidence/Sources` + `Evidence Assessment` + `Evidence Gaps` + `Research Follow-ups` + `Follow-up Actions` (Open/Completed/Skipped desde mismo summary, sin queries extra).
- **Research Tasks**: `getTasks` con filtros `status/has_outcome/person_id/opportunity_id` combinables, orden `IN_PROGRESS>OPEN>updated_at`, cards con `has_outcome` y `opportunity{score,priority}`.
- **ResearchTask Detail**: `getTask`/`updateTask`/`deleteTask` + workflow `Start/Mark Resolved/Rejected/Inconclusive`; sección `Research Outcome` + `Evidence Assessment` (status/score/counts, `Why this assessment?`, warnings) + `Evidence Gaps` (CRITICAL/WARNING/INFO, títulos/descripciones, quick actions `Add Evidence`/`Review Evidence`/`Review Contradictions`, empty `No evidence gaps detected.`) + `Research Follow-ups` (HIGH/MEDIUM/LOW, `Start follow-up`/`Add Evidence`/`Review Evidence`, oculto si `[]`, muestra `Gap: CODE`) + `Research Follow-up Actions` (lista `updated_at DESC` con `followup_code`, `Status OPEN/COMPLETED/SKIPPED`, `Notes`, botones `Mark completed`/`Skip`/`Reopen`/`Delete`, `✓ Completed — Action completed, not gap resolved`) + `Evidence` (SUPPORTS/CONTRADICTS) + `Research Case Summary` (Status/Resolution/Outcome/Evidence/Assessment/Gaps/Follow-ups/Actions + `Closure Warnings` CRITICAL/WARNING/INFO + `Timeline` ordenada + `Case Closure` cuando terminal) + `Original Research Opportunity`.
- **Research History**: `getOutcomes(limit 20, assessment_status, gap)` → tabla Date/Type/Summary/Task + `Evidence: N` + `Assessment: STATUS · score` + `Gaps: 1 warning` + `Follow-ups: 2` + `Follow-up actions: 2` (vía `followup_actions_count` sin N+1) + `View Case Summary` por outcome + filtros, paginación.
- **ResearchSources**: `getSources` con type filter, create/edit/delete.
- **SourceDetail**: `getSource` + `getCitations` con create/delete citation.
- **Evidence**: `getEvidenceList` + create con Source/Citation.
- **Person Detail**: `getPerson` + `getFindings?person_id` + `getOpportunities` + `getTasks?person_id` + `Research this person` (crea Task manual).
- **Branches**: tabla cards ordenada por `branch_score`.
- **Coverage**: `Bar` con `value%` exacto de API.

Todos los listados usan `limit/offset` (max 100) y `Pagination`.

## Componentes

`ResearchOpportunityCard, TaskStatusBadge, ScoreBadge, PriorityBadge, ConfidenceIndicator, ResearchabilityBadge, ScoreBreakdown, FindingBadge, SourceCoverageBar, BranchCard, PersonSummary, Pagination, Loading, ErrorState, Empty`

## Estados

Cada página: `Loading…`, `Success`, `Empty`, `Error` con `Retry`, sin stack traces.

## Tests

- `web/src/api/__tests__/client.test.ts` (ApiError, BASE)
- `web/src/components/__tests__/Badges.test.tsx` (Priority/Score/Confidence)
- `web/src/components/__tests__/TaskStatusBadge.test.tsx` (TaskStatus)
- `web/src/components/__tests__/OpportunityCard.test.tsx` (card + ScoreBreakdown)
- `web/src/components/__tests__/common.test.tsx` (Loading/Empty/Error/Pagination)
- `web/src/components/__tests__/Layout.test.tsx` (Research nav)
- `web/src/pages/__tests__/ResearchWorkspace.test.tsx` (overview, empty, error, gaps metrics)
- `web/src/pages/__tests__/ResearchHistory.test.tsx` (history, filters, assessment/gaps visible, gap filter)
- `web/src/pages/__tests__/ResearchTasks.test.tsx` (list, filters combinados, cards, outcome badge, has_outcome)
- `web/src/pages/__tests__/ResearchTaskDetail.test.tsx` (detail, outcome: create/edit/delete, evidence SUPPORTS/CONTRADICTS, assessment statuses, gaps CRITICAL/WARNING/INFO, quick actions)
- `web/src/pages/__tests__/ResearchSources.test.tsx` (list, create, empty, error)
- `web/src/pages/__tests__/SourceDetail.test.tsx` (detail, citations)
- `web/src/pages/__tests__/Evidence.test.tsx` (list, create)

```bash
npm run test # vitest run → 69 tests
```

## Limitaciones

Solo lectura, sin edición, sin árbol visual, sin móvil completo, sin auth, sin FamilySearch. Research Queue es el centro.

## Estructura

```
web/
  src/api/{client.ts,types.ts}  # + getTasks/createTask/updateTask/deleteTask + getOutcomes/createOutcome/updateOutcome/deleteOutcome (204)
  src/components/{Badges,common,OpportunityCard,Layout}
  src/pages/{Trees,Dashboard,ResearchQueue,OpportunityDetail,ResearchTasks,ResearchTaskDetail,Persons,PersonDetail,Findings,Branches,Sources}
  vite.config.ts (proxy /api→3000), tailwind.config.js, postcss.config.js
```
