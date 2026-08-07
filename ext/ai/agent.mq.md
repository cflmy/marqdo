---
title: ext/ai/agent
description: Document-driven agent — step / plan; tools via lib/subtask; runtime via ABI v2 agent plugin.
> ext/ai/llm.mq.md
> lib/json.mq.md
> lib/sys.mq.md
> lib/writeback.mq.md
> lib/subtask.mq.md
> lib/plugin.mq.md
---

## build_step_context
    + `agent`
    + `task`

Assemble a readable prompt: standing, task, tools, call site, source, skill, and the act protocol (human-visible).

*`standing` = > json.get value=`agent` key=standing *
1. `standing`
  *`up` = `standing`*
2. *
  *`up` = None*

*`tools` = > json.get value=`agent` key=tools *
*`tools_s` = > agent_format_tools tools=`tools` *
*`src` = > agent_module_source *
*`site` = > agent_call_site *
*`site_s` = > json.stringify value=`site` *
*`skill` = > agent_marqdo_skill *
*`task_s` = > json.stringify value=`task` *

*`esc` = > json.parse text={"a":"\n\n--- standing ---\n","b":"\n\n--- task ---\n","c":"\n\n--- tools ---\n","d":"\n\n--- call site ---\n","e":"\n\n--- source (.mq.md) ---\n","f":"\n\n--- marqdo skill ---\n","g":"\n\n--- how to act ---\nTools are ## functions in the source above. To call one, reply with exactly one line: CALL:<name>\n(Chinese ok: 调用：<name>)\nOtherwise reply with the final answer only.\nTools run via Marqdo subtask (spawn fn / wait).\n"} *
*`a` = > json.get value=`esc` key=a *
*`b` = > json.get value=`esc` key=b *
*`c` = > json.get value=`esc` key=c *
*`d` = > json.get value=`esc` key=d *
*`e` = > json.get value=`esc` key=e *
*`f` = > json.get value=`esc` key=f *
*`g` = > json.get value=`esc` key=g *

*`p` = You are a Marqdo agent. The runbook source is your ground truth — code is documentation. *
*`p` = `p` + `a` + `up` + `b` + `task_s` + `c` + `tools_s` + `g` + `d` + `site_s` + `e` + `src` + `f` + `skill` *
**`p`**

---

## extract_tool_name
    + `reply`

Find a human-readable call line: `CALL:<name>` or `调用：<name>` / `调用:<name>`.

*`esc` = > json.parse text={"nl":"\n"} *
*`nl` = > json.get value=`esc` key=nl *
*`text` = > trim value=`reply` *
*`lines` = > split value=`text` sep=`nl` *
*`found` = None*

- [`line`](`lines`)
  *`t` = > trim value=`line` *
  *`parts` = > split value=`t` sep=: *
  *`n` = > len value=`parts` *
  1. `n` > 1
    *`head` = > at value=`parts` index=0 *
    *`head` = > trim value=`head` *
    *`rest` = > at value=`parts` index=1 *
    *`rest` = > trim value=`rest` *
    1. `head` == CALL
      *`found` = `rest`*
    2. `head` == 调用
      *`found` = `rest`*
    3. *
      *`_` = 1*
  2. *
    *`_` = 1*

**`found`**

---

## run_tool
    + `tools`
    + `name`

Allowlist check, then invoke the runbook `##` via `lib/subtask` (`spawn fn=` → `wait`).

*`allowed` = > agent_tool_allowed tools=`tools` name=`name` *
1. `allowed`
  *`id` = > subtask.spawn fn=`name` *
  **> subtask.wait id=`id`**
2. *
  *`msg` = Tool not allowed: *
  **`msg` + `name`**

# agent
    + `model`
    + `tools`
    + `standing`

Load ABI v2 agent plugin once (session bag + context queries).

*`p` = > plugin.native_path name=agent *
1. `p`
  > plugin.load path=`p`
2. *
  > print text=ext/ai/agent: native agent plugin not found (build marqdo_plugin_agent or marqdo ext add agent)
  > sys.exit code=1

*`id` = > agent_alloc *
*`h` = > json.parse text={"_type":"agent"} *
*`h` = > json.set map=`h` key=id value=`id` *
*`h` = > json.set map=`h` key=model value=`model` *
*`h` = > json.set map=`h` key=tools value=`tools` *

1. `standing`
  *`h` = > json.set map=`h` key=standing value=`standing` *
2. *
  *`_` = 1*

**`h`**

## clear_history

Clear the plugin session bag for this agent id.

*`id` = > json.get value=`self` key=id *
> agent_history_clear id=`id`
****

## step
    + `task`
    + `writeback`=True

One atomic turn: context → LLM → optional tool via subtask. Returns a **map** (`status`, `task`, `decision`, and on success `result` plus optional `tool` / `tool_result`; on failure `error`). By default (`writeback=True`) persists that map under named slots `ok` / `error` at the call site; pass `writeback=False` to skip.

*`ctx` = > build_step_context agent=`self` task=`task` *
*`id` = > json.get value=`self` key=id *
*`model` = > json.get value=`self` key=model *
*`tools` = > json.get value=`self` key=tools *

*`user_turn` = > json.parse text={"role":"user"} *
*`user_turn` = > json.set map=`user_turn` key=content value=`task` *
> agent_history_append id=`id` item=`user_turn`

*`reply` = > `model`.complete prompt=`ctx` *
*`reply` = > trim value=`reply` *
*`decision` = `reply`*
*`tool_name` = > extract_tool_name reply=`reply` *
*`out` = > json.parse text={"status":"ok"} *
*`out` = > json.set map=`out` key=task value=`task` *
*`out` = > json.set map=`out` key=decision value=`decision` *

1. `tool_name`
  *`tool_out` = > run_tool tools=`tools` name=`tool_name` *
  *`tool_s` = > json.stringify value=`tool_out` *
  *`deny` = > split value=`tool_s` sep=Tool not allowed *
  *`deny_n` = > len value=`deny` *
  1. `deny_n` > 1
    *`out` = > json.set map=`out` key=status value=error *
    *`out` = > json.set map=`out` key=error value=`tool_s` *
    *`reply` = `tool_s`*
  2. *
    *`task_s` = > json.stringify value=`task` *
    *`fp` = The user asked: *
    *`fp` = `fp` + `task_s` *
    *`fp` = `fp` + . Tool *
    *`fp` = `fp` + `tool_name` *
    *`fp` = `fp` + ran via subtask and returned: *
    *`fp` = `fp` + `tool_s` *
    *`fp` = `fp` + . Reply to the user briefly. *
    *`reply` = > `model`.complete prompt=`fp` *
    *`out` = > json.set map=`out` key=tool value=`tool_name` *
    *`out` = > json.set map=`out` key=tool_result value=`tool_s` *
    *`out` = > json.set map=`out` key=result value=`reply` *
2. *
  *`out` = > json.set map=`out` key=result value=`reply` *

*`as_turn` = > json.parse text={"role":"assistant"} *
*`as_turn` = > json.set map=`as_turn` key=content value=`reply` *
> agent_history_append id=`id` item=`as_turn`

1. `writeback`
  *`body` = > json.stringify value=`out` *
  *`st` = > json.get value=`out` key=status *
  1. `st` == ok
    > writeback.record value=`body` key=ok
  2. *
    > writeback.record value=`body` key=error
2. *
  *`_` = 1*

**`out`**

## plan
    + `goal`
    + `workbook_dir`

Multi-step workbook (D2) — not implemented yet.

*`_` = `workbook_dir`*
*`msg` = plan (multi-step workbook) is not implemented yet; goal was: *
**`msg` + `goal`**
