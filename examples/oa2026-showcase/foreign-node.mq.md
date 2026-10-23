---
markup: occlusion
title: 外联：把 Node 当作文档里的计算车间
description: 一次性执行、持久会话、显式传值，以及可读的结果。
import:
  foreign: lib/foreign.mq.md
---

# main

这份文档要回答一个很具体的问题：同一批订单，按区域汇总之后，总金额分别是多少？
数据已经写在文档里，计算交给 Node，结果回到文档里继续被阅读与断言。

`订单` =

| @ | 区域 | 金额 |
|---|------|------|
| 1 | 华东 | 120 |
| 2 | 华东 | 300 |
| 3 | 华南 | 90 |
| 4 | 华南 | 140 |
| 5 | 华北 | 220 |

先做一次性执行。`foreign.call` 把`订单`编码成 JSON 放到 stdin，
Node 读完、汇总、打印 JSON；外联再把 stdout 解析成 Marqdo 的值。
没有 shell，也没有隐式的 stdin 约定——传什么、读什么，都写在代码里。

`汇总代码` =
```javascript
const incoming = JSON.parse(require('fs').readFileSync(0, 'utf8'));

const byRegion = {};
for (const row of incoming) {
  byRegion[row["区域"]] = (byRegion[row["区域"]] || 0) + Number(row["金额"]);
}

console.log(JSON.stringify(byRegion));
```

**`汇总` = > foreign.call code=`汇总代码` input=`订单`**
> print text=`汇总`

---

一次性执行适合「算完就走」。如果想让同一个 Node 内核保留变量，
就显式开启会话、显式设值、显式求值、显式关闭——状态不会藏在某个看不见的 cell 里。

**`会话` = > foreign.session_open lang="node"**
**> foreign.session_set id=`会话` name="orders" value=`订单`**

`排序代码` =
```javascript
const ranked = Object.entries(
  orders.reduce((acc, row) => {
    acc[row["区域"]] = (acc[row["区域"]] || 0) + Number(row["金额"]);
    return acc;
  }, {})
).sort((a, b) => b[1] - a[1]);

console.log(ranked.map(([region, total]) => `${region}: ${total}`).join("\n"));
```

**`第一次` = > foreign.session_eval id=`会话` code=`排序代码`**
> print text=[stdout](access(`第一次`, "stdout"))

内核状态是持续的。下面这段代码在同一个`orders`上追加一条华北订单，
再重新排序；不需要重新传一遍整张表。

`追加代码` =
```javascript
orders.push({ 区域: "华北", 金额: 80 });

const rankedAgain = Object.entries(
  orders.reduce((acc, row) => {
    acc[row["区域"]] = (acc[row["区域"]] || 0) + Number(row["金额"]);
    return acc;
  }, {})
).sort((a, b) => b[1] - a[1]);

console.log(rankedAgain.map(([region, total]) => `${region}: ${total}`).join("\n"));
```

**`第二次` = > foreign.session_eval id=`会话` code=`追加代码`**
> print text=[stdout](access(`第二次`, "stdout"))

**> foreign.session_close id=`会话`**
