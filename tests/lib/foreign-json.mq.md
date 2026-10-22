---
markup: occlusion
title: foreign invoke JSON input
import foreign:lib/foreign.mq.md
---

# main

`数值表` =

| 数值 |
|------|
| 1 |
| 2 |
| 3 |
| 4 |

`统计` =
```node
let s = "";
process.stdin.on("data", d => { s += d; });
process.stdin.on("end", () => {
  const xs = JSON.parse(s);
  console.log(JSON.stringify({ n: xs.length, sum: xs.reduce((a, b) => a + b, 0) }));
});
```

**记录 = > foreign.invoke code=`统计` input=`数值表` output="json"**
**统计结果 = [result](access(`记录`, "result"))**

> print text=[ok](access(`记录`, "ok"))
> print text=[n](access(`统计结果`, "n"))
> print text=[sum](access(`统计结果`, "sum"))
