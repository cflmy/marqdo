# linalg-transpose

Formula-document example: declare symbols with a GFM table, check \((AB)^\top = B^\top A^\top\) via `ascii` (auto-simplify).

```bash
cargo build --release -p marqdo_plugin_linalg
marqdo ext add linalg
marqdo run examples/linalg-transpose/index.mq.md
```

Design: [ext-linalg-formula-doc.md](../../doc/design/ext-linalg-formula-doc.md).
