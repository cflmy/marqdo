---
title: foreign env cwd timeout
import foreign:lib/foreign.mq.md
---

# main

`环境` =

| MODE | NOTE |
|------|------|
| fast | test |

`环境检查` =
```node
console.log(process.env.MODE + "|" + process.env.MARQDO_FOREIGN_LANG);
```

**检查 = > foreign.invoke code=`环境检查` env=`环境`**
**检查输出 = > trim [stdout](`检查`)**

> print text=`检查输出`

`慢` =
```node
setTimeout(() => console.log("late"), 5000);
```

**超时 = > foreign.invoke code=`慢` timeout=0.5**

> print text=[ok](`超时`)
> print text=[timed_out](`超时`)
