---
title: lib/datetime smoke
import dt:lib/datetime.mq.md
---

# main

*m = > dt.from_unix unix=0*
> print text=`m`[^iso]

*sh = > dt.in_zone dt=`m` zone="Asia/Shanghai"*
> print text=`sh`[^iso]

*p = > dt.parse text="2020-01-02T03:04:05+08:00"*
*u = > dt.to_unix dt=`p`*
> print text=`u`

*n = > dt.add dt=`m` days=1*
*nu = > dt.to_unix dt=`n`*
> print text=`nu`

*d = > dt.format dt=`sh` style="date"*
> print text=`d`
