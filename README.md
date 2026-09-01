# NeoGenealogy

Herramienta local de análisis genealógico. Fase 1.6 — Research Engine v2: motor explicable, priorización por ramas y cobertura documental.

```
GEDCOM → Parser → Modelo → Analysis → Research Engine → Score explicable → Research Queue
```

## Uso

```bash
cargo run -p neogenealogy -- analyze test-data/simple.ged --output analysis.json
cargo run -p neogenealogy -- analyze test-data/complex.ged --explain-score
cargo run -p neogenealogy -- analyze test-data/complex.ged --severity high
cargo run -p neogenealogy -- analyze test-data/complex.ged --severity high --sort confidence
cargo run -p neogenealogy -- import test-data/simple.ged
cargo run -p neogenealogy -- stats test-data/simple.ged
cargo run -p neogenealogy -- report test-data/simple.ged --output report.html
```

El importador mantiene etiquetas no reconocidas en `Person.raw`/`Family.raw`; los datos originales no se normalizan destructivamente. La interfaz `GedcomParser` permite sustituirlo por un parser de GEDCOM 7 o GEDCOM-X en una fase posterior.

## Research Engine

Véase `docs/RESEARCH_ENGINE.md`, `docs/SCORING.md`, `docs/SOURCE_COVERAGE.md`, `docs/ANALYSIS_RULES.md`.

Destaca:

- `neogenealogy analyze --explain-score` muestra `ScoreBreakdown { total, components: [{name, points, reason}] }` y `confidence` separado.
- `branches` con `branch_score = 0.6*max + 0.4*avg(top5)` — calidad sobre cantidad.
- `source_coverage` (birth/marriage/death/events/overall) y por rama.
- Detección de ciclos `RELATIONSHIP_ANOMALY Critical` sin romper el análisis.

## Benchmark

```bash
rustc benchmarks/generate.rs -o /tmp/gen
/tmp/gen 1000 > /tmp/bench-1000.ged
cargo run --release -p neogenealogy -- stats /tmp/bench-1000.ged
```

Véase `benchmarks/README.md` para resultados y regresión.

## Verificación

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features
cargo test --workspace
```
