---
markup: occlusion
title: S4 G05 来源与适用期
status: implemented
sources:
  - corpus.md#标准税率
applicability:
  region: CN
  valid_until: 2099-12-31
---

## 标准税率

*[当前标准税率](0.13)*

# main

**税率 = [读取标准税率](标准税率())**
**含税 = [演示含税价](100 * (1 + 税率))**
> print text=`含税`
