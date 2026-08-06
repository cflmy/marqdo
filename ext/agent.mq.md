---
title: ext/agent
description: Agent project layout via ABI plugin (English). Not stdlib — import ext/agent.mq.md.
> lib/plugin.mq.md
> lib/sys.mq.md
> lib/json.mq.md
---

## load_native

Load `plugins/agent` shared library. Order: env `MARQDO_AGENT_PLUGIN`, then installed path from `marqdo ext add agent` (`host_ext_native_path`).

*`envp` = > env_get name=MARQDO_AGENT_PLUGIN *
+ `envp`
  **> load path=`envp`**
+ *
  *`np` = > host_ext_native_path name=agent *
  + `np`
    **> load path=`np`**
  + *
    > print text=ext/agent: set MARQDO_AGENT_PLUGIN or run marqdo ext add agent
    > exit code=1

# agent
    - root

Construct a workspace handle. Optional `root=`; otherwise `agent_find_root` from cwd.

+ `root`
  *`r` = `root`*
+ *
  *`start` = > cwd *
  *`r` = > agent_find_root start=`start` markers=agents,runbooks,marqdo.agent.json *

*`t` = > parse text={"p":"{\"_type\":\"agent\",\"root\":","s":"}"} *
*`qr` = > quote text=`r` *
*`p` = > get value=`t` key=p *
*`s` = > get value=`t` key=s *
*`raw` = `p` + `qr` + `s` *
**> parse text=`raw`**

## probe

*`root` = > get value=`self` key=root *
**> agent_probe root=`root`**

## ensure_layout

*`root` = > get value=`self` key=root *
**> agent_ensure_layout root=`root`**

## scaffold
    - name
    - template
    - dest

*`root` = > get value=`self` key=root *
**> agent_scaffold root=`root` name=`name` template=`template` dest=`dest`**
