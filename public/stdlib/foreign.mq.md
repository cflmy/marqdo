---
markup: occlusion
title: lib/foreign — foreign kernels v2
description: Structured execution, JSON calls, parallel fan-out, persistent kernels and an explicit state bridge
import foreign:lib/foreign.mq.md
stdin: 21
---

# main

`lib/foreign` runs code fences and inline sources with local interpreters. v2 keeps the Markdown surface unchanged while adding structured records, JSON in/out, parallel fan-out, persistent Python/Node kernels and an explicit state bridge.

The same file is still a document: a bound fence is readable Markdown, and `foreign.*` turns it into an executable unit.

---

Probe before you run.

**node = > foreign.probe lang="node"**

> print text=[ok](access(`node`, "ok"))
> print text=[version](access(`node`, "version"))

`probe` returns `{lang, cmd, path, ok, version, error}`. Omit `lang` to inspect every known language.

---

A structured record, not just stdout.

`marker` =
```node
console.log("hello from a foreign kernel");
```

**record = > foreign.invoke code=`marker`**

> print text=[ok](access(`record`, "ok"))
> print text=[stdout](access(`record`, "stdout"))
> print text=[code](access(`record`, "code"))

`invoke` never throws on a process failure. Read `ok`, `code`, `stdout`, `stderr`, `ms`, `timed_out`, `truncated`, `result` and `error` from one record.

---

JSON in, JSON out.

`double` =
```node
let input = "";
process.stdin.on("data", chunk => { input += chunk; });
process.stdin.on("end", () => {
  console.log(JSON.stringify(JSON.parse(input) * 2));
});
```

**single = > foreign.call code=`double` input=21**

> print text=`single`

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

**doubled = > foreign.call_many code=`double` inputs=`numbers` concurrency=4**

> print text=`doubled`

`call_many` applies one program to every input, runs independently, preserves input order, and returns parsed JSON results. `run_many` does the same for full invoke-option maps.

---

A persistent kernel.

**session = > foreign.session_open lang="node"**

**first = > foreign.session_eval id=`session` source="let total = 0; for (let i = 1; i <= 5; i += 1) total += i;"**

**second = > foreign.session_eval id=`session` source="total * 2"**

> print text=[result](access(`second`, "result"))

Variables created in the first evaluation survive into the second. The kernel is not a shell: use `session_eval`, `session_get`, `session_names`, then `session_close`.

---

The explicit state bridge.

**> foreign.session_set id=`session` name="rate" value=0.08**

**rate = > foreign.session_get id=`session` name="rate"**

> print text=`rate`

**> foreign.session_close id=`session`**

This is a JSON copy, not shared memory. The kernel receives a value; Marqdo receives a value. There is no implicit mirroring and no hidden global namespace.

---

Cells in view.

`foreign.session` owns the document's kernel state. The kernel and cell order are ordinary tables passed as arguments; the names are free:

```text
`分析内核` =

| 名称 | 语言 | 命令 | 说明 |
|------|------|------|------|
| 分析 | python | | shared analysis |
| 复核 | python | | isolated review |

`分析单元` =

| 单元 | 内核 | 说明 |
|------|------|------|
| 准备数据 | 分析 | write state |
| 计算指标 | 分析 | read state |
| 独立复核 | 复核 | isolated kernel |

`会话` = > foreign.session kernels=`分析内核` cells=`分析单元`
```

`view` consumes the `会话` handle and adds Run All, Step, kernel switching, a journal and per-cell state. Without `会话`, view cells are stateless one-shot runs: same shell, no persistent kernel. There is no second CLI mode. See the complete Python walkthrough in [`features/19-foreign-view-cells.mq.md`](../features/19-foreign-view-cells.mq.md).

---

Safety defaults.

| Boundary | Default |
|---|---|
| Execution | requires the host execution capability |
| Shell | no shell; arguments are split explicitly |
| Output | bounded, with truncated |
| Timeout | process group killed on Unix / taskkill /T on Windows |
| CWD | file sandbox root; no implicit escape |
| Parallelism | capped fan-out; each item isolated |

Read the full API in [../../lib/foreign.mq.md](../../lib/foreign.mq.md) and the design in [../../doc/design/foreign-kernel.md](../../doc/design/foreign-kernel.md).
