# Research Planning — Fase 5.0

`Opportunity → Planning → User decides → Task`

## Diferencia conceptual

- **Research Opportunity**: “Esto parece digno de investigación.” (automático, `Analyzer + Scoring`)
- **Research Plan**: “Estas son las investigaciones que tienen más sentido abordar ahora.” (vista derivada, determinista, sin persistencia)
- **Research Task**: “Esta es una investigación que el usuario ha decidido realizar.” (`OPEN → IN_PROGRESS → RESOLVED/REJECTED/INCONCLUSIVE`)

El Planner **no crea Tasks automáticamente** y no modifica oportunidades, outcomes o evidencia. Solo calcula y presenta.

## Modelo

```rust
ResearchPlan {
  generated_at: String, // now_iso()
  total_candidates: usize,
  recommended: Vec<ResearchPlanItem>, // top N
  deferred: Vec<ResearchPlanItem>,    // restantes
  summary: ResearchPlanSummary
}

ResearchPlanItem {
  opportunity_id: i64,
  person_id: i64,
  title: String, // why || "Research opportunity {id}"
  priority: String, // HIGH/CRITICAL/MEDIUM/LOW (upper, reutilizada de Opportunity)
  research_score: i64, // 0..100, base principal
  planning_score: f64, // 0..100, orden práctico
  researchability: String, // HIGH/MEDIUM/LOW upper
  confidence: f64, // 0.0..1.0
  active_task: bool, // true si OPEN/IN_PROGRESS
  task_status: Option<String>, // OPEN/IN_PROGRESS/INCONCLUSIVE/None
  reasons: Vec<ResearchPlanningReason> // explicables
}

ResearchPlanningReason { code, label, description }
ResearchPlanSummary {
  total_candidates, recommended_count, deferred_count,
  active_count, inconclusive_count,
  high_priority_count, critical_gap_count
}
```

`DEFAULT_PLAN_SIZE = 10` (constante interna, sin configuración persistente). Si hay <10 candidatos, `recommended = all`.

No existe tabla `research_plans`; el plan es una vista calculada. Si cambia Opportunity/Task/Outcome/Evidence/Gap, el plan cambia en la siguiente petición.

## Planning Score

No sustituye al `Research Score`. Lo combina con contexto de planificación:

```
planning_score =
    research_score * 0.55
  + researchability_score * 0.20
  + confidence_score * 0.10
  + evidence_gap_score * 0.10
  + task_state_score * 0.05
clamp 0..100, sin redondeo prematuro
```

Pura: `calculate_planning_score(research_score, researchability, confidence, gaps, task_status) -> f64`.

### Researchability Score

Reutiliza `ResearchOpportunity.researchability` sin recalcular:

```
HIGH   → 100
MEDIUM → 60
LOW    → 20
null   → 0
```

### Confidence Score

Reutiliza `ResearchOpportunity.confidence` 0.0..1.0:

```
confidence_score = confidence * 100
```

### Evidence Gap Score

Mide contexto pendiente, no calidad. Único max:

```
CRITICAL → 100
WARNING  → 60
INFO     → 20
sin gaps → 0
múltiples → max(severity_score) // ej. 1 CRITICAL > 5 INFO
```

Gaps provienen del outcome asociado a la task de la oportunidad (si no hay task/outcome → 0). Usa `calculate_evidence_gaps` existente.

### Task State Score

Evita recomendar continuamente lo mismo:

```
NO TASK      → 100
OPEN         → 80  (penalizada vs sin task, menor que NO TASK)
IN_PROGRESS  → 40  (más penalizada que OPEN)
INCONCLUSIVE → 20  (penalizada, pero visible con razón)
RESOLVED     → 0
REJECTED     → 0
```

`RESOLVED/REJECTED` excluidas del plan (no son candidatas principales). `INCONCLUSIVE` puede reaparecer con razón `PREVIOUSLY_INCONCLUSIVE`.

## Reasons

Cada item explica por qué está donde está, con código estable (no texto libre):

```
HIGH_RESEARCH_SCORE       // research_score ≥70
HIGH_RESEARCHABILITY      // researchability == HIGH
HIGH_CONFIDENCE           // confidence ≥0.75
CRITICAL_EVIDENCE_GAP     // max gap CRITICAL
WARNING_EVIDENCE_GAP      // max gap WARNING
INFO_EVIDENCE_GAP         // max gap INFO
NO_ACTIVE_TASK            // sin task
ACTIVE_TASK               // OPEN/IN_PROGRESS → Already being researched
PREVIOUSLY_INCONCLUSIVE   // INCONCLUSIVE → Previously investigated but inconclusive
```

Cada razón: `{code, label, description}`.

## Prioridad

Se reutiliza la taxonomía de Opportunity: `Critical/High/Medium/Low` (upper en API). El `planning_score` solo ordena dentro del plan, no crea nueva prioridad.

## Ranking

```
planning_score DESC
research_score DESC
confidence DESC
opportunity_id ASC
```

Determinista: mismos datos → mismo output.

## Recommended vs Deferred

```
recommended = top 10 (limit param, max 100, default 10)
deferred = resto
```

El usuario ve primero lo recomendado, puede expandir deferred.

## Backend

`crates/storage/src/planning.rs`:

- `ResearchPlan`, `ResearchPlanItem`, `ResearchPlanningReason`, `ResearchPlanSummary`
- `calculate_planning_score(...)`, `calculate_research_plan(candidates, limit) -> ResearchPlan`
- puras, sin SQL

`crates/storage/src/repositories.rs`:

- `get_research_planning_candidates(tree_id) -> Vec<PlanningCandidate>` eficiente, sin N+1:
  1. `SELECT * FROM research_opportunities WHERE tree_id=?`
  2. `SELECT * FROM research_tasks WHERE tree_id=? AND opportunity_id IN (...)` (una query, mapea última task por oportunidad)
  3. `SELECT * FROM research_outcomes WHERE task_id IN (...)` (una query)
  4. `get_outcomes_gaps(outcome_ids)` batch (2 queries, no N)
  Total O(1) respecto a número de opportunities. Aislado por `tree_id`.

## API

```
GET /api/v1/trees/:tree_id/research/plan?limit=10&min_score=60&priority=high&researchability=high
```

- `limit` 1..100 default 10
- `min_score` 0..100 filtra `planning_score >= min_score`
- `priority` ∈ low/info/medium/warning/high/critical
- `researchability` ∈ low/medium/high
- Validación 400 si inválido, 404 `TREE_NOT_FOUND`
- Tree isolation obligatorio (solo datos del mismo árbol)

Respuesta:

```json
{
  "generated_at": "2024-...",
  "total_candidates": 42,
  "summary": {
    "total_candidates": 42,
    "recommended_count": 10,
    "deferred_count": 32,
    "active_count": 3,
    "inconclusive_count": 2,
    "high_priority_count": 7,
    "critical_gap_count": 4
  },
  "recommended": [
    {
      "opportunity_id": 123,
      "person_id": 456,
      "title": "Find the parents of Josep García",
      "priority": "HIGH",
      "research_score": 87,
      "planning_score": 90.9,
      "researchability": "HIGH",
      "confidence": 0.91,
      "active_task": false,
      "task_status": null,
      "reasons": [{ "code":"HIGH_RESEARCH_SCORE","label":"High research score","description":"..." }]
    }
  ],
  "deferred": []
}
```

OpenAPI documenta `ResearchPlan`, `ResearchPlanItem`, `ResearchPlanSummary`, `ResearchPlanningReason`, `ResearchPlanningReasonCode` y `GET /trees/{tree_id}/research/plan`.

## Frontend

Nueva sección `Research → Planning` (`/trees/:treeId/research/planning`):

- Cabecera: “Research Planning — What should I research next?” con `recommended / total_candidates`
- Lista `Recommended` con cards: Person, title, Priority, Planning Score (secundario) + Research Score (primario), Researchability, Confidence, Reasons, Task state
- `Why is this here?` muestra razones estructuradas del backend (no recalcula en TS)
- Acciones: `View Opportunity` → `/research/opportunities/:id`, `Start Research` (usa `POST /research-opportunities/:id/tasks` existente) si no hay task, `View Research Task` si existe
- Filtros: Priority, Researchability, Minimum Planning Score, Limit (10/20/50)
- Explicación: `Research Score = importancia/interés` vs `Planning Score = prioridad práctica`

Navegación `Research` ahora: Overview / Planning / Opportunities / Tasks / History + Sources/Evidence/Coverage.

## Performance y No-objetivos

- Funciona con árboles grandes (10k+ opportunities) sin N+1, sin cargar objetos completos innecesarios.
- Sin IA/LLM, sin servicios externos, sin FamilySearch/Ancestry, sin scraping, sin creación automática de Tasks, sin persistencia de planes, sin calendarios/notificaciones.

## No modifica datos

El Planner nunca crea/modifica Opportunity, Task, Outcome, Evidence, Assessment, Gaps ni Research Score. Solo calcula y presenta.

Ver `docs/STORAGE.md`, `docs/API.md`, `docs/WEB.md` para integración.
