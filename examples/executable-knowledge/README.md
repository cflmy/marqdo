# 可执行知识系统示例：订单含税总额

这是 [可执行知识系统设计](../../doc/design/executable-knowledge-system.md) 的目标示例，
已按当前 occlusion 语法落地为**可运行**订单链（Tree / Bytecode）。

| 文件 | 图中的职责 | 关键步骤 |
|---|---|---|
| `corpus/tax-policy-2026.md` | E：证据 | 税率原文 |
| `schema/order.mq.md` | C：概念 | 订单/订单行字段与单位 |
| `knowledge/tax-2026.mq.md` | K：原子知识 | `标准税率.查询` + `sources:` |
| `knowledge/negative-refund.mq.md` | 负知识 | 折扣不适用于退款 |
| `capabilities/order-pricing.mq.md` | D：推导知识 | 取元、调用、循环、汇总 |
| `docs/invoice-answer.mq.md` | N：叙述知识 | 含税结论 + NarrativeRef |

## 运行

```bash
marqdo run examples/executable-knowledge/docs/invoice-answer.mq.md --backend tree
marqdo run examples/executable-knowledge/docs/invoice-answer.mq.md --backend bytecode
# → 113
marqdo knowledge examples/executable-knowledge -o /tmp/ek
marqdo knowledge impact "标准税率.查询" examples/executable-knowledge --json
```

门禁：`cargo test --test executable_knowledge s4_examples_bookstore_chain_runs_both_backends --locked`

## 读法

`[]()` 的每一次出现都是命名步骤（observe / derive / iterate / call / narrate）：

```markdown
[订单行金额](access(行, "金额"))                 # observe
[本行含税价](金额 * (1 + 税率))                    # derive
[逐个订单行](遍历 行列表 为 行)                    # iterate
这张订单的含税总额是[含税总价](总额)。              # narrate
```

## 运行状态

| 子集 | 状态 | 证据 |
|---|---|---|
| 发票含税链双后端 `113` | **可运行** | 上表命令 / EK 门禁 |
| 来源 `sourced` + impact 下游 | **可运行** | knowledge 投影 + impact |
| 负知识文档 | **已收录** | `knowledge/negative-refund.mq.md`（发现层；非执行目标） |
