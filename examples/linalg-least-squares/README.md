# linalg-least-squares

Overdetermined least squares via `ext/linalg`: `factorize kind=qr`, `Qᵀb`, solve `R`, plus a heatmap SVG of `A`.

```bash
cargo build --release -p marqdo_plugin_linalg
marqdo ext add linalg   # once (re-run after rebuilding the plugin)
marqdo run examples/linalg-least-squares/index.mq.md
# optional: marqdo view examples/linalg-least-squares/
```

Design: [ext-linalg.md](../../doc/design/ext-linalg.md) · sibling: [linalg-svd](../linalg-svd/).
