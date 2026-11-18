# linalg-svd

Dense SVD via `ext/linalg`: singular values + structure / hinton SVG.

```bash
cargo build --release -p marqdo_plugin_linalg
marqdo ext add linalg
marqdo run examples/linalg-svd/index.mq.md
# optional: marqdo view examples/linalg-svd/
```

Design: [ext-linalg.md](../../doc/design/ext-linalg.md) · sibling: [linalg-least-squares](../linalg-least-squares/).
