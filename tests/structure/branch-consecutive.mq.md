---
title: consecutive branches restart at 1.
description: Same-indent `1.` after a completed list is a new Branch, not more arms.
---

# main

*`a` = True*
*`b` = True*

1. `a`
  > print text=A
2. *
  > print text=not-A

1. `b`
  > print text=B
2. *
  > print text=not-B

1. False
  > print text=no
2. *
  > print text=else-ok
