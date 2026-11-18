# Marqdo 知识锚点 + 自动证明示例

面向 CLI 2.8.0，原 Markdown 语法不变：

```sh
marqdo knowledge examples/knowledge-proof/
marqdo prove examples/knowledge-proof/main.mq.md --json
marqdo knowledge examples/knowledge-proof/
```

知识编译后，查看 `.marqdo/anchors.json`、`.marqdo/claims.json`、`.marqdo/refs.json`、`.marqdo/graph/edges.json` 以及 `.marqdo/proof-index.json`（具体输出目录取决于 CLI --out）。
证明调用另外生成其证明目录下的 `knowledge-links.json`。运行 CLI 时是否有 Lean 决定该定理能否获得真正的 `proved` 状态；没有 Lean 只能是 `checked` 或 `unverified`。在此案例中，`长度(轨迹)=0` 是函数级定理，并非内部的“计算结果”步骤已经独立证明正确。

`[计算知识节点](#@计算结果)` 是**同函数内的静态知识引用**。跨文件引用请从 `anchors.json` 拿完整 `mq://...#anchor-...` 地址，不要猜测哈希。两个同名标记不共享地址，会被标记为无公共锚点。
