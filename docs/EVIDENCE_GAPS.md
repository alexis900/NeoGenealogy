# Evidence Gaps — Fase 4.2

## Purpose

Evidence Gaps responde:

> "¿Qué carencias observables existen en ese respaldo?"

No responde:

> "¿Qué documento buscar?" ni "¿Es verdadera la conclusión?"

Gaps son **observaciones** derivadas de `EvidenceStats` + `Outcome.type`, no inferencias externas.

## Separation

```
Research Score      = qué merece investigación (analyzer + scoring)
Evidence Assessment = cuánto respaldo tiene la conclusión (0..100, status)
Evidence Gaps       = qué carencias observables existen (codes + severity)
```

Gaps no modifican `Evidence Score`; lo complementan con explicación accionable.

```
Assessment: WEAK · 30

Gaps:
- Single supporting evidence
- No citation
```

## Model

Derivado, no persistido:

```rust
EvidenceGap {
  code: String,       // enum controlado
  severity: String,   // INFO | WARNING | CRITICAL
  title: String,
  description: String,
}
```

Cálculo puro:

```
EvidenceStats + OutcomeType
        ↓
calculate_evidence_gaps() -> Vec<EvidenceGap>
```

Independiente de SQLite. Reutiliza `get_outcomes_evidence_stats` batch (1 query `GROUP BY`, sin N+1).

## Gap codes (6)

| Code | Severity | Title | Description |
|------|----------|-------|-------------|
| `CONFIRMED_WITHOUT_SUPPORT` | CRITICAL | Confirmed without support | This confirmed outcome has no recorded supporting evidence. |
| `NO_SUPPORTING_EVIDENCE` | CRITICAL | No supporting evidence | No supporting evidence is currently recorded for this outcome. |
| `CONTRADICTORY_EVIDENCE` | WARNING | Contradictory evidence | Contradictory evidence is recorded for this outcome. |
| `NO_CITATION` | WARNING | No citation | Supporting evidence has no citation. |
| `SINGLE_SUPPORTING_EVIDENCE` | WARNING | Single supporting evidence | This outcome currently relies on a single supporting evidence record. |
| `SINGLE_SOURCE` | INFO | Single source | Evidence currently comes from a single source. |

Un Outcome puede tener varios gaps simultáneamente (ej: `SINGLE_SUPPORTING_EVIDENCE` + `NO_CITATION` + `SINGLE_SOURCE`).

## Severity rules

- **CRITICAL**: `CONFIRMED_WITHOUT_SUPPORT` / `NO_SUPPORTING_EVIDENCE` (supporting ==0)
- **WARNING**: `CONTRADICTORY_EVIDENCE` (contradicting>0), `NO_CITATION` (supporting>0 && cited_supporting==0), `SINGLE_SUPPORTING_EVIDENCE` (supporting==1)
- **INFO**: `SINGLE_SOURCE` (evidence_total>0 && sources==1)

`CONFIRMED_WITHOUT_SUPPORT` reemplaza a `NO_SUPPORTING_EVIDENCE` cuando `type==CONFIRMED` para evitar duplicación visual. Backend define claramente.

## Calculation

Definida en `crates/storage/src/assessment.rs:calculate_evidence_gaps`:

```rust
if supporting==0 { if CONFIRMED => CONFIRMED_WITHOUT_SUPPORT else NO_SUPPORTING_EVIDENCE }
if supporting==1 => SINGLE_SUPPORTING_EVIDENCE
if supporting>0 && cited_supporting==0 => NO_CITATION
if contradicting>0 => CONTRADICTORY_EVIDENCE
if evidence_total>0 && sources==1 => SINGLE_SOURCE
```

Determinista, testeable, aislado por Tree.

No asume independencia de fuentes; `SINGLE_SOURCE` significa literalmente 1 `DISTINCT source_id`. No es score de fiabilidad.

## No persistence

No existe tabla `evidence_gaps`. Si se añade Evidence desaparece, si se elimina aparece. No requiere jobs.

`DELETE outcome` mantiene `Evidence` reutilizable; `DETACH` recalcula gaps inmediatamente.

## API

### GET /research-outcomes/:id

```json
{
  "evidence_assessment": { "score": 30, "status": "WEAK", "reasons": [...] },
  "evidence_gaps": [
    { "code": "SINGLE_SUPPORTING_EVIDENCE", "severity": "WARNING", "title": "Single supporting evidence", "description": "This outcome currently..." },
    { "code": "NO_CITATION", "severity": "WARNING", "title": "No citation", "description": "Supporting evidence has no citation." }
  ]
}
```

`[]` si no hay gaps.

### GET /research-outcomes?gap=...

Filtro server-side (igual que `assessment_status`), sin N+1. Valores válidos los 6 codes. `400 INVALID_GAP_CODE` si inválido. Puede combinarse con `assessment_status`.

List incluye gaps vía mismo batch `get_outcomes_gaps`; no carga Evidence completa.

### GET /research/summary

```json
{
  "evidence_gaps": { "critical": 1, "warning": 2, "info": 3 }
}
```

Obtenido contando `severity` sobre gaps batch; sin recorrer todos los outcomes en memoria si puede evitarse, pero sí mediante agregación de stats.

## UI

### ResearchTaskDetail

Sección `Evidence Gaps` tras Assessment:

```
Evidence Gaps
────────────────────────
No evidence gaps detected.   # si []

⚠ Contradictory evidence
Contradictory evidence is recorded...

⚠ Single supporting evidence
This outcome currently relies...
```

CRITICAL con fondo rojo, WARNING ámbar, INFO azul. Además `⚠ Critical evidence gap` prominente si existe.

Quick actions reutilizando pantallas existentes:

- `NO_SUPPORTING_EVIDENCE` / `CONFIRMED_WITHOUT_SUPPORT` → `Add Evidence`
- `NO_CITATION` → `Review Evidence`
- `CONTRADICTORY_EVIDENCE` → `Review Contradictions` (scroll a Evidence)

No bloquea editar Outcome, añadir Evidence, etc.

### Research History

`Assessment: MIXED · 45` + `Gaps: 1 warning` (o `Gaps: 2`). Filtro opcional por gap.

### Workspace

Métrica compacta `Evidence Gaps — Critical / Warnings / Info` desde `GET /research/summary` (una sola respuesta). Si requiere N+1 se omite.

## What it is NOT

```
Evidence Gap ≠ proof failure
Evidence Gap ≠ truth assessment
Evidence Gap ≠ automatic research recommendation
Evidence Gap ≠ source reliability score
Evidence Gap ≠ genealogical proof standard
```

No convierte un Gap en `Research Opportunity` automáticamente, no modifica `Task.status` ni `Outcome.type`.

## Storage

- No migración. Cálculo vía `get_outcome_evidence_stats` (`COUNT`, `SUM CASE`, `COUNT DISTINCT`) ya existente.
- `Storage::get_outcome_gaps`, `get_outcomes_gaps` batch sin N+1, usa `calculate_evidence_gaps` puro.
- `research_summary` cuenta gaps por severity.

Ver `docs/EVIDENCE_ASSESSMENT.md`, `docs/STORAGE.md`, `docs/API.md`, `docs/WEB.md`.
