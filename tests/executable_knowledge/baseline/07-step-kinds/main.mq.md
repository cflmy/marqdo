---
markup: occlusion
title: S1 步骤 kind 分类样例
status: implemented
import:
  json: lib/json.mq.md
---

# main

`列` = > json.parse text=["一","二"]

**a = [取列表首项](access(`列`, 1))**
**b = [两件商品的小计](2 * 30)**
- [篮子里的每个果子](遍历 `列` 为 `果`)
  > print text=`果`
> print text=`a`
> print text=`b`
