---
title: linalg block/kron smoke (L3)
description: Lazy kron ascii; block*block collapse.
import la:ext/linalg/linalg.mq.md
---

# main

*`A` = > la.symbol name="A" rows=2 cols=2*
*`B` = > la.symbol name="B" rows=2 cols=2*
*`Z` = > la.zeros rows=2 cols=2*

*`K` = > la.kron a=`A` b=`B`*
*`kt` = > la.ascii expr=`K`*
> print text=`kt`

`r0` =

| v |
|---|
| `A` |
| `B` |

`r1` =

| v |
|---|
| `Z` |
| `A` |

`blocks` =

| v |
|---|
| `r0` |
| `r1` |

*`L` = > la.block blocks=`blocks`*
*`P` = > la.mul a=`L` b=`L`*
*`C` = > la.collapse expr=`P`*
*`ct` = > la.ascii expr=`C`*
> print text=`ct`

*`ping` = > la.ping*
*`ok` = ping[^ok]*
1. `ok`
  > print text=ping-ok
2. *
  > print text=ping-fail
