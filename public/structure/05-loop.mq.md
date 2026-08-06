---
title: Loops
description: Condition loops and table walks
---

# main

Inside a function body, `-` starts a loop (not a parameter).

Countdown:

*`n` = 3*

- `n` > 0
  > print text=`n`
  *`n` = `n` - 1*

Walk a table (tables are collections):

`basket` =

| fruit |
|-------|
| apple |
| pear |

- [`fruit`](`basket`)
  > print text=today `fruit`
