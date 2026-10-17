---
title: notebook shared state
---

# main

`内核表` =

| 名称 | 语言 | 命令 | 说明 |
|------|------|------|------|
| 分析 | node | | 共享状态的内核 |
| 绘图 | node | | 隔离的内核 |

`会话表` =

| 单元 | 内核 | 说明 |
|------|------|------|
| 设置 | 分析 | 写入共享变量 |
| 读取 | 分析 | 读取共享变量 |
| 隔离 | 绘图 | 看不到 shared |

`设置` =
```node
let shared = 41;
```

`读取` =
```node
console.log(shared + 1);
```

`隔离` =
```node
console.log(typeof shared === "undefined" ? "isolated" : "leaked");
```
