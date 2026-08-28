---
title: agent corpus_search + mcp fixture tools (A4 offline)
description: Local RAG-lite corpus search and MCP fixture call; authority stays workbook.
import agent:ext/ai/agent.mq.md
import fs:lib/fs.mq.md
import json:lib/json.mq.md
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
---

# main

*p = > plugin.native_path name="agent"*
1. `p`
  > plugin.load path=`p`
2. *
  > print text=no-agent-plugin
  > sys.exit code=1

*paths = > json.parse text={"root":".marqdo/agent-corpus-a4","fx":".marqdo/agent-runs/a4-mcp-fixture.json","a":".marqdo/agent-corpus-a4/refunds.md","b":".marqdo/agent-corpus-a4/shipping.md"}*
*root = > json.get value=`paths` key="root"*
*fx = > json.get value=`paths` key="fx"*
*pa = > json.get value=`paths` key="a"*
*pb = > json.get value=`paths` key="b"*

1. > fs.exists path=`root`
  > fs.remove path=`root`
2. *
  *_ = 1*
> fs.make_dir path=`root`

*ba = > json.parse text={"t":"# Refund policy\n\nCustomers may request a refund within 30 days of purchase.\n"}*
*body_a = > json.get value=`ba` key="t"*
> fs.write_text path=`pa` text=`body_a`
*bb = > json.parse text={"t":"# Shipping\n\nOrders ship in two business days.\n"}*
*body_b = > json.get value=`bb` key="t"*
> fs.write_text path=`pb` text=`body_b`

*fx_body = > json.parse text={"t":"{\"note\":\"Fixture evidence only; workbook remains authority.\",\"tools\":[{\"name\":\"wiki_get\",\"description\":\"Fetch policy wiki page\"}],\"results\":{\"wiki_get\":{\"page\":\"refunds\",\"text\":\"Refunds within 30 days.\"}}}"}*
*fx_t = > json.get value=`fx_body` key="t"*
> fs.write_text path=`fx` text=`fx_t`

*hit = > agent.corpus_search query="refund 30 days" root=`root` limit=4*
*auth = > json.get value=`hit` key="authority"*
1. `auth` == workbook
  > print text=auth-ok
2. *
  > print text=auth-bad
*cnt = > json.get value=`hit` key="count"*
1. `cnt` > 0
  > print text=corpus-hit
2. *
  > print text=corpus-miss
*hits = > json.get value=`hit` key="hits"*
*hn = > len value=`hits`*
1. `hn` > 0
  *h0 = > at value=`hits` index=0*
  *p0 = > json.get value=`h0` key="path"*
  1. `p0` == refunds.md
    > print text=top-refunds
  2. *
    > print text=top-bad
2. *
  > print text=top-empty

*listed = > agent.mcp_list_tools fixture=`fx`*
*tools = > json.get value=`listed` key="tools"*
*t0 = > at value=`tools` index=0*
*tn = > json.get value=`t0` key="name"*
1. `tn` == wiki_get
  > print text=mcp-list-ok
2. *
  > print text=mcp-list-bad

*called = > agent.mcp_call name="wiki_get" fixture=`fx`*
*ok = > json.get value=`called` key="ok"*
1. `ok`
  > print text=mcp-call-ok
2. *
  > print text=mcp-call-bad
*auth2 = > json.get value=`called` key="authority"*
1. `auth2` == workbook
  > print text=mcp-auth-ok
2. *
  > print text=mcp-auth-bad

*reply = > json.parse text={"t":"CALL:corpus_search\nARGS:{\"query\":\"refund\",\"root\":\".marqdo/agent-corpus-a4\",\"limit\":3}\n"}*
*raw = > json.get value=`reply` key="t"*
*via = > agent.run_parent_tool name="corpus_search" path="." reply=`raw`*
*vc = > json.get value=`via` key="count"*
1. `vc` > 0
  > print text=parent-corpus-ok
2. *
  > print text=parent-corpus-bad
