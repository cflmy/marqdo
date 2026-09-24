---
title: agent MCP stdio client (offline)
description: Oneshoot list+call against python mock MCP server.
import agent:ext/ai/agent.mq.md
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
import json:lib/json.mq.md
---

# main

**p = > plugin.native_path name="agent"**
1. `p`
  > plugin.load path=`p`
2. *
  > print text=no-agent-plugin
  > sys.exit code=1

**argv = > json.parse text=["tests/ext/fixtures/mcp-stdio-mock.py"]**
**listed = > agent.mcp_client_list command="python3" args=`argv`**
**tools = > json.get value=`listed` key="tools"**
**n = > len value=`tools`**
1. `n` == 1
  > print text=mcp-client-list-ok
2. *
  > print text=mcp-client-list-fail
  > print text=`listed`

**call_args = > json.parse text={"msg":"hi"}**
**called = > agent.mcp_client_call command="python3" args=`argv` tool="echo" arguments=`call_args`**
**auth = > json.get value=`called` key="authority"**
1. `auth` == "workbook"
  > print text=mcp-client-call-ok
2. *
  > print text=mcp-client-call-fail
  > print text=`called`
