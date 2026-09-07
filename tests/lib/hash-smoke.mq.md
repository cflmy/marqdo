---
title: lib/hash smoke
import hash:lib/hash.mq.md
---

# main

*s = > hash.sha256 text=""*
> print text=`s`

*a = > hash.sha256 text="abc"*
> print text=`a`

*m = > hash.md5 text="hello"*
> print text=`m`

*h = > hash.hmac_sha256 key="key" text="The quick brown fox jumps over the lazy dog"*
> print text=`h`
