# Research Engine v2

```
GEDCOM
  ↓
Parser (LegacyGedcomParser, preserva RawTag, whitespace robusto)
  ↓
Modelo (GenealogyTree: Person, Family, Event, Source, Place, DateValue)
  ↓
Analysis (5 reglas + CycleRule → Vec<Finding>)
  ↓
Research Engine (scoring explicable → Vec<ResearchOpportunity>)
  ↓
Branch Analysis + Source Coverage + Ancestral Depth
  ↓
Score explicable + Research Queue (TOP N ordenado)
```

## Conceptos

### Ancestor

- `is_direct_ancestor`: aparece como ancestro de alguien o tiene descendencia.
- `generation_distance`: `max` profundidad en `tree.ancestors(id)` (visitado con `HashSet` para evitar ciclos).
- `ancestor_paths`: `Vec<Vec<String>>` desde la persona hacia raíces; múltiples caminos conservados, entrada única si es hoja.

### Cycle Detection

DFS sobre grafo hijo→padre con `stack` e `in_stack`. Si `current ∈ in_stack` se emite `Finding { kind: RELATIONSHIP_ANOMALY, severity: Critical, evidence: ["cycle: A -> B -> A"] }` y se continúa.

### Research Opportunity v2

Expone:

```
WHO              person_id + nombre
WHAT IS MISSING  missing_information
WHY IT MATTERS   why_it_matters
WHAT IS KNOWN    what_is_known (fechas, lugares, matrimonio, hijos)
RESEARCHABILITY  High | Medium | Low
POTENTIAL SOURCES potential_sources (Baptism, Marriage…)
CONFIDENCE       0.0–1.0
SCORE            0–100 + ScoreBreakdown
```

Ejemplo:

```
🔥 HIGH PRIORITY
Juan García López — 94/100 confidence 87%
Missing: Father and mother
Known: Birth ~1760 Alcalá, Marriage 1782, Children 4
Why: Direct ancestor y endpoint de línea
Researchability: HIGH
Potential: Baptism, Marriage, Parish register
Breakdown: Direct ancestor +30, Missing parent +20, ...
```

Oportunidades redundantes se agrupan por persona (una oportunidad por persona); no se generan 5 hallazgos para el mismo progenitor faltante.

### Ranking

`TOP RESEARCH OPPORTUNITIES` ordenables:

```bash
--sort score        # default, descendente por score
--sort priority     # por rank de Severity
--sort confidence   # por confidence
```

### CLI

```
neogenealogy analyze tree.ged                  # compacto
neogenealogy analyze tree.ged --explain-score  # breakdown
neogenealogy analyze tree.ged --severity high  # filtra findings >= high
neogenealogy stats tree.ged
neogenealogy report tree.ged --output report.html
```

JSON estable:

```json
{
  "summary": {"persons":..., "findings":..., "opportunities":...},
  "findings": [],
  "research_opportunities": [],
  "opportunities": [],
  "branches": [],
  "source_coverage": {"birth":..., "marriage":..., "death":..., "other_events":..., "overall":...},
  "ancestral_depth": {"maximum_depth":..., "average_depth":..., "missing_generations":[...]},
  "tree": {}
}
```

Claves históricas `chronology`, `research-gap`, `missing-source` se mantienen.

### HTML Report

Secciones: Overview, Statistics, Findings, TOP RESEARCH OPPORTUNITIES (con breakdown), BEST RESEARCH BRANCHES, Branches, Source Coverage, Ancestral Depth.
