---
title: lib/plugin — native ABI
description: Load optional shared libraries (.dll / .so / .dylib)
import plugin:lib/plugin.mq.md
---

# main

Import lib/plugin.mq.md. Functions: load(path), unload(), list(). After load, registered plugin names are callable like other functions. C ABI: include/marqdo_abi.h — see doc/design/ext-abi.md. Path must stay under the program sandbox (cwd / fs root).

*`names` = > plugin.list *

*`n` = > len `names` *

1. `n` >= 0
  > print text=plugin-ok
2. *
  > print text=plugin-bad
