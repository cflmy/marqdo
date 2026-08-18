---
title: quantum gate matches_matrix smoke
description: Named H matches an explicit 2x2 nested list.
import quantum:ext/quantum/quantum.mq.md
import json:lib/json.mq.md
---

# main

*H_matrix = > json.parse text=[[0.7071067811865475,0.7071067811865475],[0.7071067811865475,-0.7071067811865475]]*
*H = > quantum.gate name="H"*
*ok = > `H`.matches_matrix matrix=`H_matrix`*

1. `ok`
  > print text=match-ok
2. *
  > print text=match-fail

*bad = > json.parse text=[[1,0],[0,1]]*
*no = > `H`.matches_matrix matrix=`bad`*

1. not `no`
  > print text=reject-ok
2. *
  > print text=reject-fail
