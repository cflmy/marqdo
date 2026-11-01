---
markup: occlusion
title: foreign V_safe string to Node BigInt
import:
  foreign: lib/foreign.mq.md
  json: lib/json.mq.md
---

# main

**node = > foreign.session_open lang="node" timeout=30**
**s = "1152921504606846977"**

`inc` =
```javascript
const out = (BigInt(globalThis.__s) + 1n).toString();
console.log(JSON.stringify({out}));
```

**> foreign.session_set id=`node` name="__s" value=`s`**
**js = > foreign.session_eval id=`node` code=`inc`**
**js_out = > json.get value=`js` key="stdout"**
**> print text=`js_out`**
**> foreign.session_close id=`node`**
