# Web UI

React + TypeScript + Vite + Tailwind, consume exclusivamente `GET /api/v1/*`.

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

`web/src/api/client.ts` centraliza `BASE = import.meta.env.VITE_API_BASE_URL || "http://127.0.0.1:3000"` y `ApiError{code,status,message}`. Todas las páginas usan:

```
getTrees(), getTree(id), getPersons(treeId,{limit,offset}), getPerson(treeId,personId),
getFamilies, getFamily, getFindings({severity,type,person_id}), 
getResearchOpportunities({priority,min_score,sort}), getTopResearchOpportunities,
getBranches, getSourceCoverage, getAnalysisRuns
```

No hay `fetch()` disperso.

## Tipos

`web/src/api/types.ts` refleja exactamente DTOs de `docs/API.md`:

`TreeSummary, Person, Family, Finding, ResearchOpportunity {score,confidence,priority,researchability,why,what,potential_sources,breakdown:{total,components[]}}, Branch, SourceCoverage, AnalysisRun, Paginated<T>, ApiError`

No recalcula score/confidence.

## Routing

```
/              → Trees (selección)
/trees         → Trees
/trees/:treeId → Dashboard (overview + top 5)
/trees/:treeId/research → Research Queue (filtros priority/min_score/sort)
/trees/:treeId/research/:oppId → Opportunity Detail (ScoreBreakdown)
/trees/:treeId/persons → Persons list (paginado)
/trees/:treeId/persons/:personId → Person Detail (findings + opps)
/trees/:treeId/findings → Findings (severity/type)
/trees/:treeId/branches → Branches (branch_score)
/trees/:treeId/sources → SourceCoverage (barras Birth/Marriage/Death/Other/Overall)
```

Layout `web/src/components/Layout.tsx` con sidebar. TreeId se propaga por URL, no se asume `1`.

## Páginas clave

- **Dashboard**: `getTree` + `getTop(limit=5)` → overview + `Ver toda la cola →`
- **Research Queue**: `getResearchOpportunities` con filtros `Priority, Sort(score/priority/confidence), min_score` → `ResearchOpportunityCard`, `ScoreBadge`, `PriorityBadge`, `ResearchabilityBadge`, `ConfidenceIndicator`; Empty: "No research opportunities found."
- **Opportunity Detail**: `ScoreBreakdown` muestra `+30 Direct ancestor — reason` explicable.
- **Person Detail**: `getPerson` + `getFindings?person_id` + `getOpportunities` filtrado cliente (evita N+1).
- **Branches**: tabla cards ordenada por `branch_score`.
- **Sources**: `Bar` con `value%` exacto de API.

Todos los listados usan `limit/offset` (max 100) y `Pagination`.

## Componentes

`ResearchOpportunityCard, ScoreBadge, PriorityBadge, ConfidenceIndicator, ResearchabilityBadge, ScoreBreakdown, FindingBadge, SourceCoverageBar, BranchCard, PersonSummary, Pagination, Loading, ErrorState, Empty`

## Estados

Cada página: `Loading…`, `Success`, `Empty`, `Error` con `Retry`, sin stack traces.

## Tests

- `web/src/api/__tests__/client.test.ts` (ApiError, BASE)
- `web/src/components/__tests__/Badges.test.tsx` (Priority/Score/Confidence)
- `web/src/components/__tests__/OpportunityCard.test.tsx` (card + ScoreBreakdown)
- `web/src/components/__tests__/common.test.tsx` (Loading/Empty/Error/Pagination)

```bash
npm run test # vitest run → 11 tests
```

## Limitaciones

Solo lectura, sin edición, sin árbol visual, sin móvil completo, sin auth, sin FamilySearch. Research Queue es el centro.

## Estructura

```
web/
 src/api/{client.ts,types.ts}
 src/components/{Badges,common,OpportunityCard,Layout}
 src/pages/{Trees,Dashboard,ResearchQueue,OpportunityDetail,Persons,PersonDetail,Findings,Branches,Sources}
 vite.config.ts (proxy), tailwind.config.js, postcss.config.js
```
