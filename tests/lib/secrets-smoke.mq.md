---
title: lib/secrets smoke
import sec:lib/secrets.mq.md
---

# main

*t = > sec.token_hex n=8*
*L = > len `t`*
> print text=`L`

*u = > sec.token_urlsafe n=9*
*ul = > len `u`*
> print text=`ul`
