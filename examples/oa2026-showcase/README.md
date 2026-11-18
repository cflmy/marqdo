# oa2026-showcase

用于 2026 开源项目大赛作品介绍文档的可运行示例。

| 文件 | 展示的能力 | 运行 |
|------|------------|------|
| `add.mq.md` | 叙述统一、粗体即代码、斜体即返回 | `marqdo run examples/oa2026-showcase/add.mq.md` |
| `invoice.mq.md` | 表格即数据、循环、分支、断言 | `marqdo run examples/oa2026-showcase/invoice.mq.md` |
| `calculus.mq.md` | 公式即变量、求导、求根、绘图 | `marqdo run examples/oa2026-showcase/calculus.mq.md` |
| `bell.mq.md` | 量子电路、概率、断言、SVG 派生物 | `marqdo run examples/oa2026-showcase/bell.mq.md` |
| `foreign-node.mq.md` | 外联一次性执行、持久会话、显式传值 | `marqdo run examples/oa2026-showcase/foreign-node.mq.md` |

## 预期输出

```text
$ marqdo run examples/oa2026-showcase/add.mq.md
42

$ marqdo run examples/oa2026-showcase/invoice.mq.md
960
912
966
within-cap

$ marqdo run examples/oa2026-showcase/calculus.mq.md
3*x^2 - 3
9
[-1.7320508076, 0, 1.7320508076]
plot-ok

$ marqdo run examples/oa2026-showcase/foreign-node.mq.md
{华东: 420, 华北: 220, 华南: 230}
华东: 420
华南: 230
华北: 220

华东: 420
华北: 300
华南: 230

$ marqdo run examples/oa2026-showcase/bell.mq.md
{00: 0.5, 11: 0.5}
0.5
0.5
bell-00-ok
bell-11-ok
bell-draw-ok
```

`bell.mq.md` 运行后会生成 `bell-circuit.svg` 与 `bell-probs.svg`；
`calculus.mq.md` 运行后会生成 `calculus-plot.svg`。
