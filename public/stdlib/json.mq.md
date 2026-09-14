---
title: lib/json — JSON
description: Shared EN/ZH path; parse and stringify (code-as-docs pilot)
import json:lib/json.mq.md
---

# main

Shared JSON helpers for EN and ZH docs. Prefer parse, stringify, and quote.
Build maps with tables or table.put — not json.set chains.

**`obj` = [json.parse] text="{}"**
**`ty` = > type `obj`**
**打印 内容=`ty`**

**`out` = > json.stringify value=`obj`**
**打印 内容=`out`**
