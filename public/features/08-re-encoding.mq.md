---
title: Mid stdlib — regex and encoding
description: lib/re + lib/encoding without plugins (Mid M1)
import re:lib/re.mq.md
import enc:lib/encoding.mq.md
---

# Regex and encoding in lib/

Mid-tier stdlib: everyday primitives stay in `lib/`, not ABI plugins.

# main

*`ok` = > re.is_match text="user@example.com" pattern="^[^@]+@[^@]+$"*
*`tok` = > enc.base64_encode text="marqdo"*
*`hex` = > enc.hex_encode text="ok"*
> print text=`ok`
> print text=`tok`
> print text=`hex`
1. `ok`
  > print text=mid-m1-ok
2. *
  > print text=mid-m1-fail
