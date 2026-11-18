---
markup: occlusion
title: AI native flow offline smoke
import:
  flow: ext/ai/flow.mq.md
  context: ext/ai/context.mq.md
  json: lib/json.mq.md
  sys: lib/sys.mq.md
---

# main

This fixture tests the real Marqdo interface without a model provider.
It refuses an invalid choice and retrieves only the designated function
rather than guessing another SKILL or importing the full document.

**opts = > json.parse text=["chat","office"]**
**pick = > flow.parse_choice reply="选择：office" options=`opts`**
**value = > json.get value=`pick` key="value"**
1. `value` == "office"
  **_ = 1**
2. *
  > sys.exit code=11

**invalid = > flow.parse_choice reply="chat 或 office" options=`opts`**
**not_ok = > json.get value=`invalid` key="ok"**
1. `not_ok`
  > sys.exit code=12
2. *
  **_ = 1**

**ctx = > context.scope scope="function" class="main" function="probe" max_chars=200**
**ok = > json.get value=`ctx` key="ok"**
1. `ok`
  **_ = 1**
2. *
  > sys.exit code=13

Return *"ai-flow-native-ok"*.

## probe

This private helper's narrative should be extracted by the scoped selector
without including the root SKILL or neighboring function definitions.

Return *"probe"*.
