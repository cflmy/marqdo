---
title: lib/writeback — Jupyter-style writeback
description: Persist run output in the entry .mq.md file
> lib/writeback.mq.md
---

# main

Writeback stores output in `<!-- marqdo-out … -->` blocks in the **entry** file (the one you run).

- `record value=` — default: insert or replace the block **below the call**; `at_end=true` → single block at EOF
- `get` / `clear` — read or remove the adjacent block (or EOF block when `at_end=true`)
- `list` — all output blocks in the entry file

> print text=writeback overview ok
