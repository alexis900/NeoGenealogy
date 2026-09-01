# Testing

Ejecutar:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets --all-features
```

Los fixtures de `test-data/` cubren fechas, cronología, duplicados, lugares, huecos y una familia compleja.
