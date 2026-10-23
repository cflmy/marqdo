---
markup: occlusion
title: foreign call and call_many
import:
  foreign: lib/foreign.mq.md
---

# main

`双倍` =
```node
let s = "";
process.stdin.on("data", d => { s += d; });
process.stdin.on("end", () => {
  console.log(JSON.stringify(JSON.parse(s) * 2));
});
```

**单个 = > foreign.call code=`双倍` input=1**
> print text=`单个`

`数值表` =

| 数值 |
|------|
| 1 |
| 2 |
| 3 |
| 4 |

**多个 = > foreign.call_many code=`双倍` inputs=`数值表` concurrency=2**
> print text=`多个`
