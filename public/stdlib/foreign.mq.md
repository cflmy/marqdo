---
markup: occlusion
title: lib/foreign — foreign kernels v2
description: Structured execution, JSON calls, parallel fan-out, persistent kernels and an explicit state bridge
import:
  foreign: lib/foreign.mq.md
stdin: 21
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

`lib/foreign` runs code fences and inline sources with local interpreters. v2 keeps the Markdown surface unchanged while adding structured records, JSON in/out, parallel fan-out, persistent Python/Node kernels and an explicit state bridge.

The same file is still a document: a bound fence is readable Markdown, and `foreign.*` turns it into an executable unit.

---

Probe before you run.

We first **node = [Compute node](foreign.probe(lang="node"))** for the next step.

We now **[Display this step's result](> print text=[ok](access(`node`, "ok")))**.
We now **[Display this step's result](> print text=[version](access(`node`, "version")))**.

`probe` returns `{lang, cmd, path, ok, version, error}`. Omit `lang` to inspect every known language.

---

A structured record, not just stdout.

`marker` =
```node
console.log(new Intl.NumberFormat("en-US", {style:"currency", currency:"CNY"}).format(950));
```

We first **record = [Compute record](foreign.invoke(code=`marker`))** for the next step.

We now **[Display this step's result](> print text=[ok](access(`record`, "ok")))**.
We now **[Display this step's result](> print text=[stdout](access(`record`, "stdout")))**.
We now **[Display this step's result](> print text=[code](access(`record`, "code")))**.

`invoke` never throws on a process failure. Read `ok`, `code`, `stdout`, `stderr`, `ms`, `timed_out`, `truncated`, `result` and `error` from one record.

---

JSON in, JSON out.

`format_money` =
```node
let input = "";
process.stdin.on("data", chunk => { input += chunk; });
process.stdin.on("end", () => {
  console.log(JSON.stringify(new Intl.NumberFormat("en-US", {style:"currency", currency:"CNY"}).format(JSON.parse(input))));
});
```

We first **single = [Compute single](foreign.call(code=`format_money`, input=21))** for the next step.

We now **[Display this step's result](> print text=`single`)**.

The input is encoded as JSON on stdin; the last stdout value is parsed as JSON. Use `output="json"` with `invoke` when you also want the raw record.

---

Parallel fan-out.

`numbers` =

| value |
|-------|
| 1 |
| 2 |
| 3 |
| 4 |

This single-column bound table is a native list of numbers, ready for the batch call.
We **formatted_batch = [Format each amount while retaining input order](foreign.call_many(code=`format_money`, inputs=`numbers`, concurrency=4))**.

We now **[Display this step's result](> print text=`formatted_batch`)**.

`call_many` applies one program to every input, runs independently, preserves input order, and returns parsed JSON results. `run_many` does the same for full invoke-option maps.

---

A persistent kernel.

We first **session = [Compute session](foreign.session_open(lang="node"))** for the next step.

We first **first = [Compute first](foreign.session_eval(id=`session`, source="var money = new Intl.NumberFormat(\"en-US\", {style:\"currency\", currency:\"CNY\"});"))** for the next step.

We first **second = [Compute second](foreign.session_eval(id=`session`, source="money.format(21)"))** for the next step.

We now **[Display this step's result](> print text=[result](access(`second`, "result")))**.

Variables created in the first evaluation survive into the second. The kernel is not a shell: use `session_eval`, `session_get`, `session_names`, then `session_close`.

---

The explicit state bridge.

We **[Copy the explicit rate into the existing kernel](foreign.session_set(id=`session`, name="rate", value=0.08))**.

We first **rate = [Compute rate](foreign.session_get(id=`session`, name="rate"))** for the next step.

We now **[Display this step's result](> print text=`rate`)**.

After reading the copied value, we **[Close this persistent formatting kernel](foreign.session_close(id=`session`))**.

This is a JSON copy, not shared memory. The kernel receives a value; Marqdo receives a value. There is no implicit mirroring and no hidden global namespace.

---

Cells in view.

`foreign.session` owns the document's kernel state. The kernel and cell order are ordinary tables passed as arguments; the names are free:

```text
`分析内核` =

| 名称 | 语言 | 命令 | 说明 |
|------|------|------|------|
| 分析 | node | | shared analysis |
| 复核 | node | | isolated review |

`分析单元` =

| 单元 | 内核 | 说明 |
|------|------|------|
| 准备数据 | 分析 | write state |
| 计算指标 | 分析 | read state |
| 独立复核 | 复核 | isolated kernel |

`会话` = > foreign.session kernels=`分析内核` cells=`分析单元`
```

`view` consumes the `会话` handle and adds Run All, Step, kernel switching, a journal and per-cell state. Without `会话`, view cells are stateless one-shot runs: same shell, no persistent kernel. There is no second CLI mode. See the native-first Intl workflow in [`features/19-foreign-view-cells.mq.md`](../features/19-foreign-view-cells.mq.md).

---

Safety defaults.

| Boundary | Default |
|---|---|
| Execution | requires the host execution capability |
| Shell | no shell; arguments are split explicitly |
| Output | bounded, with truncated |
| Timeout | process group killed on Unix / taskkill /T on Windows |
| CWD | starts at the granted root; the child retains normal OS authority |
| Parallelism | capped fan-out; each item isolated |

Read the full API in [../../lib/foreign.mq.md](../../lib/foreign.mq.md) and the design in [../../doc/design/foreign-kernel.md](../../doc/design/foreign-kernel.md).
