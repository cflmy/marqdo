---
title: ext/ai/agent
description: Thin agent framework — ctor, run, clear_history only.
> ext/ai/llm.mq.md
---

## build_agent_context
    - agent
    - extra

Internal: assemble prompt (user prompt, extra, tool list, source, call site, history, Marqdo skill).

*`user_prompt` = > get value=`agent` key=user_prompt *
+ `user_prompt`
  *`up` = `user_prompt`*
+ *
  *`up` = None*

*`tools` = > get value=`agent` key=tools *
*`tools_s` = > host_format_tools_for_llm tools=`tools` *
*`src` = > host_module_source *
*`site` = > host_call_site *
*`site_s` = > stringify value=`site` *
*`id` = > get value=`agent` key=id *
*`hist` = > host_agent_history_get id=`id` *
*`hist_s` = > stringify value=`hist` *
*`skill` = > host_marqdo_skill *
*`extra_s` = > stringify value=`extra` *

*`sep` = > parse text={"a":"\n\n--- standing ---\n","b":"\n\n--- extra ---\n","c":"\n\n--- tools ---\n","d":"\n\n--- call site ---\n","e":"\n\n--- source ---\n","f":"\n\n--- history ---\n","g":"\n\n--- marqdo skill ---\n"} *
*`a` = > get value=`sep` key=a *
*`b` = > get value=`sep` key=b *
*`c` = > get value=`sep` key=c *
*`d` = > get value=`sep` key=d *
*`e` = > get value=`sep` key=e *
*`f` = > get value=`sep` key=f *
*`g` = > get value=`sep` key=g *

*`tool_proto` = \n\n--- tool protocol ---\nEach tool is a zero-arg Marqdo function in this module (see tools list). To call one, reply with exactly one line: TOOL:<name>. No other text on that line.\nAfter the tool runs you will get its output and should answer the user.\n *

*`p` = You are a Marqdo agent. Follow Marqdo skill rules when reading or writing .mq.md. *
*`p` = `p` + `a` + `up` + `b` + `extra_s` + `c` + `tool_proto` + `tools_s` + `d` + `site_s` + `e` + `src` + `f` + `hist_s` + `g` + `skill` *
**`p`**

# agent
    - model
    - tools
    - user_prompt

*`id` = > host_agent_alloc *
*`h` = > parse text={"_type":"agent"} *
*`h` = > map_set map=`h` key=id value=`id` *
*`h` = > map_set map=`h` key=model value=`model` *
*`h` = > map_set map=`h` key=tools value=`tools` *

+ `user_prompt`
  *`h` = > map_set map=`h` key=user_prompt value=`user_prompt` *
+ *
  *`_` = 1*

**`h`**

## clear_history

*`id` = > get value=`self` key=id *
> host_agent_history_clear id=`id`
****

## run
    - extra

*`ctx` = > build_agent_context agent=`self` extra=`extra` *
*`id` = > get value=`self` key=id *

*`user_turn` = > parse text={"role":"user"} *
*`user_turn` = > map_set map=`user_turn` key=content value=`extra` *
> host_agent_history_append id=`id` item=`user_turn`

*`model` = > get value=`self` key=model *
*`reply` = > `model`.complete prompt=`ctx` *
*`tools` = > get value=`self` key=tools *
*`reply` = > trim value=`reply` *

*`parts` = > split value=`reply` sep=: *
*`head` = > at value=`parts` index=0 *
*`tool_name` = > at value=`parts` index=1 *
*`tool_name` = > trim value=`tool_name` *

+ `head` == TOOL
  *`allowed` = > host_tool_allowed tools=`tools` name=`tool_name` *
  + `allowed`
    *`tool_out` = > call_fn name=`tool_name` *
    *`tool_s` = > stringify value=`tool_out` *
    *`extra_s` = > stringify value=`extra` *
    *`fp` = The user asked: *
    *`fp` = `fp` + `extra_s` *
    *`fp` = `fp` + . You invoked tool *
    *`fp` = `fp` + `tool_name` *
    *`fp` = `fp` + which returned: *
    *`fp` = `fp` + `tool_s` *
    *`fp` = `fp` + . Reply to the user in one brief sentence. *
    *`reply` = > `model`.complete prompt=`fp` *
  + *
    *`reply` = Tool not allowed: `tool_name` *
+ *
  *`_` = 1*

*`as_turn` = > parse text={"role":"assistant"} *
*`as_turn` = > map_set map=`as_turn` key=content value=`reply` *
> host_agent_history_append id=`id` item=`as_turn`
**`reply`**
