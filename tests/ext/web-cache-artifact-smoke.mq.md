---
title: ext/data cache smoke
import cache:ext/data/cache.mq.md
---

# main

**c = > cache.cache url="memory:"**
> `c`.set key="k" value="v" ttl=60
**got = > `c`.get key="k"**
**val = [value](got)**
1. `val` == "v"
  > print text=cache-ok
2. *
  > print text=cache-fail
