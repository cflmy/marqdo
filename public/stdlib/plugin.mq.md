---
markup: occlusion
title: lib/plugin — native ABI
description: Load optional shared libraries (.dll / .so / .dylib)
import:
  plugin: lib/plugin.mq.md
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

Import lib/plugin.mq.md. Functions: load(path), unload(), list(). After load, registered plugin names are callable like other functions. C ABI: include/marqdo_abi.h — see doc/design/ext-abi.md. Path must stay under the program sandbox (cwd / fs root).

We first **names = [Compute names](plugin.list())** for the next step.

We first **n = [Compute n](len(`names`))** for the next step.

1. `n` >= 0
  We now **[Display this step's result](> print text="plugin-ok")**.
2. *
  We now **[Display this step's result](> print text="plugin-bad")**.
