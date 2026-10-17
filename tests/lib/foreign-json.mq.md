---
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
**统计结果 = [result](`记录`)**

> print text=[ok](`记录`)
> print text=[n](`统计结果`)
> print text=[sum](`统计结果`)
