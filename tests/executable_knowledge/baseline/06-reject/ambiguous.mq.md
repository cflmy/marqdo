---
markup: occlusion
title: S0 拒绝复用：裸名无法消歧（现状基线）
status: expected_failure
description: >
  两模块均定义「商品小计」。当前默认语法要求显式路径调用；
  裸名调用不导入任何一方时，应失败而非猜一个目标。
  （同名已 import 仍误连是 S1 验收项，不在本负例。）
import:
  a: a.mq.md
  b: b.mq.md
---

# main

**小计 = [未消歧的同名调用](商品小计(数量=2, 单价=30))**
> print text=`小计`
