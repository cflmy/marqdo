---
markup: occlusion
title: 原生订单分析与 Node Intl 展示边界
import:
  native: ../foreign-native-report/index.mq.md
  foreign: lib/foreign.mq.md
  table: lib/table.mq.md
  result: lib/result.mq.md
---

订单分析应先使用已经具备的原生能力。只有本地化货币展示交给 Node Intl；一次性执行和持久会话都是明确的外联边界，文件操作与通用数据处理不需要 Python 或外部读取器。

从仓库根目录调用：`marqdo call examples/oa2026-showcase/foreign-node.mq.md main --require-ok --effects process.exec --fs-root . --json`。只需原生分析时，可直接 call 共享模块的 analyze_orders，不需要任何进程授权。

## main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result：前后两份原生分析、实际排序结果与一次性/持久格式化结果 |

成功时 value 的字段如下。先检查最外层 ok，再按字段读取；这里列出的数据都是本轮实际计算或执行得到的值。

| value 中的字段 | 值的形态 | 直接含义 |
|---|---|---|
| before | 原生分析记录 | 五笔订单的总额 870、数量 5、平均值 174 与按区域保留的统计行 |
| after | 原生分析记录 | 追加八十元订单后，总额 950、数量 6、平均值约 158.3333；不做展示舍入 |
| initial_ranking | 原生行记录列表 | 首轮按区域总额降序排列；每行保留区域、数量、总额、均值与标准化跨度 |
| updated_ranking | 原生行记录列表 | 更新后的区域总额依次为 420、300、230；排序返回新列表 |
| one_shot | 两个展示文本字段 | 一次性进程返回的 total 与 average，只表示首轮报告的本地化展示 |
| persistent_initial | 两个展示文本字段 | 持久格式器对同一首轮报告得到的 total 与 average，可与一次性结果核对 |
| persistent_updated | 两个展示文本字段 | 持久格式器展示更新后的报告；950.00 与 158.33 是展示文本，不覆盖原生数值 |


本例先处理五笔订单：总额为八百七十，随后追加一笔华北的八十元订单，总额变为九百五十。分组、计数、平均值、追加和按区域总额排序全部使用 Marqdo 固有能力；不再在 Node 中维护另一份订单分析程序。共享模块中的 analyze_orders 是唯一的分析实现，其参数与字段表可以直接阅读和调用。

外联只承担确实缺失的本地化展示。第一次通过本文件的 format_once 演示标准输入上的 JSON 往返；两次共享 format_summary 分别展示原生修改前后的结果。每次共享调用都在同一内核内建立格式器、复用它格式化平均值并关闭资源，实际生命周期写在对应函数中，而不是把一个句柄藏进跨命令的 JSON 字符串。

总额排序也保留在原生流程中：先按 total 升序，再反转得到降序记录。排序不会改变原始订单或报告，每个返回行仍包含区域、订单数和总额。追加发生在 Marqdo 集合中，随后将重新计算出的值拷贝给展示边界，避免外部内核与原生数据各保留一份会悄悄分叉的业务状态。

调用需要 process.exec 和 Node，不读写用户文件，也不安装包。先检查共享格式化任务的 ok，再使用 value；一次性 JSON 协议失败则保留原生诊断。成功结果携带真正的原生数据和展示字符串，既能继续被其他函数使用，也能核对展示没有替代数值计算。查看返回结果比只读一条“完成”打印更能证明这一轮执行发生了什么。

`订单` =

| @ | 区域 | 金额 |
|---|---|---|
| 1 | 华东 | 120 |
| 2 | 华东 | 300 |
| 3 | 华南 | 90 |
| 4 | 华南 | 140 |
| 5 | 华北 | 220 |

We **before = [在原生标准库中完成首轮订单分析](native.analyze_orders(orders=`订单`, region_key="区域", amount_key="金额"))** and
**initial_ranking = [保留首轮按区域总额降序的原生行记录](table.reverse(list=table.sort_by(list=access(before, "regions"), key="total")))** and
**one_shot = [只把首轮报告的展示交给一次性进程](format_once(report=`before`))** and
**initial = [用共享持久内核流程展示同一份原生结果](native.format_summary(report=`before`, locale="zh-CN"))**。
1. not access(`initial`, "ok")
  返回*initial*，保留真实运行失败。

`追加` =

| 区域 | 金额 |
|---|---|
| 华北 | 80 |

We **orders = [在原生集合中追加新订单](table.append(list=`订单`, item=`追加`))** and
**after = [用同一份原生实现重新计算完整报告](native.analyze_orders(orders=`orders`, region_key="区域", amount_key="金额"))** and
**updated_ranking = [在原生集合中保留更新后的降序记录](table.reverse(list=table.sort_by(list=access(after, "regions"), key="total")))** and
**updated = [把更新后的原生结果交给展示边界](native.format_summary(report=`after`, locale="zh-CN"))**。
1. not access(`updated`, "ok")
  返回*updated*，保留持久内核的失败出口。
We **persistent_initial = [保留首轮实际格式化文本](access(initial, "value"))** and **persistent_updated = [保留更新后的实际展示文本](access(updated, "value"))**。

`观察` =

| before | after | initial_ranking | updated_ranking | one_shot | persistent_initial | persistent_updated |
|---|---|---|---|---|---|---|
| `before` | `after` | `initial_ranking` | `updated_ranking` | `one_shot` | `persistent_initial` | `persistent_updated` |

完成原生分析与明确的展示外联后，我们返回*[本次实际观察到的数据与结果](result.ok(value=`观察`))*。

## format_once

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| report | map | required | 原生分析结果，包含 total 与 average 数值 |
| locale | text | "zh-CN" | Node Intl 使用的展示语言区域 |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | 直接返回 total/average 本地化文本；子进程或 JSON 失败保留原生诊断 |

使用 `report` 和 `locale`="zh-CN" 补足本地化货币展示。一次性外联只接收已经计算好的总额与平均值，不分组、不排序、不修改订单。传入数据由 Marqdo 的 input 通道编码为 JSON；Node 从标准输入流接收它，不读取任何路径，也不额外生成文件。

代码围栏明确写出进程如何消费输入、建立 Intl 格式器以及输出唯一的 JSON 记录。标准输出只承载这个协议值，诊断保留在错误通道；混入其他打印会导致 JSON 解析失败，而不是被当作部分成功。这里选择人民币展示，不声称进行汇率换算或货币精度计算。

调用需要 process.exec 与已安装的 Node，超时为五秒。foreign.call 在进程失败或输出不是合法 JSON 时给出原生诊断，因此本函数的成功值就是格式化记录，不再套一个假造的通过标记。这个一次性子进程结束后没有可复用状态；持久会话的创建、两次求值和关闭由共享原生报表手册中的 format_summary 展示。

`输入` =

| report | locale |
|---|---|
| `report` | `locale` |

`格式化代码` =
```node
let raw = "";
process.stdin.setEncoding("utf8");
process.stdin.on("data", chunk => { raw += chunk; });
process.stdin.on("end", () => {
  const input = JSON.parse(raw);
  const money = new Intl.NumberFormat(input.locale, {style: "currency", currency: "CNY"});
  console.log(JSON.stringify({total: money.format(input.report.total), average: money.format(input.report.average)}));
});
```

接收完整输入后，我们返回*[执行有界的一次性展示边界并解析真实 JSON 返回](foreign.call(code=`格式化代码`, input=`输入`, timeout=5))*。
