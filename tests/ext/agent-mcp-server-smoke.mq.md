---
title: agent mcp server smoke
description: Offline assemble mcp_server + one tool (no serve).
import agent:ext/ai/agent.mq.md
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
import demo:agent-mcp-server-helpers.mq.md
---

# main

*p = > plugin.native_path name="agent"*
1. `p`
  > plugin.load path=`p`
2. *
  > sys.exit code=1

*srv = > agent.mcp_server name="marqdo-test"*
*srv = > `srv`.tool name="ping" fn="demo.ping" description="ping tool"*
*tools = srv[^tools]*
*n = > len value=`tools`*
1. `n` == 1
  > print text=mcp-tool-ok
2. *
  > print text=mcp-tool-fail

*t0 = tools[^1]*
*fn = t0[^fn]*
1. `fn` == "demo.ping"
  > print text=mcp-fn-ok
2. *
  > print text=mcp-fn-fail
