# 代码即文档 · 周末咖啡手账

可读可跑的 v0.3 样例：`.mq.md` 同时是说明文和程序。

```bash
marqdo run examples/code-as-docs/index.mq.md
```

## 这份样例在示范什么

| 习惯 | 写法 |
|------|------|
| 数据给人读 | GFM 表当菜单 / 订单 |
| 步骤嵌在句子里 | `**总额 = 价 * 杯数**`、返回 `*总额*` |
| 装饰性强调 | `**说明**` 不执行 |
| 段落分隔 | `---` 只是分割线 |
| 浮点 | `折扣`=0.85 |
| 取元 | `[拿铁](菜单)` / `` [`名`](菜单) ``（与循环同形；脚注 `[^…]` 仅过渡） |
| 循环 | `- [行](今日订单)` |
| 调用 | `**摘要 = 结账摘要 …**`（粗体 RHS 可省略 `>`） |

设计：[collection-access-link.md](../../doc/design/collection-access-link.md)、[code-as-docs-gaps.md](../../doc/design/code-as-docs-gaps.md)。
