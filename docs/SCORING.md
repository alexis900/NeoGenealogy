# Research Score v2

El Research Score es `0–100` y representa la prioridad de investigación para una persona, no una conclusión genealógica.

## Dimensiones

El motor evalúa seis dimensiones conceptuales:

- **Genealogical importance**: ser antepasado directo (+30) o rama colateral (+12)
- **Data gap**: progenitores ausentes (+20) o un progenitor faltante (+12), fecha/lugar faltantes implícito
- **Researchability**: `High` (+9), `Medium` (+5), `Low` (+1) según disponibilidad de lugar y fecha
- **Evidence quality**: hallazgos asociados (+10), calidad de fuente (sin fuente +8, genérica +5, con cita +2)
- **Source potential**: potencial de encontrar bautismo/matrimonio si hay localidad y fecha aproximada
- **Confidence**: separado del score (0.0–1.0), basado en precisión de fecha, lugar y citas

## Fórmula

```
score = clamp( sum(component.points), 0, 100 )
confidence = clamp( base(0.5) + precision_bonus + place_bonus + citation_bonus - anomaly_penalty, 0.1, 0.98 )
priority = Critical >=85 | High >=65 | Medium >=35 | Low <35
researchability = High si lugar && fecha, Medium si uno, Low si ninguno o año <1500
```

Cada oportunidad expone `ScoreBreakdown { total, components: Vec<ScoreComponent> }` donde:

```rust
struct ScoreComponent { name: String, points: i32, reason: String }
```

Ejemplo `--explain-score`:

```
Direct ancestor              +30
Missing parent               +20
Known locality               +15
Known birth date             +10
Existing related evidence    +10
No source linked              +8
High researchability           +9
                              ---
Total                         92 (confidence 81%)
```

## Separación score / confidence / priority

- **score**: importancia + viabilidad para priorizar.
- **confidence**: fiabilidad de los datos de partida.
- **priority**: derivada del score para ordenación.

No se asignan valores arbitrarios: todo proviene de `GenealogyTree` y `Findings` reales.

## Branch Score

Deriva de oportunidades reales de la rama (apellido):

```
top5 = oportunidades de la rama ordenadas descendente, take 5
max  = máximo score en la rama
avg_top5 = promedio de top5
branch_score = round(0.6 * max + 0.4 * avg_top5)  ∈ [0,100]
```

La calidad importa más que la cantidad: 200 problemas triviales (20) no superan un único hueco crítico (94).
