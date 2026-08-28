# quantum-entanglement

Bell |Φ⁺⟩ **entanglement lab** via `ext/quantum` Q7: density matrix, partial trace, Schmidt entropy, and advanced SVG (hinton / qsphere / multibloch / city).

```bash
cargo build --release -p marqdo_plugin_quantum
marqdo ext add quantum   # once
marqdo run examples/quantum-entanglement/index.mq.md
```

Prints probabilities, full/reduced purity, ⟨ZZ⟩, and Schmidt entropy; writes SVGs next to the program (sandbox cwd).

Design: [ext-quantum-q7.md](../../doc/design/ext-quantum-q7.md) · sibling demo: [quantum-bell](../quantum-bell/).
