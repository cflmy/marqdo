---
markup: occlusion
title: 一张订单，一份可执行的业务说明
description: 叙述、数据、计算与断言共享同一个 .mq.md 源文件。
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

这份文档不是代码旁边的说明，也不是说明里贴的代码。下表既是给人读的订单，
也是运行时直接使用的记录集合；下面的句子既解释业务规则，也参与计算。

`订单` =

| @ | 客户 | 单价 | 数量 | 提前天数 |
|---|------|------|------|----------|
| 1 | 青山实验室 | 120 | 3 | 10 |
| 2 | 远望中心 | 80 | 5 | 4 |
| 3 | 星尘工坊 | 200 | 1 | 14 |

逐行把「单价 × 数量」累加进`小计`。读者看到的是清单，机器执行的是同一张表。

We first **小计 = 0** for the next step.
- [行](遍历 订单 为 行)
  We first **小计 = (`小计` + ([单价](access(`行`, "单价")) * [数量](access(`行`, "数量"))))** for the next step.

业务约定写在叙述里：提前不少于七天，整单按九五折；否则原价。
本单最早提前 14 天。

We first **最早提前 = 14** for the next step.
1. `最早提前` >= 7
  We first **折后 = ((`小计` * 95) / 100)** for the next step.
2. *
  We first **折后 = `小计`** for the next step.

把结果、依据和判断一起写进文档。下面的断言如果跑不通，文档本身就是错的。

We first **含税 = ((`折后` * 106) / 100)** for the next step.
We now **[Display this step's result](> print text=`小计`)**.
We now **[Display this step's result](> print text=`折后`)**.
We now **[Display this step's result](> print text=`含税`)**.

1. `含税` < 10000
  We now **[Display this step's result](> print text="within-cap")**.
2. *
  We now **[Display this step's result](> print text="over-cap")**.
