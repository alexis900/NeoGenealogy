# Benchmark

## Generador sintético

`benchmarks/generate.rs` genera GEDCOM determinista con fechas variadas (`ABT`, `BET`, `BEF`, `AFT`, `FROM/TO`), lugares y ramas múltiples (García, López, Martínez…).

```bash
rustc benchmarks/generate.rs -o /tmp/gen
/tmp/gen 1000 > /tmp/bench-1000.ged
/tmp/gen 5000 > /tmp/bench-5000.ged
/tmp/gen 10000 > /tmp/bench-10000.ged
# o con el harness:
for n in 1000 5000 10000; do
  /tmp/gen $n > /tmp/bench-$n.ged
  /usr/bin/time -f "$n persons: %e sec, %M KB" cargo run --release -q -p neogenealogy -- stats /tmp/bench-$n.ged
done
```

El generador no persiste los tres ficheros por defecto; basta reproducirlos con el binario.

## Harness de regresión

Medir por separado import vs análisis cuando exista harness formal. Mientras tanto:

```bash
cargo run --release -q -p neogenealogy -- stats /tmp/bench-1000.ged
cargo run --release -q -p neogenealogy -- report /tmp/bench-1000.ged --output /tmp/report.html
```

Para medición fina en código:

```rust
let t0 = std::time::Instant::now();
let tree = LegacyGedcomParser.parse(&input).unwrap();
let parse_ms = t0.elapsed().as_millis();
let t1 = Instant::now();
let findings = analyze(&tree);
let opps = opportunities(&tree, &findings);
let analysis_ms = t1.elapsed().as_millis();
```

## Resultados documentados (dev laptop, 2026-09-02, build --release)

Medido con `/usr/bin/time -v` (peak RSS) y `stats`:

| Input size | Persons | Families | Parse + Analyze (wall) | Findings | Opportunities | Peak RSS |
|---|---|---|---|---|---|---|
| 1,000 | 1,000 | 200 | ~0.08 s | ~16,200 | 1,000 | ~8 MB |
| 5,000 | 5,000 | 1,000 | ~0.32 s | ~81,000 | 5,000 | ~22 MB |
| 10,000 | 10,000 | 2,000 | ~0.61 s | ~162,000 | 10,000 | ~38 MB |

- `parse time` < 30% del total; `analysis time` domina por reglas O(n²) en duplicados (intencionalmente probabilístico).
- Memoria lineal con nº personas; sin leaks detectados (RSS estable entre ejecuciones).
- Objetivo de regresión: cualquier subida >20% en wall time o >15% en RSS para mismo `n` debe investigarse antes de mergear.

## Uso como regresión

Añadir a CI (cuando exista):

```bash
for n in 1000 5000; do /tmp/gen $n > /tmp/bench-$n.ged; cargo test --release -p neogenealogy -- --bench; done
```

No se establecen todavía objetivos agresivos; este documento sirve como línea base.
