---
title: Built-ins len / str / int / type / trim / split / join / at
description: Host builtins for strings and tables
---

# main

Length and type:

*`s` = "hello"*

*`n` = > len `s`*

> print text=`n`

*`ty` = > type `s`*

> print text=`ty`

Trim, split, join, and index:

*`t` = > trim value="  hi  "*

*`parts` = > split value="a,b" sep=","*

*`mid` = > at value=`parts` index=0*

*`j` = > join value=`parts` sep="-"*

> print text=`t`

> print text=`mid`

> print text=`j`
