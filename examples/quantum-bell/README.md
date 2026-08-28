# quantum-bell

Bell |Φ⁺⟩ via `ext/quantum` circuit table + draw (default `theme=dark` tech lab SVG).

```bash
cargo build -p marqdo_plugin_quantum
marqdo ext add quantum   # after rebuild, so view loads the new .so
marqdo run examples/quantum-bell/index.mq.md
```

Writes `bell.svg` next to the program (sandbox cwd). Themes: `theme="dark"|"light"|"bw"` — see [ext-quantum-viz-style.md](../../doc/design/ext-quantum-viz-style.md).
