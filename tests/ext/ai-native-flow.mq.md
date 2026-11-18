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
  **debug_pick = > json.stringify value=`pick`**
  > print text=`debug_pick`
  > sys.exit code=11

**structured = > flow.parse_choice reply="{\"choice\":\"chat\"}" options=`opts`**
**structure_value = > json.get value=`structured` key="value"**
1. `structure_value` == "chat"
  **_ = 1**
2. *
  > sys.exit code=16

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

**packed = > context.compose user="Test the route" class="main" function="probe" budget_tokens=2048**
**pack_schema = > json.get value=`packed` key="schema"**
1. `pack_schema` == "ContextPackV2"
  **_ = 1**
2. *
  > sys.exit code=15

**gate = > flow.input id="ai_native_flow_ci_gate" question="Continue?" options=`opts` source_revision="fixture-v1" freeform=False**
**answered = > flow.reply id="ai_native_flow_ci_gate" reply="office" source_revision="fixture-v1"**
**answer = > json.get value=`answered` key="value"**
1. `answer` == "office"
  **_ = 1**
2. *
  > sys.exit code=14

Return *"ai-flow-native-ok"*.

## probe

This private helper's narrative should be extracted by the scoped selector
without including the root SKILL or neighboring function definitions.

Return *"probe"*.
