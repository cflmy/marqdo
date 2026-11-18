---
markup: occlusion
title: lib/url smoke
import:
  url: lib/url.mq.md
---

# main

**u = > url.parse text="https://example.com:8443/a/b?x=1&y=2#frag"**
> print text=[scheme](access(`u`, "scheme"))
> print text=[host](access(`u`, "host"))
> print text=[port](access(`u`, "port"))
> print text=[path](access(`u`, "path"))
> print text=[query](access(`u`, "query"))
> print text=[fragment](access(`u`, "fragment"))

**q = > url.query_parse text="a=1&b=hi%20x&a=2"**
> print text=[b](access(`q`, "b"))
**as = [a](access(`q`, "a"))**
**a0 = > at value=`as` index=0**
**a1 = > at value=`as` index=1**
> print text=`a0`
> print text=`a1`

**s = > url.query_stringify map=`q`**
> print text=`s`
