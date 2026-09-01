# Benchmark

El benchmark de importación se puede ejecutar con el binario optimizado y `/usr/bin/time`:

```bash
for n in 1000 5000 10000; do
  /usr/bin/time -f "$n persons: %e sec, %M KB" cargo run --release -q -p neogenealogy -- stats test-data/complex.ged
done
```

La medición debe registrar por separado importación/análisis cuando se añada el harness formal; esta fase evita optimizar antes de tener datos reales.
