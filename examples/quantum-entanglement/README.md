# quantum-entanglement

Bell |Φ⁺⟩ **entanglement lab** via `ext/quantum` Q7/Q8: density matrix, partial trace, Schmidt entropy, and SVG plots (circuit themed dark by default; hinton / qsphere / multibloch / city).

```bash
cargo build --release -p marqdo_plugin_quantum
marqdo ext add quantum   # once (re-run after rebuilding the plugin)
marqdo run examples/quantum-entanglement/index.mq.md
# optional: marqdo view examples/quantum-entanglement/
```

Prints probabilities, full/reduced purity, ⟨ZZ⟩, and Schmidt entropy; writes SVGs next to the program (sandbox cwd). Circuit/probs/bloch accept `theme="dark"|"light"|"bw"`.

Design: [ext-quantum-q7.md](../../doc/design/ext-quantum-q7.md) · [ext-quantum-viz-style.md](../../doc/design/ext-quantum-viz-style.md) · sibling: [quantum-bell](../quantum-bell/).
