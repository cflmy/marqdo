---
markup: occlusion
title: Persist an actual formatting result through native writeback
import:
  guide: ../../doc/library-development.mq.md
  formatter: ../../examples/foreign-native-report/index.mq.md
  writeback: lib/writeback.mq.md
  result: lib/result.mq.md
---

The source remains the manual for this file operation. Its callable main computes
native data, crosses a genuine formatting boundary, checks the result and records
only the successful value. Invoke with process.exec,fs.write and --fs-root .;
static View uses the same source and embeds its actual observed output.

## main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with the actual string recorded in this source's total slot |

A static View snapshot embeds executed output in HTML. Explicit writeback
instead keeps a selected value in this source file's named slot. A caller should
choose that file effect deliberately: this main needs process.exec and fs.write,
with --fs-root pointing at the repository. Reading the page alone does not write.

The amount is computed by the native programming guide, then the shared example
uses Node Intl only for locale-aware currency formatting. Its persistent kernel
is closed before this page receives the result. No Python process, foreign file
writer or duplicated arithmetic participates. The exact returned string is then
passed to lib/writeback, so the file operation remains a Marqdo capability.

The total slot below is actual recorded output. Repeated execution replaces that
same key rather than appending another slot. An unavailable interpreter or failed
formatting returns before the write; native write-policy or IO faults retain
their diagnostics. Writeback persists this selected ordinary value, not a live
kernel or arbitrary process memory. Review source changes with the repository's
normal diff workflow. For published docs use a static snapshot unless a living
auditable source result is intended, as it explicitly is in this demonstration.

We **amount = [Compute the native amount](guide.计算金额(单价=190, 数量=5))**.

`report` =

| total | average |
|---|---|
| `amount` | `amount` |

We **formatted = [Use only the actual Intl presentation gap](formatter.format_summary(report=`report`))**.
1. not access(`formatted`, "ok")
  Return *formatted* before writing a result slot.
We **text = [Select the actual formatter output](access(access(formatted, "value"), "total"))**.
We **written = [Persist this actual output through Marqdo's native writeback](writeback.record(value=`text`, key="total"))**.
<!-- marqdo-out total
CN¥950.00
-->

After the explicit native file operation, return *[The persisted formatted value](result.ok(value=`text`))*.
