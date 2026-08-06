---
title: ext/agent
description: Agent development framework (layout ABI + match/assign). Compose with ext/llm.
> lib/plugin.mq.md
> ext/llm.mq.md
---

## load_native

Load `plugins/agent` shared library. Order: env `MARQDO_AGENT_PLUGIN`, then `marqdo ext add agent` (`host_ext_native_path`).

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

Workspace handle. Optional `root=`; else `agent_find_root` from cwd.

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

## match_skill
    - skill
    - members

Deterministic skill match over a list of member maps (`技能`/`skills`, `负载`/`load`). Picks lowest load among matches. Returns member map or `None`.

**> agent_match_skill skill=`skill` members=`members`**

## assign_task
    - task_id
    - assignee

Record a successful assignment (map). Caller updates load separately.

*`t` = > parse text={"p":"{\"success\":true,\"task_id\":","m":",\"assignee\":","s":"}"} *
*`qid` = > quote text=`task_id` *
*`qa` = > stringify value=`assignee` *
*`p` = > get value=`t` key=p *
*`m` = > get value=`t` key=m *
*`s` = > get value=`t` key=s *
*`raw` = `p` + `qid` + `m` + `qa` + `s` *
**> parse text=`raw`**

## update_load
    - member
    - delta

**> agent_bump_load member=`member` delta=`delta`**

## create_ticket
    - title
    - detail

Stub ticket: ensure layout, write `ticket-stub.txt` under cwd (sandbox). Returns the filename.

*`_n` = > `self`.ensure_layout *
*`body` = `title` *
*`out` = ticket.txt *
> host_write_text path=ticket.txt text=`body`
**`out`**

## notify
    - to
    - text

Stub channel: print a notification line.

> print text=notify
> print text=`to`
> print text=`text`
****

## draft_message
    - model
    - prompt

Optional LLM helper: `model` is an `ext/llm` handle.

**> `model`.complete prompt=`prompt`**
