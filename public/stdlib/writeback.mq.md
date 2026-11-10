---
markup: occlusion
title: lib/writeback — Jupyter-style writeback
description: Persist run output in the entry .mq.md file
import:
  writeback: lib/writeback.mq.md
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

Writeback stores output in `<!-- marqdo-out … -->` blocks in the **entry** file (the one you run).

record value= — default: insert or replace the block below the call; at_end=true → single block at EOF.

get / clear — read or remove the adjacent block (or EOF block when at_end=true).

list — all output blocks in the entry file.

We now **[Display this step's result](> print text="writeback overview ok")**.
