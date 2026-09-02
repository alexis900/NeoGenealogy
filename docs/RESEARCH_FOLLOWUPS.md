# Research Follow-ups — Fase 4.3

## Purpose

Research Follow-ups responde:

> "Tengo este hueco de evidencia. ¿Qué acción de investigación genérica podría realizar ahora?"

No responde:

> "¿Qué documento exacto buscar?" ni "¿Es verdadera la conclusión?"

Follow-ups son **acciones sugeridas y deterministas** derivadas del estado actual de un `Research Outcome` y sus `Evidence Gaps`. No son hechos genealógicos ni recomendaciones externas.

```
Research Opportunity  → ¿Qué merece investigación?        (analyzer + scoring)
Research Task        → ¿Qué ha decidido hacer el usuario? (OPEN→RESOLVED)
Evidence Assessment  → ¿Cuánto respaldo hay registrado?   (0..100, status)
Evidence Gaps        → ¿Qué carencia observable existe?  (codes + severity)
Research Follow-up   → ¿Qué acción genérica podría abordar ese hueco? (sugerida)
```

```
Evidence Gap
      ↓
Research Follow-up
      ↓ (usuario decide)
Research Task (opcional, manual)
```

Un Follow-up **no crea automáticamente** `Research Task`, `Research Opportunity` ni modifica `Outcome.type`.

## Model

Derivado, no persistido:

```rust
ResearchFollowUp {
  code: String,       // enum controlado
  priority: String,   // HIGH | MEDIUM | LOW
  title: String,
  description: String,
  gap_code: String,   // gap que lo originó
}
```

Cálculo puro:

```
EvidenceStats + OutcomeType + EvidenceGap[]
          ↓
calculate_research_followups() -> Vec<ResearchFollowUp>
```

Determinista, sin SQLite, HTTP, filesystem ni AI. Preferentemente junto a `calculate_evidence_assessment()` / `calculate_evidence_gaps()` en `crates/storage/src/assessment.rs`.

No existe tabla `research_followups`. Si cambia Evidence, se recalcula inmediatamente.

## Follow-up codes (5)

| Code | Priority | Title | Description |
|------|----------|-------|-------------|
| `ADD_SUPPORTING_EVIDENCE` | HIGH | Add supporting evidence | `This confirmed outcome has no supporting evidence recorded.` (si `CONFIRMED_WITHOUT_SUPPORT`) o `No supporting evidence is currently recorded for this outcome.` (si `NO_SUPPORTING_EVIDENCE`) |
| `ADD_CITATION` | MEDIUM | Add a citation | Supporting evidence is recorded without a citation. |
| `REVIEW_CONTRADICTION` | HIGH | Review contradictory evidence | Supporting and contradicting evidence are both recorded for this outcome. |
| `ADD_SECOND_SUPPORTING_EVIDENCE` | MEDIUM | Add another supporting evidence | This outcome currently has a single supporting evidence record. |
| `REVIEW_SOURCE_COVERAGE` | LOW | Review source coverage | Evidence for this outcome currently comes from a single source. |

Prioridades no reutilizan `Critical/High/Medium/Low` del Research Score.

## Mapping Gap → Follow-up

| Gap | Follow-up | Priority |
|-----|-----------|----------|
| `CONFIRMED_WITHOUT_SUPPORT` | `ADD_SUPPORTING_EVIDENCE` | HIGH |
| `NO_SUPPORTING_EVIDENCE` | `ADD_SUPPORTING_EVIDENCE` | HIGH |
| `NO_CITATION` | `ADD_CITATION` | MEDIUM |
| `SINGLE_SUPPORTING_EVIDENCE` | `ADD_SECOND_SUPPORTING_EVIDENCE` | MEDIUM |
| `CONTRADICTORY_EVIDENCE` | `REVIEW_CONTRADICTION` | HIGH |
| `SINGLE_SOURCE` | `REVIEW_SOURCE_COVERAGE` | LOW |

Un Outcome puede producir varios Follow-ups simultáneamente.

Ejemplo:

```
Gaps:
- CONTRADICTORY_EVIDENCE
- NO_CITATION
- SINGLE_SOURCE

Follow-ups:
HIGH   Review contradictory evidence
MEDIUM Add a citation
LOW    Review source coverage
```

## Pure calculation

```rust
pub fn calculate_research_followups(
    outcome_type: &str,
    stats: &EvidenceStats,
    gaps: &[EvidenceGap],
) -> Vec<ResearchFollowUp>
```

- pura, determinista, sin I/O
- deduplica por `code` si varios gaps generan el mismo follow-up (ej. `CONFIRMED_WITHOUT_SUPPORT` + `NO_SUPPORTING_EVIDENCE` nunca coexisten pero se protege)
- ordena por prioridad `HIGH → MEDIUM → LOW` y dentro de la misma prioridad en orden estable (`ADD_SUPPORTING_EVIDENCE` → `REVIEW_CONTRADICTION` → `ADD_CITATION` → `ADD_SECOND_SUPPORTING_EVIDENCE` → `REVIEW_SOURCE_COVERAGE`)
- no usa `HashMap` con orden aleatorio

## Deduplicación y orden

Si varios gaps generan `ADD_SUPPORTING_EVIDENCE`, solo se devuelve una vez. `gap_code` conserva el gap originario (primero en aparecer). Relación 1:1 `Follow-up → gap_code` mantiene el modelo simple.

Orden estable garantiza tests y UI deterministas.

## API

### GET /research-outcomes/:id

```json
{
  "evidence_assessment": { "score": 25, "status": "WEAK", "reasons": [...] },
  "evidence_gaps": [{ "code": "NO_CITATION", "severity": "WARNING", "title": "No citation", "description": "..." }],
  "research_followups": [
    {
      "code": "REVIEW_CONTRADICTION",
      "priority": "HIGH",
      "title": "Review contradictory evidence",
      "description": "Supporting and contradicting evidence are both recorded for this outcome.",
      "gap_code": "CONTRADICTORY_EVIDENCE"
    }
  ]
}
```

`[]` si no hay follow-ups.

### GET /research-outcomes (list)

Incluye `research_followups` derivados mediante batch (`get_outcomes_evidence_stats` + `get_outcomes_gaps` → `calculate_research_followups`), sin N+1. Reutiliza la información ya agregada para assessment/gaps.

### GET /research/summary

```json
{
  "research_followups": { "high": 6, "medium": 9, "low": 4 }
}
```

Calculado server-side contando `priority` sobre followups batch (una sola respuesta, sin recorrer todos los outcomes en React).

## UI

### ResearchTaskDetail

Sección `Research Follow-ups` después de `Evidence Assessment` y `Evidence Gaps`:

```
Research Follow-ups
────────────────────

HIGH
Review contradictory evidence
Supporting and contradicting evidence are both
recorded for this outcome.
[Review Evidence]
```

Otro:

```
MEDIUM
Add a citation
Supporting evidence is recorded without a citation.
[Review Evidence]
```

Otro:

```
HIGH
Add supporting evidence
No supporting evidence is currently recorded.
[Add Evidence]
```

- Oculta la sección cuando está vacía (no mostrar "No follow-ups").
- Mantiene visible el `Evidence Gap` que originó la acción para que el usuario entienda el porqué.

Quick actions reutilizan pantallas existentes:

| Follow-up | Acción | Destino |
|-----------|--------|---------|
| `ADD_SUPPORTING_EVIDENCE` | Add Evidence | workflow creación Evidence |
| `ADD_SECOND_SUPPORTING_EVIDENCE` | Add Evidence | mismo |
| `ADD_CITATION` | Review Evidence | área Evidence/Citation existente |
| `REVIEW_CONTRADICTION` | Review Evidence | sección Evidence |
| `REVIEW_SOURCE_COVERAGE` | Review Evidence | sección Evidence |

No crea workflow nuevo.

### ResearchWorkspace

Métrica opcional `Research Follow-ups — High / Medium / Low` desde `GET /research/summary` (una sola respuesta). Si añade complejidad significativa, se omite (4.3 la deja fuera).

### ResearchHistory

Por Outcome: `Follow-ups: 2` junto a `Assessment` y `Gaps`. No carga Evidence adicional.

## What it is NOT

```
Research Follow-up ≠ verdad genealógica
Research Follow-up ≠ recomendación de fuente concreta
Research Follow-up ≠ creación automática de Task/Opportunity
Research Follow-up ≠ cambio automático de Outcome.type o Task.status
Research Follow-up ≠ AI / LLM / búsqueda externa / FamilySearch / OCR
Research Follow-up ≠ proof standard / Bayesian reasoning
```

Especialmente: **NO intenta resolver el Gap**, solo sugiere la acción genérica que podría abordarlo.

## Separation from Gaps/Opportunities/Tasks

```
Gap  → carencia observable
Follow-up → acción sugerida para abordar ese hueco (derivada, no persistida)
Task → trabajo decidido por el usuario (persistido, manual)
Opportunity → qué merece investigación según el sistema (scoring)
```

Ver `docs/EVIDENCE_GAPS.md`, `docs/EVIDENCE_ASSESSMENT.md`, `docs/RESEARCH_WORKFLOW.md`.

## Storage

- No migración. Cálculo vía `EvidenceStats` + `EvidenceGap[]` ya existente.
- `Storage::get_outcome_followups`, `get_outcomes_followups` batch sin N+1, usan `calculate_research_followups` puro.
- `research_summary` cuenta followups por prioridad.

## Testing

Unit: 6 gaps → 5 follow-ups, duplicate prevention, priority ordering, empty.

Integración: attach/detach evidence y citation recalculan follow-ups.

Frontend: `ResearchTaskDetail` no follow-ups / uno / varios / prioridades / quick actions.

## Golden rule

> NeoGenealogy puede decir: "Una acción razonable para abordar ese hueco sería revisar la evidencia contradictoria." Pero no debe decir: "Busca este documento concreto." Eso será otra capa.
