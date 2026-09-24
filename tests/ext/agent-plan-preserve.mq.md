---
title: plan must not overwrite existing llm_free resources
description: force+confirm plan keeps a pre-seeded compiled skill workbook intact (offline).
import agent:ext/ai/agent.mq.md
import fs:lib/fs.mq.md
import json:lib/json.mq.md
import llm:ext/ai/llm.mq.md
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
import text:lib/text.mq.md
---

# main

**p = > plugin.native_path name="agent"**
1. `p`
  > plugin.load path=`p`
2. *
  > print text=no-agent-plugin
  > sys.exit code=1

**kb = ".marqdo/agent-kb-preserve-test"**
**ex = > fs.exists path=`kb`**
1. `ex`
  > fs.remove_tree path=`kb`
2. *
  **_ = 1**
> fs.make_dirs path=`kb`
**parts = > json.parse text={"res":"/resources/","ext":".mq.md","dir":"/resources"}**
**res_dir = kb + [dir](`parts`)**
> fs.make_dirs path=`res_dir`

**goal = "Preserve compiled skill body under force plan"**
**slug = > agent_goal_slug goal=`goal`**
**res = kb + [res](`parts`) + slug + [ext](`parts`)**
**body = > fs.read_text path="fixtures/preserve-skill.mq.md"**
> fs.write_text path=`res` text=`body`

**model = > llm.create**
`tools` =

| 工具 |
|------|

**ag = > agent.agent model=`model` tools=`tools` standing="preserve probe"**
**out = > `ag`.plan goal=`goal` kb_dir=`kb` force=True confirm=True writeback=False reuse=False**
**st = > json.get value=`out` key="status"**
> print text=`st`

**after = > fs.read_text path=`res`**
**kept = > text.contains text=`after` sub="kept"**
**no_skel = > text.contains text=`after` sub="worker.step"**
> print text=`kept`
> print text=`no_skel`
