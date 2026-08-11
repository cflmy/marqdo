---
title: Parent CALL:lib whitelist (offline)
description: lib.fs.exists + ARGS; reject lib.net; catalog callable.
> ext/ai/agent.mq.md
> lib/json.mq.md
> lib/fs.mq.md
> lib/sys.mq.md
---

# main

*`parts` = > json.parse text={"a":"CALL:lib.fs.exists","b":"ARGS:{\"path\":\".\"}","nl":"\n"} *
*`a` = > json.get value=`parts` key=a *
*`b` = > json.get value=`parts` key=b *
*`nl` = > json.get value=`parts` key=nl *
*`t_ok` = `a` + `nl` + `b` + `nl` *
*`act` = > agent.extract_plan_act reply=`t_ok` *
*`kind` = > json.get value=`act` key=kind *
*`name` = > json.get value=`act` key=name *
> print text=`kind`
> print text=`name`

*`out` = > agent.run_parent_tool name=`name` path=. reply=`t_ok` *
1. `out`
  > print text=exists-ok
2. *
  > print text=exists-bad

*`deny` = > agent.run_parent_tool name=lib.net.get path=. reply=CALL:lib.net.get *
*`sep` = > json.parse text={"s":"not allowed"} *
*`s` = > json.get value=`sep` key=s *
*`dparts` = > split value=`deny` sep=`s` *
*`pn` = > len value=`dparts` *
1. `pn` > 1
  > print text=deny-ok
2. *
  > print text=deny-bad

*`cat` = > agent.lib_catalog *
*`callable` = > json.get value=`cat` key=callable *
*`cn` = > len value=`callable` *
1. `cn` > 3
  > print text=callable-ok
2. *
  > print text=callable-bad
