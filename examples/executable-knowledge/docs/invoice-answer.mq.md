---
markup: occlusion
id: document.invoice-answer
type: Answer
import:
  pricing: ../capabilities/order-pricing.mq.md
  json: lib/json.mq.md
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

发票回答：演示订单一行图书，未税金额 100。

We first **订单 = [Compute 订单](json.parse(text="{\"行\":[{\"金额\":100,\"品类\":\"书\"}]}"))** for the next step.
We first **总额 = [Compute 总额](pricing.订单定价.含税总额(订单=`订单`))** for the next step.

这张订单的含税总额是[含税总价](总额)。

We now **[Display this step's result](> print text=`总额`)**.
