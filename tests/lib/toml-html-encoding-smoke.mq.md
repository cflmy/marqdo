---
markup: occlusion
title: lib/toml html encoding M8 smoke
import toml:lib/toml.mq.md
import html:lib/html.mq.md
import enc:lib/encoding.mq.md
---

# main

**cfg = > toml.parse text="name = \"marqdo\"\ncount = 2\n[pkg]\nver = \"1\"\n"**
> print text=[name](access(`cfg`, "name"))
> print text=[count](access(`cfg`, "count"))
**pkg = [pkg](access(`cfg`, "pkg"))**
> print text=[ver](access(`pkg`, "ver"))

**e = > html.escape text="a<b>&\"c'"**
> print text=`e`
**u = > html.unescape text=`e`**
> print text=`u`

**b = > enc.base32_encode text="hello"**
> print text=`b`
**back = > enc.base32_decode text=`b`**
> print text=`back`
