# FamilySearch Provider — Phase 6.1

> **FamilySearch Result ≠ Evidence** — Un resultado externo es solo un candidato. NeoGenealogy no escribe ni sincroniza automáticamente el Family Tree.

## 1. Qué soporta actualmente la integración

Adapter preparado detrás de `ResearchProvider`:

```
ResearchQuery → FamilySearchProvider → Tree Person Search → ResearchResult
```

- **FamilySearch Family Tree Search** (`GET https://api.familysearch.org/platform/tree/search`) como primera capacidad real.
  - Parámetros usados: `q.givenName`, `q.surname`, `q.birthLikeDate` (extraído heurísticamente de la query), `count=10`.
  - Autenticación vía `Authorization: Bearer <token>` + `Accept: application/x-gedcomx-atom+json`.
  - Normalización a `ResearchResult` con `external_id`, `title`, `description`, `url=https://www.familysearch.org/tree/person/details/{id}`, `record_type=PERSON`, `date`/`place` de Birth, `metadata` con `raw_query`, `familysearch_url`, `surname`, etc.
  - URLs validadas `http/https` únicamente.
  - Paginación no avanzada (esta fase: `count=10` fijo, `offset` por defecto 0). Soporte completo de faceting/filtering queda para fase posterior.

- **MockResearchProvider** sigue funcionando sin cambios funcionales.

- **Isolación y seguridad:**
  - Secretos/tokens jamás expuestos en API responses, logs, errores, HTML o JSON de resultados.
  - Logs solo con `provider`, `query_id`, `execution_id`, `status`, `duration_ms`, `result_count`.
  - Errores mapeados a códigos normalizados existentes.

Arquitectura por capas:

```
Domain
ResearchProvider abstraction (trait ResearchProvider)
FamilySearchProvider (impl)
  ├─ FamilySearchConfig (env)
  ├─ Query Translation (ResearchQuery → FamilySearchSearchRequest)
  ├─ HTTP Client (ReqwestExecutor con timeout, headers encapsulados)
  ├─ Authentication (Bearer token, unauthenticated_session o access_token)
  └─ Normalization (GedcomX → ResearchResultCandidate)
HTTP/API (FamilySearch Platform API)
```

El resto del proyecto no conoce detalles HTTP de FamilySearch.

## 2. Qué NO soporta (fuera de alcance de 6.1)

- Escritura en FamilySearch (crear/editar personas, relaciones, sources)
- Modificación del Family Tree ni sync bidireccional
- Importación completa de árboles FamilySearch
- Auto-matching / auto-merge
- IA o conversión automática `ResearchResult → Evidence`
- Búsqueda avanzada por facets/filters (`f.*`, `c.*`) más allá de `q.*`
- Historical Records Search directa — ver §3
- Segundo provider real, multi-user NeoGenealogy, plugins genéricos

## 3. Requisitos de acceso / configuración

Basado **únicamente** en documentación oficial https://developers.familysearch.org y https://www.familysearch.org/developers/docs:

- Registrar aplicación en **Innovate / Developer Portal**: https://www.familysearch.org/innovate/apply → obtener `client_id` (app key). Acceso a la API REST y endpoints es gratuito pero requiere aprobación.
- No todas las capabilities están disponibles sin autenticación. `Tree Person Search` **sí** permite `unauthenticated_session` (ver Authentication Guide).
- Sin `client_id` aprobado, el provider queda **no configurado** y devuelve `AUTH_REQUIRED` normalizado ("FamilySearch is not configured").
- Producción requiere app aprobada y respetar throttling, caching, términos de uso (Terms of Use, Compatibility Review Process).

## 4. Variables de entorno

Todas con prefijo `NEOGENEALOGY_FAMILYSEARCH_`, coherente con el diseño existente (`NEOGENEALOGY_HOST`, `DATABASE_URL`, etc.):

| Variable | Requerida | Default | Descripción |
|----------|-----------|---------|-------------|
| `NEOGENEALOGY_FAMILYSEARCH_CLIENT_ID` | Sí si se usa FamilySearch | `None` | App key / client_id de FamilySearch |
| `NEOGENEALOGY_FAMILYSEARCH_ACCESS_TOKEN` | No | `None` | Bearer token inyectado manualmente (para E2E/tests locales). Si se define, se usa directamente y no se solicita token via `ident` |
| `NEOGENEALOGY_FAMILYSEARCH_BASE_URL` | No | `https://api.familysearch.org` | Base URL de Platform API. Configurable solo para integración/beta (`https://apibeta.familysearch.org`, `https://api-integ.familysearch.org`) |
| `NEOGENEALOGY_FAMILYSEARCH_IDENT_BASE_URL` | No | `https://ident.familysearch.org` | Base URL del Identity server (`/cis-web/oauth2/v3/token`) |
| `NEOGENEALOGY_FAMILYSEARCH_TIMEOUT_MS` | No | `10000` | Timeout HTTP en ms para search y token |
| `NEOGENEALOGY_FAMILYSEARCH_ENABLED` | No | `true` | Si `false`/`0`/`off`, provider deshabilitado (aunque configurado) |

Nunca hardcodear `client_id`, `secret`, `tokens` ni endpoints privados en código. Nunca exponer en logs.

## 5. Flujo de autenticación (soportado oficialmente)

Documentación oficial distingue 4 grant types:

- **authorization_code** — web redirect a FamilySearch login, devuelve `code` → `POST /cis-web/oauth2/v3/token` con `client_id`, `code`, `redirect_uri`. Para apps web con usuario. **No implementado en 6.1** (requiere redirect URI, manejo de sesiones).
- **unauthenticated_session** — `POST /cis-web/oauth2/v3/token` con `grant_type=unauthenticated_session`, `client_id`, `ip_address`. Devuelve `access_token` sin credenciales de usuario. **Permitido solo para endpoints específicos**: `Places`, `Date Authority`, `Person Search`, `Person Matches Query`, `Relationship Finder`. NeoGenealogy lo usa como **primera capacidad**: si `client_id` está configurado y no hay `ACCESS_TOKEN`, el provider solicita automáticamente un token `unauthenticated_session` antes de buscar. Si falla → `AUTH_REQUIRED`.
- **client_credentials** — requiere permiso especial email `devsupport@familysearch.org` + `client_secret` firmado. **No disponible para uso general**. No implementado; documentado como limitación.
- **password** — legacy, restringido a keys aprobadas.

**Token handling en 6.1:**
- Tokens **no** se persisten en BD, no se guardan en `ResearchQuery/ResearchResult/ResearchTask/Evidence/Source/Citation/Outcome`. Solo memoria del proceso.
- `ACCESS_TOKEN` de env se usa como cache inmediato; `unauthenticated_session` se solicita por ejecución (no se cachea persistentemente). Expira a las 24h o 60min sin uso según docs.
- Errores 401/403 → `AUTH_REQUIRED` ("FamilySearch connection required") sin filtrar token.
- No se almacenan passwords de FamilySearch jamás.
- No se implementa multi-user auth de NeoGenealogy todavía; alcance es conectar provider, no crear sistema de cuentas.

Si en el futuro se necesita OAuth de usuario (Authorization Code), se diseñará separación explícita:

```
FamilySearch application configuration ≠ FamilySearch user authorization ≠ Genealogy domain data
```

y persistencia de tokens aislada sin exposición via endpoints normales.

### 5.1 OAuth interactivo — Conectar con FamilySearch (incluye inicio con Google)

Desde 6.1.1 NeoGenealogy expone login interactivo **sin multi-user** (single-tenant, token global `familysearch_connections id=1`):

- `GET /api/v1/auth/familysearch/authorize` → genera `state` (uuid v4, 10 min TTL en `familysearch_oauth_states`), persiste y devuelve `{authorization_url, state}`. `authorization_url = https://ident.familysearch.org/cis-web/oauth2/v3/authorization?response_type=code&client_id=&redirect_uri=&state=&scope=openid`. `redirect_uri` por defecto `http://127.0.0.1:3000/api/v1/auth/familysearch/callback` (configurable `NEOGENEALOGY_FAMILYSEARCH_REDIRECT_URI`, debe estar pre-registrado en el portal FamilySearch).
- Frontend `Conectar con FamilySearch` (`web/src/pages/ResearchTaskDetail.tsx:380`, `web/src/pages/FamilySearchGlobalSearch.tsx`) hace `window.location.href = authorization_url`. El usuario ve la **página de login de FamilySearch**, que puede ofrecer **"Sign in with Google"** — es UI de FamilySearch, no de NeoGenealogy. Tras login, FamilySearch redirige a `redirect_uri?code=&state=`.
- `GET /api/v1/auth/familysearch/callback?code=&state=&error=` valida `state` (consume y expira), intercambia `code` por token vía `POST /cis-web/oauth2/v3/token {grant_type=authorization_code, client_id, code, redirect_uri}` (`familysearch.rs:fetch_token_authorization_code`), guarda `access_token` + `expires_at` en `familysearch_connections` y redirige a `NEOGENEALOGY_FAMILYSEARCH_FRONTEND_REDIRECT` (`http://localhost:5173?fanitysearch=connected`). Errores `error=access_denied` también redirigen con `?familysearch_error=`.
- `GET /api/v1/auth/familysearch/status` → `{configured, enabled, connected, status: connected|configured|not_configured|disabled, expires_at, requires_auth, redirect_uri}`. `connected` true si hay token válido en BD o `NEOGENEALOGY_FAMILYSEARCH_ACCESS_TOKEN`. `list_providers` también refleja `connected`.
- `POST /api/v1/auth/familysearch/disconnect` → `DELETE FROM familysearch_connections WHERE id=1`.
- Tokens nunca en `ResearchQuery/Result/Task/Evidence/Source/Citation/Outcome`, nunca en JSON/logs, solo `tracing::info` sin secreto. `effective_familysearch_config()` en `crates/api/src/handlers/external_research.rs:800` y `familysearch_auth.rs` prioriza: `stored token (si válido) > env ACCESS_TOKEN > unauthenticated_session`.

No se crea sistema multi-user; token es global para la instancia. Para multi-user futuro se extendería `familysearch_connections` con `user_id`.

### 5.2 Búsqueda global sin árbol

Para no depender de un único árbol (`crates/storage/migrations/007_external_research.sql` exige `tree_id`), se añade endpoint global:

- `GET /api/v1/familysearch/search?q=&givenName=&surname=&birthLikeDate=&birthLikePlace=` (`familysearch_auth.rs:familysearch_global_search`). Construye `free_q` o `q.givenName` etc., traduce con `translate_query` o usa explícitos, llama `FamilySearchProvider::search` con `effective_config` y devuelve `{provider, query, results[], result_count, disclaimer}` sin persistir `research_queries/executions`. No requiere `tree_id` ni `task_id`.
- UI `GET /familysearch` (`web/src/pages/FamilySearchGlobalSearch.tsx`) permite `q` libre o campos separados, muestra estado de conexión, botón Conectar/Desconectar, y resultados con `FamilySearch Result ≠ Evidence`. Accesible desde `Layout` (`web/src/components/Layout.tsx`) sin seleccionar árbol.
- El flujo clásico `Árbol → Task → Query → Execution → Result` sigue intacto; global es adicional para búsquedas rápidas.

## 6. Entorno de desarrollo (oficial)

FamilySearch documenta 3 entornos:

- **Production**: `https://api.familysearch.org`, `https://ident.familysearch.org`
- **Beta**: `https://apibeta.familysearch.org`, `https://identbeta.familysearch.org`
- **Integration**: `https://api-integ.familysearch.org`, `https://identint.familysearch.org`

Seleccionable vía `NEOGENEALOGY_FAMILYSEARCH_BASE_URL` / `NEOGENEALOGY_FAMILYSEARCH_IDENT_BASE_URL` solo cuando está documentado. Útil para pruebas sin tocar producción.

## 7. Limitaciones conocidas (verificadas en docs actuales)

1. **Historical Records Search NO es un endpoint público** para apps externas en la doc actual. `Records` en https://www.familysearch.org/developers/docs/api/resources aparece como *"No resources are available"*. `Historical Records Archive` (`/platform/collections/records`) solo lista colecciones, no búsqueda. La capacidad disponible más cercana es **Tree Person Search** sobre el Family Tree colaborativo, que sí está documentada y soporta `unauthenticated_session`. Por eso 6.1 implementa `Tree Person Search` como primera capacidad y deja adapter preparado para futura búsqueda de registros históricos si FamilySearch la expone.
2. **Application approval requerida**: sin `client_id` aprobado, FamilySearch rate-limits o devuelve 401. NeoGenealogy sigue funcionando completamente con `Mock`.
3. **Unauthenticated token limitado**: no todos los endpoints lo aceptan; `Tree Person Search` sí, pero otros (escritura, datos privados) requieren auth de usuario.
4. **No resultados = 204 No Content** o `entries: []` con 200. NeoGenealogy lo normaliza a `COMPLETED` con 0 resultados, no `FAILED`.
5. **Rate limiting y throttling**: docs indican 429 con warnings header; provider lo mapea a `RATE_LIMITED`.
6. **Throttling no documentado con límites exactos**: comportamiento queda encapsulado; no se hardcodean thresholds.
7. **GedcomX Atom**: requiere header `Accept: application/x-gedcomx-atom+json`; error si ausente.
8. **Datos sensibles**: Tree Person Search no retorna personas vivas ni datos sin permiso.

## 8. Rate limiting / comportamiento ante errores

Mapeo a errores normalizados existentes (ver `ProviderErrorCode`):

| Condición Familia | HTTP / Error | Código NeoGenealogy |
|-------------------|--------------|---------------------|
| 400 Bad Request, query inválida | 400 | `INVALID_QUERY` |
| 401 / 403 Unauthorized | token ausente/expirado, client_id inválido | `AUTH_REQUIRED` → HTTP 200 con execution `FAILED`, `error_code=AUTH_REQUIRED` (frontend muestra "FamilySearch connection required") |
| No configurado (sin client_id ni token) | pre-check | `AUTH_REQUIRED` con mensaje "FamilySearch is not configured" |
| 429 Throttled | 429 | `RATE_LIMITED` |
| Timeout | `reqwest::Error::is_timeout()` / 408/504 | `TIMEOUT` |
| 5xx Service failure | 500-599 | `PROVIDER_UNAVAILABLE` |
| 0 resultados exitoso | 204 o 200 con `entries:[]` | `COMPLETED` con 0 (no es `NO_RESULTS` como FAILED) |
| Otro inesperado | parse error, etc | `UNKNOWN` |

Nunca se oculta `AUTH_REQUIRED` devolviendo `NO_RESULTS`. Detalles internos quedan en `tracing::error!` seguro (sin token), API pública mantiene contrato.

Re-run preserva historial: cada `POST .../run` crea nueva `research_query_executions` y sus `research_results`; listado y conteo batch sin N+1, igual que `mock`.

## 9. Arquitectura del adapter

```
crates/storage/src/familysearch.rs
  ├─ FamilySearchConfig::from_env() / is_configured() / enabled()
  ├─ FamilySearchSearchRequest {givenName, surname, birthLikeDate}
  │   └─ translate_query(&str) → Result<Request, ProviderError> (testeable)
  ├─ map_http_status(u16) → ProviderErrorCode
  ├─ normalize_search_response(Value) → Vec<ResearchResultCandidate>
  ├─ FamilySearchHttpExecutor trait (fetch_search, fetch_token_unauthenticated)
  │   └─ ReqwestExecutor (ruta real con timeout, headers, URL segura)
  └─ FamilySearchProvider implements ResearchProvider
        └─ search(&query) → ProviderError o ResearchProviderResponse

crates/storage/src/external_research.rs
  └─ ResearchProviderRegistry::new() inserta "mock" y "familysearch"

crates/api/src/handlers/external_research.rs
  ├─ execute_query() usa registry.get(provider_name) genérico (mock o familysearch)
  └─ list_providers_generic / list_providers_tree (nuevo)

web/src/api/client.ts + types.ts
  └─ getResearchProviders(provider list)

web/src/pages/ResearchTaskDetail.tsx
  └─ selector Mock / FamilySearch, estados no configurado / auth required, resultados con provider y URL externa
```

Diseño asegura:

```
FamilySearchProvider
    ↓
adapter preparado
    ↓
capacidad real condicionada por configuración/acceso
```

Proyecto sigue funcionando completamente con `MockResearchProvider` si FamilySearch no está configurado o falla.

## 10. Regla fundamental

```
FamilySearch Result ≠ Evidence
FamilySearch Result ≠ Source
FamilySearch Result ≠ Citation
```

Un resultado de FamilySearch es **solo un candidato**, nunca se convierte automáticamente en `Source`, `Citation`, `Evidence` u `Outcome`. La revisión humana es obligatoria. NeoGenealogy **no escribe ni sincroniza automáticamente** el Family Tree.

---

## Verificación y referencias oficiales

- Developer Portal: https://developers.familysearch.org
- API Resources: https://www.familysearch.org/developers/docs/api/resources
- Authentication Guide: https://www.familysearch.org/developers/docs/guides/authentication
- Access Token Resource: `POST https://ident.familysearch.org/cis-web/oauth2/v3/token` (`/cis-web/oauth2/v3/token` en prod/beta/integ)
- Authorization Resource: `GET/POST https://ident.familysearch.org/cis-web/oauth2/v3/authorization`
- Tree Person Search: `GET https://api.familysearch.org/platform/tree/search?q.givenName=&q.surname=&q.birthLikeDate=`
- Family Tree Search Guide: https://developers.familysearch.org/main/docs/family-tree-search
- Historical Records Archive: `GET https://api.familysearch.org/platform/collections/records` (solo lectura, sin búsqueda)
- Collections: `GET https://api.familysearch.org/platform/collections`
- Change Log / Compatibility: https://www.familysearch.org/developers/docs/change-log, https://developers.familysearch.org/main/docs/compatibility-review-process

Todas las decisiones sobre endpoints, scopes, parámetros y flujos se basaron **únicamente** en esa documentación oficial. No se inventaron URLs, scopes ni parámetros.

## E2E y pruebas

- Sin credenciales reales ni Internet, la fase se verifica E2E con **Mock provider + fixtures HTTP / integration tests** (ver `familysearch::tests` y `external_research::tests`).
- Con `NEOGENEALOGY_FAMILYSEARCH_CLIENT_ID` o `NEOGENEALOGY_FAMILYSEARCH_ACCESS_TOKEN` válidos, una `ResearchQuery` con `provider=familysearch` puede ejecutarse contra el entorno real y persistir `research_results` mediante el flujo existente. Documentar resultado E2E real en commit/PR sin exponer secretos.

## Configuración de ejemplo

```bash
export NEOGENEALOGY_FAMILYSEARCH_CLIENT_ID="a1b2c3-..."
# opcional para pruebas directas con token ya obtenido:
export NEOGENEALOGY_FAMILYSEARCH_ACCESS_TOKEN="eyJ..."
# opcional entorno beta:
export NEOGENEALOGY_FAMILYSEARCH_BASE_URL="https://apibeta.familysearch.org"
export NEOGENEALOGY_FAMILYSEARCH_IDENT_BASE_URL="https://identbeta.familysearch.org"
export NEOGENEALOGY_FAMILYSEARCH_TIMEOUT_MS="15000"

cargo run -p neogenealogy -- serve --db neogenealogy.db
```

Frontend mostrará:

- `FamilySearch` en selector cuando provider disponible
- `FamilySearch is not configured` si `configured==false`
- `FamilySearch connection required` si ejecución devuelve `AUTH_REQUIRED`
- Resultados con `FamilySearch` como provider, título, fact, URL `https://www.familysearch.org/tree/person/details/{id}` validada, y badge `External Research Result — This result is not evidence`.

