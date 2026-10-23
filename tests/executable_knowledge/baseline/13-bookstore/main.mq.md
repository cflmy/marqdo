---
markup: occlusion
title: S4 书店订单含税（可运行子集）
status: implemented
import:
  tax: tax.mq.md
---

# main

一行订单：金额 100，按标准税率含税。

**税率 = [适用税率](tax.标准税率())**
**含税 = [本行含税价](100 * (1 + 税率))**
这是[含税总价](含税)的可读结论。
> print text=`含税`
