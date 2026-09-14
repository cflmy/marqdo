---
title: lib/toml html encoding M8 smoke
import toml:lib/toml.mq.md
import html:lib/html.mq.md
import enc:lib/encoding.mq.md
---

# main

**cfg = > toml.parse text="name = \"marqdo\"\ncount = 2\n[pkg]\nver = \"1\"\n"**
> print text=[name](`cfg`)
> print text=[count](`cfg`)
**pkg = [pkg](`cfg`)**
> print text=[ver](`pkg`)

**e = > html.escape text="a<b>&\"c'"**
> print text=`e`
**u = > html.unescape text=`e`**
> print text=`u`

**b = > enc.base32_encode text="hello"**
> print text=`b`
**back = > enc.base32_decode text=`b`**
> print text=`back`
