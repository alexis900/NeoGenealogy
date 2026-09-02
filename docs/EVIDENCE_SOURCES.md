# Evidence & Sources — Fase 4.0

## Modelo conceptual

```
Research Opportunity
        ↓
Research Task
        ↓
Research Outcome
        ↓
Evidence  (qué encontramos)
        ↓
Source + Citation (de dónde procede)
```

Una `Source` describe **de dónde** procede la información (libro, registro parroquial, censo).  
Una `Citation` indica **dónde exactamente** dentro de la fuente (folio 42, página 18).  
Una `Evidence` registra **qué encontramos** (statement + notes) y referencia `Source` (obligatorio) y `Citation` (opcional).  
Un `Outcome` representa **qué concluimos** y puede estar respaldado por múltiples `Evidence` con relación `SUPPORTS` o `CONTRADICTS`.

```
Source 1 ──* Citation
   │            │
   └────*───────┘
          │
       Evidence * ──* OutcomeEvidence (SUPPORTS|CONTRADICTS) ──* Outcome
```

- `Source` puede tener N `Citation` y N `Evidence`
- `Citation` puede tener N `Evidence`
- `Outcome` puede tener N `Evidence` (via `outcome_evidence`)
- `Evidence` es reutilizable entre Outcomes

## Storage

Migración: `crates/storage/migrations/004_evidence_sources.sql`

Nota: `sources`/`citations` GEDCOM ya existían en `001_initial.sql`. Para no colisionar, las tablas de investigación usan `research_sources`/`research_citations`; `evidence`/`outcome_evidence` mantienen el nombre del spec.

```sql
research_sources(id, tree_id FK CASCADE, title NOT NULL, author, publication, date, type CHECK(BOOK...OTHER), created_at, updated_at)
research_citations(id, source_id FK CASCADE, locator, text, created_at, updated_at)
evidence(id, tree_id FK CASCADE, source_id FK CASCADE, citation_id FK SET NULL, statement NOT NULL, notes, created_at, updated_at)
outcome_evidence(outcome_id FK CASCADE, evidence_id FK CASCADE, relationship CHECK(SUPPORTS|CONTRADICTS), PK(outcome_id,evidence_id))
```

Indices: `research_sources(tree_id,type)`, `research_citations(source_id)`, `evidence(tree_id,source_id,citation_id)`, `outcome_evidence(outcome_id,evidence_id)`.

Isolación por `tree_id`: `Evidence.tree_id` y `research_sources.tree_id` deben coincidir; `Citation` hereda tree via `source`; `OutcomeEvidence` valida que `outcome.tree_id == evidence.tree_id`.

Cascadas:
- Delete `Source` → `Citation` CASCADE, `Evidence` CASCADE (via source), `outcome_evidence` via evidence.
- Delete `Citation` → `Evidence.citation_id` SET NULL.
- Delete `Evidence` → `outcome_evidence` CASCADE.
- Delete `Outcome` → `outcome_evidence` CASCADE, `Evidence` permanece (reutilizable).

## Tipos

`Source.type`:
`BOOK, REGISTER, CENSUS, CIVIL_RECORD, PARISH_RECORD, NEWSPAPER, WEBSITE, OTHER` (validado 400 `INVALID_SOURCE_TYPE`)

`outcome_evidence.relationship`:
`SUPPORTS, CONTRADICTS` (validado 400 `INVALID_EVIDENCE_RELATIONSHIP`, duplicado 409 `EVIDENCE_ALREADY_ATTACHED`)

`Evidence.statement` obligatorio no vacío (`400 INVALID_STATEMENT`).

## API

### Sources
```
GET    /api/v1/trees/:tree_id/sources?type=PARISH_RECORD&limit=20&offset=0 -> {items, pagination}
GET    /api/v1/trees/:tree_id/sources/:source_id -> {id,tree_id,title,author,publication,date,type,created_at,updated_at}
POST   /api/v1/trees/:tree_id/sources {title,author,publication,date,type} -> 201
PATCH  /api/v1/trees/:tree_id/sources/:source_id {title,author,publication,date,type} -> 200
DELETE /api/v1/trees/:tree_id/sources/:source_id -> 204
```
Errores: `400 INVALID_SOURCE_TYPE/INVALID_TITLE`, `404 SOURCE_NOT_FOUND/TREE_NOT_FOUND`, cross-tree `404`.

### Citations
```
GET    /api/v1/trees/:tree_id/sources/:source_id/citations?limit=&offset= -> {items, pagination}
GET    /api/v1/trees/:tree_id/citations/:citation_id -> {id,source_id,locator,text,created_at,updated_at}
POST   /api/v1/trees/:tree_id/sources/:source_id/citations {locator,text} -> 201
PATCH  /api/v1/trees/:tree_id/citations/:citation_id {locator,text} -> 200
DELETE /api/v1/trees/:tree_id/citations/:citation_id -> 204
```
Validación tree via source.

### Evidence
```
GET    /api/v1/trees/:tree_id/evidence?limit=&offset= -> {items, pagination} (con source/citation embebidos)
GET    /api/v1/trees/:tree_id/evidence/:evidence_id -> {id,tree_id,source_id,citation_id,statement,notes,source{title,type},citation{locator}}
POST   /api/v1/trees/:tree_id/evidence {source_id,citation_id,statement,notes} -> 201
PATCH  /api/v1/trees/:tree_id/evidence/:evidence_id {statement,notes,citation_id} -> 200
DELETE /api/v1/trees/:tree_id/evidence/:evidence_id -> 204
```
Validación: `source` debe pertenecer al tree, `citation` debe pertenecer al source.

### Outcome Evidence
```
GET    /api/v1/trees/:tree_id/research-outcomes/:outcome_id/evidence -> {items: [{id,relationship,statement,notes,source{...},citation{...},created_at,updated_at}]}
POST   /api/v1/trees/:tree_id/research-outcomes/:outcome_id/evidence/:evidence_id {relationship} -> 201
DELETE /api/v1/trees/:tree_id/research-outcomes/:outcome_id/evidence/:evidence_id -> 204
```
Codes: `409 EVIDENCE_ALREADY_ATTACHED`, `400 INVALID_EVIDENCE_RELATIONSHIP`, `404 OUTCOME_NOT_FOUND/EVIDENCE_NOT_FOUND` (cross-tree oculto como 404).

### Outcome enriquecido
```
GET /api/v1/trees/:tree_id/research-outcomes/:outcome_id
-> {id,type,summary,details,created_at,updated_at,evidence: [{id,relationship,statement,source{...},citation{...}}]}
```
Si no hay evidencia: `"evidence":[]`. Evita N+1 via batch `list_outcome_evidence_detailed`.

## Web

**Navegación** (`web/src/components/Layout.tsx`):
```
Dashboard
Research
  ├─ Overview
  ├─ Opportunities
  ├─ Tasks
  └─ History
Sources        -> /trees/:treeId/sources (ResearchSources)
Evidence       -> /trees/:treeId/evidence
Persons / Findings / Branches / Coverage
```

Rutas (`web/src/App.tsx`):
```
/trees/:treeId/sources -> ResearchSources (list + Create Source)
/trees/:treeId/sources/:sourceId -> SourceDetail (edit/delete + Citations)
/trees/:treeId/evidence -> Evidence list + Create Evidence
/trees/:treeId/evidence/:evidenceId -> EvidenceDetail
/trees/:treeId/research/tasks/:taskId -> ResearchTaskDetail integra Evidence
```

**Sources page**: tabla Title/Type/Author/Date/Publication, filtros type, paginación, create/edit/delete. Mensajes empty/error/loading.

**SourceDetail**: muestra Source + lista Citations, permite crear/editar/eliminar Citation sin adjuntos.

**Evidence**: list, create con Source select (fetch sources), Citation select (fetch citations), statement, notes. Muestra source/citation embebidos.

**ResearchTaskDetail — Evidence**:
```
Research Outcome
  Type CONFIRMED
  Summary ...
  Evidence (N attached)
    ✓ SUPPORTS  Registro parroquial — folio 42 — "La partida identifica..."
    ⚠ CONTRADICTS ...
  + Add Evidence -> [Source dropdown] [Citation dropdown] [SUPPORTS|CONTRADICTS] [Statement] [Notes] -> createEvidence + attach
  Edit Outcome
```
Distingue visualmente SUPPORTS (emerald) vs CONTRADICTS (orange). Reutilizable, no deduplica. Remove desconecta relación (`DELETE outcome_evidence`) sin borrar Evidence.

**History**: muestra `Evidence: N` por outcome (fetch `GET outcome evidence` paralelo). Orden `created_at DESC`.

**Workspace**: métrica `Evidence recorded: X · Sources: Y` desde `GET /research/summary` (`sources.total`, `evidence.total`).

## Tests

- Storage `evidence_sources.rs`: CRUD sources/citations/evidence, validation, tree isolation, pagination, type filter, cascade, outcome_evidence (SUPPORTS/CONTRADICTS, duplicate 409, cross-tree, reuse).
- API `evidence_sources.rs`: 9 tests cubren todos endpoints, status codes, JSON, pagination, filtros, cross-tree, duplicate, invalid type/relationship, outcome enrichment, cascade.

## Verificación

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
npm --prefix web run build
npm --prefix web run test
docker compose build && docker compose up -d
# E2E: Opportunity -> Task -> Outcome -> Source -> Citation -> Evidence -> SUPPORTS -> Outcome visible -> History -> detach -> Evidence remains
```

Evidence no modifica árbol ni GEDCOM, permanece como registro estructurado.
