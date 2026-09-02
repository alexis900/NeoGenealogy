# Web UI

React + TypeScript + Vite + Tailwind, consume `GET /api/v1/*` y `POST/PATCH/DELETE` para Research Tasks y Outcomes.

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
getTasks({status,person_id,opportunity_id}), getTask, createTask, createTaskFromOpportunity, updateTask, deleteTask,
getOutcomes({type,task_id,person_id}), getOutcome, createOutcome, updateOutcome, deleteOutcome
```

No hay `fetch()` disperso.

## Tipos

`web/src/api/types.ts` refleja exactamente DTOs de `docs/API.md`:

`TreeSummary, Person, Family, Finding, ResearchOpportunity {score,confidence,priority,researchability,why,what,potential_sources,breakdown:{total,components[]}}, ResearchTask {id,tree_id,opportunity_id,person_id,title,description,status,created_at,updated_at,started_at,completed_at,resolution,outcome}, ResearchOutcome {id,tree_id,task_id,type,summary,details,created_at,updated_at}, OutcomeType, Branch, SourceCoverage, AnalysisRun, Paginated<T>, ApiError`

No recalcula score/confidence.

## Routing

```
 /              → Trees (selección)
 /trees         → Trees
 /trees/:treeId → Dashboard (overview + top 5 + Research Tasks summary)
 /trees/:treeId/research → Research Queue (filtros priority/min_score/sort)
 /trees/:treeId/research/tasks → Research Tasks (filtros status, paginación)
 /trees/:treeId/research/tasks/:taskId → Research Task Detail (edición status/resolution + Outcome Record/Edit/Delete)
 /trees/:treeId/research/:oppId → Opportunity Detail (ScoreBreakdown + Start Research)
 /trees/:treeId/persons → Persons list (paginado)
 /trees/:treeId/persons/:personId → Person Detail (findings + opps + tasks)
 /trees/:treeId/findings → Findings (severity/type)
 /trees/:treeId/branches → Branches (branch_score)
 /trees/:treeId/sources → SourceCoverage (barras Birth/Marriage/Death/Other/Overall)
 ```

Layout `web/src/components/Layout.tsx` con sidebar. TreeId se propaga por URL, no se asume `1`.

## Páginas clave

- **Dashboard**: `getTree` + `getTop(limit=5)` + `getTasks(limit=100)` → overview + Research Tasks summary (Open/In Progress/Resolved) + `Ver toda la cola →`
- **Research Queue**: `getResearchOpportunities` con filtros `Priority, Sort(score/priority/confidence), min_score` → `ResearchOpportunityCard`; distinción `Research Queue` (automático) vs `Research Tasks` (humano).
- **Opportunity Detail**: `ScoreBreakdown` + `Start Research` (`POST /research-opportunities/:id/tasks`) o `View Research Task` si ya existe.
- **Research Tasks**: `getTasks` con filtros `status/person_id/opportunity_id`, paginación; `ResearchTasks.tsx` lista.
- **ResearchTask Detail**: `getTask`/`updateTask`/`deleteTask` → edita `title/description/status/resolution`; sección `Research Outcome` → `getTask.outcome` embebido, sin outcome muestra `Record Outcome` (type/summary/details), con outcome muestra badge + summary/details + `Edit Outcome` / `Delete Outcome`; muestra `Original Research Opportunity` con `ScoreBreakdown` (persiste tras Outcome CRUD).
- **Person Detail**: `getPerson` + `getFindings?person_id` + `getOpportunities` + `getTasks?person_id` → muestra tasks asociadas.
- **Branches**: tabla cards ordenada por `branch_score`.
- **Sources**: `Bar` con `value%` exacto de API.

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
- `web/src/pages/__tests__/ResearchTasks.test.tsx` (list, filters, empty)
- `web/src/pages/__tests__/ResearchTaskDetail.test.tsx` (detail, update, outcome: create/edit/delete, 5 tipos, no duplicate form, Original Opportunity persiste, error/retry, 204 handling)

```bash
npm run test # vitest run → 27 tests
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
