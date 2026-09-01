# Source Coverage

## Definición

Mide qué proporción de entidades tienen documentación básica. Todos los valores son porcentajes 0–100.

- **Birth coverage**: `persons_with(birth_date || birth_place) / persons`
- **Marriage coverage**: `families_with(marriage_date || marriage_place) / families` (0 si no hay familias)
- **Death coverage**: `persons_with(death_date || death_place) / persons`
- **Other event coverage**: `persons_with(event_type ∉ {BIRT,DEAT,MARR}) / persons` (bautismo, residencia, etc.)
- **Overall**: `avg(birth, marriage, death, other) + persons_with_source) / 2` donde `persons_with_source = persons_with(sources.nonEmpty)/persons *100`

## Cobertura por rama

Para cada apellido:

```
source_coverage_branch = persons_in_branch_with_source / persons_in_branch *100
```

Se muestra junto al branch score:

```
García — 92/100
Source Coverage: 38%  ⚠ Low source coverage
```

Ramas con `<40%` se consideran de baja cobertura documental y elevan prioridad práctica.

## Calidad de fuentes

Se distingue:

- `source exists` — `0 @S1@ SOUR` definido
- `source linked` — `1 SOUR @S1@` en persona/familia
- `source has citation` — `SOUR` con `TITL/AUTH/PUBL/PAGE/TEXT`
- `source has evidence` — cita vinculada a evento/persona con `PAGE` específica

Un `SOUR` con `Book 4, Page 127, Entry 32` tiene más calidad que un `SOUR` genérico `FamilySearch` sin cita. `scoring::confidence_for` bonifica +0.15 si hay cita vs +0.08 si solo genérico; `overall` no inventa calidad sin evidencia.

## Integración

`analyzer::source_coverage(tree) -> SourceCoverage` y `branch_analyses` se consumen en CLI `--explain-score`, `stats`, `report.html` y JSON `source_coverage`.
