---
markup: occlusion
title: foreign config load
import foreign:lib/foreign.mq.md
---

# main

**载入 = > foreign.load_config**
> print text=`载入`

**运行 = > foreign.invoke lang="tinycfg" source="console.log('cfg')"**
> print text=[stdout](access(`运行`, "stdout"))
