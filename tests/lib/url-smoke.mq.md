---
title: lib/url smoke
import url:lib/url.mq.md
---

# main

**u = > url.parse text="https://example.com:8443/a/b?x=1&y=2#frag"**
> print text=[scheme](`u`)
> print text=[host](`u`)
> print text=[port](`u`)
> print text=[path](`u`)
> print text=[query](`u`)
> print text=[fragment](`u`)

**q = > url.query_parse text="a=1&b=hi%20x&a=2"**
> print text=[b](`q`)
**as = [a](`q`)**
**a0 = > at value=`as` index=0**
**a1 = > at value=`as` index=1**
> print text=`a0`
> print text=`a1`

**s = > url.query_stringify map=`q`**
> print text=`s`
