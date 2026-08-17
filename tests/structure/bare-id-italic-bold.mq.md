---
title: bare id in italic and bold
description: Inside *…* / **…**, bare ids are variables; strings need quotes.
---

# main

*answer = 1*
*n = answer + 1*
*ticked = 2*
*s = 'quoted'*

> print text=`n`
> print text=`ticked`
> print text=`s`

*a = > ret_var x=7*
> print text=`a`

*b = > ret_str*
> print text=`b`

*c = > ret_single*
> print text=`c`

## ret_var
    + `x`

**x**

## ret_str

**"ok"**

## ret_single

**'hi'**
