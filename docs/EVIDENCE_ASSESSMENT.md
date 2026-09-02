# Evidence Assessment — Fase 4.1

## Purpose

Evidence Assessment responde:

> "¿Qué tan respaldada está esta conclusión?"

No responde:

> "¿Es verdadera?"

Evalúa cuánto respaldo documental registrado tiene un `ResearchOutcome` a partir de `Evidence` vinculadas con relación `SUPPORTS` / `CONTRADICTS`, sin confundirlo con `ResearchScore` ni modificar el árbol, el Outcome o las relaciones genealógicas.

## Difference from Research Score

```
Research Score  = qué merece ser investigado. (opportunity score 0..100, prioridad)
Evidence Score  = cuánto respaldo documental registrado tiene el resultado de esa investigación.
```

| Concepto | Pregunta | Origen | Mutabilidad |
|----------|----------|--------|-------------|
| Research Score | ¿Qué investigar? | Analyzer + Scoring (findings, gaps, branches) | Recalculado en cada import |
| Evidence Assessment | ¿Cuánto respaldo tiene lo concluido? | Evidence vinculadas al Outcome | Recalculado al attach/detach evidence |

No mezclar ambos. Un `CONFIRMED` con `NO_EVIDENCE` es posible y se advierte, no se bloquea.

## Input: EvidenceStats

Agregación SQL por outcome (sin N+1):

```rust
EvidenceStats {
  evidence_total: i64,
  supporting_count: i64,
  contradicting_count: i64,
  sources_count: i64,          // DISTINCT source_id
  cited_count: i64,            // citation_id IS NOT NULL
  uncited_count: i64,
  cited_supporting_count: i64, // SUPPORTS AND citation_id IS NOT NULL
}
```

Batch: `get_outcomes_evidence_stats(&[id])` hace `GROUP BY oe.outcome_id` en una sola query; los faltantes se rellenan con ceros.

## Formula

Implementada en `crates/storage/src/assessment.rs:calculate_evidence_assessment`. No recalcular en frontend; el backend es autoridad y expone `reasons`.

### Bonuses

```
+30 SUPPORTS exists               (supporting_count >=1)         code: SUPPORTING_EVIDENCE
+20 >= 2 SUPPORTS                 (supporting_count >=2)         code: MULTIPLE_SUPPORTING_EVIDENCE
+15 supporting evidence has citation (cited_supporting_count >=1) code: SUPPORTING_EVIDENCE_HAS_CITATION
+10 >= 2 distinct sources         (sources_count >=2)            code: MULTIPLE_SOURCES
+10 >= 2 evidence                (evidence_total >=2)           code: MULTIPLE_EVIDENCE
+5  all evidence has source       (evidence_total>0 && sources_count>0) code: ALL_EVIDENCE_HAS_SOURCE
```

Nota: `ALL_EVIDENCE_HAS_SOURCE` es siempre cierto si existe evidence (source_id NOT NULL), se mantiene por trazabilidad.

### Penalties

```
-30 contradiction exists          (contradicting_count >=1)      code: CONTRADICTING_EVIDENCE
-15 contradicting >= supporting   (contradicting >= supporting && contradicting>0) code: CONTRADICTS_DOMINANT
-10 no citation                  (cited_count==0 && evidence_total>0) code: NO_CITATION
```

### Clamp

```
score = clamp(score, 0, 100)
```

Score 0..100, explicable vía `reasons: Vec<{code,points,message}>`.

## Status rules

La implementación existente es la autoridad (`assessment.rs`).

```
NO_EVIDENCE        supporting =0 && contradicting =0
MIXED              supporting >=1 && contradicting >=1
STRONGLY_SUPPORTED supporting >=2 && contradicting==0 && cited_supporting>=1 && evidence_total>=2
SUPPORTED          supporting >=2 && contradicting==0
WEAK               remaining cases
```

Orden de evaluación en código:

```rust
if supporting==0 && contradicting==0 => NO_EVIDENCE
else if supporting>=1 && contradicting>=1 => MIXED
else if supporting>=2 && contradicting==0 && cited_supporting>=1 && evidence_total>=2 => STRONGLY_SUPPORTED
else if supporting>=2 && contradicting==0 => SUPPORTED
else => WEAK
```

## Important semantic warning

Evidence Score is NOT:
- probability of truth
- genealogical confidence
- source reliability
- proof standard

Es solamente:

> **una medida explicable del respaldo documental registrado en NeoGenealogy.**

No bloquea acciones, no cambia automáticamente `Outcome.type`, no altera `Research Score`.

## API

### GET /research-outcomes/:id

```json
{
  "id": 10,
  "type": "CONFIRMED",
  "summary": "...",
  "evidence": [{ "id": 1, "relationship": "SUPPORTS", "statement": "...", "source": {...}, "citation": {...} }],
  "evidence_assessment": {
    "score": 75,
    "status": "SUPPORTED",
    "evidence_total": 3,
    "supporting_count": 2,
    "contradicting_count": 0,
    "sources_count": 2,
    "cited_count": 2,
    "uncited_count": 1,
    "reasons": [
      { "code": "SUPPORTING_EVIDENCE", "points": 30, "message": "Supporting evidence exists" },
      { "code": "MULTIPLE_SUPPORTING_EVIDENCE", "points": 20, "message": "Multiple supporting evidence" }
    ]
  }
}
```

### GET /research-outcomes?assessment_status=...

Filtro server-side (no frontend-only). `assessment_status` ∈ `NO_EVIDENCE,WEAK,MIXED,SUPPORTED,STRONGLY_SUPPORTED`. Implementado batch sin N+1; no carga Evidence completa por outcome salvo lo ya incluido en `evidence:[]` (detallado) y `evidence_assessment`.

### GET /research/summary

```json
{
  "opportunities": { "high": 2, "medium": 3, "low": 5 },
  "tasks": { "open": 1, "in_progress": 2, "resolved": 1, "rejected": 0, "inconclusive": 0 },
  "outcomes": { "total": 12 },
  "sources": { "total": 8 },
  "evidence": { "total": 20, "supporting": 15, "contradicting": 5 },
  "assessment": { "no_evidence": 4, "weak": 10, "mixed": 3, "supported": 15, "strongly_supported": 8 }
}
```

### Errors

`assessment_status` inválido → `400 INVALID_ASSESSMENT_STATUS`.

## UI

### ResearchTaskDetail

Sección `Evidence Assessment` muestra status legible, `score / 100`, counts (`supporting/contradicting/sources/citations`) y `Why this assessment?` con `reasons` del backend. No recalcula score en React.

Warnings cerca del Assessment (no bloqueantes):
- `CONFIRMED + NO_EVIDENCE` → "This outcome is marked as CONFIRMED but has no recorded supporting evidence."
- `CONFIRMED + MIXED` → "This outcome has contradictory evidence."

Evidence con `CONTRADICTS` lleva badge `⚠ CONTRADICTS` (naranja) frente a `✓ SUPPORTS` (verde).

### ResearchHistory

Por Outcome: `Outcome · Evidence: N · Assessment: STATUS · score`. Filtro `Assessment` (All/No Evidence/Weak/Mixed/Supported/Strongly Supported) usa `assessment_status` del API; mantiene loading/empty/error/retry sin N+1.

### ResearchWorkspace

Bloque `Evidence Assessment` agregado si `GET /research/summary` lo retorna (una sola respuesta, sin queries adicionales):

```
No Evidence        4
Weak              10
Mixed              3
Supported         15
Strongly Supported 8
```

Si el endpoint no lo proporcionara, se omite y se mantiene `Evidence recorded / Sources`.

## Storage

- Migration `004_evidence_sources.sql` crea `research_sources`, `research_citations`, `evidence`, `outcome_evidence`.
- `EvidenceStats` vía `COUNT`, `SUM CASE`, `COUNT DISTINCT` en SQLite.
- `detach` mantiene `Evidence` reutilizable; `DELETE outcome` borra `outcome_evidence` (FK CASCADE) pero no `Evidence/Source/Citation`.

Ver `docs/STORAGE.md`, `docs/API.md`, `docs/WEB.md`.
