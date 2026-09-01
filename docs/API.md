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
