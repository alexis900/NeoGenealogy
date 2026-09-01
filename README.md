# NeoGenealogy

Herramienta local de análisis genealógico. La Fase 1 ofrece importación GEDCOM conservadora, detección de hallazgos, puntuación de oportunidades y reportes JSON/HTML.

## Uso

```bash
cargo run -p neogenealogy -- analyze test-data/simple.ged --output analysis.json
cargo run -p neogenealogy -- import test-data/simple.ged
cargo run -p neogenealogy -- stats test-data/simple.ged
cargo run -p neogenealogy -- report test-data/simple.ged --output report.html
```

El importador mantiene etiquetas no reconocidas en `Person.raw`/`Family.raw`; los datos originales no se normalizan destructivamente. La interfaz `GedcomParser` permite sustituirlo por un parser de GEDCOM 7 o GEDCOM-X en una fase posterior.

