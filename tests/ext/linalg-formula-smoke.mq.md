---
title: linalg formula smoke
description: (AB)^T simplifies to B^T*A^T; ping ok.
import la:ext/linalg/linalg.mq.md
---

# main

*`A` = > la.symbol name="A" rows=2 cols=3*
*`B` = > la.symbol name="B" rows=3 cols=2*
*`P` = > la.mul a=`A` b=`B`*
*`T` = > la.transpose expr=`P`*
*`S` = > la.simplify expr=`T`*
*`txt` = > la.ascii expr=`S`*
> print text=`txt`

*`ping` = > la.ping*
*`ok` = ping[^ok]*
1. `ok`
  > print text=ping-ok
2. *
  > print text=ping-fail
