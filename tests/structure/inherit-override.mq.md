---
title: inherit method override
---

# Greeter

## hello
    + `who`

*msg = "Hello, `who`!"*
**msg**

# Loud = > Greeter

## hello
    + `who`

*msg = "HELLO, `who`!"*
**msg**

# main

*g = > Greeter*
*m = > `g`.hello who="world"*
> print text=`m`
*l = > Loud*
*m2 = > `l`.hello who="world"*
> print text=`m2`
