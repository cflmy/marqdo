---
markup: occlusion
id: knowledge.negative.discount-not-for-refund
type: Failure
title: 折扣不适用于退款
---

# 折扣不适用于退款

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Object/type namespace for the child functions below; call the intended child directly |

负知识：退款结算不得套用会员折扣。Agent 复用折扣能力时应拒绝该适用面。

## 说明

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | The value explicitly returned below; conditional returns follow the selected branch |

After these steps, return *[不适用](`false`)*.
