# Analysis Rules

Los hallazgos son señales de investigación, no afirmaciones de hechos. Cada resultado conserva `evidence` y `confidence` 0–1, y puede enlazar en el futuro `Finding → Evidence → Source → Citation`.

| Tipo | Criterio | Severidad | Descripción |
|---|---|---|---|
| `missing-data` | Falta campo biográfico (nacimiento,lugar,defunción,matrimonio) | `Info` | Datos incompletos |
| `chronology` | Nacimiento posterior a defunción | `High` | `birth year > death year` |
| `AGE_ANOMALY` | Edad parental <14 o >55 al nacer hijo | `High`/`Warning` | Umbrales configurables vía `AnalysisConfig` |
| `RELATIONSHIP_ANOMALY` | Hijo nacido antes de matrimonio registrado | `Warning` | Puede indicar otra unión |
| `RELATIONSHIP_ANOMALY` | Ciclo genealógico (A padre de B, B padre de A) | `Critical` | Detectado por DFS con `in_stack`; no rompe el análisis |
| `POSSIBLE_DUPLICATE` | ≥2 de: mismo nombre, año ±2, mismo lugar | `Warning` | Probabilístico, `confidence = 0.55 + 0.12*m` |
| `research-gap` | Persona sin familia de origen (`FAMC` ausente) | `High` | Hueco contextualizado, sugiere bautismos/matrimonios |
| `missing-source` | Persona sin fuentes vinculadas | `Medium` | Sin citas de fuente |

Severidades ordenadas (asc): `Low (0) < Info (1) < Medium (2) < Warning (3) < High (4) < Critical (5)`. El filtro `--severity high` muestra `High` y `Critical`.

## Robustez

El análisis continúa aunque existan personas/familias incompletas, referencias rotas, etiquetas desconocidas, fechas inválidas o ciclos. Una entrada problemática produce un `Finding` en lugar de un crash.

## Separación de estados

Se mantiene distinción estricta:

- `KNOWN` — dato con evidencia
- `UNKNOWN` — dato ausente
- `UNCERTAIN` — `ABT/BEF/AFT/BET/FROM/TO`, precisión aproximada
- `SUSPICIOUS` — anomalía que requiere verificación (edad inusual, duplicado)
- `IMPOSSIBLE` — contradicción cronológica o ciclo; marcado como `SUSPICIOUS`, no como hecho.

NeoGenealogy detecta/compara/prioriza/sugiere pero **no inventa** padres, fechas, lugares ni fuentes.
