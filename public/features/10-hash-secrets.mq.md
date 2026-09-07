---
title: Mid stdlib — hash and secrets
description: lib/hash + lib/secrets without plugins (Mid M3)
import hash:lib/hash.mq.md
import sec:lib/secrets.mq.md
---

# Digests and secure tokens

Integrity digests and cryptographic tokens stay in `lib/` — do not use `math.random` for secrets.

# main

*`d` = > hash.sha256 text="marqdo"*
*`t` = > sec.token_hex n=8*
*`L` = > len `t`*
> print text=`d`
> print text=`L`
1. `L` == 16
  > print text=mid-m3-ok
2. *
  > print text=mid-m3-fail
