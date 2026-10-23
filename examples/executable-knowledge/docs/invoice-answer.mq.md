---
markup: occlusion
id: document.invoice-answer
type: Answer
import:
  pricing: ../capabilities/order-pricing.mq.md
  json: lib/json.mq.md
---

# main

发票回答：演示订单一行图书，未税金额 100。

`订单` = > json.parse text={"行":[{"金额":100,"品类":"书"}]}
**总额 = > pricing.订单定价.含税总额 订单=`订单`**

这张订单的含税总额是[含税总价](总额)。

> print text=`总额`
