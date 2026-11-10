---
markup: occlusion
title: Native report tools and a persistent Intl formatter
import:
  foreign: lib/foreign.mq.md
  table: lib/table.mq.md
  stats: lib/stats.mq.md
  math: lib/math.mq.md
  result: lib/result.mq.md
---

This module keeps order analysis in native Marqdo and uses Node Intl only for
locale-aware currency formatting. Import it from a report page or call its
two public functions separately. There is no executable main in this module;
the English and Chinese public feature pages compose the full callable view.

From the repository root, call analyze_orders with an orders row list and
--effects none. It returns ordinary report data. Pass that report to
format_summary with --effects process.exec for the bounded persistent formatter.
A missing Node executable is an explicit failed Result; native analysis remains
available without it. No Python interpreter is required.

## analyze_orders

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| orders | list | required | Order rows with a region and numeric amount; no file or network IO |
| region_key | text | "region" | Field whose value identifies a group |
| amount_key | text | "amount" | Numeric amount field in every row |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Native report with regions, total, count and average |

Use `orders`, `region_key`="region" and `amount_key`="amount" for the
whole analysis. Grouping, ordering, arithmetic, means and population deviation
all use Marqdo and its standard libraries. Each region retains orders, total,
mean and z_span, so moving the boundary does not remove the analysis. The data
is a row list, not a JSON program or an opaque host dataframe.

The report has one row per region in sorted key order. Population variance is
the mean squared deviation; a singleton or equal-valued group has zero spread,
for which the normalization denominator is one and z_span is zero. The maximum
normalized deviation is the same statistic the previous kernel example displayed.
Values are ordinary runtime numbers, not a financial decimal model; the formatter
later chooses presentation precision without changing this report.

Empty input returns no regions and zero totals/average. Missing fields and invalid
amounts preserve native diagnostics; do not replace them with invented zeroes.
The function is pure and needs no interpreter, plugin, process, file or network
grant. Its actual return can be checked with marqdo call before preparing a view.
The native report is also the only analysis value copied into external kernels:
there is no second implementation of grouping or statistics in another language.

We **groups = [Group native rows and order their original keys](table.sort_by(list=table.group_by(list=`orders`, key=`region_key`), key="key"))** and **regions = [Start native summary rows](table.empty_list())**.
- [Summarize each region through native values](for groups as group)
  We **rows = [Read this group's original rows](access(group, "rows"))** and **region = [Retain its region key](access(group, "key"))** and **values = [Read numeric amounts](table.pluck(list=`rows`, key=`amount_key`))**.
  We **mean = [Compute the native arithmetic mean](stats.mean(list=`values`))** and **squared = 0.0**.
  - [Accumulate population squared deviation](for values as value)
    We **squared = squared + (value - mean) * (value - mean)**.
  We **sigma = [Compute population spread](math.sqrt(value=`squared` / len(values)))**.
  1. `sigma` <= 0
    We **sigma = 1** so equal-valued groups keep a zero normalized span.
  We **count = [Count this region's values](len(values))** and **total = [Sum the region in Marqdo](table.aggregate(list=`values`))** and **maximum = [Read its largest amount](table.aggregate(list=`values`, op="max"))** and **z_span = (maximum - mean) / sigma**.

  `row` =

  | region | orders | total | mean | z_span |
  |---|---|---|---|---|
  | `region` | `count` | `total` | `mean` | `z_span` |

  We **regions = [Retain this native summary row](table.append(list=`regions`, item=`row`))**.
We **count = [Count all input orders](len(orders))** and **total = [Sum all native amounts](table.aggregate(list=`orders`, key=`amount_key`))** and **average = 0**.
1. `count` > 0
  We **average = [Use the native mean without integer division](stats.mean(list=table.pluck(list=`orders`, key=`amount_key`)))** for a nonempty report.

`report` =

| regions | total | count | average |
|---|---|---|---|
| `regions` | `total` | `count` | `average` |

After the native analysis we return *report*.

## format_summary

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| report | map | required | Native analyze_orders result with total and average |
| locale | text | "en-US" | Locale understood by Node Intl; currency is CNY |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result containing localized total and average strings |

Use `report` and `locale`="en-US" only for the presentation gap:
locale-aware currency formatting is provided here by Node's built-in Intl.
The source makes that external boundary explicit. It does not read or write files,
recalculate the native report, import a package or use Python. Node must already
be installed; probe it before creating a kernel and return a real failure if
it is unavailable.

One persistent kernel receives JSON copies of the report and locale. The first
evaluation creates a formatter and formats the total; the second reuses that
same formatter for the already-computed native average. A fresh process without
its previous state could not run the second expression successfully. The helper
checks each evaluation's ok field and closes a live kernel before returning,
including handled evaluation errors. A dead kernel has already been retired and
must not be closed a second time.

This function needs process.exec. Native policy or serialization diagnostics
remain diagnostics, not synthetic currency strings. Kernel values cannot become
guessed handles, and an opaque handle never crosses a CLI JSON boundary. Only
the two actual formatted strings leave this task in its successful Result.

We **probe = [Check the explicit external interpreter](foreign.probe(lang="node"))**.
1. not access(`probe`, "ok")
  Return *[Keep the missing interpreter visible](result.error(code="example.node", message=str(`probe`), operation="format_summary"))*.
We **kernel = [Open one persistent formatter kernel](foreign.session_open(lang="node"))** and **copied = [Copy the native report as ordinary data](foreign.session_set(id=`kernel`, name="report", value=`report`))** and **selected = [Copy the requested display locale](foreign.session_set(id=`kernel`, name="locale", value=`locale`))**.

`formatter_code` =
```node
var money = new Intl.NumberFormat(locale, {style: "currency", currency: "CNY"});
money.format(report.total)
```

We **first = [Create the formatter and format the native total](foreign.session_eval(id=`kernel`, code=`formatter_code`))**.
1. not access(`first`, "ok")
  1. access(`first`, "alive")
    We **closed = [Close a still-live failed formatter](foreign.session_close(id=`kernel`))**.
  Return *[Keep the actual formatter failure](result.error(code="example.format", message=str(`first`), operation="format_summary"))*.

`average_code` =
```node
money.format(report.average)
```

We **second = [Reuse the formatter's persistent state](foreign.session_eval(id=`kernel`, code=`average_code`))**.
1. not access(`second`, "ok")
  1. access(`second`, "alive")
    We **closed = [Close a live kernel after the second failure](foreign.session_close(id=`kernel`))**.
  Return *[Keep the actual reuse failure](result.error(code="example.format", message=str(`second`), operation="format_summary"))*.
We **total = [Read the actual formatted total](access(first, "result"))** and **average = [Read the actual formatted average](access(second, "result"))** and **closed = [Close the consumed formatter kernel](foreign.session_close(id=`kernel`))**.

`formatted` =

| total | average |
|---|---|
| `total` | `average` |

Return *[The real locale-aware strings](result.ok(value=`formatted`))*.
