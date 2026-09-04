# External Research Core — Phase 6.0

> **External Research finds candidates. The researcher decides what constitutes evidence.**

## Concepto

```
Research Opportunity
        ↓
Research Planning
        ↓
Research Session
        ↓
Research Task (unidad de trabajo)
        ↓
Research Query (intención: "quiero buscar esto")
        ↓
Research Query Execution (ejecución concreta)
        ↓
Research Provider (abstracción)
        ↓
Research Results (candidatos, ordenados)
        ↓
User Review
        ↓
Evidence (futura fase)
        ↓
Evidence Assessment → Research Outcome
```

Invariante crítico: `ResearchResult ≠ Evidence`. Un resultado externo es solo un candidato.

## Modelos

### ResearchQuery

```sql
research_queries(id, tree_id FK CASCADE, task_id FK CASCADE, provider, query, status PENDING|RUNNING|COMPLETED|FAILED, created_at, started_at, completed_at, error_code, error_message)
```

- Asociada a `ResearchTask`, no directamente a Opportunity.
- `status` refleja último estado de ejecuciones; `PENDING` al crear (no ejecuta automáticamente).
- `provider` normalizado a lowercase (solo `mock` en 6.0).

### ResearchQueryExecution

```sql
research_query_executions(id, query_id FK CASCADE, status, started_at, completed_at, error_code, error_message, provider_request_id, provider_metadata JSON, created_at)
```

- Permite re-runs sin destruir historial: `Query → Execution #1 → Results`, `Execution #2 → Results`.
- `provider_metadata` y `provider_request_id` no almacenan secretos.

### ResearchResult

```sql
research_results(id, execution_id FK CASCADE, query_id FK CASCADE, provider, external_id, title, description, url, record_type, date, place, metadata JSON, position, created_at)
```

- `position` conserva orden del provider, sin ranking propio.
- `metadata` genérico JSON para datos específicos del provider sin contaminar dominio.
- URLs solo `http/https`, validadas; no `javascript:`, `data:`, `file:`.
- No se convierte automáticamente en `Source/Citation/Evidence/Outcome`.

Relación: `research_query_executions` → `research_results` FK `ON DELETE CASCADE`; borrar query borra ejecuciones y resultados.

## Estados y semántica

| Estado | Significado |
|--------|-------------|
| PENDING | Query existe, no ejecutada |
| RUNNING | Provider ejecutando |
| COMPLETED | Provider respondió correctamente (0..N results) |
| FAILED | No se pudo completar |

Regla: `COMPLETED + 0 results` = búsqueda exitosa sin resultados. `FAILED` nunca es `NO_EVIDENCE`.

Re-run: `COMPLETED → RUNNING` y `FAILED → RUNNING` crean nueva `Execution` y nuevos `Results`; historial se preserva.

## Provider abstracción

```rust
#[async_trait::async_trait]
pub trait ResearchProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn search(&self, query: &str) -> Result<ResearchProviderResponse, ProviderError>;
}

pub struct ResearchProviderResponse {
    pub provider: String,
    pub results: Vec<ResearchResultCandidate>,
    pub provider_request_id: Option<String>,
    pub provider_metadata: serde_json::Value,
}
```

Errores normalizados: `NO_RESULTS, PROVIDER_UNAVAILABLE, AUTH_REQUIRED, RATE_LIMITED, INVALID_QUERY, TIMEOUT, UNKNOWN`. La API diferencia `provider failure` de `successful with zero results`.

No acoplado a HTTP.

### MockResearchProvider

Determinista, sin Internet:

- `query` contiene `fail`/`error` → `PROVIDER_UNAVAILABLE`
- `rate_limited` → `RATE_LIMITED`
- `timeout` → `TIMEOUT`
- `auth` → `AUTH_REQUIRED`
- `no-results` / `empty` → 0 results
- otro → 2 results (`Baptism` + `Census`, orden fijo, URLs `https://example.com/...`)

Permite probar todos los flujos.

### Registry

```rust
ResearchProviderRegistry::new() // contiene "mock"
registry.get("mock") -> Arc<dyn ResearchProvider>
```

Fácil sustituir por `FamilySearchProvider` sin tocar storage/API/domain.

## API

```
POST   /api/v1/trees/:tree_id/research-tasks/:task_id/research-queries  {provider, query} → 201 PENDING
GET    /api/v1/trees/:tree_id/research-queries?task_id=&provider=&status=&limit=&offset=
GET    /api/v1/trees/:tree_id/research-queries/:query_id  (incluye latest_execution + result_count)
DELETE /api/v1/trees/:tree_id/research-queries/:query_id  (cascade)
POST   /api/v1/trees/:tree_id/research-queries/:query_id/run  → Execution (COMPLETED/FAILED)
GET    /api/v1/trees/:tree_id/research-queries/:query_id/executions  (paginado)
GET    /api/v1/trees/:tree_id/research-queries/:query_id/results  (latest execution, paginado, orden por position)
GET    /api/v1/trees/:tree_id/research-results/:result_id
```

Además rutas genéricas sin `tree_id` (`/api/v1/research-tasks/:task_id/research-queries`, `/api/v1/research-queries/:id/...`) para compatibilidad; todas validan `tree_id` del query/task.

Errores: formato `{error:{code,message}}`, sin filtrar secretos/stack traces. `INVALID_PROVIDER` 400, `PROVIDER_UNAVAILABLE` se persiste como `FAILED` con `error_code`.

Tree isolation: todo listado/validadores comprueban `row.tree_id == path tree_id`.

## Frontend

- `Task Detail` → bloque **External Research**: selector `Mock`, input query, `Create Query`, `Suggest Query`, lista `Queries` con `RUN/Run Again/View Results/Detail`, resultados con tarjeta `Baptism record — ...`, badge `External Research Result` + `This result is not evidence.` + `Possible matching record` + `Open external source` + `Review Result`.
- `Query Detail` (`/trees/:treeId/research/queries/:queryId`): query, provider, status, `latest_execution`, lista executions (re-runs), lista results (pos sort), `Run Again`.
- `Result Detail` (`/trees/:treeId/research/results/:resultId`): Title, Description, Provider, External ID, Record Type, Date, Place, URL (validada http/https), Metadata, warnings candidato.
- `Research Workspace` Overview → `External Research` metrics `Queries/Successful/Failed/Results`.
- `Session Detail` → `External Research` `Queries/Results` agregados desde tasks.

No matching automático de personas/familias, no creación de Evidence.

## Storage & Performance

Repositorios: `create/get/list/delete` queries; `create/get/list/latest/update_status` executions; `create/get/list_by_execution/count_by_execution` results.

Evita N+1: `get_latest_executions_for_queries` + `count_results_for_executions` batch; `get_queries_with_latest_and_counts`; `external_research_summary` con agregaciones SQL; `list_latest_results_for_query` usa latest execution.

Paginación `limit/offset/page` en queries/results/executions (0..100).

## Seguridad

- URL solo http/https
- No se guardan API keys/tokens/cookies
- No se loguea query completa si sensible (solo provider, query_id, execution_id, status, duration, result_count)
- No se registran credenciales

## Invariantes arquitectónicos

1. `ResearchResult != Evidence`
2. `ResearchResult != Outcome`
3. `Provider failure != NO_EVIDENCE`
4. `0 results != Evidence`
5. External Research nunca modifica `Person/Family/Event/Source/Citation/Evidence/Outcome` automáticamente
6. Usuario decide conversión a Evidence
7. Evidence interpretable sin acceso live al provider
8. Research Score / Planning Score no cambian por resultados externos
9. Evidence Assessment no cuenta Results
10. Dominio funciona sin providers

## Futuro

Integrar provider real solo implementando `ResearchProvider`; no reescribir Task/Session/Evidence/Outcome.

## Diagrama

```
Research Task → Research Query → Query Execution → Research Provider → Research Results → User Review → Evidence → Assessment → Outcome
                └─ Execution #1 → Results
                └─ Execution #2 → Results   (historial preservado)

                 USER REVIEW
                     ↓
Result ───────────┼────────── Evidence
                     ↑
               NEVER AUTOMATIC
```
