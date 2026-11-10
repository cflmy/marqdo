---
markup: occlusion
title: Native analysis and an explicit Intl formatting boundary
import:
  foreign: lib/foreign.mq.md
  table: lib/table.mq.md
  result: lib/result.mq.md
  native: ../../examples/foreign-native-report/index.mq.md
---

This page keeps order analysis in Marqdo and uses Node Intl only for locale-aware
currency formatting. Read [the actual native report tools](../../examples/foreign-native-report/index.mq.md)
for their independently callable input and return contracts. That single module
contains grouping, population statistics and the bounded persistent formatter.

Call this page's main with process.exec for a full workflow, or call analyze_orders
in that module with no effects for native analysis alone. View evaluates the same
cells with shared state and isolated kernels. No Python interpreter is needed.

## main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | View-session plan plus the native report and checked localized formatting |

This complete page uses six ordinary orders: their native total is 950, count
is six, and the four regional rows retain count, total, mean and normalized span.
The pure analysis is independently callable in the linked module. Formatting is a separate
capability with an explicit process.exec grant and an installed Node runtime.
The successful return contains report and formatting alongside the view-session
plan; it contains no live kernel handle and can be returned by marqdo call.

The view gets the same native report copied into two named kernels. Its cells
perform presentation rather than reimplementing native grouping or arithmetic.
Run All first initializes the formatter in the analysis kernel, then a second
cell reuses it. The review kernel receives its own report copy and formatter.
This demonstrates both shared state and kernel isolation. Running the dependent
formatting cell before its initializer should show the real missing-state error.

Static View export evaluates those same cells and embeds actual stdout.
Interactive View retains their kernels for later runs. The low-level formatter
used to check the public return is opened and closed separately, so it cannot
secretly supply state to the view kernels. No stdin, notebook command, generated
file-reader program or package installation is hidden in this example.
If Node or formatting fails, the page returns that failure before declaring a
ready view. Native-only consumers can still call analyze_orders with no effects.
The table cells below are data, while the three bound node fences are explicit
foreign code. The distinction is visible without translating a Python workflow
into another ad hoc command sequence.

`orders` =

| @ | region | amount |
|---|---|---|
| 1 | 华东 | 120 |
| 2 | 华东 | 300 |
| 3 | 华南 | 90 |
| 4 | 华南 | 140 |
| 5 | 华北 | 220 |
| 6 | 西部 | 80 |

We **report = [Perform all analysis in Marqdo](native.analyze_orders(orders=`orders`))** and **formatted = [Use the genuine external formatting boundary](native.format_summary(report=`report`))**.
1. not access(`formatted`, "ok")
  Return *formatted* before declaring an executable view ready.

`kernels` =

| @ | 名称 | 语言 |
|---|---|---|
| 1 | 分析 | node |
| 2 | 复核 | node |

`cells` =

| @ | 单元 | 内核 |
|---|---|---|
| 1 | 计算 | 分析 |
| 2 | 格式化 | 分析 |
| 3 | 复核 | 复核 |

`initial` =

| @ | 内核 | 名称 | 值 |
|---|---|---|---|
| 1 | 分析 | report | `report` |
| 2 | 复核 | report | `report` |

We **会话 = [Declare the view with native values and explicit kernels](foreign.session(kernels=`kernels`, cells=`cells`, values=`initial`))**.

`计算` =
```node
var money = new Intl.NumberFormat("en-US", {style:"currency", currency:"CNY"});
console.log(JSON.stringify(report));
```

`格式化` =
```node
console.log(money.format(report.total));
console.log(money.format(report.average));
```

`复核` =
```node
var reviewMoney = new Intl.NumberFormat("en-US", {style:"currency", currency:"CNY"});
console.log(JSON.stringify({count:report.count, total:reviewMoney.format(report.total)}));
```

We **formatting = [Retain the checked persistent-kernel result](access(formatted, "value"))**.

`evidence` =

| report | formatting |
|---|---|
| `report` | `formatting` |

Return *[The executable view plan with native analysis evidence](table.merge(a=`会话`, b=`evidence`))*.
